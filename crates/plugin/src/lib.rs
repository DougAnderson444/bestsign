#[allow(warnings)]
mod bindings;

use std::fs::OpenOptions;
use std::io::Write;

use bindings::{
    component::extension::types::{Error, Message},
    exports::component::extension::handlers::Guest,
};

/// The provenance log.
use bestsign_core::{serde_cbor, utils, Log, Vlad};

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
        // Simple check to see if the bytes deserialize to a Plog (or not)
        let maybe_log: Result<Log, _> = serde_cbor::from_slice(&data);

        if let Ok(log) = maybe_log {
            let display =
                utils::get_display_data(&log).map_err(|e| Error::HandlerError(e.to_string()))?;

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

                // if vlad is verified, save the plog to disk
                if vlad.verified {
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(&vlad.encoded)
                        .map_err(|e| Error::IoError(e.to_string()))?;

                    writeln!(file, "{:?}", data).map_err(|e| Error::IoError(e.to_string()))?;

                    println!("Vlad is verified and saved to disk");
                    // return 1 for true
                    return Ok(vec![1]);
                } else {
                    println!("Vlad is not verified");
                }
            }
            // return 0 for false
            Ok(vec![0])
        } else {
            println!("Received some data that is not a log");
            // return 0 for false
            Ok(vec![0])
        }
    }
}

bindings::export!(Component with_types_in bindings);
