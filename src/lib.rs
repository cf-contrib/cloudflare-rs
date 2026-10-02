//! The Rust SDK for the Cloudflare API: its types, and a client, generated
//! from Cloudflare's OpenAPI document.
//!
//! Everything is under [`v4`]:
//!
//! ```
//! use cloudflare_sdk::v4::*;
//! ```
//!
//! # What is in it
//!
//! - **Types**: a request and response type for every schema in the
//!   document, such as [`v4::ZonesZone`].
//! - **Client** (`client` feature): `HttpClient`, a method per operation, which
//!   sends the API token it is given as a bearer token to
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
    // operations the generator can't encode, which return an error up front.
    #![allow(
        unused_imports,
        non_camel_case_types,
        unreachable_code,
        unused_mut,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_if,
        clippy::doc_lazy_continuation,
        clippy::doc_overindented_list_items,
        clippy::double_must_use,
        clippy::ifs_same_cond,
        clippy::match_single_binding,
        clippy::needless_else,
        clippy::nonminimal_bool,
        clippy::redundant_field_names,
        clippy::tabs_in_doc_comments,
        clippy::too_many_arguments,
        clippy::unnecessary_to_owned,
        clippy::vec_init_then_push
    )]

    include!(concat!(env!("OUT_DIR"), "/v4/mod.rs"));
}
