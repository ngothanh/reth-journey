use tokio::io;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub const MAX_FRAME_LEN: usize = 1024 * 1024;

pub type FramedTransport = Framed<TcpStream, LengthDelimitedCodec>;
pub fn codec() -> LengthDelimitedCodec {
    LengthDelimitedCodec::builder()
        .max_frame_length(MAX_FRAME_LEN)
        .length_field_length(4)
        .big_endian()
        .new_codec()
}

pub fn frame(stream: TcpStream) -> FramedTransport {
    Framed::new(stream, codec())
}

pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<FramedTransport> {
    let stream = TcpStream::connect(addr).await?;
    Ok(frame(stream))
}
