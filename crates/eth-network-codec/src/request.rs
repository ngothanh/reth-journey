use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;

pub type RequestId = u64;

pub struct MessageRequest<R> {
    id: RequestId,
    rx: oneshot::Receiver<R>,
}

pub struct RequestHandler<R> {
    id: RequestId,
    tx: oneshot::Sender<R>,
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("peer dropped before responding to request {0}")]
    Cancelled(RequestId),
}

impl<R> MessageRequest<R> {
    pub fn id(&self) -> RequestId {
        self.id
    }
}

impl<R> RequestHandler<R> {
    pub fn id(&self) -> RequestId {
        self.id
    }

    pub fn complete(self, response: R) -> Result<(), R> {
        self.tx.send(response)
    }
}

pub fn channel<R>(id: RequestId) -> (MessageRequest<R>, RequestHandler<R>) {
    let (tx, rx) = oneshot::channel();
    (MessageRequest { id, rx }, RequestHandler { tx, id })
}

impl<R> Future for MessageRequest<R> {
    type Output = Result<R, RequestError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let id = self.id;
        Pin::new(&mut self.rx)
            .poll(cx)
            .map_err(|_a| RequestError::Cancelled(id))
    }
}
