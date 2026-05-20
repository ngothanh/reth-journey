use bytes::Bytes;
use eth_network_codec::{EthMessage, EthMessageCodec, MessageStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio_util::codec::FramedWrite;

#[tokio::test]
async fn round_trips_eth_messages_over_tcp() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let messages = vec![
        EthMessage::Status {
            protocol_version: 68,
            chain_id: 1,
            total_difficulty: 999_999,
            best_hash: [0xab; 32],
            genesis_hash: [0xcd; 32],
        },
        EthMessage::GetBlockHeaders {
            request_id: 7,
            start_block: 4_242,
            limit: 16,
            skip: 0,
            reverse: true,
        },
        EthMessage::BlockHeaders {
            request_id: 7,
            headers: vec![Bytes::from_static(b"h0"), Bytes::from_static(b"header-one")],
        },
        EthMessage::BlockBodies {
            request_id: 9,
            bodies: vec![Bytes::from_static(b"body")],
        },
        EthMessage::NewBlock {
            block: Bytes::from_static(b"alpha-block"),
            total_difficulty: 1_000_000,
        },
    ];

    let sent = messages.clone();
    let server = tokio::spawn(async move {
        let (sock, _) = listener.accept().await.unwrap();
        let mut writer = FramedWrite::new(sock, EthMessageCodec::new());
        for message in sent {
            writer.send(message).await.unwrap();
        }
    });

    let client = TcpStream::connect(addr).await.unwrap();
    let mut stream = MessageStream::new(client, EthMessageCodec::new());

    let mut received = Vec::new();
    while let Some(frame) = stream.next().await {
        received.push(frame.unwrap());
    }
    server.await.unwrap();
    assert_eq!(received, messages);
}
