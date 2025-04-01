#![allow(dead_code)]

use std::{fmt::{Debug, Formatter}, sync::{Arc, RwLock}};

use http_body_util::Empty;
use hyper::Request;
use tokio::net::TcpStream;
use hyper::body::Bytes;
use fastwebsockets::handshake;
use fastwebsockets::FragmentCollector;

type ElbowClientCallback = Box<dyn Fn() + Send + Sync>;

enum Callback {
    None,
    Callback(ElbowClientCallback)
}

impl Debug for Callback {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Callback::None => write!(f, "None"),
            Callback::Callback(_) => write!(f, "Callback"),
        }
    }
}


#[derive(Debug)]
pub struct ElbowConnection {
    inner: Arc<ElbowConnectionInner>
}

#[derive(Debug)]
struct ElbowConnectionInner {
    host: String,
    callback: Callback
}

impl ElbowConnection {
    pub fn connect(host: String, callback: ElbowClientCallback) -> ElbowConnection {
        let inner = Arc::new(ElbowConnectionInner {
            callback: Callback::Callback(callback),
            host
        });
        let inner2 = inner.clone();
        let _jh = tokio::spawn(ElbowConnection::run(inner2));
        println!("Spawned connection");
        ElbowConnection {
            inner
        }
    }

    async fn run(inner: Arc<ElbowConnectionInner>) {
        loop {
            let host = &inner.host;
            println!("Connecting to {}", host);
            let stream = TcpStream::connect(host).await.unwrap();

            let req = Request::builder()
              .method("GET")
              .uri(format!("http://{}/",host))
              .header("Host", "localhost:8080")
              .header(hyper::header::UPGRADE, "websocket")
              .header(hyper::header::CONNECTION, "upgrade")
              .header("Sec-WebSocket-Key", fastwebsockets::handshake::generate_key())
              .header("Sec-WebSocket-Version", "13")
              .body(Empty::<Bytes>::new()).unwrap();
          
    
            let (ws, _) = handshake::client(&SpawnExecutor, req, stream).await.unwrap();
            let mut ws = FragmentCollector::new(ws);
            break;
        }
    }

}

// Tie hyper's executor to tokio runtime
struct SpawnExecutor;

impl<Fut> hyper::rt::Executor<Fut> for SpawnExecutor
where
Fut: Future + Send + 'static,
Fut::Output: Send + 'static,
{
    fn execute(&self, fut: Fut) {
        tokio::task::spawn(fut);
    }
} 

mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let conn = ElbowConnection::connect("localhost:8080".to_string(), Box::new(|| {
            println!("Callback called");
        }));
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}