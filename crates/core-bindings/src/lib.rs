use bestsign_core::ops::config::defaults::DEFAULT_PUBKEY;
use bestsign_core::utils::get_display_data;
use bestsign_core::{
    ops::{
        config::{
            CidGen, KeyParams, LockScript, UnlockScript, UseStr, VladCid, VladConfig, VladKey,
        },
        create,
        open::config::{Config, NewLogBuilder},
        update::UpdateConfig,
        update_plog, CryptoManager,
    },
    Codec, Key, Log, Multikey, Multisig, Script,
};
use js_sys::Function;
use multibase::Base;
use multicid::{Cid, EncodedCid, EncodedVlad, Vlad};
use multihash::EncodedMultihash;
use multikey::views::Views;
use multiutil::{BaseEncoded, CodecInfo, DetectedEncoder, EncodingInfo};
use provenance_log::{LogValue, Pairs};
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
#[derive(Debug, Serialize, Deserialize)]
pub struct SignArgs {
    mk: Multikey,
    data: Vec<u8>,
}

/// Struct that will implement KeyManager
pub struct KeyHandler {
    key: Option<Multikey>,
    get_key_callback: Function,
    sign_callback: Function,
}

impl KeyHandler {
    /// Create a new KeyHandler with the given callback functions
    pub fn new(get_key: &Function, prove: &Function) -> Self {
        KeyHandler {
            get_key_callback: get_key.clone(),
            sign_callback: prove.clone(),
            key: None,
        }
    }

    /// Set the key
    pub fn set_key(&mut self, key: Multikey) {
        self.key = Some(key);
    }

    /// Get the /pubkey
    pub fn get_key(&self, key: &Key) -> Result<Multikey, bestsign_core::Error> {
        let key_args = KeyArgs {
            key: key.to_string(),
            codec: Codec::Ed25519Priv.to_string(),
            threshold: 1,
            limit: 1,
        };
        let k = self
            .get_key_callback
            .call1(
                &JsValue::NULL,
                &serde_wasm_bindgen::to_value(&key_args).unwrap(),
            )
            .map_err(|e| {
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

        let mk: Multikey = serde_wasm_bindgen::from_value(k).map_err(|e| {
            bestsign_core::Error::Generic(format!("Error converting result to Multikey: {}", e))
        })?;

        Ok(mk)
    }
}

impl CryptoManager for KeyHandler {
    /// Get Multikey using the given callback function.
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

        // if Key is "/pubkey" then set the key
        if key.to_string() == DEFAULT_PUBKEY {
            self.key = Some(mk.clone());
        }
        Ok(mk)
    }

    /// Binds the prover to the given callback function
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

    // Add a Key Value (String) to the log
    #[wasm_bindgen]
    pub fn add_string(&mut self, op: JsValue) -> Result<(), JsValue> {
        let val: UseStr = serde_wasm_bindgen::from_value(op)?;
        self.inner
            .with_use_str(val)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    /// Add data to the log, encoded as a CID
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
    pub fn create(&mut self) -> Result<JsValue, JsValue> {
        let config = self
            .inner
            .clone()
            .try_build()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let log = create(&config, &mut self.key_manager).map_err(|e| {
            tracing::error!("Error creating log: {}", e);
            JsValue::from_str(&e.to_string())
        })?;

        // serialize the log to serde_cbor and then to JsValue for return
        let log_cbor = serde_cbor::to_vec(&log).map_err(|e| {
            tracing::error!("Error serializing log to CBOR: {}", e);
            JsValue::from_str(&e.to_string())
        })?;

        // use js_sys
        let log_js = js_sys::Uint8Array::from(log_cbor.as_slice()).into();

        Ok(log_js)
    }
}

/// Loads a Plog so it can be displayed and updated in the UI
#[wasm_bindgen]
pub struct ProvenanceLog {
    config: UpdateConfig,
    log: bestsign_core::Log,
    key_manager: KeyHandler,
}

#[wasm_bindgen]
impl ProvenanceLog {
    /// Load a Plog from a serialized config
    #[wasm_bindgen(constructor)]
    pub fn new(
        log: &[u8],
        unlock: String,
        get_key: &Function,
        prove: &Function,
    ) -> Result<ProvenanceLog, JsValue> {
        // deserialize the log from CBOR
        let log: Log = serde_cbor::from_slice(log)
            .map_err(|e| JsValue::from_str(&format!("Error deserializing log: {}", e)))?;

        // start with Default Config, user can update it as desired
        let key_manager = KeyHandler::new(get_key, prove);

        // setup /pubkey with Key from DEFAULT_PUBKEY
        let pubkey_rust = key_manager
            .get_key(&Key::try_from(DEFAULT_PUBKEY).unwrap())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let config = UpdateConfig::new(Script::Code(Key::default(), unlock), pubkey_rust);

        Ok(ProvenanceLog {
            config,
            log,
            key_manager,
        })
    }

    /// Get a structured representation of the Plog for display
    #[wasm_bindgen]
    pub fn plog(&self) -> Result<JsValue, JsValue> {
        let display_data =
            get_display_data(&self.log).map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&display_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize display data: {}", e)))
    }

    // Add a Key Value (String) to the log
    #[wasm_bindgen]
    pub fn add_string(&mut self, op: JsValue) -> Result<(), JsValue> {
        let val: UseStr = serde_wasm_bindgen::from_value(op)?;
        self.config.add_op(
            val.try_into()
                .map_err(|e: bestsign_core::Error| JsValue::from_str(&e.to_string()))?,
        );
        Ok(())
    }

    /// Set the unlock script in self.config
    #[wasm_bindgen]
    pub fn set_unlock(&mut self, script: &str) {
        let script = Script::Code(Key::default(), script.to_string());
        self.config.entry_unlock_script = script;
    }

    /// Add a lock script in self.config
    #[wasm_bindgen]
    pub fn add_lock_script(&mut self, key_path: &str, script: &str) -> Result<(), JsValue> {
        let script = Script::Code(
            Key::try_from(key_path).map_err(|e| JsValue::from_str(&e.to_string()))?,
            script.to_string(),
        );
        self.config.add_lock(key_path, script);
        Ok(())
    }

    /// Update the Plog in-place with the current config
    #[wasm_bindgen]
    pub fn update(&mut self) -> Result<(), JsValue> {
        tracing::info!("Updating Plog with config");

        let config = self.config.build();

        tracing::info!("Config set");

        update_plog(&mut self.log, &config, &mut self.key_manager)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(())
    }
}
