// SPDX-License-Identifier: MIT OR Apache-2.0

use esplora_testenv::Config;

#[test]
fn default_config_enables_esplora_http_api() {
    assert!(Config::default().electrsd.http_enabled);
}
