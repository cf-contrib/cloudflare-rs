# cloudflare-rs

> The [Cloudflare API](https://developers.cloudflare.com/api/), as Cloudflare's
> OpenAPI document, and the Rust generated from it: the types, and a client.

[![CI](https://github.com/cf-contrib/cloudflare-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/cf-contrib/cloudflare-rs/actions/workflows/ci.yml)
[![Update Spec](https://github.com/cf-contrib/cloudflare-rs/actions/workflows/update-spec.yml/badge.svg)](https://github.com/cf-contrib/cloudflare-rs/actions/workflows/update-spec.yml)
[![Rust (edition 2024)](https://img.shields.io/badge/Rust-2024-black?logo=rust)](https://www.rust-lang.org/)
[![Nix Flake](https://img.shields.io/badge/Nix-Flake-5277C3?logo=nixos&logoColor=white)](https://nixos.wiki/wiki/Flakes)
[![License: MIT](https://img.shields.io/github/license/cf-contrib/cloudflare-rs)](LICENSE)
[![OpenAPI: Cloudflare v4](https://img.shields.io/badge/OpenAPI-Cloudflare%20v4-F38020?logo=cloudflare&logoColor=white)](https://github.com/cloudflare/api-schemas)

This isn't published to crates.io, and won't be: it's here until Cloudflare
releases an official Rust SDK generated from its document, as it has for Go,
TypeScript and Python. The `cloudflare` crate on crates.io is Cloudflare's own
[cloudflare-rs](https://github.com/cloudflare/cloudflare-rs), not this.

## Using it

Depend on it from git, with a feature for each Cloudflare product you use, and
`client` for the client:

```toml
[dependencies]
cloudflare = { git = "https://github.com/cf-contrib/cloudflare-rs", rev = "<commit>", features = ["client", "dns", "zones"] }
```

Pin a `rev`: the document changes weekly, and with it the types. Everything is under `v4`:

```rust
use cloudflare::v4::*;
```

To use it beside the `cloudflare` crate from crates.io, rename one of them:

```toml
cloudflare-api = { package = "cloudflare", git = "https://github.com/cf-contrib/cloudflare-rs", rev = "<commit>", features = ["client", "dns"] }
```

The build script generates the code from Cloudflare's document, which takes
some 15 seconds unoptimized, whenever the features you use change. Optimizing
build scripts makes that 5:

```toml
[profile.dev.build-override]
opt-level = 3
```

## What is in it

There is a feature per Cloudflare product, and nothing is generated for a
product that isn't on.

| Feature | What it adds |
|---|---|
| A product, such as `dns`, `zones`, `workers`, `kv` or `r2` | Its operations, and a struct or enum for every type they use, such as `ZonesZone`. |
| `full` | Every product. |
| `client` | `HttpClient`, a method per operation of the products that are on, over reqwest. |

The products are Cloudflare's own SDKs' resources: the first part of each
operation's `x-fern-sdk-group-name` in the document, so `dns.records` is in
`dns`. The few operations without one are in `other`. Cargo.toml lists them
all, with how many operations each has.

## Calling the API

```rust
use cloudflare::v4::HttpClient;

// The API token is sent as a bearer token, to https://api.cloudflare.com/client/v4.
let client = HttpClient::new().with_api_key(std::env::var("CLOUDFLARE_API_TOKEN")?);

let zone = client.zones_get("023e105f4ecef8ad9ca31a8372d0c353").await?;
```

Methods are named as in Cloudflare's own SDKs, by the product, resource and
action each operation's `x-fern-sdk-group-name` and `x-fern-sdk-method-name`
give it: `GET /zones/{zone_id}/dns_records` is `dns_records_list`, and
`GET /accounts/{account_id}/r2/buckets/{bucket_name}/objects/{object_key}` is
`r2_objects_get`. The few the document names twice, such as an account's and a
zone's version of one operation, keep their `operationId`. Their error types
are named after them too, as are responses defined inline: `ZonesListResponse`
and `ZonesListApiError`.

Operations the document marks deprecated are left out: most have a
replacement, and some only return 410 Gone.

Methods take the operation's parameters in the document's order, then its body,
with optional ones as `Option`s. An operation with more than three optional parameters, as
most list operations have, also has a builder, which takes the required ones
and has a setter for each of the rest:

```rust
let zones = client.zones_list_builder().name("example.com").per_page(50.0).send().await?;
```

An error the API answers with is an `ApiOpError::Api`, which holds the status,
the body, and the body parsed as that operation's error type when it matches
one.

[`examples/list_zones.rs`](examples/list_zones.rs) lists the zones a token can
read:

```sh
CLOUDFLARE_API_TOKEN=... cargo run --features client,zones --example list_zones
```

## Generated code

[`openapi/cloudflare/v4/openapi.yaml`](openapi/cloudflare/v4/openapi.yaml) is the
source, vendored unchanged from
[cloudflare/api-schemas](https://github.com/cloudflare/api-schemas) at
`03a6de21e114bf8f998013d5b477d7c20fa70475`.
[`overlay.yaml`](openapi/cloudflare/v4/overlay.yaml) beside it is an
[OpenAPI Overlay](https://spec.openapis.org/overlay/v1.1.0.html) of the fixes the
generator needs, which says what each one is.

`build.rs` runs [openapi-to-rust](https://github.com/gpu-cli/openapi-to-rust)
over the two into `OUT_DIR` on every build that changes them, as
[`openapi-to-rust.toml`](openapi-to-rust.toml) says, for the products that are
on. None of it is checked in or edited by hand.

The product features in Cargo.toml are generated from the document too, and the
build fails if they aren't the document's. To rewrite them:

```sh
CLOUDFLARE_SYNC_FEATURES=1 cargo check
```

Compiling every product takes some 10 minutes and over 10 GB of memory, so CI
compiles `dns` and `zones`, and `CLOUDFLARE_CHECK=1` has build.rs generate
the rest in memory, which takes seconds:

```sh
CLOUDFLARE_CHECK=1 cargo clippy --features client,dns,zones --all-targets
```

Every Monday, [a workflow](.github/workflows/update-spec.yml) opens a pull
request with Cloudflare's latest document and the features for it, saying
whether the generator still accepts it. If it doesn't, the fix goes in the
overlay. To do the same by hand:

```sh
curl -fsSL -o openapi/cloudflare/v4/openapi.yaml \
  https://raw.githubusercontent.com/cloudflare/api-schemas/main/openapi.yaml
CLOUDFLARE_SYNC_FEATURES=1 cargo check
```

and update the commit above.

## Development

`nix develop` has the toolchain, from [`flake.nix`](flake.nix), and so has the
[dev container](.devcontainer/devcontainer.json).

## Size

The document has some 3,600 operations and 7,000 schemas. With `full`, the
generated code is over a million lines, and `cargo check` takes several minutes
and over 10 GB of memory. `dns` and `zones` with `client` are some 36,000 lines,
and take seconds.

## Known gaps

- The generator can't encode a multipart request whose schema has typed
  additional properties, so uploading Workers assets returns `HttpError::Config`
  without sending anything.
- A few constraints the document states but Rust types can't, such as "at least
  one of `to`, `cc` or `bcc`", are the API's to enforce, not the types'.

## License

[MIT](LICENSE). The Cloudflare document in [`openapi/cloudflare/v4`](openapi/cloudflare/v4)
is Cloudflare's, under its own [BSD 3-Clause license](openapi/cloudflare/v4/LICENSE).
