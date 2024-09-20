//! A Basic wallet that provides get_mk and sign functions.
//!
//! Pass in [Credentials] such as `{"username":"username","password":"password","encrypted_seed":null}` to create a wallet,
//!
//! Or use a seed {"username":"username","password":"password","encrypted_seed":[46,236,62,136,201,70,17,15,212,216,99,70,0,242,150,190,15,58,71,131,148,196,18,158,104,110,121,170,241,22,47,63,211,192,118,233,214,196,223,34]}

use std::collections::HashMap;

use bestsign_core::ops::config::defaults::{DEFAULT_ENTRYKEY, DEFAULT_VLAD_KEY};
use bestsign_core::{Codec, Key, Multikey};
use multikey::{mk, EncodedMultikey, Views as _};
use seed_keeper_core::credentials::{Credentials, Wallet};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// The arguments for the get_key callback
#[derive(Serialize, Deserialize)]
pub struct KeyArgs {
    key: String,
    codec: String,
    threshold: usize,
    limit: usize,
}

/// Holds the wallet and keys
#[wasm_bindgen]
pub struct WasmWallet {
    /// The seed-keeper wallet
    wallet: Wallet,

    /// A map of encoded public keys to their corresponding Multikey and Key
    keys: HashMap<String, (Multikey, Key)>,
}

/// Helper fn which does `|e| JsValue::from_str(&e.to_string())`
fn into_js_val<T>(e: T) -> JsValue
where
    T: std::fmt::Display,
{
    JsValue::from_str(&e.to_string())
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    tracing::info!("Initialized logging");
}

#[wasm_bindgen]
impl WasmWallet {
    #[wasm_bindgen(constructor)]
    pub fn new(credentials: JsValue) -> Result<WasmWallet, JsValue> {
        let credentials: Credentials =
            serde_wasm_bindgen::from_value(credentials).map_err(into_js_val)?;
        let wallet = Wallet::new(credentials).map_err(into_js_val)?;
        let keys = HashMap::new();
        Ok(WasmWallet { wallet, keys })
    }

    pub fn get_mk(&mut self, args: JsValue) -> Result<JsValue, JsValue> {
        // deserialize and destructure the args
        let KeyArgs {
            key,
            codec,
            threshold,
            limit,
        } = serde_wasm_bindgen::from_value(args).map_err(into_js_val)?;

        let codec = Codec::try_from(codec.as_str()).map_err(into_js_val)?;
        // if key is DEFAULT_ENTRYKEY or DEFAULT_VLAD_KEY, generate random key.
        // Otherwise, use the key from seed.
        let mk = match key.as_str() {
            DEFAULT_ENTRYKEY | DEFAULT_VLAD_KEY => {
                let mut rng = rand::thread_rng();
                let mk = mk::Builder::new_from_random_bytes(codec, &mut rng)
                    .map_err(into_js_val)?
                    .try_build()
                    .map_err(into_js_val)?;

                Ok::<Multikey, JsValue>(mk)
            }
            _ => {
                // get seed from wallet.seed
                let seed = self.wallet.seed();

                let mk = mk::Builder::new_from_seed(codec, seed)
                    .map_err(into_js_val)?
                    .try_build()
                    .map_err(into_js_val)?;

                Ok(mk)
            }
        }?;

        // save the key in the HashMap
        let key = Key::try_from(key).map_err(into_js_val)?;
        let pk = mk
            .conv_view()
            .map_err(into_js_val)?
            .to_public_key()
            .map_err(into_js_val)?;

        let epk = EncodedMultikey::from(pk);
        self.keys.insert(epk.to_string(), (mk.clone(), key));

        // return the epk string as JsValue
        Ok(JsValue::from_str(&epk.to_string()))
    }

    /// Genertaes Proof, such as Signature, over the data with the Multikey that corresponds to the given key
    pub fn prove(&mut self, encoded_pubkey: JsValue, data: Vec<u8>) -> Result<JsValue, JsValue> {
        let encoded_pubkey: String = encoded_pubkey
            .as_string()
            .ok_or(JsError::new("Invalid encoded public key"))?;
        let (mk, key) = self.keys.get(&encoded_pubkey).ok_or(JsError::new(&format!(
            "Key not found: {:?}",
            encoded_pubkey
        )))?;

        let signature = mk
            .sign_view()
            .map_err(into_js_val)?
            .sign(&data, true, None)
            .map_err(into_js_val)?;

        // remove the key if it is DEFAULT_ENTRYKEY or DEFAULT_VLAD_KEY
        if key.to_string() == DEFAULT_ENTRYKEY || key.to_string() == DEFAULT_VLAD_KEY {
            self.keys.remove(&encoded_pubkey);
        }

        let sig_js = serde_wasm_bindgen::to_value(&signature).map_err(into_js_val)?;
        Ok(sig_js)
    }
}
