//! APPROACH 3: Shared trait.
//!
//! Both the real `esplora_client::BlockingClient` and
//! `MempoolClientTrait` implement a common `EsploraApi` trait with
//! identical method signatures. Unlike approaches 1 and 2, this gives
//! real type-level substitutability: a function generic over
//! `impl EsploraApi` accepts either concrete client. It does NOT give you
//! free forwarding though -- every method still needs its own impl, the
//! same cost as approach 2's manual forwarding, just organized under a
//! trait boundary instead of inherent methods.
//!
//! `EsploraApi` is defined in this file, so implementing it for the
//! foreign `esplora_client::BlockingClient` type is allowed under Rust's
//! orphan rule (at least one of the trait or the type must be local --
//! here, the trait is).
//!
//! Run with: `cargo run --example approach3_trait -p mempool-client`

use esplora_client::{BlockingClient, Builder, Error};
use serde::Deserialize;

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

/// The shared trait -- same method names/signatures for both clients.
pub trait EsploraApi {
    fn get_height(&self) -> Result<u32, Error>;
}

impl EsploraApi for BlockingClient {
    fn get_height(&self) -> Result<u32, Error> {
        self.get_height()
    }
}

pub struct MempoolClientTrait {
    inner: BlockingClient,
}

impl MempoolClientTrait {
    pub fn new(base_url: &str) -> Self {
        Self { inner: Builder::new(base_url).build_blocking() }
    }

    /// Genuinely new, mempool-only endpoint -- outside the shared trait
    /// entirely, using this file's own request helper.
    pub fn get_recommended_fees(&self) -> Result<RecommendedFees, String> {
        get_json_directly(self.inner.url(), "/v1/fees/recommended")
    }
}

// Same trait, same method name/signature as the real BlockingClient's
// impl above -- but note this crate's `get_height` behavior could
// diverge (e.g. hit a different endpoint or apply extra validation)
// while still satisfying the same trait signature, so generic code
// doesn't care which concrete client it's holding.
impl EsploraApi for MempoolClientTrait {
    fn get_height(&self) -> Result<u32, Error> {
        self.inner.get_height()
    }
}

/// The thing Deref and allow-list can't give you: real substitutability.
/// This function works with EITHER concrete client type, unmodified.
fn print_height<C: EsploraApi>(client: &C) {
    match client.get_height() {
        Ok(height) => println!("-> (generic fn) height = {height}"),
        Err(e) => println!("-> (generic fn) get_height failed: {e}"),
    }
}

fn main() {
    let esplora = Builder::new("https://mempool.space/api").build_blocking();
    let mempool = MempoolClientTrait::new("https://mempool.space/api");

    print_height(&esplora);
    print_height(&mempool);
    println!();

    match mempool.get_recommended_fees() {
        Ok(fees) => println!("-> recommended fees = {fees:?}"),
        Err(e) => println!("-> get_recommended_fees failed: {e}"),
    }
}
