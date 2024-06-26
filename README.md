# Nostr PPG

This is my Nostr Personal Playground. This is a way for me to play with Rust and Nostr.

For now, this consists of a simple REST server with only a few end points, a Nostr client that subscribes to the signer's text notes, and a RocksDB instance to persists related event notifications.

This uses [nostr-sdk](https://github.com/rust-nostr/nostr/tree/master/crates/nostr-sdk) for the client,  [warp](https://github.com/seanmonstar/warp) to create a REST server, and [nostr-rocksdb](https://github.com/rust-nostr/nostr/tree/master/crates/nostr-rocksdb) for the DB backing. In each case, the crate was chosen because it had sufficient documentation for my purposes and the crates were referenced in multiple locations.

## Example usage

Spark this up with:

> $ cargo run

You can optionally create a `.nsec` file with your secret-key to be used, but if not, the server will just create one.

The following are examples of invoking the REST API.

1. You can query for a new public/secret key pair (these won't be used) with:

> $ curl -X GET http://127.0.0.1:8080/new-keys
> {"secret":"nsec19cdpzqauf3mkh8d6z27qy77g7qq5eux5f06sm62esr3uk4r2eyyqpv78vk","pubkey":"npub1ekc0z9u5snhkd86tqk4xl4ltxsef3z7pc0f6llazdvd268gqzeaqehe5nc"}

2. You can post a text note using the current signer (either from the `.nsec` file or the sever generated one) with:

> $ curl -X POST -H "Content-Type: application/json" -d '{"msg": "GM Nostriches"}' http://127.0.0.1:8080/publish-text-note

After executing the above command, you should see the server emit a message once the text note is received.

3. You can query for all the most recent text notes persisted (up to 10) with:

> $ curl -X GET http://127.0.0.1:8080/latest-text-notes?limit=10
[{"author_bech32":"npub1t3zx06xh8qhnle30qp0vqsx2ywye64u750q8l2j20jpj98s6kdwsxk2l50","content":"Yet another post, but this one is persisted locally.","created_at":1712621522}]

where `limit=10` is optional.