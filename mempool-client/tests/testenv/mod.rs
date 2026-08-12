// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(unused)]

use std::collections::HashMap;
use std::ops::Deref;

use mempool_client::{AsyncClient, BlockingClient, Builder};

pub(crate) use esplora_testenv::Config as EnvConfig;

pub(crate) struct TestEnv {
    inner: esplora_testenv::TestEnv,
}

impl Deref for TestEnv {
    type Target = esplora_testenv::TestEnv;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TestEnv {
    pub(crate) fn new() -> Self {
        Self::new_with_config(EnvConfig::default())
    }

    pub(crate) fn new_with_config(config: EnvConfig<'_>) -> Self {
        Self {
            inner: esplora_testenv::TestEnv::new_with_config(config),
        }
    }

    pub(crate) fn setup_clients(&self) -> (BlockingClient, AsyncClient) {
        let builder = Builder::new(self.inner.esplora_url());
        build_clients(builder)
    }

    pub(crate) fn setup_clients_with_headers(
        &self,
        url: &str,
        headers: HashMap<String, String>,
    ) -> (BlockingClient, AsyncClient) {
        let mut builder = Builder::new(&format!("http://{url}"));
        for (key, value) in &headers {
            builder = builder.header(key, value);
        }
        build_clients(builder)
    }
}

fn build_clients(builder: Builder) -> (BlockingClient, AsyncClient) {
    let blocking_client = builder
        .clone()
        .header("User-Agent", "blocking")
        .build_blocking();
    let async_client = builder.header("User-Agent", "async").build_async().unwrap();
    (blocking_client, async_client)
}
