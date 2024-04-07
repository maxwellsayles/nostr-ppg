use std::fs;
use std::time::Duration;

use nostr_sdk::prelude::*;

#[allow(dead_code)]
fn generate_keys() -> Result<()> {
    let keys = Keys::generate();
    let seckey = keys.secret_key()?.to_bech32()?;
    fs::write(".nsec", seckey)?;
    Ok(())
}

#[allow(dead_code)]
async fn set_metadata(client: &Client) -> Result<()> {
    let metadata = Metadata::new()
        .name("fysx-nostr-ppg")
        .display_name("Fysx Nostr PPG")
        .about("Personal Playground for Nostr dev.")
        .lud16("weirdmexican50@walletofsatoshi.com");
    client.set_metadata(&metadata).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create client from secret key.
    let keys = Keys::parse(fs::read_to_string(".nsec")?)?;

    // Show bech32 public key
    let pubkey = keys.public_key();
    let bech32_pubkey: String = pubkey.to_bech32()?;
    println!("Bech32 PubKey: {}", bech32_pubkey);

    // Create new client
    let client = Client::new(&keys);
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;

    // Get metadata.
    let filter = Filter::new().author(pubkey).kind(Kind::Metadata);
    let events = client
        .get_events_of(vec![filter], Some(Duration::from_secs(10)))
        .await?;
    println!("{events:#?}");

    // Get text notes.
    let filter = Filter::new()
        .author(pubkey)
        .kind(Kind::TextNote)
        .limit(3);
    let events = client
        .get_events_of(
            vec![filter],
            Some(Duration::from_secs(10)),
        )
        .await?;
    println!("{events:#?}");

    Ok(())
}
