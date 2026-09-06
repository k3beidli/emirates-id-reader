# Rust snippets

Run from the repository root:

```sh
cargo run --example read_identity
cargo run --example read_photo
cargo run --example watch_removal
```

These programs require a connected PC/SC reader and a card. To compile them
without accessing hardware, run `cargo check --examples --locked`.
