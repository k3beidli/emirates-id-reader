# Troubleshooting

Start with `cargo run --features cli -- probe`, then
`cargo run --features cli -- read --identity-only`. The CLI redacts reads by
default; keep personal values out of logs and issues.

| Symptom | Check |
| --- | --- |
| Cargo cannot find a runnable binary | Add `--features cli`; the default is library-only |
| Missing PC/SC library during build | Install the platform PC/SC development files; see [installation and platforms](Platforms) |
| Linker or Windows SDK build error | Install/configure the Rust Windows C++ build prerequisites |
| No readers discovered | USB connection and driver; check Device Manager on Windows or the platform reader tools |
| PC/SC service failure | Smart Card service on Windows, pcscd on Linux, or system reader support on macOS |
| Reader listed but no card | Contact orientation, chip-first insertion, card seating |
| Sharing/transaction failure | Another app may be using/resetting the reader; close it and retry |
| `Protocol` error on first read | Inserted card may not expose the Emirates ID application |
| `Unknown` generation | ATR is outside the known list; read may still work, support is not guaranteed |
| No photo after identity-only read | Use `.with_photo(true)` and inspect `read_status.photo` |
| No optional field despite `Read` status | File was read but the field may be blank/absent |
| Protected photo or extended data | Library cannot authenticate protected files; handle missing data |
| Getter returns old data after reinsertion | Discard old snapshot/session and reconnect |
| UI freezes during reading | Move blocking PC/SC work off the UI thread |
| `InvalidData` | A required value or card response failed validation; see [errors and read statuses](Error-Handling) |

Report the library revision, Rust version, operating system/version, reader model, generation,
error kind/status word, and reproduction steps. Do not submit photographs,
card dumps, full debug snapshots, personal identifiers, or proprietary keys.
See [errors and read statuses](Error-Handling) for what each error kind and
group status means, and [testing and hardware validation](Testing) for a
generation-specific checklist.
