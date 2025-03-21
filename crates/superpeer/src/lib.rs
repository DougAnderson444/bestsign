use std::{
    future::Future,
    pin::Pin,
    sync::{atomic::AtomicBool, Arc},
};

use bestsign_core::{
    provenance_log::{
        multicid::{self, vlad, Cid},
        Entry, Log,
    },
    resolve::{resolve_plog, Resolver},
    Null,
};
use futures::StreamExt;
use peerpiper::{
    core::events::{PublicEvent, SystemCommand},
    AllCommands, Events, Libp2pEvent, PeerPiper, ReturnValues,
};
use peerpiper_native::NativeBlockstoreBuilder;
use peerpiper_server::web_server;
use tokio::sync::Mutex;

// Whether the web server has started or not.
static WEB_SERVER_STARTED: AtomicBool = AtomicBool::new(false);

/// Use PeerPiper to create a SuperPeer.
///
/// Since Resolve uses PeerPiper Get, the data will be pulled into the
/// blockstore available for other peers to fetch.
#[derive(Clone, Default)]
pub struct SuperPeer {
    peerpiper: Arc<Mutex<Option<PeerPiper>>>,
}

impl SuperPeer {
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let blockstore = NativeBlockstoreBuilder::default().open().await.unwrap();

        let peerpiper = PeerPiper::new(blockstore, Default::default());
        let mut rx_evts = peerpiper.events().await?;
        self.peerpiper.lock().await.replace(peerpiper);

        loop {
            tokio::select! {
                event = rx_evts.select_next_some() => {
                    if let Err(e) = self.handle_event(event).await {
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

    async fn handle_event(&self, event: Events) -> Result<(), Box<dyn std::error::Error>> {
        println!("Received event: {:?}", event);
        match event {
            Events::Outer(PublicEvent::ListenAddr { address, .. }) => {
                tracing::debug!("Received Node Address: {:?}", address);
                if !WEB_SERVER_STARTED.load(std::sync::atomic::Ordering::Relaxed) {
                    tokio::spawn(web_server::serve(address.clone()));
                    WEB_SERVER_STARTED.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            }
            Events::Inner(Libp2pEvent::PutRecordRequest { source: _, record }) => {
                // Validate the record is a Vlad with a valid Plog.
                // The record key:value will be vlad:HeadEntryCid
                // To validate, we fetch all the Plog data by the head Cid
                // using peerpiper and its bitswap client.
                // Once we have all the Plog Cid data, we can reconstruct the Plog.
                let vlad = vlad::Vlad::try_from(record.key.to_vec().as_slice())?;
                let head = Cid::try_from(record.value.as_slice())?;

                let _resolved_plog = resolve_plog(&vlad, &head, self.clone()).await?;

                // If we made it this far, it means we have a valid Plog and we should Put the
                // Record.
                let put_record = AllCommands::PutRecord {
                    key: vlad.into(),
                    value: head.into(),
                };
                self.peerpiper
                    .lock()
                    .await
                    .as_ref()
                    .unwrap()
                    .order(put_record)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Resolver for SuperPeer {
    type Error = TestError;

    fn resolve(
        &self,
        cid: &multicid::Cid,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Self::Error>> + Send>> {
        let command = AllCommands::System(SystemCommand::Get {
            key: cid.clone().into(),
        });
        let peerpiper = self.peerpiper.clone();
        Box::pin(async move {
            let ReturnValues::Data(data) = peerpiper
                .lock()
                .await
                .as_ref()
                .unwrap()
                .order(command)
                .await?
            else {
                return Err(TestError::PerrPiper(peerpiper::Error::NotConnected));
            };
            Ok(data)
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TestError {
    #[error("PeerPiper error: {0}")]
    PerrPiper(#[from] peerpiper::Error),
}
