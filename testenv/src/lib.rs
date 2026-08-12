// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shared regtest process orchestration for Esplora-compatible client tests.

use std::str::FromStr;
use std::time::{Duration, Instant};

use bitcoin::Address;
use electrsd::bitcoind::BitcoinD;
use electrsd::electrum_client::ElectrumApi;
use electrsd::ElectrsD;

pub use bitcoin;
pub use electrsd::bitcoind;

const SETUP_BLOCK_COUNT: usize = 101;
const ELECTRS_SYNC_TIMEOUT: Duration = Duration::from_secs(60);

/// Configuration for the Bitcoin Core and Electrs processes.
pub struct Config<'a> {
    /// Configuration for the Bitcoin Core node.
    pub bitcoind: bitcoind::Conf<'a>,
    /// Configuration for Electrs.
    pub electrsd: electrsd::Conf<'a>,
}

impl Default for Config<'_> {
    fn default() -> Self {
        let mut electrsd = electrsd::Conf::default();
        electrsd.http_enabled = true;

        Self {
            bitcoind: bitcoind::Conf::default(),
            electrsd,
        }
    }
}

/// A Bitcoin Core regtest node connected to an Electrs Esplora server.
pub struct TestEnv {
    bitcoind: BitcoinD,
    electrsd: ElectrsD,
    esplora_url: String,
}

impl TestEnv {
    /// Start the environment with [`Config::default`].
    pub fn new() -> Self {
        Self::new_with_config(Config::default())
    }

    /// Start the environment with custom process configuration.
    pub fn new_with_config(config: Config<'_>) -> Self {
        let bitcoind_exe = std::env::var("BITCOIND_EXE")
            .ok()
            .or_else(|| bitcoind::downloaded_exe_path().ok())
            .expect("provide BITCOIND_EXE or enable an electrsd bitcoind version feature");
        let bitcoind =
            BitcoinD::with_conf(bitcoind_exe, &config.bitcoind).expect("failed to start bitcoind");

        let electrs_exe = std::env::var("ELECTRS_EXE")
            .ok()
            .or_else(electrsd::downloaded_exe_path)
            .expect("provide ELECTRS_EXE or enable an electrsd version feature");
        let electrsd = ElectrsD::with_conf(electrs_exe, &bitcoind, &config.electrsd)
            .expect("failed to start electrs");
        let esplora_url = format!(
            "http://{}",
            electrsd
                .esplora_url
                .as_ref()
                .expect("electrs Esplora HTTP API is disabled")
        );

        let env = Self {
            bitcoind,
            electrsd,
            esplora_url,
        };
        env.mine_blocks(SETUP_BLOCK_COUNT);
        env.wait_until_electrum_sees_block(SETUP_BLOCK_COUNT);
        env
    }

    /// Return the complete HTTP URL of the Electrs Esplora API.
    pub fn esplora_url(&self) -> &str {
        &self.esplora_url
    }

    /// Return the Bitcoin Core RPC client.
    pub fn bitcoind_client(&self) -> &bitcoind::Client {
        &self.bitcoind.client
    }

    /// Mine `count` blocks to the deterministic test address.
    pub fn mine_blocks(&self, count: usize) {
        self.bitcoind
            .client
            .generate_to_address(count, &self.get_mining_address())
            .expect("failed to mine blocks");
    }

    /// Wait for Electrs to index at least `min_height` blocks.
    pub fn wait_until_electrum_sees_block(&self, min_height: usize) {
        let deadline = Instant::now() + ELECTRS_SYNC_TIMEOUT;
        let mut header = self
            .electrsd
            .client
            .block_headers_subscribe()
            .expect("failed to subscribe to Electrs block headers");

        while header.height < min_height {
            header = self.poll_exp_backoff(deadline, || {
                self.electrsd.trigger().expect("failed to trigger Electrs");
                self.electrsd.client.ping().expect("failed to ping Electrs");
                self.electrsd
                    .client
                    .block_headers_pop()
                    .expect("failed to read an Electrs block header")
            });
        }
    }

    /// Mine `count` blocks and wait for Electrs to index them.
    pub fn mine_and_wait(&self, count: usize) {
        let current_height = self
            .electrsd
            .client
            .block_headers_subscribe()
            .expect("failed to subscribe to Electrs block headers")
            .height;
        self.mine_blocks(count);
        self.wait_until_electrum_sees_block(current_height + count);
    }

    fn poll_exp_backoff<T, F>(&self, deadline: Instant, mut poll: F) -> T
    where
        F: FnMut() -> Option<T>,
    {
        let mut delay = Duration::from_millis(64);
        loop {
            if let Some(data) = poll() {
                return data;
            }
            assert!(
                Instant::now() < deadline,
                "electrs did not synchronize within {:?}",
                ELECTRS_SYNC_TIMEOUT
            );
            std::thread::sleep(delay);
            delay = delay.saturating_mul(2).min(Duration::from_millis(512));
        }
    }

    /// Return a deterministic legacy regtest address.
    pub fn get_legacy_address(&self) -> Address {
        parse_address("mvUsRD2pNeQQ8nZq8CDEx6fjVQsyzqyhVC")
    }

    /// Return a deterministic nested SegWit regtest address.
    pub fn get_nested_segwit_address(&self) -> Address {
        parse_address("2N2bJevrSwzv5C6dGm9kQAivDYnvDBPbUxM")
    }

    /// Return a deterministic Bech32 regtest address.
    pub fn get_bech32_address(&self) -> Address {
        parse_address("bcrt1qedegah48k0uft3ez7u8ywg2hf0ygexgvhps0wp")
    }

    /// Return a deterministic Bech32m regtest address.
    pub fn get_bech32m_address(&self) -> Address {
        parse_address("bcrt1p970nsjmz8ls34ty229n6zu534mumc2j74skuxe2lzcqdqxuwwhxsftk7al")
    }

    /// Return the deterministic address used for coinbase outputs.
    pub fn get_mining_address(&self) -> Address {
        parse_address("bcrt1qj5gx4t0n8lrl0clddmpn0pee4r4fds7stwyj0j")
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_address(address: &str) -> Address {
    Address::from_str(address)
        .expect("hard-coded regtest address must be valid")
        .assume_checked()
}
