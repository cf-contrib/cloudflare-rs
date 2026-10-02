//! Lists the zones an API token can read.
//!
//! ```sh
//! CLOUDFLARE_API_TOKEN=... cargo run --features client --example list_zones
//! ```

use cloudflare_sdk::v4::HttpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("CLOUDFLARE_API_TOKEN")?;
    let client = HttpClient::new().with_api_key(token);

    // name, status, type, account.id, account.name, page, per_page, order,
    // direction, match.
    let zones = client
        .zones_get(
            None::<&str>,
            None,
            None,
            None::<&str>,
            None::<&str>,
            None,
            Some(50.0),
            None,
            None,
            None,
        )
        .await?;
    for zone in zones.result.unwrap_or_default() {
        println!("{}\t{}", zone.id, zone.name);
    }
    Ok(())
}
