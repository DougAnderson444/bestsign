use std::{future::Future, pin::Pin};

use bestsign_core::{Entry, Null};
use futures::StreamExt;
use multicid::{vlad, Cid};
use peerpiper::{
    core::events::{PublicEvent, SystemCommand},
    AllCommands, Events, Libp2pEvent, PeerPiper, ReturnValues,
};
use peerpiper_native::NativeBlockstoreBuilder;
use peerpiper_server::web_server;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let blockstore = NativeBlockstoreBuilder::default().open().await.unwrap();

    let (peerpiper, ready) = PeerPiper::new(blockstore, Default::default());

    let mut rx_evts = ready.await?;

    loop {
        tokio::select! {
            event = rx_evts.select_next_some() => {
                if let Err(e) = handle_event(event, &peerpiper).await {
                    tracing::error!(%e, "Error handling event");
                }
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Received ctrl-c");
                break;
            }
        }
    }
    Ok(())
}

async fn handle_event(
    event: Events,
    peerpiper: &PeerPiper,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Received event: {:?}", event);
    match event {
        Events::Outer(PublicEvent::ListenAddr { address, .. }) => {
            tracing::debug!("Received Node Address: {:?}", address);
            tokio::spawn(web_server::serve(address.clone()));
        }
        Events::Inner(Libp2pEvent::PutRecordRequest { source, record }) => {
            // Validate the record is a Vlad with a valid Plog.
            // The record key:value will be vlad:Cid
            // To validate, we fetch all the Plog data by the head Cid
            // using the peerpiper variable and it bitswap client.
            // Once we have all the Plog Cid data, we can reconstruct the Plog.
            let head_cid = Cid::try_from(record.value.as_slice()).unwrap();

            let command = AllCommands::System(SystemCommand::Get {
                key: head_cid.into(),
            });
            if let ReturnValues::Data(head_bytes) = peerpiper.order(command).await? {
                // Now that we have the entry, we can get the previous entry from it prev field
                // and continue until we have all the entries.

                // try to convert bytes to Entry
                let entry = Entry::try_from(head_bytes.as_slice())?;

                // and so on...until prev is Cid::null
            }
        }
        _ => {}
    }
    Ok(())
}
