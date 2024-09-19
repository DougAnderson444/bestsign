use std::fmt::Display;

use bestsign_core::{
    ops::{
        config::{
            CidGen, KeyParams, LockScript, UnlockScript, UseStr, VladCid, VladConfig, VladKey,
        },
        create,
        open::config::{Config, ConfigBuilder},
        EntrySigner, KeyManager,
    },
    Codec, Key, Multikey, Multisig, Script,
};
use wasm_bindgen::prelude::*;

pub struct Manager;

impl KeyManager for &Manager {
    fn get_mk(
        &mut self,
        key: &Key,
        codec: Codec,
        threshold: usize,
        limit: usize,
    ) -> Result<Multikey, bestsign_core::Error> {
        unimplemented!()
    }
}

impl EntrySigner for &Manager {
    fn sign(&self, mk: &Multikey, data: &[u8]) -> Result<Multisig, bestsign_core::Error> {
        unimplemented!()
    }
}

#[wasm_bindgen]
pub struct WasmConfigBuilder {
    inner: ConfigBuilder,
    config: Option<Config>,
    // key_manage must impl both KeyManager and EntrySigner
    key_manager: Option<Manager>,
}

#[wasm_bindgen]
impl WasmConfigBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(lock: &str, unlock: &str) -> Self {
        let lock = Script::Code(Key::default(), lock.to_string());
        let unlock = Script::Code(Key::default(), unlock.to_string());

        Self {
            inner: ConfigBuilder::new(LockScript(lock), UnlockScript(unlock)),
            config: None,
            key_manager: None,
        }
    }

    // Mirror Methods of inner type
    //
    // with_entry_lock_script
    #[wasm_bindgen]
    pub fn set_entry_lock_script(&mut self, script: &str) {
        let script = Script::Code(Key::default(), script.to_string());
        self.inner.entry_lock_script = script;
    }

    // with_entry_unlock_script
    #[wasm_bindgen]
    pub fn set_entry_unlock_script(&mut self, script: &str) {
        let script = Script::Code(Key::default(), script.to_string());
        self.inner.entry_unlock_script = script;
    }

    /// Set additional Key params
    #[wasm_bindgen]
    pub fn add_key(&mut self, op: JsValue) -> Result<(), JsValue> {
        let val: KeyParams = serde_wasm_bindgen::from_value(op)?;
        self.inner.with_key_params(val);
        Ok(())
    }

    // with_use_str
    #[wasm_bindgen]
    pub fn add_string(&mut self, op: JsValue) -> Result<(), JsValue> {
        let val: UseStr = serde_wasm_bindgen::from_value(op)?;
        self.inner.with_use_str(val);
        Ok(())
    }

    /// Add a Cid to the log
    #[wasm_bindgen]
    pub fn add_cid(&mut self, op: JsValue) -> Result<(), JsValue> {
        let val: CidGen = serde_wasm_bindgen::from_value(op)?;
        self.inner.with_cid_gen(val);
        Ok(())
    }

    /// Add (VladKey, VladCid)
    #[wasm_bindgen]
    pub fn set_vlad_params(&mut self, vlad_key: JsValue, vlad_cid: JsValue) -> Result<(), JsValue> {
        let vlad_key_params: VladKey = serde_wasm_bindgen::from_value(vlad_key)?;
        let vlad_cid_params: VladCid = serde_wasm_bindgen::from_value(vlad_cid)?;
        self.inner.vlad_params = Some(VladConfig {
            key: vlad_key_params,
            cid: vlad_cid_params,
        });
        Ok(())
    }

    /// Build the Config
    #[wasm_bindgen]
    pub fn try_build(mut self) -> Result<(), JsValue> {
        let config = self
            .inner
            .try_build()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.config = Some(config.clone());

        Ok(())
    }

    /// Creates a new Plog using self.config
    #[wasm_bindgen]
    pub fn create(&self) -> Result<(), JsValue> {
        let config = self.config.as_ref().ok_or("Config not built")?;
        let mut key_manager = self.key_manager.as_ref().ok_or("KeyManager not set")?;
        let log = create(config, &mut key_manager).map_err(|e| JsValue::from_str(&e.to_string()));
        Ok(())
    }
}
