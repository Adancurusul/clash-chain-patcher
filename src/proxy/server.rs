//! SOCKS5 proxy server implementation

use crate::proxy::{config::ProxyConfig, relay, upstream::UpstreamProxy};
use anyhow::{Context, Result};
use fast_socks5::server::Socks5ServerProtocol;
use fast_socks5::Socks5Command;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

/// SOCKS5 proxy server
pub struct ProxyServer {
    config: ProxyConfig,
    upstream: Arc<UpstreamProxy>,
}

impl ProxyServer {
    /// Create a new proxy server
    pub fn new(config: ProxyConfig) -> Self {
        let upstream = Arc::new(UpstreamProxy::new(config.upstream.clone()));
        Self { config, upstream }
    }

    /// Start the proxy server
    ///
    /// This function will block until the server is stopped.
    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.config.listen_addr)
            .await
            .context("Failed to bind to listen address")?;

        info!(
            "Proxy server listening on {} (upstream: {}:{})",
            self.config.listen_addr, self.config.upstream.host, self.config.upstream.port
        );

        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    debug!("Accepted connection from {}", peer_addr);

                    let upstream = self.upstream.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream, peer_addr.to_string(), upstream).await
                        {
                            error!("Client error ({}): {}", peer_addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

/// Handle a single client connection
async fn handle_client(
    socket: TcpStream,
    peer_addr: String,
    upstream: Arc<UpstreamProxy>,
) -> Result<()> {
    debug!("Handling client {}", peer_addr);

    // Perform SOCKS5 handshake - accept no auth for simplicity
    // In the future, we can add authentication support
    let (proto, cmd, target_addr) = Socks5ServerProtocol::accept_no_auth(socket)
        .await
        .context("Failed to accept SOCKS5 connection")?
        .read_command()
        .await
        .context("Failed to read SOCKS5 command")?;

    // Convert TargetAddr to string for connecting
    let (target_host, target_port) = match &target_addr {
        fast_socks5::util::target_addr::TargetAddr::Ip(sock_addr) => {
            (sock_addr.ip().to_string(), sock_addr.port())
        }
        fast_socks5::util::target_addr::TargetAddr::Domain(domain, port) => {
            (domain.clone(), *port)
        }
    };

    info!(
        "Client {} requesting connection to {}:{}",
        peer_addr, target_host, target_port
    );

    // Only support TCP CONNECT for now
    match cmd {
        Socks5Command::TCPConnect => {
            // Connect to target through upstream proxy
            let upstream_stream = upstream
                .connect(&target_host, target_port)
                .await
                .context("Failed to connect through upstream")?;

            info!(
                "Connected {} -> {}:{} via upstream",
                peer_addr, target_host, target_port
            );

            // Complete the SOCKS5 handshake with success reply
            // Create a dummy SocketAddr for reply (actual bind address doesn't matter for CONNECT)
            let reply_addr = "0.0.0.0:0".parse().unwrap();
            let client_stream = proto
                .reply_success(reply_addr)
                .await
                .context("Failed to send SOCKS5 reply")?;

            // Relay traffic
            let (sent, received) = relay::relay_traffic(client_stream, upstream_stream)
                .await
                .context("Relay failed")?;

            info!(
                "Connection closed: {} -> {}:{} (sent: {}, received: {})",
                peer_addr, target_host, target_port, sent, received
            );

            Ok(())
        }
        _ => {
            warn!("Unsupported command: {:?}", cmd);
            proto
                .reply_error(&fast_socks5::ReplyError::CommandNotSupported)
                .await
                .context("Failed to send error reply")?;
            Err(anyhow::anyhow!("Command not supported"))
        }
    }
}
