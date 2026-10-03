//! Generates the Cloudflare `v4` API from `openapi/cloudflare/v4/openapi.yaml`,
//! with `overlay.yaml` applied, into `OUT_DIR`, which `src/lib.rs` mounts as
//! `v4`. How is in `openapi-to-rust.toml`.
//!
//! The crate's features pick what is generated. There is one per Cloudflare
//! product, which adds that product's operations and the types they use, and
//! `client`, which adds `HttpClient`. Nothing generated is checked in, so the
//! document and its overlay are the only things to edit.
//!
//! A product is the first part of an operation's `x-fern-sdk-group-name`, the
//! grouping Cloudflare's own SDKs use, so `dns.records` is in `dns`, and the
//! few operations with none are in `other`. The product features in
//! Cargo.toml are generated from the document too:
//! `CLOUDFLARE_SYNC_FEATURES=1 cargo check` rewrites them when it changes.
//!
//! Operations are named after Cloudflare's own SDKs too: `x-fern-sdk-group-name`
//! and `x-fern-sdk-method-name`, so `GET /zones/{zone_id}/dns_records` is
//! `dns_records_list` rather than its operationId,
//! `dns-records-for-a-zone-list-dns-records`. The few whose names the document
//! repeats keep their operationId.
//!
//! `CLOUDFLARE_CHECK=1` also generates the whole API, client and all, and
//! throws it away: it takes seconds where compiling it takes minutes, so CI
//! knows every feature generates without building them all.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use openapi_to_rust::{
    CodeGenerator, ConfigFile, GeneratorConfig, SchemaAnalyzer, TypeMapper,
    config::ClientSection,
    overlay::preprocess_spec,
    spec_source::{parse_spec, validate_oas_document},
};
use serde_json::Value;

const CONFIG: &str = "openapi-to-rust.toml";
const SPEC: &str = "openapi/cloudflare/v4/openapi.yaml";
const OVERLAY: &str = "openapi/cloudflare/v4/overlay.yaml";
const MANIFEST: &str = "Cargo.toml";

const SYNC_FEATURES: &str = "CLOUDFLARE_SYNC_FEATURES";
const CHECK: &str = "CLOUDFLARE_CHECK";

/// The product features in Cargo.toml are between these lines.
const FEATURES_BEGIN: &str = "# BEGIN products";
const FEATURES_END: &str = "# END products";

/// The product of the few operations the document puts in no group.
const UNGROUPED: &str = "other";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={CONFIG}");
    println!("cargo:rerun-if-changed={SPEC}");
    println!("cargo:rerun-if-changed={OVERLAY}");
    println!("cargo:rerun-if-changed={MANIFEST}");
    println!("cargo:rerun-if-env-changed={SYNC_FEATURES}");
    println!("cargo:rerun-if-env-changed={CHECK}");

    let out_dir = PathBuf::from(env::var("OUT_DIR")?).join("v4");
    let mut config = ConfigFile::load(CONFIG.as_ref())?.into_generator_config();
    config.output_dir = out_dir.clone();

    let spec = parse_spec(&fs::read_to_string(&config.spec_path)?, SPEC)?;
    let mut spec = preprocess_spec(spec, &config.schema_extensions, &config.overlays)?;
    name_operations(&mut spec);
    if let Some(warning) = validate_oas_document(&spec)? {
        println!("cargo:warning={warning}");
    }
    // The client's default base URL, https://api.cloudflare.com/client/v4,
    // comes from the document's `servers`.
    config.apply_spec_server_default(&spec);

    let products = products(&spec)?;
    sync_features(&products)?;

    // Every product, client and all, as `full` and `client` would generate it.
    if env::var_os(CHECK).is_some() {
        let every_operation = products.values().flatten().cloned().collect();
        generate(select(config.clone(), every_operation), spec.clone())?;
    }

    // The operations of every product feature that is on.
    let operations: Vec<String> = products
        .iter()
        .filter(|(product, _)| env::var_os(feature_env(product)).is_some())
        .flat_map(|(_, operations)| operations.iter().cloned())
        .collect();

    // A file from a feature since turned off would otherwise linger. Nothing
    // includes it, but it's stale all the same.
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir)?;
    }

    // No operations means all of them to the generator, so with no product on
    // there is nothing to generate.
    if operations.is_empty() {
        fs::create_dir_all(&out_dir)?;
        fs::write(out_dir.join("mod.rs"), "")?;
        return Ok(());
    }

    let client = env::var_os("CARGO_FEATURE_CLIENT").is_some();
    let (generator, mut result) = generate(select(config, operations), spec)?;
    if !client {
        result
            .files
            .retain(|file| file.path != Path::new("client.rs"));
    }

    // `include!` takes no inner attributes or inner doc comments, and the
    // generated module root opens with both. src/lib.rs says what they did.
    result.mod_file.content = result
        .mod_file
        .content
        .lines()
        .filter(|line| !line.starts_with("//!") && !line.starts_with("#!["))
        .filter(|line| client || !matches!(*line, "pub mod client;" | "pub use client::*;"))
        .collect::<Vec<_>>()
        .join("\n");

    generator.write_files(&result)?;
    Ok(())
}

/// Generates `operations` and the types they use. The client is generated
/// whether or not `client` is on, because pruning the types to those the
/// operations use is the client's to do. Without `client`, it's dropped.
fn select(mut config: GeneratorConfig, operations: Vec<String>) -> GeneratorConfig {
    config.enable_async_client = true;
    config.client = Some(ClientSection {
        operations,
        prune_models: true,
    });
    config
}

fn generate(
    config: GeneratorConfig,
    spec: Value,
) -> Result<(CodeGenerator, openapi_to_rust::GenerationResult), Box<dyn Error>> {
    let mapper = TypeMapper::new(config.types.clone());
    let mut analysis = SchemaAnalyzer::with_type_mapper(spec, mapper)?.analyze()?;
    let generator = CodeGenerator::new(config).with_source_provenance(SPEC);
    let result = generator.generate_all(&mut analysis)?;
    Ok((generator, result))
}

/// Renames every operation to its name in Cloudflare's own SDKs, its
/// `x-fern-sdk-group-name` and `x-fern-sdk-method-name` in snake case, unless
/// another operation would have the same one. Of two that would, one that
/// Cloudflare's SDKs leave out (`x-fern-ignore`) keeps its operationId and the
/// other takes the name: a deprecated operation beside its replacement. Any
/// others keep their operationIds: a v1 beside a v2.
fn name_operations(spec: &mut Value) {
    let Some(paths) = spec["paths"].as_object_mut() else {
        return;
    };
    let mut operations: Vec<&mut Value> = paths
        .values_mut()
        .filter_map(Value::as_object_mut)
        .flat_map(|item| item.values_mut())
        .filter(|operation| operation["operationId"].is_string())
        .collect();

    let ids: Vec<String> = operations
        .iter()
        .map(|operation| snake_case(operation["operationId"].as_str().unwrap_or_default()))
        .collect();
    let fern_names: Vec<Option<String>> = operations
        .iter()
        .map(|operation| {
            let group = operation["x-fern-sdk-group-name"].as_str()?;
            let method = operation["x-fern-sdk-method-name"].as_str()?;
            Some(snake_case(&format!("{group}.{method}")))
        })
        .collect();

    // Every operation whose Fern name no other has takes it, and of those
    // that share one, the only one Cloudflare's SDKs don't leave out.
    let ignored: Vec<bool> = operations
        .iter()
        .map(|operation| operation["x-fern-ignore"].as_bool() == Some(true))
        .collect();
    let mut counts = HashMap::<&str, usize>::new();
    let mut kept_counts = HashMap::<&str, usize>::new();
    for (name, ignored) in fern_names.iter().zip(&ignored) {
        if let Some(name) = name {
            *counts.entry(name).or_default() += 1;
            if !ignored {
                *kept_counts.entry(name).or_default() += 1;
            }
        }
    }
    let mut renamed: Vec<bool> = fern_names
        .iter()
        .zip(&ignored)
        .map(|(name, ignored)| {
            name.as_deref().is_some_and(|name| {
                counts[name] == 1 || (!ignored && kept_counts.get(name) == Some(&1))
            })
        })
        .collect();

    // Until no name is taken twice, one that does goes back to its id. The
    // ids are unique, so the operations that keep theirs never clash.
    loop {
        let kept: HashSet<&str> = ids
            .iter()
            .zip(&renamed)
            .filter(|(_, renamed)| !**renamed)
            .map(|(id, _)| id.as_str())
            .collect();
        let mut changed = false;
        for (index, name) in fern_names.iter().enumerate() {
            if renamed[index] && name.as_deref().is_some_and(|name| kept.contains(name)) {
                renamed[index] = false;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    for ((operation, name), renamed) in operations.iter_mut().zip(fern_names).zip(renamed) {
        if let (Some(name), true) = (name, renamed) {
            operation["operationId"] = Value::String(name);
        }
    }
}

/// `accounts.subscriptions.createBulk` and `workers-for-platforms` as
/// `accounts_subscriptions_create_bulk` and `workers_for_platforms`.
fn snake_case(name: &str) -> String {
    let mut snake = String::new();
    let mut previous_lower = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            if c.is_ascii_uppercase() && previous_lower {
                snake.push('_');
            }
            previous_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
            snake.push(c.to_ascii_lowercase());
        } else {
            if !snake.is_empty() && !snake.ends_with('_') {
                snake.push('_');
            }
            previous_lower = false;
        }
    }
    snake.trim_end_matches('_').to_string()
}

/// Every product, and its operations as `METHOD /path`: unlike operationIds,
/// which the document repeats, those are unique.
fn products(spec: &Value) -> Result<BTreeMap<String, Vec<String>>, Box<dyn Error>> {
    let mut products = BTreeMap::<String, Vec<String>>::new();
    let paths = spec["paths"]
        .as_object()
        .ok_or("the document has no paths")?;
    for (path, item) in paths {
        let Some(item) = item.as_object() else {
            continue;
        };
        for (method, operation) in item {
            if operation["operationId"].as_str().is_none() {
                continue;
            }
            let selector = format!("{} {path}", method.to_ascii_uppercase());
            let product = match operation["x-fern-sdk-group-name"].as_str() {
                Some(group) => feature_name(group.split('.').next().unwrap_or(group)),
                None => UNGROUPED.to_string(),
            };
            products.entry(product).or_default().push(selector);
        }
    }
    Ok(products)
}

/// Checks the product features in Cargo.toml are the document's, and with
/// `CLOUDFLARE_SYNC_FEATURES` set, rewrites them if they aren't.
fn sync_features(products: &BTreeMap<String, Vec<String>>) -> Result<(), Box<dyn Error>> {
    let manifest = fs::read_to_string(MANIFEST)?;
    let begin = manifest
        .find(FEATURES_BEGIN)
        .ok_or_else(|| format!("{MANIFEST} has no `{FEATURES_BEGIN}` line"))?;
    let end = manifest
        .find(FEATURES_END)
        .ok_or_else(|| format!("{MANIFEST} has no `{FEATURES_END}` line"))?;

    let mut block = format!(
        "{FEATURES_BEGIN}: generated by build.rs from the document, one feature per \
         Cloudflare product.\n# Every product.\nfull = [\n"
    );
    for product in products.keys() {
        block += &format!("  \"{product}\",\n");
    }
    block += "]\n";
    for (product, operations) in products {
        let count = operations.len();
        let noun = if count == 1 {
            "operation"
        } else {
            "operations"
        };
        block += &format!("{product} = [\"types\"] # {count} {noun}\n");
    }

    if manifest[begin..end] == block {
        return Ok(());
    }
    if env::var_os(SYNC_FEATURES).is_none() {
        return Err(format!(
            "the product features in {MANIFEST} aren't the document's: run \
             `{SYNC_FEATURES}=1 cargo check` to rewrite them"
        )
        .into());
    }
    let manifest = format!("{}{block}{}", &manifest[..begin], &manifest[end..]);
    fs::write(MANIFEST, manifest)?;
    // This build saw the features as they were.
    Err(format!("rewrote the product features in {MANIFEST}: build again").into())
}

/// A group name as a feature name: `fieldExtractors` and `analytics_engine`
/// are `field-extractors` and `analytics-engine`.
fn feature_name(group: &str) -> String {
    let mut name = String::new();
    let mut previous_lower = false;
    for c in group.chars() {
        if c.is_ascii_uppercase() && previous_lower {
            name.push('-');
        }
        previous_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
        name.push(match c {
            '_' => '-',
            c => c.to_ascii_lowercase(),
        });
    }
    name
}

/// The variable Cargo sets for a feature that is on.
fn feature_env(feature: &str) -> String {
    format!(
        "CARGO_FEATURE_{}",
        feature.to_ascii_uppercase().replace('-', "_")
    )
}
