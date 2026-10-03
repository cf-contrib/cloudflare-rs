//! The Rust SDK for the Cloudflare API: its types, and a client, generated
//! from Cloudflare's OpenAPI document.
//!
//! Everything is under [`v4`]:
//!
//! ```
//! use cloudflare::v4::*;
//! ```
//!
//! # What is in it
//!
//! There is a feature per Cloudflare product, such as `dns`, `zones` or
//! `workers`, and `full` for all of them. Nothing is generated for a product
//! that isn't on, so turn on only those you use: all of them are over a million
//! lines.
//!
//! - **Types**: a request and response type for every schema the products'
//!   operations use, such as `ZonesZone` (`zones` feature).
//! - **Client** (`client` feature): `HttpClient`, a method per operation of
//!   the products, which sends the API token it is given as a bearer token to
//!   `https://api.cloudflare.com/client/v4`.
//!
//! # Generated code
//!
//! `openapi/cloudflare/v4/openapi.yaml` is the source, vendored unchanged from
//! [cloudflare/api-schemas](https://github.com/cloudflare/api-schemas), and
//! `overlay.yaml` beside it the fixes the generator needs. `build.rs` runs
//! [openapi-to-rust](https://github.com/gpu-cli/openapi-to-rust) over them into
//! `OUT_DIR`, so none of it is checked in or edited by hand.

/// Everything for the Cloudflare `v4` API: the types, and the client the
/// crate's features enable.
pub mod v4 {
    // The generated module root opens with `unused_imports`, which `include!`
    // can't take: build.rs strips it, and it's restated here. The rest are the
    // generator's style, not ours to fix. `unreachable_code` is the multipart
    // operations the generator can't encode, which return an error up front,
    // and `dead_code` the helpers only some products use.
    #![allow(
        unused_imports,
        dead_code,
        non_camel_case_types,
        unreachable_code,
        unused_mut,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_if,
        clippy::doc_lazy_continuation,
        clippy::doc_overindented_list_items,
        clippy::double_must_use,
        clippy::match_single_binding,
        clippy::needless_else,
        clippy::nonminimal_bool,
        clippy::redundant_field_names,
        clippy::result_large_err,
        clippy::tabs_in_doc_comments,
        clippy::too_many_arguments,
        clippy::unnecessary_to_owned,
        clippy::vec_init_then_push
    )]

    include!(concat!(env!("OUT_DIR"), "/v4/mod.rs"));
}
