mod request;

use bytes::Bytes;
use eth_network_codec::{connect, frame, MAX_FRAME_LEN};
use futures_util::{SinkExt, StreamExt};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

#[tokio::test]
async fn round_trip_payload() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let listener_addr = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut framed = frame(stream);

        while let Some(Ok(buf)) = framed.next().await {
            framed.send(buf.freeze()).await.unwrap();
        }
    });

    let mut client = connect(listener_addr).await.unwrap();
    for payload in [&b"hello"[..], &b"world"[..], &vec![0xab; 8192]] {
        client.send(Bytes::copy_from_slice(payload)).await.unwrap();
        let echoed = client.next().await.unwrap().unwrap();
        assert_eq!(&echoed[..], payload);
    }
    drop(client);
    server.await.unwrap();
}

#[tokio::test]
async fn rejects_oversized_frame() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let listener_addr = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut framed = frame(stream);
        framed.next().await
    });

    let mut raw = TcpStream::connect(listener_addr).await.unwrap();
    let oversized_len = (MAX_FRAME_LEN + 1) as u32;
    let x = oversized_len.to_be_bytes();
    raw.write_all(&x).await.unwrap();
    raw.flush().await.unwrap();
    drop(raw);

    let item = server
        .await
        .unwrap()
        .expect("server should have produced an item");
    let err = item.expect_err("oversized prefix must be rejected by the codec");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
}
