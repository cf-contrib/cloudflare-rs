# Cloudflare SDK

> The [Cloudflare API](https://developers.cloudflare.com/api/), as Cloudflare's
> OpenAPI document, and the Rust generated from it: the types, and a client.

Everything is under `v4`:

```rust
use cloudflare_sdk::v4::*;
```

## What is in it

| Feature | What it adds |
|---|---|
| (none) | The types: a struct or enum for every schema in the document, such as `ZonesZone`. |
| `client` | `HttpClient`, a method per operation, over reqwest. |

## Calling the API

```rust
use cloudflare_sdk::v4::HttpClient;

// The API token is sent as a bearer token, to https://api.cloudflare.com/client/v4.
let client = HttpClient::new().with_api_key(std::env::var("CLOUDFLARE_API_TOKEN")?);

let zone = client.zones_0_get("023e105f4ecef8ad9ca31a8372d0c353").await?;
```

Method names are the document's `operationId`s, in snake case. They take the
operation's parameters in the document's order, then its body. Optional
parameters are `Option`s, so a list operation reads
`client.zones_get(None::<&str>, None, ..)`.

An error the API answers with is an `ApiOpError::Api`, which holds the status,
the body, and the body parsed as that operation's error type when it matches
one.

[`examples/list_zones.rs`](examples/list_zones.rs) lists the zones a token can
read:

```sh
CLOUDFLARE_API_TOKEN=... cargo run --features client --example list_zones
```

## Generated code

[`openapi/cloudflare/v4/openapi.yaml`](openapi/cloudflare/v4/openapi.yaml) is the
source, vendored unchanged from
[cloudflare/api-schemas](https://github.com/cloudflare/api-schemas) at
`50d6b84ee50aa35f62703bf64a24f3bf76c4908a`.
[`overlay.yaml`](openapi/cloudflare/v4/overlay.yaml) beside it is an
[OpenAPI Overlay](https://spec.openapis.org/overlay/v1.1.0.html) of the fixes the
generator needs, which says what each one is.

`build.rs` runs [openapi-to-rust](https://github.com/gpu-cli/openapi-to-rust)
over the two into `OUT_DIR` on every build that changes them, with the client
only when its feature is on. None of it is checked in or edited by hand. How it
runs is in [`openapi-to-rust.toml`](openapi-to-rust.toml), which the CLI reads
too, so CI checks the whole API generates in seconds without compiling it:

```sh
openapi-to-rust generate --config openapi-to-rust.toml --dry-run
```

Every Monday, [a workflow](.github/workflows/refresh-spec.yml) opens a pull
request with Cloudflare's latest document, saying whether the generator still
accepts it. If it doesn't, the fix goes in the overlay. To do the same by hand:

```sh
curl -fsSL -o openapi/cloudflare/v4/openapi.yaml \
  https://raw.githubusercontent.com/cloudflare/api-schemas/main/openapi.yaml
```

and update the commit above.

## Size

The document has some 3,600 operations and 7,000 schemas, and the generated
code is over a million lines. Expect `cargo check` of this crate to take
several minutes and over 10 GB of memory, the first time and whenever the
document changes. After that, cargo reuses it.

## Known gaps

- There are no per-operation builders. openapi-to-rust can generate them, but in
  0.19 the ones for a dozen of Cloudflare's operations, whose bodies are unions,
  don't compile.
- The generator can't encode a multipart field that is a JSON object, so the 13
  operations that take one return `HttpError::Config` without sending anything:
  uploading a Worker's script, content, version or settings (including under
  Workers for Platforms), writing a KV value with metadata, uploading an image
  or creating a direct upload URL, and converting to Markdown with Workers AI.
- A few constraints the document states but Rust types can't, such as "at least
  one of `to`, `cc` or `bcc`", are the API's to enforce, not the types'.
