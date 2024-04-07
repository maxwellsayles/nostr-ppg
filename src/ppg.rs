use std::collections::HashMap;
use std::convert::Infallible;
use std::fs;
use std::thread;

use nostr_sdk::prelude as nostr;
use nostr_sdk::prelude::Result;
use nostr_sdk::{NostrDatabase as _, ToBech32 as _};

use serde_derive::{Deserialize, Serialize};

use warp;
use warp::Filter as _;

fn with_client(
    client: nostr_sdk::Client,
) -> impl warp::Filter<Extract = (nostr_sdk::Client,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

async fn new_keys_route() -> Result<impl warp::Reply, warp::Rejection> {
    let keys = nostr::Keys::generate();
    match get_bech32(&keys) {
        Ok((pubkey, seckey)) => Ok(warp::reply::json(&HashMap::from([
            ("pubkey", &pubkey),
            ("secret", &seckey),
        ]))),
        Err(_) => Err(warp::reject::reject()),
    }
}

#[derive(Serialize, Deserialize)]
struct PublishTextNoteQuery {
    msg: String,
}

async fn publish_text_note_route(
    client: nostr_sdk::Client,
    params: PublishTextNoteQuery,
) -> Result<impl warp::Reply, warp::Rejection> {
    match client.publish_text_note(params.msg, []).await {
        Ok(_) => Ok(warp::http::StatusCode::OK),
        Err(_) => Err(warp::reject::reject()),
    }
}

fn get_bech32(keys: &nostr::Keys) -> Result<(String, String)> {
    let pubkey = keys.public_key().to_bech32()?;
    let seckey = keys.secret_key()?.to_bech32()?;
    Ok((pubkey, seckey))
}

/**
 * Attempt to load the .nsec file to use a persistent signer across runs.
 * Otherwise, generate new keys so messages can still be signed.
 */
fn load_keys() -> Result<nostr::Keys> {
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
    let client = nostr::ClientBuilder::new().signer(keys).opts(opts).build();
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;
    Ok(client)
}

async fn notification_handler(
    db: &dyn nostr_sdk::NostrDatabase<Err = nostr_database::DatabaseError>,
    client: &nostr_sdk::Client,
) -> Result<()> {
    client
        .handle_notifications(|notif| async {
            match notif {
                nostr_sdk::RelayPoolNotification::Event { event, .. } => {
                    if event.kind() == nostr::Kind::TextNote {
                        println!("Event: {event:?}");
                        db.save_event(&event).await?;
                    }
                }
                _ => {
                    println!("Unknown: {notif:?}");
                }
            }
            Ok(false) // false => continue looping
        })
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = nostr_rocksdb::RocksDatabase::open("rocksdb")
        .await
        .expect("Unable to open or create rocksdb.");
    println!(
        "DB \"rocksdb\" opened and stores {} events.",
        db.count(vec![nostr::Filter::new()]).await.unwrap()
    );

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

    let client2 = client.clone();
    thread::spawn(move || {
        println!("Listening to nostr event notifications on a seprate thread.");
        let _ = futures::executor::block_on(notification_handler(&db, &client2));
    });

    // Set up the routes for the REST server.
    println!("Listening to REST requests on http://127.0.0.1:8080.");
    let new_keys = warp::get()
        .and(warp::path("new-keys"))
        .and(warp::path::end())
        .and_then(new_keys_route);
    let publish_text_note = warp::post()
        .and(warp::path("publish-text-note"))
        .and(warp::path::end())
        .and(with_client(client.clone()))
        .and(warp::body::json())
        .and_then(publish_text_note_route);
    let routes = new_keys.or(publish_text_note);
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
    Ok(())
}
