#[allow(warnings)]
mod bindings;

use std::fs::File;
use std::io::Write;
use std::{fs::OpenOptions, io::Read as _};

use bindings::{
    component::extension::logging,
    component::extension::peer_piper_commands,
    component::extension::types::{Error, Message},
    exports::component::extension::handlers::Guest,
};
use chrono::{DateTime, Local, TimeZone};

/// The provenance log.
use bestsign_core::{serde_cbor, utils, Base, EncodedVlad, Log, Vlad};

struct Component;

impl Guest for Component {
    /// Say hello!
    fn handle_message(msg: Message) -> Result<String, Error> {
        // topic: String, peer: String, data: Vec<u8>
        let Message { topic, peer, data } = msg;

        let phrase = format!("Hello, {peer}! You sent me: {data:?} about topic {topic:?}");

        // if the log file does not exist, create it.  if the log file exists, append the phrase to the end of the file
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("log.txt")
            .map_err(|e| Error::IoError(e.to_string()))?;

        writeln!(file, "{}", phrase).map_err(|e| Error::IoError(e.to_string()))?;

        println!("{}", phrase);

        Ok(phrase)
    }

    /// Respond to a request with the given bytes
    fn handle_request(data: Vec<u8>) -> Result<Vec<u8>, Error> {
        // Simple check to see what kind of data we are dealing with
        if let Ok(log) = serde_cbor::from_slice(&data) {
            return log_handler(&log, &data);
        }

        if let Ok(vlad) = Vlad::try_from(data.as_slice()) {
            return vlad_handler(vlad);
        }

        Err(Error::UnsupportedMessageType)
    }
}

/// handle if Vlad is requested to be provided
fn vlad_handler(vlad: Vlad) -> Result<Vec<u8>, Error> {
    // convert vlad to
    let encoded = EncodedVlad::new(Base::Base36Lower, vlad).to_string();

    // read from disk and provide the Plog saved to disk as the response
    let mut file = OpenOptions::new()
        .read(true)
        .open(&encoded)
        .map_err(|e| Error::IoError(e.to_string()))?;

    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| Error::IoError(e.to_string()))?;

    // [datetime]: Sent Plog for Vlad: encoded
    // change system time to dd/mm/yyyy hh:mm format
    let unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    // Convert Unix timestamp to a DateTime<Local>
    let local_time: DateTime<Local> = Local.timestamp_opt(unix, 0).unwrap();

    // Format the DateTime to the desired string representation
    local_time.format("%d/%m/%Y %H:%M").to_string();

    let msg = format!("[{}] Sent Plog for Vlad: {}", local_time, encoded);
    logging::log(&msg);

    Ok(data)
}

/// For [Log] types, handle the request
fn log_handler(log: &Log, data: &[u8]) -> Result<Vec<u8>, Error> {
    // TODO: Ensure the encoding is the same as decoding Base
    let display = utils::get_display_data(log).map_err(|e| Error::HandlerError(e.to_string()))?;

    // display is likely  DisplayData::ReturnValue { vlad, entries_count, kvp_data }
    if let utils::DisplayData::ReturnValue {
        vlad,
        entries_count,
        kvp_data,
    } = display
    {
        println!("Received a log with {} entries", entries_count);
        println!("Vlad details: {:?}", vlad);
        println!("KVP data: {:?}", kvp_data);

        // if vlad is verified, save the plog to disk, overwrtiting if it exists
        if vlad.verified {
            let mut file =
                File::create(&vlad.encoded).map_err(|e| Error::IoError(e.to_string()))?;

            // write the binary data
            file.write_all(data)
                .map_err(|e| Error::IoError(e.to_string()))?;

            println!("Vlad is verified and saved to disk");

            // start providing on the DHT
            // TODO: Use the Blake3 hash instead of the bytes
            peer_piper_commands::start_providing(&vlad.bytes);

            // return 1 for true
            return Ok(vec![1]);
        } else {
            println!("Vlad is not verified");
        }
    }
    // return 0 for false
    Ok(vec![0])
}

bindings::export!(Component with_types_in bindings);
