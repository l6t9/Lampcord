use std::str::FromStr;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use lampcord_core::traits::{GatewayMessage, GatewayTransport, TransportError};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{
        Message,
        http::{HeaderName, HeaderValue, Request, Uri},
        protocol::CloseFrame,
    },
};

pub type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Default)]
pub struct TungsteniteGatewayTransport {
    socket: Option<Socket>,
    close_code: Option<u16>,
}

#[async_trait]
impl GatewayTransport for TungsteniteGatewayTransport {
    async fn connect(
        &mut self,
        url: &str,
        headers: &[(String, String)],
    ) -> Result<(), TransportError> {
        let uri = Uri::from_str(url).map_err(|error| TransportError(error.to_string()))?;
        let mut request = Request::builder()
            .uri(uri)
            .body(())
            .map_err(|error| TransportError(error.to_string()))?;

        for (name, value) in headers {
            let name =
                HeaderName::from_str(name).map_err(|error| TransportError(error.to_string()))?;
            let value =
                HeaderValue::from_str(value).map_err(|error| TransportError(error.to_string()))?;
            request.headers_mut().insert(name, value);
        }

        let (socket, _) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(|error| TransportError(error.to_string()))?;
        self.socket = Some(socket);
        self.close_code = None;
        Ok(())
    }

    async fn receive(&mut self) -> Result<GatewayMessage, TransportError> {
        let Some(socket) = self.socket.as_mut() else {
            return Err(TransportError("gateway socket is not connected".to_owned()));
        };

        loop {
            match socket.next().await {
                Some(Ok(Message::Text(text))) => {
                    return Ok(GatewayMessage::Text(text.to_string()));
                }
                Some(Ok(Message::Ping(data))) => {
                    socket
                        .send(Message::Pong(data))
                        .await
                        .map_err(|error| TransportError(error.to_string()))?;
                }
                Some(Ok(Message::Close(frame))) => {
                    self.close_code = frame.map(|frame| frame.code.into());
                    return Ok(GatewayMessage::Closed);
                }
                Some(Ok(_)) => continue,
                Some(Err(error)) => {
                    return Err(TransportError(error.to_string()));
                }
                None => return Ok(GatewayMessage::Closed),
            }
        }
    }

    async fn send(&mut self, text: &str) -> Result<(), TransportError> {
        let socket = self
            .socket
            .as_mut()
            .ok_or_else(|| TransportError("gateway socket is not connected".to_owned()))?;
        socket
            .send(Message::Text(text.to_owned().into()))
            .await
            .map_err(|error| TransportError(error.to_string()))
    }

    async fn close(&mut self, code: u16, reason: &str) -> Result<(), TransportError> {
        let socket = self
            .socket
            .as_mut()
            .ok_or_else(|| TransportError("gateway socket is not connected".to_owned()))?;
        socket
            .send(Message::Close(Some(CloseFrame {
                code: code.into(),
                reason: reason.into(),
            })))
            .await
            .map_err(|error| TransportError(error.to_string()))?;
        self.close_code = Some(code);
        Ok(())
    }

    fn close_code(&self) -> Option<u16> {
        self.close_code
    }
}
