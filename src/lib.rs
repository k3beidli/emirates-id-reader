//! A Rust library for reading data from Emirates ID chips.
//!
//! The crate talks to contact cards through Windows PC/SC. Cardholder data is
//! returned in memory and is never persisted by the SDK.
//!
//! # Quick start
//!
//! ```no_run
//! use emirates_id_reader::{CardSession, ReadOptions};
//!
//! # fn main() -> Result<(), emirates_id_reader::Error> {
//! let session = CardSession::connect_first()?;
//! let card = session.read_with_options(ReadOptions::identity_only())?;
//! let name = card.get_name();
//! let id = card.get_id_number();
//! // Bind borrowed values to your UI without logging personal data.
//! # Ok(())
//! # }
//! ```
//!
//! Use [`CardSession::read`] when photographs and all other supported public
//! groups are required. Inspect [`EmiratesIdData::read_status`] to distinguish
//! a blank field from an optional group that was protected, unavailable, or
//! deliberately skipped.

#![deny(missing_docs)]

mod apdu;
mod data;
mod decode;
mod error;
mod protocol;
mod session;
mod transport;

pub use data::*;
pub use error::{Error, ErrorKind};
pub use session::CardSession;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod sdk_tests;

// Compile the consumer examples in the shipped guides as Rustdoc tests.
#[cfg(doctest)]
#[doc = concat!(
    include_str!("../README.md"),
    "\n", include_str!("../docs/getting-started.md"),
    "\n", include_str!("../docs/integration.md"),
    "\n", include_str!("../docs/error-handling.md"),
    "\n", include_str!("../docs/data-model.md"),
)]
mod guide_examples {}
