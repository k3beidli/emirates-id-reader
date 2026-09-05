# Platform setup

The Rust SDK supports native PC/SC on Windows, Linux, and macOS through
the [pcsc bindings](https://github.com/bluetech/pcsc-rust). The same
`CardSession` and data accessors work on each platform. Contactless and
browser readers are outside the current scope.

| Platform | Native library | Build prerequisite | Runtime prerequisite |
| --- | --- | --- | --- |
| Windows 10/11 | WinSCard | Rust MSVC toolchain, C++ Build Tools/Windows SDK | Smart Card service and reader driver |
| Linux | pcsc-lite (`libpcsclite`) | C toolchain, pkg-config, pcsc-lite headers | pcscd service and CCID/reader driver |
| macOS | System PCSC framework | Rust, Xcode Command Line Tools | Compatible reader/driver; system smart-card service |

## Windows

Install Rust using the MSVC toolchain and its C++ build prerequisites. Windows
supplies WinSCard. Connect the reader and confirm the Smart Card service is
available. No proprietary ICP toolkit is required.

## Linux

On Debian/Ubuntu, install build and runtime prerequisites:

```sh
sudo apt-get update
sudo apt-get install build-essential pkg-config libpcsclite-dev pcscd libccid
sudo systemctl enable --now pcscd.socket
```

On Fedora-family distributions, the corresponding packages include
`pcsc-lite-devel`, `pcsc-lite`, `pcsc-lite-ccid`, and `pkgconf-pkg-config`.
Service management and access policy depend on the distribution. Run the
application as your ordinary user; configure the system's PC/SC access policy
if it denies access. Do not use elevated execution as the application's default.

Cross-compilation requires the target PC/SC library and pkg-config setup, not
just a Rust target. Native CI builds avoid that ambiguity.

## macOS

Install Xcode Command Line Tools (`xcode-select --install`) and Rust. The
binding links Apple's system PCSC framework; installing a second PC/SC stack
through Homebrew is not required. Use a reader supported by macOS or install
the manufacturer's appropriate driver.

## Validate locally

```sh
cargo run --features cli -- probe
cargo run --features cli -- read --identity-only
cargo run --features cli -- read
```

Reads redact by default. `probe` reports reader/ATR connectivity without
reading personal fields. Run the [hardware checklist](testing.md) for your
reader and card generation. CI exercises native builds and synthetic tests;
it does not attach a physical reader. Historical V1/V2 hardware evidence is
Windows-specific and is not evidence of Linux/macOS reader compatibility.

## Sample application

The desktop sample is being introduced on Windows first so its appearance and
interaction can be reviewed before Linux/macOS distribution packages are
finalized. Its native Rust UI and SDK integration are kept portable. The CLI
and Rust examples can already be built on all three systems.
