use tokio::{io::AsyncWriteExt, net::TcpListener, sync::Notify};

use super::*;

async fn run_server(b: &[u8]) {
    

}


#[tokio::test]
async fn it_works() {

    let listen_notify = Arc::new(Notify::new());
    let listen_notify2 = listen_notify.clone();
    let serverhandle = tokio::spawn(async move {
        let serverbytes = b"ElbowServer_1   "; //exactly 16 Bytes
        let listener = TcpListener::bind("127.0.0.1:8181").await.unwrap();
        listen_notify.notify_waiters();
        let (mut socket, _) = listener.accept().await.unwrap();
        println!("Accepted connection");
        let mut buf = [0; 16];
        socket.peek(&mut buf).await.unwrap();
        println!("Received: {:?}", buf);
        socket.shutdown().await.unwrap();
    });
    listen_notify2.notified().await;        

    let conn = ElbowConnection::connect("localhost:8181".to_string(), Box::new(|| {
        println!("Callback called");
    }));
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
}