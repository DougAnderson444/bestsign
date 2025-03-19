use peerpiper_server::web_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("bestsign_superpeer=debug,peerpiper_native=debug,peerpiper_core=debug,libp2p_webrtc=info,libp2p_ping=debug")
        .try_init();

    tracing::info!("Starting bestsign_superpeer BINARY");

    superpeer::run().await?;

    Ok(())
}
