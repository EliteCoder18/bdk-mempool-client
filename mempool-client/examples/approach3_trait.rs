//! APPROACH 3: Shared trait.
//!
//! Both the real `esplora_client::BlockingClient` and
//! `MempoolClientTrait` implement a common `EsploraApi` trait. Unlike
//! approaches 1 and 2, this gives real type-level substitutability: a
//! function generic over `impl EsploraApi` accepts either concrete
//! client. It does NOT give free forwarding though -- every method still
//! needs its own impl, the same cost as approach 2's manual forwarding,
//! just organized under a trait boundary instead of inherent methods.
//! `#[deprecated]` on a trait method's signature warns at every call
//! site through any implementor, without warning at the impl sites
//! themselves -- verified separately before writing this.
//!
//! `EsploraApi` is defined in this file, so implementing it for the
//! foreign `esplora_client::BlockingClient` type is allowed under Rust's
//! orphan rule (at least one of the trait or the type must be local --
//! here, the trait is). Response types come from `mempool_types`.
//!
//! Exactly three functions, matched across all three approach files for a
//! fair comparison:
//! - one private helper (`get_json_directly`), not part of this type's
//!   public API,
//! - one legacy `get_fee_estimates`, part of the shared trait, and
//! - one new dedicated `get_recommended_fees`, outside the trait.
//!
//! `main()` shows only how a downstream user calls the last two.
//!
//! Run with: `cargo run --example approach3_trait -p mempool-client`

use std::collections::HashMap;

use esplora_client::{BlockingClient, Builder, Error};
use mempool_types::{FeeRate, RecommendedFees};

/// The shared trait -- same method name/signature for both clients.
/// Deprecated at the trait level: mempool.space is deprecating the
/// underlying endpoint, and this applies regardless of which concrete
/// client a caller is holding.
pub trait EsploraApi {
    #[deprecated(note = "mempool.space is deprecating /fee-estimates; use get_recommended_fees() instead")]
    fn get_fee_estimates(&self) -> Result<HashMap<u16, FeeRate>, Error>;
}

impl EsploraApi for BlockingClient {
    fn get_fee_estimates(&self) -> Result<HashMap<u16, FeeRate>, Error> {
        // Resolves to BlockingClient's own inherent method -- Rust always
        // prefers an inherent method over a trait method of the same
        // name, so this is not recursive.
        self.get_fee_estimates()
    }
}

pub struct MempoolClientTrait {
    inner: BlockingClient,
}

impl MempoolClientTrait {
    pub fn new(base_url: &str) -> Self {
        Self { inner: Builder::new(base_url).build_blocking() }
    }

    /// Private: same shape as approaches 1 and 2, not part of this
    /// type's public API. Only used internally, below.
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

    /// The new dedicated endpoint -- outside the shared trait entirely,
    /// using the private helper above.
    pub fn get_recommended_fees(&self) -> Result<RecommendedFees, String> {
        self.get_json_directly("/v1/fees/recommended")
    }
}

impl EsploraApi for MempoolClientTrait {
    fn get_fee_estimates(&self) -> Result<HashMap<u16, FeeRate>, Error> {
        self.inner.get_fee_estimates()
    }
}

/// Real substitutability: this function works with EITHER concrete
/// client type, unmodified, calling the same trait method we're
/// comparing across all three approaches.
fn print_fee_estimates<C: EsploraApi>(client: &C) {
    #[allow(deprecated)]
    match client.get_fee_estimates() {
        Ok(fees) => println!("-> (generic fn) get_fee_estimates = {fees:?}"),
        Err(e) => println!("-> (generic fn) get_fee_estimates failed: {e}"),
    }
}

fn main() {
    let esplora = Builder::new("https://mempool.space/api").build_blocking();
    let mempool = MempoolClientTrait::new("https://mempool.space/api");

    // How a downstream user calls the deprecated method -- through the
    // shared trait, on either concrete client.
    print_fee_estimates(&esplora);
    print_fee_estimates(&mempool);
    println!();

    // How a downstream user calls the new, dedicated replacement.
    match mempool.get_recommended_fees() {
        Ok(fees) => println!("-> get_recommended_fees = {fees:?}"),
        Err(e) => println!("-> get_recommended_fees failed: {e}"),
    }
}
