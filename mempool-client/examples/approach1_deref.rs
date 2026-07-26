//! APPROACH 1: Deref + shadowing.
//!
//! `MempoolClientDeref` wraps the real `esplora_client::BlockingClient` and
//! derefs to it, so every compatible method is inherited for free. The
//! legacy `get_fee_estimates` is shadowed with an inherent method of the
//! same name (rustc always prefers an inherent method over one reached
//! through `Deref`), marked `#[deprecated]` since it still works today but
//! mempool.space is deprecating the underlying endpoint. All response
//! types come from `mempool_types`, which re-exports `esplora_types` in
//! full and adds `RecommendedFees` on top.
//!
//! Exactly three functions, matched across all three approach files for a
//! fair comparison:
//! - one private helper (`get_json_directly`), not part of this type's
//!   public API,
//! - one legacy `get_fee_estimates`, and
//! - one new dedicated `get_recommended_fees`.
//!
//! `main()` shows only how a downstream user calls the last two -- the
//! private helper is never callable from outside this type.
//!
//! Run with: `cargo run --example approach1_deref -p mempool-client`

use std::collections::HashMap;
use std::ops::Deref;

use esplora_client::{BlockingClient, Builder, Error};
use mempool_types::{FeeRate, RecommendedFees};

pub struct MempoolClientDeref {
    inner: BlockingClient,
}

impl MempoolClientDeref {
    pub fn new(base_url: &str) -> Self {
        Self { inner: Builder::new(base_url).build_blocking() }
    }

    /// Private: reuses nothing from `esplora-client`, not even its public
    /// `get_json` extension point. Only used internally, below.
    fn get_json_directly<T: for<'de> serde::Deserialize<'de>>(&self, path: &str) -> Result<T, String> {
        let url = format!("{}{path}", self.inner.url());
        println!("[private helper] GET {url}");
        bitreq::get(url)
            .with_timeout(10)
            .send()
            .map_err(|e| e.to_string())?
            .json::<T>()
            .map_err(|e| e.to_string())
    }

    /// Shadows `BlockingClient::get_fee_estimates`. Still works today, but as
    /// mempool.space is deprecating the underlying endpoint.
    #[deprecated(note = "mempool.space is deprecating /fee-estimates; use get_recommended_fees() instead")]
    pub fn get_fee_estimates(&self) -> Result<HashMap<u16, FeeRate>, Error> {
        self.inner.get_fee_estimates()
    }

    /// Genuinely new, mempool-only endpoint, built with the private
    /// helper above rather than any `esplora-client` internal.
    pub fn get_recommended_fees(&self) -> Result<RecommendedFees, String> {
        self.get_json_directly("/v1/fees/recommended")
    }
}

impl Deref for MempoolClientDeref {
    type Target = BlockingClient;
    fn deref(&self) -> &BlockingClient {
        &self.inner
    }
}

fn main() {
    let client = MempoolClientDeref::new("https://mempool.space/api");

    // How a downstream user calls the still-present, deprecated method --
    // this compiles, but produces a deprecation warning at this call site.
    #[allow(deprecated)]
    match client.get_fee_estimates() {
        Ok(fees) => println!("-> get_fee_estimates (deprecated) = {fees:?}\n"),
        Err(e) => println!("-> get_fee_estimates failed: {e}\n"),
    }

    // How a downstream user calls the new, dedicated replacement.
    match client.get_recommended_fees() {
        Ok(fees) => println!("-> get_recommended_fees = {fees:?}"),
        Err(e) => println!("-> get_recommended_fees failed: {e}"),
    }
}
