use bytes::Bytes;
use eth_network_codec::MessageStream;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

#[tokio::test]
async fn read_frame_from_tcp() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (sock, _) = listener.accept().await.unwrap();
        let mut writer = FramedWrite::new(sock, LengthDelimitedCodec::new());
        let payloads = [&b"alpha"[..], &b"beta"[..]];
        for payload in payloads {
            writer.send(Bytes::from(payload)).await.unwrap();
        }
    });

    let client = TcpStream::connect(addr).await.unwrap();
    let codec = LengthDelimitedCodec::builder()
        .max_frame_length(1024 * 1024)
        .big_endian()
        .new_codec();
    let mut stream = MessageStream::new(client, codec);

    let mut received = Vec::new();
    while let Some(frame) = stream.next().await {
        received.push(frame.unwrap());
    }
    server.await.unwrap();
    assert_eq!(received, vec![b"alpha".to_vec(), b"beta".to_vec()]);
}
