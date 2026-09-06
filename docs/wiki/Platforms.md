<a id="platform-setup"></a>

# Installation and platforms

The library uses native PC/SC through the
[pcsc bindings](https://github.com/bluetech/pcsc-rust), with one Rust API for
Windows, Linux, and macOS.

**Hardware testing is limited to the HID OMNIKEY 3121 on Windows.** Linux and
macOS are expected to work through their native PC/SC backends, but neither has
been hardware-tested. No other reader models have been tested. CI builds and
synthetic tests do not verify communication with a physical reader.

See [testing and hardware validation](Testing) for compatibility details.

## Requirements

- Windows 10/11, Linux with pcsc-lite, or macOS with the system PCSC framework.
- Rust 1.85 or newer and the platform build toolchain.
- A PC/SC contact reader with its normal driver, and an inserted Emirates ID.

Card reading works locally without a proprietary toolkit.

| Platform | Native library | Build prerequisite | Runtime prerequisite |
| --- | --- | --- | --- |
| Windows 10/11 | WinSCard | Rust MSVC toolchain, C++ Build Tools/Windows SDK | Smart Card service and reader driver |
| Linux | pcsc-lite (`libpcsclite`) | C toolchain, pkg-config, pcsc-lite headers | pcscd service and CCID/reader driver |
| macOS | System PCSC framework | Rust, Xcode Command Line Tools | Compatible reader/driver; system smart-card service |

## Add the dependency

```toml
[dependencies]
emirates-id-reader = { git = "https://github.com/k3beidli/emirates-id-reader" }
```

For development alongside this checkout, use a path dependency instead:

```toml
[dependencies]
emirates-id-reader = { path = "../emirates-id-reader" }
```

Commit your application's `Cargo.lock` to retain the resolved Git revision. For
a release you can also pin `rev = "<full reviewed commit SHA>"`.

## Optional serialization

The default build does not depend on Serde. Add `features = ["serde"]` to the
dependency to enable `Serialize` for snapshots, records, statuses, and errors.
The desktop app enables this feature for its Tauri bridge. No `Deserialize`
implementation is provided for hardware snapshots.

## Windows

Install Rust with the MSVC toolchain and its C++ build prerequisites. Windows
supplies WinSCard. Connect the reader and confirm the Smart Card service is
running. No proprietary ICP toolkit is required.

## Linux

On Debian and Ubuntu:

```sh
sudo apt-get update
sudo apt-get install build-essential pkg-config libpcsclite-dev pcscd libccid
sudo systemctl enable --now pcscd.socket
```

On Fedora-family distributions the corresponding packages are
`pcsc-lite-devel`, `pcsc-lite`, `pcsc-lite-ccid`, and `pkgconf-pkg-config`.
Service management and access policy vary by distribution. Run as your ordinary
user and adjust the system's PC/SC access policy if it denies access; elevated
execution should not be the application's default.

Cross-compilation needs the target's PC/SC library and pkg-config setup, not
just a Rust target.

## macOS

Install Xcode Command Line Tools (`xcode-select --install`) and Rust. The
binding links Apple's system PCSC framework, so a second PC/SC stack from
Homebrew is unnecessary. Use a reader macOS supports, or install the
manufacturer's driver.

<a id="sample-application"></a>

## Validate locally

```sh
cargo run --features cli -- probe
cargo run --features cli -- read --identity-only
cargo run --features cli -- read
```

Reads redact by default. `probe` reports reader and ATR connectivity without
reading personal fields. Run the [hardware checklist](Testing) for your
reader and card generation.

Continue with [your first read](Getting-Started).
