// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Mempool Client
//!
//! A blocking client for mempool.space, built by composing
//! [`esplora_client::BlockingClient`] rather than duplicating it.
//!
//! [`MempoolClient`] derefs to the inner [`BlockingClient`], so every
//! Esplora-compatible method (`get_tx`, `get_block_status`, address and
//! scripthash stats, ...) is available unchanged, with no code written
//! here for any of them. Methods whose behavior genuinely differs on
//! mempool.space are shadowed with an inherent method of the same name,
//! marked `#[deprecated]` where the old behavior still works today but
//! is going away. New mempool.space-only endpoints are built on
//! [`BlockingClient::get_json`], the one deliberate public extension
//! point `esplora-client` exposes for this purpose.
//!
//! This crate is a prototype scoped to prove out that pattern -- it does
//! not yet cover the async client or the rest of the mempool.space
//! `/api/v1/*` surface.

#![warn(missing_docs)]

// Re-exported so a downstream user only ever needs `mempool-client` as a
// dependency: `Builder`/`Error`/`BlockingClient` come straight from
// `esplora-client`, unmodified, and every response type (`Transaction`,
// `Txid`, `TxStatus`, ...) comes from `mempool-types`, which itself
// re-exports all of `esplora-types`. Nothing here is a copy -- these are
// the exact same types, just reachable without an extra `use` or an
// extra direct Cargo dependency.
pub use esplora_client::{BlockingClient, Builder, Error};
pub use mempool_types::*;

use std::collections::HashMap;
use std::ops::Deref;

/// A blocking client for mempool.space.
///
/// Derefs to [`BlockingClient`] for every method this crate doesn't
/// explicitly shadow or add.
pub struct MempoolClient {
    inner: BlockingClient,
}

impl MempoolClient {
    /// Create a [`MempoolClient`] from an already-configured
    /// [`esplora_client::Builder`].
    ///
    /// Use this when you need to set a timeout, proxy, custom headers, or
    /// retry count -- [`Builder`] is reused directly from `esplora-client`,
    /// unmodified; there is no separate mempool-specific builder type.
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use mempool_client::{Builder, MempoolClient};
    ///
    /// let client = MempoolClient::from_builder(
    ///     Builder::new("https://mempool.space/api")
    ///         .timeout(Duration::from_secs(10))
    ///         .max_retries(3),
    /// );
    /// ```
    pub fn from_builder(builder: Builder) -> Self {
        Self { inner: builder.build_blocking() }
    }

    /// Create a [`MempoolClient`] for the given mempool.space-compatible
    /// server base URL, using default configuration.
    ///
    /// Equivalent to `MempoolClient::from_builder(Builder::new(base_url))`.
    /// Use [`Self::from_builder`] directly if you need to configure the
    /// client beyond just its base URL.
    pub fn new(base_url: &str) -> Self {
        Self::from_builder(Builder::new(base_url))
    }

    /// Get fee estimates keyed by confirmation target, in sat/vB.
    ///
    /// Shadows [`BlockingClient::get_fee_estimates`]: the underlying
    /// `/fee-estimates` endpoint this calls is being deprecated by
    /// mempool.space. It still works today, but new code should prefer
    /// [`Self::get_recommended_fees`].
    #[deprecated(
        note = "mempool.space is deprecating /fee-estimates; use get_recommended_fees() instead"
    )]
    pub fn get_fee_estimates(&self) -> Result<HashMap<u16, FeeRate>, Error> {
        #[allow(deprecated)]
        self.inner.get_fee_estimates()
    }

    /// Get mempool.space's own recommended fee-rate tiers, in sat/vB.
    ///
    /// A genuinely new endpoint, not present in the Esplora API this
    /// crate otherwise mirrors via [`Deref`]. Built on
    /// [`BlockingClient::get_json`] rather than any private
    /// `esplora-client` internal.
    pub fn get_recommended_fees(&self) -> Result<RecommendedFees, Error> {
        self.inner.get_json("/v1/fees/recommended")
    }
}

impl Deref for MempoolClient {
    type Target = BlockingClient;

    fn deref(&self) -> &BlockingClient {
        &self.inner
    }
}
