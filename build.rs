//! Generates the Cloudflare `v4` API from `openapi/cloudflare/v4/openapi.yaml`,
//! with `overlay.yaml` applied, into `OUT_DIR`, which `src/lib.rs` mounts as
//! `v4`. How is in `openapi-to-rust.toml`, which the CLI reads too.
//!
//! The crate's features pick what is generated: the model types always, the
//! reqwest client with `client`. Nothing generated is checked in, so the
//! document and its overlay are the only things to edit.

use std::{env, error::Error, fs, path::PathBuf};

use openapi_to_rust::{
    CodeGenerator, ConfigFile, SchemaAnalyzer, TypeMapper,
    overlay::preprocess_spec,
    spec_source::{parse_spec, validate_oas_document},
};

const CONFIG: &str = "openapi-to-rust.toml";
const SPEC: &str = "openapi/cloudflare/v4/openapi.yaml";
const OVERLAY: &str = "openapi/cloudflare/v4/overlay.yaml";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={CONFIG}");
    println!("cargo:rerun-if-changed={SPEC}");
    println!("cargo:rerun-if-changed={OVERLAY}");

    let out_dir = PathBuf::from(env::var("OUT_DIR")?).join("v4");
    let client = env::var_os("CARGO_FEATURE_CLIENT").is_some();

    let mut config = ConfigFile::load(CONFIG.as_ref())?.into_generator_config();
    config.output_dir = out_dir.clone();
    config.enable_async_client = client;

    let spec = parse_spec(&fs::read_to_string(&config.spec_path)?, SPEC)?;
    let spec = preprocess_spec(spec, &config.schema_extensions, &config.overlays)?;
    if let Some(warning) = validate_oas_document(&spec)? {
        println!("cargo:warning={warning}");
    }
    // The client's default base URL, https://api.cloudflare.com/client/v4,
    // comes from the document's `servers`.
    config.apply_spec_server_default(&spec);

    let mapper = TypeMapper::new(config.types.clone());
    let mut analysis = SchemaAnalyzer::with_type_mapper(spec, mapper)?.analyze()?;
    let generator = CodeGenerator::new(config).with_source_provenance(SPEC);
    let mut result = generator.generate_all(&mut analysis)?;

    // `include!` takes no inner attributes or inner doc comments, and the
    // generated module root opens with both. src/lib.rs says what they did.
    result.mod_file.content = result
        .mod_file
        .content
        .lines()
        .filter(|line| !line.starts_with("//!") && !line.starts_with("#!["))
        .collect::<Vec<_>>()
        .join("\n");

    // A file from a feature since turned off would otherwise linger. Nothing
    // includes it, but it's stale all the same.
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir)?;
    }
    generator.write_files(&result)?;
    Ok(())
}
