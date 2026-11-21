// SPDX-License-Identifier: GPL-3.0-or-later
//! API keys in the OS credential store (the macOS Keychain; Windows Credential Manager or the
//! Linux Secret Service through the same crate).
//!
//! Ported from looking_glass's `secrets.rs` (MIT; see `crates/cruncha-acp/LICENSE-LOOKING-GLASS`).
//! Keys are stored under the service "cruncha", one account per connection
//! (`connection:<id>`). They are read only when an agent starts, and handed to it in its
//! environment. They are never written to disk by Cruncha and never sent to the web UI.

const SERVICE: &str = "cruncha";

/// Where a connection's key would come from right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum KeySource {
    /// The app's own environment has the provider's variable (it wins over the Keychain).
    Environment,
    Keychain,
    Missing,
    /// The provider needs no key (a local model server).
    NotNeeded,
}

fn entry(account: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, account).map_err(|e| e.to_string())
}

/// The stored key, if any. May show a macOS access prompt the first time a newly built
/// binary reads it.
pub fn get(account: &str) -> Option<String> {
    entry(account).ok()?.get_password().ok().filter(|k| !k.is_empty())
}

pub fn set(account: &str, value: &str) -> Result<(), String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("the key is empty".into());
    }
    entry(account)?.set_password(value).map_err(|e| e.to_string())
}

pub fn delete(account: &str) -> Result<(), String> {
    match entry(account)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Where the key for variable `var` (stored under `account`) would come from.
pub fn source(var: &str, account: &str) -> KeySource {
    if var.is_empty() {
        KeySource::NotNeeded
    } else if std::env::var_os(var).is_some_and(|v| !v.is_empty()) {
        KeySource::Environment
    } else if get(account).is_some() {
        KeySource::Keychain
    } else {
        KeySource::Missing
    }
}

/// The key for `var` to hand an agent: the app's environment first, then the Keychain.
pub fn resolve(var: &str, account: &str) -> Option<String> {
    if var.is_empty() {
        return None;
    }
    std::env::var(var).ok().filter(|v| !v.is_empty()).or_else(|| get(account))
}
