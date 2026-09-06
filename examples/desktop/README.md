# Emirates ID Reader sample app

A Windows app for reading Emirates ID cards without writing code.

## Run the executable

Run `emirates-id-reader-desktop.exe` directly; no app installer is needed.
The machine needs Microsoft Edge WebView2 Runtime and your reader's driver.

## Using the app

Select a connected reader and insert a card.

- **Automatic** reads on insertion and is the default on every launch.
- **Manual** reads when you press **Read card**. Use it to retry a failed read.
- **Stop reading** clears the display and pauses reading; **Resume reading** restarts it.
- **Refresh** updates the reader list. Removing the card clears its details.
- Copy buttons copy the displayed value. **Escape** outside a text field stops reading.

The app displays bilingual fields, formatted names, and dates such as **12 Jul 2008**.
Photos and signatures appear when available; unsupported image formats show a
preview-unavailable message.

Card data and preferences are not saved. Copied values remain on the system
clipboard and may be retained by clipboard history.

## Build and run from source

Install Node.js 22+, Rust with the MSVC toolchain, and the
[Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/).
From this directory:

```powershell
npm ci
npm run desktop
# Build the portable executable:
npm run desktop:build -- -- --locked
```

Output: `src-tauri/target/release/emirates-id-reader-desktop.exe`.
The executable includes the interface and assets. The app builds separately
and is excluded from the library's Cargo package.

## Development checks

```powershell
npm test
npm run build
npm run format:check
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Tests use synthetic fixtures. Check insertion/removal, reader disconnection,
and automatic/manual reading with hardware; also check narrow windows and
long bilingual names.

Asset sources and font licenses are listed in [asset credits](public/assets/README.md).

## Build downloads

The Desktop app workflow saves the Windows EXE as a `windows-x64` artifact for
14 days. Run it manually from Actions when needed. Pushing a version tag such
as `v0.1.0` also attaches the EXE to a draft release for testing before publication.

The Rust backend enables the library's optional `serde` feature for IPC and
requests all groups explicitly. Rust API naming changes do not change the JSON
fields consumed by the interface.
