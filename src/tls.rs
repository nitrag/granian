use hyper::server::conn::{AddrIncoming, AddrStream};
use std::{fs, io, net::TcpListener, sync::Arc};
use tls_listener::hyper::WrappedAccept;
use tokio_rustls::{
    TlsAcceptor,
    rustls::{Certificate, PrivateKey, ServerConfig},
    server::TlsStream
};


pub(crate) type TlsListener = tls_listener::TlsListener<WrappedAccept<AddrIncoming>, TlsAcceptor>;
pub(crate) type TlsAddrStream = TlsStream<AddrStream>;

pub(crate) fn tls_listen(config: Arc<ServerConfig>, tcp: TcpListener) -> TlsListener {
    let listener = tokio::net::TcpListener::from_std(tcp).unwrap();
    let incoming = AddrIncoming::from_listener(listener).unwrap();
    TlsListener::new_hyper(tokio_rustls::TlsAcceptor::from(config), incoming)
}

fn tls_error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

pub(crate) fn load_certs(filename: &str) -> io::Result<Vec<Certificate>> {
    let certfile = fs::File::open(filename)
        .map_err(|e| tls_error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(certfile);

    let certs = rustls_pemfile::certs(&mut reader)
        .map_err(|_| tls_error("failed to load certificate".into()))?;
    Ok(certs.into_iter().map(Certificate).collect())
}

pub(crate) fn load_private_key(filename: &str) -> io::Result<PrivateKey> {
    let keyfile = fs::File::open(filename)
        .map_err(|e| tls_error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(keyfile);

    let keys = rustls_pemfile::read_all(&mut reader)
        .map_err(|_| tls_error("failed to load private key".into()))?;
    if keys.len() != 1 {
        return Err(tls_error("expected a single private key".into()));
    }

    let key = match &keys[0] {
        rustls_pemfile::Item::RSAKey(key) => PrivateKey(key.to_vec()),
        rustls_pemfile::Item::PKCS8Key(key) => PrivateKey(key.to_vec()),
        rustls_pemfile::Item::ECKey(key) => PrivateKey(key.to_vec()),
        _ => {
            return Err(tls_error("failed to load private key".into()));
        }
    };

    Ok(key)
}
