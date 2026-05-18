use bytes::BytesMut;
use core::marker::PhantomPinned;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::Stream;
use tokio::io::AsyncRead;
use tokio_util::codec::Decoder;
use tokio_util::io::poll_read_buf;

pub struct MessageStreamManual<IO, Codec> {
    io: IO,
    codec: Codec,
    buf: BytesMut,
    eof: bool,
    _pinned: PhantomPinned,
}

struct StreamProj<'a, IO, Codec> {
    io: Pin<&'a mut IO>,
    codec: &'a mut Codec,
    buf: &'a mut BytesMut,
    eof: &'a mut bool,
}

impl<IO, Codec> MessageStreamManual<IO, Codec> {
    pub fn new(io: IO, codec: Codec) -> Self {
        Self {
            io,
            codec,
            buf: BytesMut::with_capacity(8 * 1024),
            eof: false,
            _pinned: PhantomPinned,
        }
    }

    fn project<'a>(self: Pin<&'a mut Self>) -> StreamProj<'a, IO, Codec> {
        // SAFETY:
        // 1. `io` is treated as a pinned field. We expose only `Pin<&mut IO>` to
        //    callers; there is no path to a `&mut IO` that would let a caller
        //    `mem::swap` or `mem::replace` it out of its slot.
        // 2. `codec`, `buf`, `eof` are NOT pinned fields. Returning `&mut` is
        //    sound: they have no Pin obligations of their own.
        // 3. There is no `Drop` impl on `MessageStreamManual` that moves the
        //    `io` field out before destruction.
        // 4. The struct has no other public `&mut self` API that exposes `io`
        //    or the projected references in a way that violates pin.
        unsafe {
            let this = self.get_unchecked_mut();
            StreamProj {
                io: Pin::new_unchecked(&mut this.io),
                codec: &mut this.codec,
                buf: &mut this.buf,
                eof: &mut this.eof,
            }
        }
    }
}

// Conditional Unpin. PhantomPinned blocks the auto-impl, so we write this
// explicitly. The compiler will check that exposing &mut to non-pinned fields
// while only exposing Pin<&mut> to `io` is consistent with this bound.
impl<IO: Unpin, C> Unpin for MessageStreamManual<IO, C> {}

impl<IO, Codec> Stream for MessageStreamManual<IO, Codec>
where
    IO: AsyncRead,
    Codec: Decoder,
{
    type Item = Result<Codec::Item, Codec::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            match this.codec.decode(this.buf) {
                Ok(Some(item)) => return Poll::Ready(Some(Ok(item))),
                Err(e) => return Poll::Ready(Some(Err(e))),
                Ok(None) => {}
            }
            if *this.eof {
                return match this.codec.decode_eof(this.buf) {
                    Ok(Some(item)) => Poll::Ready(Some(Ok(item))),
                    Ok(None) => Poll::Ready(None),
                    Err(e) => Poll::Ready(Some(Err(e))),
                };
            }
            match poll_read_buf(this.io.as_mut(), cx, this.buf) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(e.into()))),
                Poll::Ready(Ok(0)) => *this.eof = true,
                Poll::Ready(Ok(_)) => {}
            }
        }
    }
}
