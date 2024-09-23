use bestsign_core::{
    ops::{
        config::{
            CidGen, KeyParams, LockScript, UnlockScript, UseStr, VladCid, VladConfig, VladKey,
        },
        create,
        open::config::{Config, NewLogBuilder},
        CryptoManager,
    },
    Codec, Key, Multikey, Multisig, Script,
};
use multikey::{mk, EncodedMultikey, Views as _};
use multitrait::{EncodeInto, TryDecodeFrom};
//use js_sys::Function;
use js_sys::Function;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    tracing::info!("Initialized logging in wasm bindings");
}

/// The arguments for the get_key callback
#[derive(Serialize, Deserialize)]
pub struct KeyArgs {
    key: String,
    codec: String,
    threshold: usize,
    limit: usize,
}

/// The arguments for the sign callback
#[derive(Serialize, Deserialize)]
pub struct SignArgs {
    mk: Multikey,
    data: Vec<u8>,
}

/// Struct that will implement KeyManager
#[wasm_bindgen]
pub struct KeyHandler {
    get_key_callback: Function,
    sign_callback: Function,
}

#[wasm_bindgen]
impl KeyHandler {
    #[wasm_bindgen(constructor)]
    pub fn new(get_key: &Function, prove: &Function) -> Self {
        KeyHandler {
            get_key_callback: get_key.clone(),
            sign_callback: prove.clone(),
        }
    }
}

impl CryptoManager for &KeyHandler {
    fn get_mk(
        &mut self,
        key: &Key,
        codec: Codec,
        threshold: usize,
        limit: usize,
    ) -> Result<Multikey, bestsign_core::Error> {
        // use the callback to get the key
        let this = JsValue::NULL;
        let args = KeyArgs {
            key: key.to_string(),
            codec: codec.to_string(),
            threshold,
            limit,
        };

        let args_js = serde_wasm_bindgen::to_value(&args).map_err(|e| {
            bestsign_core::Error::Generic(format!("Error converting args to JsValue: {e}"))
        })?;

        // use apply to call the callback with the args
        let result = self.get_key_callback.call1(&this, &args_js).map_err(|e| {
            tracing::error!(
                "Error calling get_key: {}",
                e.as_string()
                    .unwrap_or("No Error message found".to_string())
            );
            bestsign_core::Error::Generic(format!(
                "Error calling get_key: {}",
                e.as_string()
                    .unwrap_or("No Error message found".to_string())
            ))
        })?;

        // convert the result to a Multikey
        let mk: Multikey = serde_wasm_bindgen::from_value(result).map_err(|e| {
            bestsign_core::Error::Generic(format!("Error converting result to Multikey: {}", e))
        })?;

        Ok(mk)
    }

    fn prove(&self, mk: &Multikey, data: &[u8]) -> Result<Multisig, bestsign_core::Error> {
        // use the callback to sign the data
        let this = JsValue::NULL;

        let args = SignArgs {
            mk: mk.clone(),
            data: data.to_vec(),
        };

        let args_js = serde_wasm_bindgen::to_value(&args).map_err(|e| {
            bestsign_core::Error::Generic(format!("Error converting args to JsValue: {e}"))
        })?;

        let result = self.sign_callback.call1(&this, &args_js).map_err(|e| {
            bestsign_core::Error::Generic(format!(
                "Error calling sign: {}",
                e.as_string()
                    .unwrap_or("No Error message found".to_string())
            ))
        })?;

        let sig: Multisig = serde_wasm_bindgen::from_value(result).map_err(|e| {
            bestsign_core::Error::Generic(format!("Error converting result to Multisig: {}", e))
        })?;

        Ok(sig)
    }
}

#[wasm_bindgen]
pub struct ProvenanceLogBuilder {
    /// The inner ConfigBuilder
    inner: NewLogBuilder,
    // key_manage must impl both KeyManager and EntrySigner
    key_manager: KeyHandler,
}

#[wasm_bindgen]
impl ProvenanceLogBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(lock: &str, unlock: &str, get_key: &Function, prove: &Function) -> Self {
        let lock = Script::Code(Key::default(), lock.to_string());
        let unlock = Script::Code(Key::default(), unlock.to_string());

        Self {
            inner: NewLogBuilder::new(LockScript(lock), UnlockScript(unlock)),
            key_manager: KeyHandler::new(get_key, prove),
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
        self.inner.vlad_params = VladConfig {
            key: vlad_key_params,
            cid: vlad_cid_params,
        };
        Ok(())
    }

    /// Creates a new Plog using self.config
    #[wasm_bindgen]
    pub fn create(&self) -> Result<JsValue, JsValue> {
        let config = self
            .inner
            .clone()
            .try_build()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let mut key_manager = &self.key_manager;

        let log = create(&config, &mut key_manager).map_err(|e| {
            tracing::error!("Error creating log: {}", e);
            JsValue::from_str(&e.to_string())
        })?;

        // serialize the log to a JsValue
        let log_js =
            serde_wasm_bindgen::to_value(&log).map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(log_js)
    }
}
