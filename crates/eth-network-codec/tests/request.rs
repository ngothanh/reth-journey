use eth_network_codec::channel;

#[tokio::test]
async fn deliver_response() {
    let (request, handler) = channel::<u32>(42);
    assert_eq!(request.id(), 42);
    handler.complete(7).unwrap();
    assert_eq!(request.await.unwrap(), 7);
}

#[tokio::test]
async fn cancelled_when_sender_drop() {
    let (request, handler) = channel::<u32>(42);
    drop(handler);
    assert!(request.await.is_err());
}
