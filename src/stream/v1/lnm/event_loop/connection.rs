use std::{future::Future, sync::Arc};

use fastwebsockets::{FragmentCollector, Frame, OpCode, WebSocketError, handshake};
use http_body_util::Empty;
use hyper::{
    Request, Uri,
    body::Bytes,
    header::{CONNECTION, HOST, UPGRADE},
    upgrade::Upgraded,
};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use tokio_rustls::{
    TlsConnector,
    rustls::{ClientConfig, RootCertStore, pki_types::ServerName},
};
use webpki_roots::TLS_SERVER_ROOTS;

use super::super::super::{
    error::{ConnectionResult, StreamConnectionError},
    models::{StreamJsonRpcMessage, StreamJsonRpcRequest},
};

#[derive(Clone, Debug)]
pub(super) enum LnmStreamResponse {
    Close,
    JsonRpc(StreamJsonRpcMessage),
    Ping(Vec<u8>),
    Pong,
}

struct SpawnExecutor;

impl<Fut> hyper::rt::Executor<Fut> for SpawnExecutor
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    fn execute(&self, fut: Fut) {
        tokio::task::spawn(fut);
    }
}

pub(super) struct StreamApiConnection(FragmentCollector<TokioIo<Upgraded>>);

struct StreamEndpoint {
    uri: Uri,
    host: String,
    authority: String,
    port: u16,
}

impl StreamEndpoint {
    fn parse(endpoint: &str) -> ConnectionResult<Self> {
        let uri: Uri = endpoint
            .parse()
            .map_err(StreamConnectionError::InvalidEndpointUri)?;

        if uri.scheme_str() != Some("wss") {
            return Err(StreamConnectionError::InvalidEndpoint(endpoint.to_string()));
        }

        let host = uri
            .host()
            .ok_or_else(|| StreamConnectionError::InvalidEndpoint(endpoint.to_string()))?
            .to_string();
        let authority = uri
            .authority()
            .ok_or_else(|| StreamConnectionError::InvalidEndpoint(endpoint.to_string()))?
            .as_str()
            .to_string();
        let port = uri.port_u16().unwrap_or(443);

        Ok(Self {
            uri,
            host,
            authority,
            port,
        })
    }
}

impl StreamApiConnection {
    pub async fn new(endpoint: &str) -> ConnectionResult<Self> {
        let endpoint = StreamEndpoint::parse(endpoint)?;
        let endpoint_addr = format!("{}:{}", endpoint.host, endpoint.port);

        let server_name = ServerName::try_from(endpoint.host.clone())
            .map_err(StreamConnectionError::InvalidDnsName)?;

        let tls_connector = {
            let mut root_cert_store = RootCertStore::empty();
            root_cert_store.extend(TLS_SERVER_ROOTS.iter().cloned());

            let config = ClientConfig::builder()
                .with_root_certificates(root_cert_store)
                .with_no_client_auth();

            TlsConnector::from(Arc::new(config))
        };

        let tcp_stream = TcpStream::connect(&endpoint_addr)
            .await
            .map_err(StreamConnectionError::CreateTcpStream)?;
        let tls_stream = tls_connector
            .connect(server_name, tcp_stream)
            .await
            .map_err(StreamConnectionError::ConnectTcpStream)?;

        let req = Request::builder()
            .method("GET")
            .uri(endpoint.uri.clone())
            .header(HOST, endpoint.authority.as_str())
            .header(UPGRADE, "websocket")
            .header(CONNECTION, "upgrade")
            .header("Sec-WebSocket-Key", handshake::generate_key())
            .header("Sec-WebSocket-Version", "13")
            .body(Empty::<Bytes>::new())
            .map_err(StreamConnectionError::HttpUpgradeRequest)?;

        let (ws, _) = handshake::client(&SpawnExecutor, req, tls_stream)
            .await
            .map_err(StreamConnectionError::Handshake)?;
        let ws = FragmentCollector::new(ws);

        Ok(Self(ws))
    }

    async fn send_frame(&mut self, frame: Frame<'_>) -> ConnectionResult<()> {
        self.0
            .write_frame(frame)
            .await
            .map_err(StreamConnectionError::WriteFrame)
    }

    pub async fn send_json_rpc(&mut self, req: &StreamJsonRpcRequest) -> ConnectionResult<()> {
        let payload = req.try_to_bytes()?.into();
        let frame = Frame::text(payload);
        self.send_frame(frame).await
    }

    pub async fn send_close(&mut self) -> ConnectionResult<()> {
        let frame = Frame::close(1000, &[]);
        self.send_frame(frame).await
    }

    pub async fn send_pong(&mut self, payload: Vec<u8>) -> ConnectionResult<()> {
        let frame = Frame::pong(payload.into());
        self.send_frame(frame).await
    }

    pub async fn send_ping(&mut self) -> ConnectionResult<()> {
        let frame = Frame::new(true, OpCode::Ping, None, Vec::new().into());
        self.send_frame(frame).await
    }

    pub async fn read_response(&mut self) -> ConnectionResult<LnmStreamResponse> {
        let frame = match self.0.read_frame().await {
            Ok(frame) => frame,
            Err(WebSocketError::ConnectionClosed) => return Ok(LnmStreamResponse::Close),
            Err(e) => return Err(StreamConnectionError::ReadFrame(e)),
        };

        let response = match frame.opcode {
            OpCode::Text => {
                let text = String::from_utf8(frame.payload.to_vec())
                    .map_err(StreamConnectionError::DecodeText)?;
                let json_rpc_message = serde_json::from_str::<StreamJsonRpcMessage>(&text)
                    .map_err(StreamConnectionError::DecodeJson)?;
                LnmStreamResponse::JsonRpc(json_rpc_message)
            }
            OpCode::Close => LnmStreamResponse::Close,
            OpCode::Ping => LnmStreamResponse::Ping(frame.payload.to_vec()),
            OpCode::Pong => LnmStreamResponse::Pong,
            unhandled_opcode => {
                return Err(StreamConnectionError::UnhandledOpCode(unhandled_opcode));
            }
        };

        Ok(response)
    }
}
