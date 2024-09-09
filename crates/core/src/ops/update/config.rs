// SPDX-License-Identifier: FSL-1.1
use crate::ops::update::op_params::OpParams;
use multikey::Multikey;
use provenance_log::Script;

/// the configuration for opening a new provenance log
#[derive(Clone, Debug, Default)]
pub struct Config {
    /// clear all lock scripts?
    pub clear_lock_scripts: bool,

    /// The set of lock scripts define the conditions which must be met by the next entry in the plog for it to be valid.
    pub add_entry_lock_scripts: Vec<(String, Script)>,

    /// remove lock scripts
    pub remove_entry_lock_scripts: Vec<String>,

    /// Unlock script which solves one of the previous entry lock scripts
    pub entry_unlock_script: Script,

    /// The signing key that matches a current Plog pubkey
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

    /// Add an Entry lock script to the configuration
    pub fn add_lock(mut self, key_path: impl AsRef<str>, script: Script) -> Self {
        self.add_entry_lock_scripts
            .push((key_path.as_ref().to_owned(), script));
        self
    }

    /// Remove an Entry lock script from the configuration
    pub fn remove_lock(mut self, key_path: impl AsRef<str>) -> Self {
        self.remove_entry_lock_scripts
            .push(key_path.as_ref().to_owned());
        self
    }

    /// Add an entry operation to the configuration
    pub fn add_op(mut self, op: OpParams) -> Self {
        self.entry_ops.push(op);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::update::op_params::OpParams;
    use multicodec::Codec;
    use multikey::mk;
    use provenance_log::{Key, Script};

    #[test]
    fn test_config() -> Result<(), Box<dyn std::error::Error>> {
        let script = Script::Code(Key::default(), "script".to_string());
        let mut rng = rand::rngs::OsRng;
        let mk = mk::Builder::new_from_random_bytes(Codec::Ed25519Priv, &mut rng)?.try_build()?;
        let op = OpParams::UseStr {
            key: Key::default(),
            s: "test".to_string(),
        };

        let config = Config::new(script.clone(), mk.clone())
            .add_op(op.clone())
            .add_lock("test".to_string(), script.clone())
            .remove_lock("test".to_string());

        assert_eq!(config.entry_unlock_script, script);
        assert_eq!(config.entry_signing_key, mk);
        assert_eq!(config.entry_ops, vec![op]);
        assert_eq!(
            config.add_entry_lock_scripts,
            vec![("test".to_string(), script.clone())]
        );
        assert_eq!(config.remove_entry_lock_scripts, vec!["test".to_string()]);

        Ok(())
    }
}
