/* This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich */
#![allow(dead_code)]

mod client;

use std::sync::{Arc, RwLock};
use std::fmt::{Debug, Formatter};

use fastwebsockets::{CloseCode, Frame, OpCode, WebSocket, WebSocketError};
use tokio::io::{AsyncRead, AsyncWriteExt};
use tokio::sync::Notify;

use dataflowgrid_commons::cursedbuffer::CursedBuffer;

enum Callback<T> {
    None,
    Callback(Box<dyn Fn(&T) + Send + Sync>),
}

impl<T> Debug for Callback<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Callback::None => write!(f, "None"),
            Callback::Callback(_) => write!(f, "Callback"),
        }
    }
}

#[derive(Debug)]
pub struct ElbowServer {
    shutdown_notify: Arc<Notify>,
    started_notify: Arc<Notify>,
    callback: Callback<usize>,
    connections: Vec<Arc<ElbowServerConnection>>,
}

#[derive(Debug)]
pub struct ElbowServerConnection {
    buffer: CursedBuffer<u8>,
    buffer_out: CursedBuffer<u8>,
    shutdown_notify: Notify,
    callback: Callback<usize>,
    connection_id: usize,
    server: Arc<RwLock<ElbowServer>>,
}

impl ElbowServerConnection {
    pub fn new(server: Arc<RwLock<ElbowServer>>) -> Result<Arc<ElbowServerConnection>, std::io::Error> {
        let mut conn = ElbowServerConnection {
            buffer: CursedBuffer::new(),
            buffer_out: CursedBuffer::new(),
            shutdown_notify: Notify::new(),
            callback: Callback::None,
            server: server.clone(),
            connection_id: 0,
        };
        conn.connection_id = server.write().unwrap().connections.len(); //TODO: This is not thread safe
        let conn = Arc::new(conn);
        server.write().unwrap().connections.push(conn.clone());
        Ok(conn)
    }

    pub async fn handle_ws<T: AsyncRead+AsyncWriteExt+Unpin>(&self, ws: WebSocket<T>) -> Result<(), WebSocketError> {
        let mut ws = fastwebsockets::FragmentCollector::new(ws);
        let reader = self.buffer_out.reader(0);

        match self.callback {
            Callback::Callback(ref cb) => cb(&self.connection_id),
            _ => {}
        }
        loop {
            tokio::select! {
                readframe = ws.read_frame() => {
                    match readframe {
                        Ok(frame) => {
                            match frame.opcode {
                                OpCode::Close => {
                                    self.buffer.close();
                                    self.shutdown_notify.notify_waiters();
                                    break
                                }
                                OpCode::Text | OpCode::Binary => {
                                    println!("Received data: {:?}", frame.payload);
                                    let data = frame.payload.to_vec();
                                    self.buffer.awrite(data).await.unwrap();
                                }
                                _ => {
                                    self.buffer.close();
                                    self.shutdown_notify.notify_one();
                                    ws.write_frame(Frame::close(CloseCode::Normal.into(), b"")).await?;
                                    ws.into_inner().shutdown().await?;
                                    break;
                                }
                            }
                        },
                        Err(e) => {
                            self.shutdown_notify.notify_waiters();
                            return Err(e)
                        }
                    }
                }
                writedata = reader.anext_chunk() => {
                    ws.write_frame( Frame::new(true, OpCode::Binary, None, fastwebsockets::Payload::Borrowed(writedata.unwrap().as_slice()))).await?;
                }
            }
        }
        Ok(())
    }

}

impl ElbowServer {
    pub fn new() -> ElbowServer {
        ElbowServer {
            shutdown_notify: Arc::new(Notify::new()),
            started_notify: Arc::new(Notify::new()),
            callback: Callback::None,
            connections: Vec::new(),
        }
    }

    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: Fn(&usize) + Send + Sync + 'static,
    {
        self.callback = Callback::Callback(Box::new(callback));
    }
}


#[cfg(test)]
mod tests {
    use std::sync::RwLock;

    use super::*;

use fastwebsockets::handshake;
use fastwebsockets::upgrade;
use fastwebsockets::FragmentCollector;
use fastwebsockets::WebSocketError;
use http_body_util::Empty;
use hyper::body::Bytes;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Request;
use hyper::Response;
use tokio::net::TcpListener;

use tokio::net::TcpStream;
use tokio::select;

    async fn server_upgrade(
            mut req: Request<Incoming>,
            server: Arc<RwLock<ElbowServer>>,
        ) -> Result<Response<Empty<Bytes>>, WebSocketError> {
        let (response, fut) = upgrade::upgrade(&mut req)?;
            
        let conn = ElbowServerConnection::new(server.clone()).unwrap();
        tokio::task::spawn(async move {
            if let Err(e) = tokio::task::unconstrained(async {
                let ws = fut.await?;
                conn.handle_ws(ws).await?;
                Ok::<(), WebSocketError>(())
            }).await {
                eprintln!("Error in websocket connection: {}", e);
            }
        });
        server.read().unwrap().started_notify.notify_waiters();
        Ok(response)
    }

    async fn runserver(server: Arc<RwLock<ElbowServer>>) -> Result<(), WebSocketError> {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;
        println!("Server started, listening on {}", "127.0.0.1:8080");
        let shutdown_notify = server.read().unwrap().shutdown_notify.clone();

        server.read().unwrap().started_notify.notify_waiters();
        loop {
            select! {
                _ = shutdown_notify.notified() => {
                    println!("Shutting down server");
                    break;
                }
                r = listener.accept() => {
                    println!("Client connected");
                    let server_clone = server.clone();
                    tokio::spawn(async move {
                        let io = hyper_util::rt::TokioIo::new(r.unwrap().0);
                        let conn_fut = http1::Builder::new()
                            .serve_connection(io, service_fn(move |req| {
                                server_upgrade(req, server_clone.clone())
                            }))
                            .with_upgrades();
                        if let Err(e) = conn_fut.await {
                            println!("An error occurred: {:?}", e);
                        }
                    });
                }
            }
        }
        return Ok(());
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

    #[tokio::test]
    async fn it_works() {
        let mut server = ElbowServer::new();
        let connref = Arc::new(RwLock::new(Box::new(0)));
        server.set_callback(move |conn| {
            println!("Callback called {}", conn);
            **connref.write().unwrap() = *conn;
        });
        let server = Arc::new(RwLock::new(server));
        let startupnotifiy = server.read().unwrap().started_notify.clone();
        tokio::spawn(runserver(server.clone()));
        startupnotifiy.notified().await; //wait until server is actually started

        let stream = TcpStream::connect("localhost:8080").await.unwrap();

        let req = Request::builder()
          .method("GET")
          .uri("http://localhost:8080/")
          .header("Host", "localhost:8080")
          .header(hyper::header::UPGRADE, "websocket")
          .header(hyper::header::CONNECTION, "upgrade")
          .header("Sec-WebSocket-Key", fastwebsockets::handshake::generate_key())
          .header("Sec-WebSocket-Version", "13")
          .body(Empty::<Bytes>::new()).unwrap();
      

        let (ws, _) = handshake::client(&SpawnExecutor, req, stream).await.unwrap();
        let mut ws = FragmentCollector::new(ws);

        let shutdownnotifiy = server.read().unwrap().connections[0].clone();

        ws.write_frame(Frame::new(true, OpCode::Binary, None, fastwebsockets::Payload::Owned(b"test".to_vec()))).await.unwrap();
        server.write().unwrap().connections[0].buffer_out.write(b"hallo".to_vec()).unwrap();
        let f = ws.read_frame().await.unwrap();
        println!("Back from Server: {:?}", f.payload);
        ws.write_frame(Frame::close(1000, b"")).await.unwrap();
        
        shutdownnotifiy.shutdown_notify.notified().await; //wait until server is actually down
    }
}
