// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Mempool Types
//!
//! mempool.space-specific types, layered on top of [`esplora-types`]. Most
//! of mempool.space's legacy `/api/*` endpoints return byte-compatible
//! Esplora response shapes, so this crate re-exports [`esplora-types`] in
//! full rather than redefining them, and only adds the types unique to
//! mempool.space's `/api/v1/*` endpoints.
//!
//! [`esplora-types`]: https://docs.rs/esplora-types

#![warn(missing_docs)]

pub use esplora_types::*;

use serde::Deserialize;

/// Recommended fee rates, in sat/vB, as returned by
/// `GET /api/v1/fees/recommended`.
///
/// Unlike [`esplora_types`]'s confirmation-target-keyed fee estimates, this
/// is mempool.space's own human-facing tiering, and is not part of the
/// Esplora API this crate's types otherwise mirror.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RecommendedFees {
    /// Fee rate for next-block confirmation.
    #[serde(rename = "fastestFee")]
    pub fastest_fee: u32,
    /// Fee rate for confirmation within ~30 minutes.
    #[serde(rename = "halfHourFee")]
    pub half_hour_fee: u32,
    /// Fee rate for confirmation within ~1 hour.
    #[serde(rename = "hourFee")]
    pub hour_fee: u32,
    /// Fee rate with no particular confirmation target.
    #[serde(rename = "economyFee")]
    pub economy_fee: u32,
    /// The minimum fee rate the mempool will currently accept.
    #[serde(rename = "minimumFee")]
    pub minimum_fee: u32,
}
