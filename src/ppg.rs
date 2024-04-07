use std::collections::HashMap;
use std::fs;
use std::time::Duration;

use nostr_sdk::prelude as nostr;
use nostr_sdk::prelude::{Result};
use nostr_sdk::{ToBech32 as _};

use warp;
use warp::{Filter as _};

fn get_bech32(keys: &nostr::Keys) -> Result<(String, String)> {
    let pubkey = keys.public_key().to_bech32()?;
    let seckey = keys.secret_key()?.to_bech32()?;
    Ok((pubkey, seckey))
}

async fn new_keys_route() -> Result<impl warp::Reply, warp::Rejection> {
    let keys = nostr::Keys::generate();
    match get_bech32(&keys) {
	Ok((pubkey, seckey)) => {
	    Ok(warp::reply::json(
		&HashMap::from([
		    ("pubkey", &pubkey),
		    ("secret", &seckey),
		]),
	    ))
	},
	Err(_) => Err(warp::reject::reject()),
    }
}

#[allow(dead_code)]
async fn set_metadata(client: &nostr::Client) -> Result<()> {
    let metadata = nostr::Metadata::new()
        .name("fysx-nostr-ppg")
        .display_name("Fysx Nostr PPG")
        .about("Personal Playground for Nostr dev.")
        .lud16("weirdmexican50@walletofsatoshi.com");
    client.set_metadata(&metadata).await?;
    Ok(())
}

#[allow(dead_code)]
async fn get_events() -> Result<()> {
    // Create client from secret key.
    let keys = nostr::Keys::parse(fs::read_to_string(".nsec")?)?;

    // Show bech32 public key
    let pubkey = keys.public_key();
    let bech32_pubkey: String = pubkey.to_bech32()?;
    println!("Bech32 PubKey: {}", bech32_pubkey);

    // Create new client
    let client = nostr::Client::new(&keys);
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;

    // Get metadata.
    let filter = nostr::Filter::new().author(pubkey).kind(nostr::Kind::Metadata);
    let events = client
        .get_events_of(vec![filter], Some(Duration::from_secs(10)))
        .await?;
    println!("{events:#?}");

    // Get text notes.
    let filter = nostr::Filter::new()
        .author(pubkey)
        .kind(nostr::Kind::TextNote)
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

#[tokio::main]
async fn main() -> Result<()> {
    let routes = warp::get()
	.and(warp::path("new_keys"))
	.and_then(new_keys_route);
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
    Ok(())
}
