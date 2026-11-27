//! Local patch (see vendor/README.md): lets an external HTTP/3 implementation carry requests that wreq would otherwise send over TCP.
//!
//! The transport decides per request whether it takes it (it knows which origins advertised `Alt-Svc: h3`, which are marked broken, and it
//! owns the QUIC connections). wreq still does everything around the exchange: default and ordered headers, cookies, redirects, retries,
//! decompression, timeouts. The response body type is [`ResBody`] so TCP and HTTP/3 bodies flow through the same layers.

use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_util::{future::BoxFuture, Stream};
use http::HeaderMap;
use http_body::{Body as HttpBody, Frame, SizeHint};
use wreq_proto::body::Incoming;

use crate::error::BoxError;

/// An HTTP/3 transport, set with `ClientBuilder::http3_transport`.
pub trait Http3Transport: Send + Sync + 'static {
    /// Whether a request to this authority (`host` or `host:port`) goes over HTTP/3 now. Resolves to the session to send it on, or `None`
    /// to use TCP (no Alt-Svc known, origin marked broken, QUIC handshake failed, ...). Called before anything of the request is consumed.
    fn route(&self, authority: &str) -> BoxFuture<'static, Option<Arc<dyn Http3Session>>>;

    /// Called with the headers of every response, to learn `Alt-Svc` from.
    fn observe(&self, authority: &str, headers: &HeaderMap);
}

/// One HTTP/3 connection.
pub trait Http3Session: Send + Sync {
    /// Sends one request and resolves when the response head has arrived.
    fn send(&self, head: Http3Head, body: Http3Body) -> BoxFuture<'static, Result<Http3Response, BoxError>>;
}

/// The head of a request for HTTP/3.
pub struct Http3Head {
    /// The request method.
    pub method: String,
    /// `:authority`.
    pub authority: String,
    /// `:path` (with the query).
    pub path: String,
    /// In wire order, names lowercase.
    pub headers: Vec<(String, String)>,
}

/// The body of a request for HTTP/3.
pub enum Http3Body {
    /// No body.
    Empty,
    /// A small body held in memory.
    Full(Bytes),
    /// A body sent as it is produced.
    Stream(Pin<Box<dyn Stream<Item = Result<Bytes, BoxError>> + Send>>),
}

/// The response to an HTTP/3 request.
pub struct Http3Response {
    /// The status code.
    pub status: u16,
    /// The headers, in the order received.
    pub headers: Vec<(String, String)>,
    /// The body as it arrives.
    pub body: Pin<Box<dyn Stream<Item = Result<Bytes, BoxError>> + Send + Sync>>,
}

/// The body of a response received over TCP or HTTP/3.
pub enum ResBody {
    /// Received over TCP.
    Tcp(Incoming),
    /// Received over HTTP/3.
    H3(Pin<Box<dyn Stream<Item = Result<Bytes, BoxError>> + Send + Sync>>),
}

impl HttpBody for ResBody {
    type Data = Bytes;
    type Error = BoxError;

    fn poll_frame(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.get_mut() {
            ResBody::Tcp(incoming) => Pin::new(incoming).poll_frame(cx).map(|frame| frame.map(|r| r.map_err(Into::into))),
            ResBody::H3(stream) => stream.as_mut().poll_next(cx).map(|item| item.map(|r| r.map(Frame::data))),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            ResBody::Tcp(incoming) => incoming.is_end_stream(),
            ResBody::H3(_) => false,
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            ResBody::Tcp(incoming) => incoming.size_hint(),
            ResBody::H3(_) => SizeHint::default(),
        }
    }
}
