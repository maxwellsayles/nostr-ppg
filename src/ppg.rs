use std::collections::HashMap;
use std::fs;
use std::thread;

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

fn load_keys() -> Result<nostr::Keys> {
    // Attempt to load the .nsec file to use a persistent signer.
    // If not, this will be read only mode.
    let keys = match fs::read_to_string(".nsec") {
	Ok(nsec) => nostr::Keys::parse(nsec)?,
	Err(_) => {
	    println!("No .nsec present. Generating new signing keys.");
	    nostr::Keys::generate()
	}
    };
    Ok(keys)
}

async fn make_client(keys: &nostr::Keys) -> Result<nostr_sdk::Client> {
    let opts = nostr::Options::new().wait_for_send(false);
    let client = nostr::ClientBuilder::new()
	.signer(keys)
	.opts(opts)
	.build();
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;
    Ok(client)
}

async fn notification_handler(client: &nostr_sdk::Client) -> Result<()> {
    client
	.handle_notifications(|notif| async {
	    match notif {
		nostr_sdk::RelayPoolNotification::Event {
		    event,
		    ..
		} => {
		    if event.kind() == nostr::Kind::TextNote {
			println!("Event: {event:?}");
		    }
		},
		_ => {
		    println!("Unknown: {notif:?}");
		},
	    }
	    Ok(false) // false => continue looping
	})
	.await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let keys = load_keys()?;
    let pubkey = keys.public_key();
    println!("Client will sign as {}", pubkey.to_bech32()?);
    let client = make_client(&keys).await?;

    // Subscribe to the signer's notes.
    let subscription = nostr::Filter::new()
        .author(pubkey)
        .kind(nostr::Kind::TextNote)
        .since(nostr_sdk::Timestamp::now());
    client.subscribe(vec![subscription], None).await;

    thread::spawn(move || {
	println!("Listening to nostr event notifications on a seprate thread.");
	let _ = futures::executor::block_on(notification_handler(&client));
    });

    // Set up the routes for the REST server.
    println!("Listening to REST requests on http://127.0.0.1:8080.");
    let routes = warp::get()
	.and(warp::path("new_keys"))
	.and_then(new_keys_route);
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
    Ok(())
}
