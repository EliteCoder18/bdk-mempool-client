//! APPROACH 1: Deref + shadowing.
//!
//! `MempoolClientDeref` wraps the real `esplora_client::BlockingClient` and
//! derefs to it, so every compatible method is inherited for free. Methods
//! that behave differently on mempool.space are shadowed with an inherent
//! method of the same name (rustc always prefers an inherent method over
//! one reached through `Deref`). The new mempool-only endpoint is built
//! with this file's own tiny request helper -- it never touches anything
//! private inside `esplora-client`.
//!
//! This is a standalone comparison file, independent of the real
//! `mempool_client::MempoolClient` in `src/lib.rs` -- kept separate so all
//! three approaches in this `examples/` directory are built the exact same
//! way and are directly comparable.
//!
//! Run with: `cargo run --example approach1_deref -p mempool-client`

use std::collections::HashMap;
use std::ops::Deref;

use esplora_client::{BlockingClient, Builder, Error, FeeRate};
use serde::Deserialize;

/// This file's own request helper -- reuses nothing from
/// `esplora-client`, not even its public `get_json` extension point.
fn get_json_directly<T: for<'de> Deserialize<'de>>(base_url: &str, path: &str) -> Result<T, String> {
    let url = format!("{base_url}{path}");
    println!("[mempool_client's own helper] GET {url}");
    bitreq::get(url)
        .with_timeout(10)
        .send()
        .map_err(|e| e.to_string())?
        .json::<T>()
        .map_err(|e| e.to_string())
}

#[derive(Deserialize, Debug)]
pub struct RecommendedFees {
    #[serde(rename = "fastestFee")]
    pub fastest_fee: u32,
}

pub struct MempoolClientDeref {
    inner: BlockingClient,
}

impl MempoolClientDeref {
    pub fn new(base_url: &str) -> Self {
        Self { inner: Builder::new(base_url).build_blocking() }
    }

    /// Shadows `BlockingClient::get_fee_estimates`. Still works today, but
    /// mempool.space is deprecating the underlying endpoint.
    #[deprecated(note = "mempool.space is deprecating /fee-estimates; use get_recommended_fees() instead")]
    pub fn get_fee_estimates(&self) -> Result<HashMap<u16, FeeRate>, Error> {
        #[allow(deprecated)]
        self.inner.get_fee_estimates()
    }

    /// Genuinely new, mempool-only endpoint, built with this file's own
    /// helper rather than any `esplora-client` internal.
    pub fn get_recommended_fees(&self) -> Result<RecommendedFees, String> {
        get_json_directly(self.inner.url(), "/v1/fees/recommended")
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

    // 1. Inherited via Deref -- never defined on MempoolClientDeref at all.
    match client.get_height() {
        Ok(height) => println!("-> height = {height}\n"),
        Err(e) => println!("-> get_height failed: {e}\n"),
    }

    // 2. Shadowed -- this method wins over the one reachable via Deref.
    #[allow(deprecated)]
    match client.get_fee_estimates() {
        Ok(fees) => println!("-> old_fees (deprecated path) = {fees:?}\n"),
        Err(e) => println!("-> get_fee_estimates failed: {e}\n"),
    }

    // 3. Brand new, via this file's own helper.
    match client.get_recommended_fees() {
        Ok(fees) => println!("-> new_fees = {fees:?}"),
        Err(e) => println!("-> get_recommended_fees failed: {e}"),
    }
}
