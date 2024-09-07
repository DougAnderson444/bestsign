// SPDX-License-Identifier: FSL-1.1
use crate::ops::update::op_params::OpParams;
use multikey::Multikey;
use provenance_log::Script;

/// the configuration for opening a new provenance log
#[derive(Clone, Debug, Default)]
pub struct Config {
    /// clear all lock scripts?
    pub clear_lock_scripts: bool,

    /// entry lock script
    pub add_entry_lock_scripts: Vec<(String, Script)>,

    /// remove lock scripts
    pub remove_entry_lock_scripts: Vec<String>,

    /// entry unlock script
    pub entry_unlock_script: Script,

    /// entry signing key
    pub entry_signing_key: Multikey,

    /// entry operations
    pub entry_ops: Vec<OpParams>,
}

impl Config {
    /// Create a new Config with the given unlock script and entry signing key
    pub fn new(entry_unlock_script: Script, entry_signing_key: Multikey) -> Self {
        Self {
            clear_lock_scripts: false,
            add_entry_lock_scripts: Vec::new(),
            remove_entry_lock_scripts: Vec::new(),
            entry_unlock_script,
            entry_signing_key,
            entry_ops: Vec::new(),
        }
    }
}
