/* This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich */
use std::sync::Arc;

use fastwebsockets::{OpCode, WebSocket, WebSocketError};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::Notify;

#[derive(Debug)]
pub struct ElbowServer {
    shutdown_notify: Arc<Notify>,
    started_notify: Arc<Notify>
}

impl ElbowServer {
    pub fn new() -> ElbowServer {
        ElbowServer {
            shutdown_notify: Arc::new(Notify::new()),
            started_notify: Arc::new(Notify::new())
        }
    }

    pub async fn handle_ws<T: AsyncRead+AsyncWrite+Unpin>(&self, ws: WebSocket<T>) -> Result<(), WebSocketError> {
        let mut ws = fastwebsockets::FragmentCollector::new(ws);

        loop {
            match ws.read_frame().await {
                Ok(frame) => {
                    match frame.opcode {
                        OpCode::Close => {
                            self.shutdown_notify.notify_one();
                            break
                        }
                        OpCode::Text | OpCode::Binary => {
                            let _x = frame.payload.to_vec();
                            ws.write_frame(frame).await?;
                        }
                        _ => {}
                        }
                },
                Err(e) => {
                    self.shutdown_notify.notify_waiters();
                    return Err(e)
                }
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::sync::LazyLock;

    use super::*;

use fastwebsockets::upgrade;
use fastwebsockets::OpCode;
use fastwebsockets::WebSocketError;
use http_body_util::Empty;
use hyper::body::Bytes;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Request;
use hyper::Response;
use tokio::net::TcpListener;

use tokio::select;
use tokio::sync::Notify;
use tokio::sync::OnceCell;

    async fn handle_client(fut: upgrade::UpgradeFut, server:Arc<ElbowServer>) -> Result<(), WebSocketError> {
        let ws = fut.await?;
        server.handle_ws(ws).await?;
        Ok(())
        }

    async fn server_upgrade(
            mut req: Request<Incoming>,
            server: Arc<ElbowServer>,
        ) -> Result<Response<Empty<Bytes>>, WebSocketError> {
        let (response, fut) = upgrade::upgrade(&mut req)?;

        tokio::task::spawn(async move {
            if let Err(e) = tokio::task::unconstrained(handle_client(fut, server)).await {
                eprintln!("Error in websocket connection: {}", e);
            }
        });
        Ok(response)
    }

    async fn runserver(server: ElbowServer) -> Result<(), WebSocketError> {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;
        println!("Server started, listening on {}", "127.0.0.1:8080");
        let shutdown_notify = server.shutdown_notify.clone();

        let server = Arc::new(server);
        server.started_notify.notify_waiters();
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

    #[tokio::test]
    async fn it_works() {
        let server = ElbowServer::new();
        let n = server.shutdown_notify.clone();
        tokio::spawn(runserver(server));
        n.notified().await
    }
}
