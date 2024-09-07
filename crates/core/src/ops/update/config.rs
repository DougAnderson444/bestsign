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

    /// Add an entry operation to the configuration
    pub fn with_entry_op(mut self, op: OpParams) -> Self {
        self.entry_ops.push(op);
        self
    }

    /// Add an Entry lock script to the configuration
    pub fn with_lock_script(mut self, name: String, script: Script) -> Self {
        self.add_entry_lock_scripts.push((name, script));
        self
    }

    /// Remove an Entry lock script from the configuration
    pub fn remove_lock_script(mut self, name: String) -> Self {
        self.remove_entry_lock_scripts.push(name);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::update::op_params::OpParams;
    use multicodec::Codec;
    use multikey::{mk, Multikey};
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
            .with_entry_op(op.clone())
            .with_lock_script("test".to_string(), script.clone())
            .remove_lock_script("test".to_string());

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
