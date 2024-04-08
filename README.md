# Nostr PPG

This is my Nostr Personal Playground. This is a way for me to play with Rust and Nostr.

For now, this is a small REST server with a few end points, and a Nostr client that subscribes to the signer's text notes.

I'm planning to persist notes to a DB and to provide rest APIs for querying the DB.

This uses [nostr-sdk](https://github.com/rust-nostr/nostr/tree/master/crates/nostr-sdk) for the client, and [warp](https://github.com/seanmonstar/warp) to create a REST server. Both were chosen because they had sufficient documentation for my purposes and were referenced in many locations.