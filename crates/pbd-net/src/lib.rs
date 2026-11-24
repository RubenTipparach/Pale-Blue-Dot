//! The transport between a host and its players (`multiplayer` design 6
//! and 20, task 6.1): QUIC, on its own thread with its own async runtime.
//!
//! The game and a dedicated server never wait on a socket. They hand
//! messages in and take them out through channels each frame:
//! - [`Host::bind`] listens; each connection is numbered, its messages come
//!   out as [`HostEvent`]s and go in with [`Host::send`].
//! - [`Client::connect`] connects; [`Client::poll`] and [`Client::send`].
//!
//! On the wire: one reliable, ordered stream each way, the client's to
//! open, each message a `u32` little-endian length and then the protocol's
//! bytes (`pbd_core::net::protocol`). The certificate is self-signed per run
//! and a client accepts any (design 20): the join check, not TLS, is what
//! refuses another world.

use pbd_core::net::protocol::{Decode, ToClient, ToServer};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc as tmpsc;

/// The largest message either side accepts, in bytes: a snapshot's bound
/// and a little.
pub const MAX_FRAME: usize = pbd_core::net::protocol::MAX_SNAPSHOT + 64;

/// What happened on a host's connections.
#[derive(Debug, PartialEq)]
pub enum HostEvent {
    /// A connection opened its stream; its first message follows.
    Opened(u64),
    /// A message from a connection.
    Message(u64, ToServer),
    /// Bytes that are not a message this build reads: the connection is
    /// closed.
    Unreadable(u64, Decode),
    /// The connection closed.
    Closed(u64),
}

/// A listening host.
pub struct Host {
    /// Behind a lock so a `Host` can be shared (a game resource is).
    events: Mutex<mpsc::Receiver<HostEvent>>,
    out: tmpsc::UnboundedSender<(u64, Vec<u8>)>,
    close: tmpsc::UnboundedSender<u64>,
    addr: SocketAddr,
    _thread: std::thread::JoinHandle<()>,
}

impl Host {
    /// Listen on `port` on every interface (0: any free port, for tests).
    pub fn bind(port: u16) -> std::io::Result<Host> {
        Self::bind_to(SocketAddr::from(([0, 0, 0, 0], port)))
    }

    /// Listen on `addr`.
    pub fn bind_to(addr: SocketAddr) -> std::io::Result<Host> {
        let (events_tx, events) = mpsc::channel();
        let (out, mut out_rx) = tmpsc::unbounded_channel::<(u64, Vec<u8>)>();
        let (close, mut close_rx) = tmpsc::unbounded_channel::<u64>();
        let (ready_tx, ready_rx) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("pbd_net_host".into())
            .spawn(move || {
                let runtime = match runtime() {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                runtime.block_on(async move {
                    let endpoint = match server_endpoint(addr) {
                        Ok(endpoint) => endpoint,
                        Err(e) => {
                            let _ = ready_tx.send(Err(e));
                            return;
                        }
                    };
                    let _ = ready_tx.send(endpoint.local_addr());
                    let writers: Arc<Mutex<HashMap<u64, tmpsc::UnboundedSender<Vec<u8>>>>> =
                        Arc::default();
                    let connections: Arc<Mutex<HashMap<u64, quinn::Connection>>> = Arc::default();
                    // Outgoing messages to their connection's writer.
                    let routed = writers.clone();
                    tokio::spawn(async move {
                        while let Some((to, bytes)) = out_rx.recv().await {
                            if let Some(w) = routed.lock().unwrap().get(&to) {
                                let _ = w.send(bytes);
                            }
                        }
                    });
                    let closing = connections.clone();
                    tokio::spawn(async move {
                        while let Some(id) = close_rx.recv().await {
                            if let Some(c) = closing.lock().unwrap().remove(&id) {
                                c.close(0u32.into(), b"closed");
                            }
                        }
                    });
                    let mut next = 0u64;
                    while let Some(incoming) = endpoint.accept().await {
                        let id = next;
                        next += 1;
                        let events = events_tx.clone();
                        let writers = writers.clone();
                        let connections = connections.clone();
                        tokio::spawn(async move {
                            let Ok(connection) = incoming.await else {
                                return;
                            };
                            connections.lock().unwrap().insert(id, connection.clone());
                            let Ok((send, recv)) = connection.accept_bi().await else {
                                let _ = events.send(HostEvent::Closed(id));
                                return;
                            };
                            let _ = events.send(HostEvent::Opened(id));
                            let (tx, rx) = tmpsc::unbounded_channel();
                            writers.lock().unwrap().insert(id, tx);
                            tokio::spawn(write_frames(send, rx));
                            let reason =
                                read_frames(recv, |bytes| match ToServer::decode(&bytes) {
                                    Ok(m) => {
                                        let _ = events.send(HostEvent::Message(id, m));
                                        Ok(())
                                    }
                                    Err(e) => {
                                        let _ = events.send(HostEvent::Unreadable(id, e));
                                        Err(())
                                    }
                                })
                                .await;
                            if reason.is_err() {
                                connection.close(1u32.into(), b"unreadable");
                            }
                            writers.lock().unwrap().remove(&id);
                            connections.lock().unwrap().remove(&id);
                            let _ = events.send(HostEvent::Closed(id));
                        });
                    }
                });
            })?;
        let addr = ready_rx
            .recv()
            .map_err(|_| std::io::Error::other("the network thread stopped"))??;
        Ok(Host {
            events: Mutex::new(events),
            out,
            close,
            addr,
            _thread: thread,
        })
    }

    /// The address it listens on.
    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }

    /// Everything that happened since the last poll, in order.
    pub fn poll(&self) -> Vec<HostEvent> {
        self.events
            .lock()
            .map(|e| e.try_iter().collect())
            .unwrap_or_default()
    }

    /// Send one message to a connection.
    pub fn send(&self, connection: u64, message: &ToClient) {
        let _ = self.out.send((connection, message.encode()));
    }

    /// Close a connection (after a refused join's message has gone).
    pub fn close(&self, connection: u64) {
        let _ = self.close.send(connection);
    }
}

/// A connected client.
pub struct Client {
    /// Behind a lock so a `Client` can be shared (a game resource is).
    inbox: Mutex<mpsc::Receiver<Result<ToClient, Decode>>>,
    out: tmpsc::UnboundedSender<Vec<u8>>,
    closed: Arc<std::sync::atomic::AtomicBool>,
    _thread: std::thread::JoinHandle<()>,
}

impl Client {
    /// Connect to a host, waiting at most `timeout` for it to answer.
    pub fn connect(addr: SocketAddr, timeout: std::time::Duration) -> std::io::Result<Client> {
        let (inbox_tx, inbox) = mpsc::channel();
        let (out, out_rx) = tmpsc::unbounded_channel::<Vec<u8>>();
        let (ready_tx, ready_rx) = mpsc::channel::<std::io::Result<()>>();
        let closed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = closed.clone();
        let thread = std::thread::Builder::new()
            .name("pbd_net_client".into())
            .spawn(move || {
                let runtime = match runtime() {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                runtime.block_on(async move {
                    let opened = tokio::time::timeout(timeout, open(addr)).await;
                    let (endpoint, connection, send, recv) = match opened {
                        Ok(Ok(parts)) => parts,
                        Ok(Err(e)) => {
                            let _ = ready_tx.send(Err(e));
                            return;
                        }
                        Err(_) => {
                            let _ = ready_tx.send(Err(std::io::Error::new(
                                std::io::ErrorKind::TimedOut,
                                "the host did not answer",
                            )));
                            return;
                        }
                    };
                    let _ = ready_tx.send(Ok(()));
                    tokio::spawn(write_frames(send, out_rx));
                    let _ = read_frames(recv, |bytes| {
                        let message = ToClient::decode(&bytes);
                        let bad = message.is_err();
                        let _ = inbox_tx.send(message);
                        if bad { Err(()) } else { Ok(()) }
                    })
                    .await;
                    flag.store(true, std::sync::atomic::Ordering::Release);
                    connection.close(0u32.into(), b"done");
                    endpoint.wait_idle().await;
                });
            })?;
        ready_rx
            .recv()
            .map_err(|_| std::io::Error::other("the network thread stopped"))??;
        Ok(Client {
            inbox: Mutex::new(inbox),
            out,
            closed,
            _thread: thread,
        })
    }

    /// Every message since the last poll, in order. An `Err` is bytes this
    /// build cannot read; the connection closes after it.
    pub fn poll(&self) -> Vec<Result<ToClient, Decode>> {
        self.inbox
            .lock()
            .map(|i| i.try_iter().collect())
            .unwrap_or_default()
    }

    /// Send one message.
    pub fn send(&self, message: &ToServer) {
        let _ = self.out.send(message.encode());
    }

    /// Whether the host has closed the connection.
    pub fn is_closed(&self) -> bool {
        self.closed.load(std::sync::atomic::Ordering::Acquire)
    }
}

/// Both ends ping every few seconds, so a connection stays open through a
/// quiet spell (a joining game building its world takes most of a minute
/// on a slow machine), and one that stops answering is closed in a minute.
fn transport() -> Arc<quinn::TransportConfig> {
    let mut t = quinn::TransportConfig::default();
    t.keep_alive_interval(Some(std::time::Duration::from_secs(4)));
    t.max_idle_timeout(Some(
        quinn::IdleTimeout::try_from(std::time::Duration::from_secs(60)).expect("a minute fits"),
    ));
    Arc::new(t)
}

fn runtime() -> std::io::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
}

fn server_endpoint(addr: SocketAddr) -> std::io::Result<quinn::Endpoint> {
    let cert = rcgen::generate_simple_self_signed(vec!["pale-blue-dot".into()])
        .map_err(std::io::Error::other)?;
    let der = rustls::pki_types::CertificateDer::from(cert.cert);
    let key = rustls::pki_types::PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
    let mut config = quinn::ServerConfig::with_single_cert(vec![der], key.into())
        .map_err(std::io::Error::other)?;
    config.transport_config(transport());
    quinn::Endpoint::server(config, addr)
}

async fn open(
    addr: SocketAddr,
) -> std::io::Result<(
    quinn::Endpoint,
    quinn::Connection,
    quinn::SendStream,
    quinn::RecvStream,
)> {
    let bind: SocketAddr = if addr.is_ipv6() {
        "[::]:0".parse().expect("an address")
    } else {
        "0.0.0.0:0".parse().expect("an address")
    };
    let mut endpoint = quinn::Endpoint::client(bind)?;
    let crypto = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_protocol_versions(&[&rustls::version::TLS13])
    .map_err(std::io::Error::other)?
    .dangerous()
    .with_custom_certificate_verifier(Arc::new(AnyCertificate))
    .with_no_client_auth();
    let quic =
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto).map_err(std::io::Error::other)?;
    let mut client = quinn::ClientConfig::new(Arc::new(quic));
    client.transport_config(transport());
    endpoint.set_default_client_config(client);
    let connection = endpoint
        .connect(addr, "pale-blue-dot")
        .map_err(std::io::Error::other)?
        .await
        .map_err(std::io::Error::other)?;
    let (send, recv) = connection.open_bi().await.map_err(std::io::Error::other)?;
    Ok((endpoint, connection, send, recv))
}

/// Write each message as a length and its bytes, in order.
async fn write_frames(mut send: quinn::SendStream, mut rx: tmpsc::UnboundedReceiver<Vec<u8>>) {
    while let Some(bytes) = rx.recv().await {
        let len = (bytes.len() as u32).to_le_bytes();
        if send.write_all(&len).await.is_err() || send.write_all(&bytes).await.is_err() {
            return;
        }
    }
    let _ = send.finish();
}

/// Read messages until the stream ends or `each` refuses one. `Err` when it
/// stopped on a refusal or a frame past [`MAX_FRAME`].
async fn read_frames(
    mut recv: quinn::RecvStream,
    mut each: impl FnMut(Vec<u8>) -> Result<(), ()>,
) -> Result<(), ()> {
    loop {
        let mut len = [0u8; 4];
        if recv.read_exact(&mut len).await.is_err() {
            return Ok(());
        }
        let len = u32::from_le_bytes(len) as usize;
        if len > MAX_FRAME {
            return Err(());
        }
        let mut bytes = vec![0u8; len];
        if recv.read_exact(&mut bytes).await.is_err() {
            return Ok(());
        }
        each(bytes)?;
    }
}

/// Accepts any certificate: see the module's note.
#[derive(Debug)]
struct AnyCertificate;

impl rustls::client::danger::ServerCertVerifier for AnyCertificate {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

#[cfg(test)]
mod tests;
