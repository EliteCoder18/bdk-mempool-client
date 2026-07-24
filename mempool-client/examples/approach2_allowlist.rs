//! APPROACH 2: Allow-list / manual forwarding, NO Deref.
//!
//! `MempoolClientAllowlist` defines every method it exposes by hand, one
//! line each, forwarding to the real `esplora_client::BlockingClient`.
//! Nothing is inherited automatically -- anything not explicitly
//! forwarded simply does not exist on this type, a compile error at the
//! call site rather than a runtime surprise. The new mempool-only
//! endpoint uses this file's own request helper, same as approach 1.
//!
//! Run with: `cargo run --example approach2_allowlist -p mempool-client`

use std::str::FromStr;

use esplora_client::{BlockingClient, Builder, Error, Txid};
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

pub struct MempoolClientAllowlist {
    inner: BlockingClient,
}

impl MempoolClientAllowlist {
    pub fn new(base_url: &str) -> Self {
        Self { inner: Builder::new(base_url).build_blocking() }
    }

    // (1) Manually forwarded, one line, deliberately chosen and vetted.
    pub fn get_height(&self) -> Result<u32, Error> {
        self.inner.get_height()
    }

    pub fn get_tx_status(&self, txid: &Txid) -> Result<esplora_client::TxStatus, Error> {
        self.inner.get_tx_status(txid)
    }

    // (2) get_fee_estimates is deliberately NOT forwarded at all -- no
    // #[deprecated] shadow trick needed, it simply never exists here.
    //
    // get_block_status is ALSO deliberately not forwarded, to prove
    // exclusion works for more than just the one method we thought about.

    // (3) The replacement gets its own name, using this file's own
    // request helper, not any esplora_client internal.
    pub fn get_recommended_fees(&self) -> Result<RecommendedFees, String> {
        get_json_directly(self.inner.url(), "/v1/fees/recommended")
    }
}

fn main() {
    let client = MempoolClientAllowlist::new("https://mempool.space/api");

    match client.get_height() {
        Ok(height) => println!("-> height = {height}"),
        Err(e) => println!("-> get_height failed: {e}"),
    }

    // A syntactically valid but nonexistent txid -- good enough to
    // demonstrate the call; the server is expected to report "not found".
    let txid = Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000")
        .expect("valid txid hex");
    match client.get_tx_status(&txid) {
        Ok(status) => println!("-> tx status = {status:?}\n"),
        Err(e) => println!("-> get_tx_status failed (expected for a made-up txid): {e}\n"),
    }

    match client.get_recommended_fees() {
        Ok(fees) => println!("-> recommended fees = {fees:?}\n"),
        Err(e) => println!("-> get_recommended_fees failed: {e}\n"),
    }

    println!("(get_fee_estimates and get_block_status are not callable on MempoolClientAllowlist at all)");

    // Uncomment either line below to see the allow-list exclude things at
    // COMPILE TIME, not at runtime:
    //
    // let _ = client.get_fee_estimates();
    // let _ = client.get_block_status(&esplora_client::BlockHash::all_zeros());
}
