//! APPROACH 2: Allow-list / manual forwarding, NO Deref.
//!
//! `MempoolClientAllowlist` defines every method it exposes by hand.
//! Nothing is inherited automatically, so `get_fee_estimates` simply does
//! not exist on this type at all -- a compile error at the call site
//! rather than a runtime surprise, no `#[deprecated]` shadow needed.
//! Response types come from `mempool_types`.
//!
//! Exactly three functions, matched across all three approach files for a
//! fair comparison:
//! - one private helper (`get_json_directly`), not part of this type's
//!   public API,
//! - one legacy `get_fee_estimates` -- here, deliberately absent, and
//! - one new dedicated `get_recommended_fees`.
//!
//! `main()` shows only how a downstream user calls the last two -- for
//! `get_fee_estimates` that means showing it isn't callable at all.
//!
//! Run with: `cargo run --example approach2_allowlist -p mempool-client`

use esplora_client::{BlockingClient, Builder};
use mempool_types::RecommendedFees;

pub struct MempoolClientAllowlist {
    inner: BlockingClient,
}

impl MempoolClientAllowlist {
    pub fn new(base_url: &str) -> Self {
        Self { inner: Builder::new(base_url).build_blocking() }
    }

    /// Private: same shape as approach 1, not part of this type's public
    /// API. Only used internally, below.
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

    // get_fee_estimates is deliberately NOT forwarded at all -- it simply
    // does not exist on this type. No shadow, no #[deprecated] needed.

    /// The new dedicated endpoint, built via the private helper above.
    pub fn get_recommended_fees(&self) -> Result<RecommendedFees, String> {
        self.get_json_directly("/v1/fees/recommended")
    }
}

fn main() {
    let client = MempoolClientAllowlist::new("https://mempool.space/api");

    // A downstream user CANNOT call get_fee_estimates here -- it does not
    // exist on MempoolClientAllowlist. Uncomment to see the compile error:
    //
    // let _ = client.get_fee_estimates();
    println!("-> get_fee_estimates is not callable on MempoolClientAllowlist at all\n");

    // How a downstream user calls the new, dedicated replacement.
    match client.get_recommended_fees() {
        Ok(fees) => println!("-> get_recommended_fees = {fees:?}"),
        Err(e) => println!("-> get_recommended_fees failed: {e}"),
    }
}
