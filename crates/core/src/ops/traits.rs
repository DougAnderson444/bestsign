use multicodec::Codec;
use multikey::Multikey;
use multisig::Multisig;
use provenance_log::Key;

use crate::Error;

/// Users implement this trait to provide the keys for the log
pub trait CryptoManager {
    /// Get a mulitkey for the requested [Key] path.
    ///
    /// with the given parameters based on the user's preference for
    /// key generation (new random, from seed, etc.).
    fn get_mk(
        &mut self,
        key: &Key,
        codec: Codec,
        start: usize,
        end: usize,
    ) -> Result<Multikey, Error>;

    /// Generates proof for this key, such as a signature, over the data.
    fn prove(&self, mk: &Multikey, data: &[u8]) -> Result<Multisig, Error>;
}
