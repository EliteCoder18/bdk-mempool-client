# Shared Test Environment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract regtest process orchestration into one dev-only crate consumable by both Esplora HTTP clients without coupling it to either client implementation.

**Architecture:** A private workspace crate, `esplora-testenv`, owns Bitcoin Core and Electrs lifecycle, synchronization, mining, and deterministic addresses. Each client test directory keeps a small adapter that builds that client's blocking and asynchronous clients while delegating environment operations to the shared crate.

**Tech Stack:** Rust 2021, Cargo workspace, `bitcoin` 0.32, `electrsd` 0.40, Tokio integration tests.

## Global Constraints

- Keep all resulting Git changes unstaged.
- Preserve the existing Rust 1.75.0 minimum supported version.
- Do not make the shared crate depend on `esplora-client` or `mempool-client`.
- Preserve existing Esplora integration-test call sites.

---

### Task 1: Define the shared boundary through client tests

**Files:**
- Modify: `Cargo.toml`
- Modify: `esplora-client/Cargo.toml`
- Modify: `esplora-client/tests/testenv/mod.rs`
- Modify: `esplora-client/tests/transaction.rs`
- Modify: `mempool-client/Cargo.toml`
- Create: `mempool-client/tests/testenv/mod.rs`

**Interfaces:**
- Consumes: `esplora_testenv::{Config, TestEnv}`.
- Produces: client-local `TestEnv::setup_clients` and `TestEnv::setup_clients_with_headers` methods with the existing signatures.

- [ ] Replace the Esplora test helper implementation with a thin adapter that delegates environment methods.
- [ ] Move the transaction error-propagation test out of the helper module so it runs once per suite.
- [ ] Add the equivalent Mempool client adapter.
- [ ] Run `cargo check -p esplora-client --tests --no-default-features --features async,blocking,tokio` and confirm it fails because `esplora-testenv` has not been implemented.

### Task 2: Implement the client-neutral environment crate

**Files:**
- Create: `testenv/Cargo.toml`
- Create: `testenv/src/lib.rs`
- Create: `testenv/tests/config.rs`

**Interfaces:**
- Produces: `Config<'a>`, `TestEnv::new`, `TestEnv::new_with_config`, `TestEnv::esplora_url`, `TestEnv::bitcoind_client`, mining and synchronization methods, and deterministic address accessors.
- Consumes: `electrsd::{BitcoinD, ElectrsD}` and `bitcoin::Address`.

- [ ] Add a contract test asserting that the default configuration enables Electrs' HTTP API.
- [ ] Implement process setup and environment operations by extracting the existing client-neutral code.
- [ ] Run `cargo test -p esplora-testenv --test config` and confirm the contract test passes without launching external processes.

### Task 3: Verify the migration

**Files:**
- Modify mechanically if required: `Cargo-minimal.lock`
- Modify mechanically if required: `Cargo-recent.lock`

**Interfaces:**
- Consumes: both client adapters and the shared crate.
- Produces: a formatted workspace whose test targets compile under the relevant feature set.

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo check -p esplora-client --tests --no-default-features --features async,blocking,tokio`.
- [ ] Run `cargo check -p mempool-client --tests --no-default-features --features async,blocking,tokio`.
- [ ] Run the shared crate's focused test and inspect `git diff --check`.
- [ ] Confirm `git diff --cached` is empty and report any full integration tests that were not run.
