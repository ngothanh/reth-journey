use crate::codec::Codec;
use bytes::BytesMut;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::Stream;
use pin_project_lite::pin_project;
use tokio::io::AsyncRead;
use tokio_util::io::poll_read_buf;

pin_project! {
    pub struct MessageStream<C: Codec, IO> {
        #[pin]
        io: IO,
        codec: C,
        buf: BytesMut,
        eof: bool,
    }
}

impl<C: Codec, IO> MessageStream<C, IO> {
    pub fn new(io: IO, codec: C) -> Self {
        Self {
            io,
            codec,
            buf: BytesMut::with_capacity(8 * 1024),
            eof: false,
        }
    }
}

impl<C: Codec, IO: AsyncRead> Stream for MessageStream<C, IO> {
    type Item = Result<C::Item, C::CodecError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            match this.codec.decode(this.buf) {
                Ok(None) => {}
                Err(e) => return Poll::Ready(Some(Err(e))),
                Ok(Some(item)) => return Poll::Ready(Some(Ok(item))),
            }

            if *this.eof {
                return match this.codec.decode(this.buf) {
                    Ok(None) => Poll::Ready(None),
                    Err(e) => Poll::Ready(Some(Err(e))),
                    Ok(Some(item)) => Poll::Ready(Some(Ok(item))),
                };
            }

            match poll_read_buf(this.io.as_mut().as_mut(), cx, this.buf) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(e.into()))),
                Poll::Ready(Ok(0)) => {
                    *this.eof = true;
                }
                Poll::Ready(Ok(_n)) => {}
            }
        }
    }
}
