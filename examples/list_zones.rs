//! Lists the zones an API token can read.
//!
//! ```sh
//! CLOUDFLARE_API_TOKEN=... cargo run --features client,zones --example list_zones
//! ```

use cloudflare::v4::HttpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("CLOUDFLARE_API_TOKEN")?;
    let client = HttpClient::new().with_api_key(token);

    let zones = client.zones_list_builder().per_page(50.0).send().await?;
    for zone in zones.result.unwrap_or_default() {
        println!("{}\t{}", zone.id, zone.name);
    }
    Ok(())
}
