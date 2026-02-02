//! Traffic relay between client and upstream

use anyhow::Result;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::{debug, warn};

/// Relay traffic bidirectionally between client and upstream
///
/// # Arguments
/// * `client` - Client stream (implements AsyncRead + AsyncWrite)
/// * `upstream` - Upstream stream (implements AsyncRead + AsyncWrite)
///
/// # Returns
/// * `Ok((bytes_sent, bytes_received))` - Total bytes transferred in each direction
/// * `Err` - Relay failed
pub async fn relay_traffic<C, U>(client: C, upstream: U) -> Result<(u64, u64)>
where
    C: AsyncRead + AsyncWrite + Unpin,
    U: AsyncRead + AsyncWrite + Unpin,
{
    let (mut client_read, mut client_write) = tokio::io::split(client);
    let (mut upstream_read, mut upstream_write) = tokio::io::split(upstream);

    // Client to upstream
    let client_to_upstream = async {
        let mut buf = vec![0u8; 8192];
        let mut total = 0u64;

        loop {
            match client_read.read(&mut buf).await {
                Ok(0) => {
                    // Client closed connection
                    debug!("Client closed connection (sent {} bytes)", total);
                    break;
                }
                Ok(n) => {
                    if let Err(e) = upstream_write.write_all(&buf[..n]).await {
                        warn!("Failed to write to upstream: {}", e);
                        return Err(e);
                    }
                    total += n as u64;
                }
                Err(e) => {
                    warn!("Failed to read from client: {}", e);
                    return Err(e);
                }
            }
        }

        // Shutdown write half
        let _ = upstream_write.shutdown().await;
        Ok(total)
    };

    // Upstream to client
    let upstream_to_client = async {
        let mut buf = vec![0u8; 8192];
        let mut total = 0u64;

        loop {
            match upstream_read.read(&mut buf).await {
                Ok(0) => {
                    // Upstream closed connection
                    debug!("Upstream closed connection (received {} bytes)", total);
                    break;
                }
                Ok(n) => {
                    if let Err(e) = client_write.write_all(&buf[..n]).await {
                        warn!("Failed to write to client: {}", e);
                        return Err(e);
                    }
                    total += n as u64;
                }
                Err(e) => {
                    warn!("Failed to read from upstream: {}", e);
                    return Err(e);
                }
            }
        }

        // Shutdown write half
        let _ = client_write.shutdown().await;
        Ok(total)
    };

    // Run both directions concurrently
    match tokio::try_join!(client_to_upstream, upstream_to_client) {
        Ok((sent, received)) => {
            debug!("Relay completed: sent={}, received={}", sent, received);
            Ok((sent, received))
        }
        Err(e) => {
            // One direction failed, but that's normal when connection closes
            debug!("Relay ended: {}", e);
            // Return 0 for both directions on error
            Ok((0, 0))
        }
    }
}

/// Alternative implementation using tokio::io::copy_bidirectional
///
/// This is simpler but provides less control over the relay process.
#[allow(dead_code)]
pub async fn relay_traffic_simple<C, U>(mut client: C, mut upstream: U) -> Result<(u64, u64)>
where
    C: AsyncRead + AsyncWrite + Unpin,
    U: AsyncRead + AsyncWrite + Unpin,
{
    match tokio::io::copy_bidirectional(&mut client, &mut upstream).await {
        Ok((sent, received)) => {
            debug!("Relay completed: sent={}, received={}", sent, received);
            Ok((sent, received))
        }
        Err(e) => {
            warn!("Relay error: {}", e);
            Err(e.into())
        }
    }
}
