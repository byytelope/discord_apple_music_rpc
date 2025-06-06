use crate::core::error::{AppError, AppResult};
use crate::ipc::commands::{IpcMessage, IpcRequest};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, oneshot};

pub struct IpcServer {
    socket_path: PathBuf,
    request_tx: mpsc::UnboundedSender<IpcRequest>,
}

impl IpcServer {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<IpcRequest>) {
        let socket_path = std::env::temp_dir().join("pipeboom.sock");
        let (request_tx, request_rx) = mpsc::unbounded_channel();

        (
            Self {
                socket_path,
                request_tx,
            },
            request_rx,
        )
    }

    pub async fn start(&mut self) -> AppResult<()> {
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)
                .map_err(|e| AppError::Ipc(format!("Failed to remove existing socket: {}", e)))?;
        }

        let listener = UnixListener::bind(&self.socket_path)
            .map_err(|e| AppError::Ipc(format!("Failed to bind Unix socket: {}", e)))?;

        log::info!("IPC server listening on {:?}", self.socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let request_tx = self.request_tx.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, request_tx).await {
                            log::error!("Client handler error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    log::error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    async fn handle_client(
        mut stream: UnixStream,
        request_tx: mpsc::UnboundedSender<IpcRequest>,
    ) -> AppResult<()> {
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();

        match reader.read_line(&mut line).await {
            Ok(0) => return Ok(()),
            Ok(_) => {
                let message = serde_json::from_str::<IpcMessage>(line.trim())
                    .map_err(|e| AppError::Ipc(format!("Failed to parse IPC message: {}", e)))?;

                let (response_tx, response_rx) = oneshot::channel();

                let request = IpcRequest {
                    command: message.command,
                    response_tx,
                };

                request_tx
                    .send(request)
                    .map_err(|e| AppError::Ipc(format!("Failed to send request: {}", e)))?;

                let response = response_rx
                    .await
                    .map_err(|e| AppError::Ipc(format!("Failed to receive response: {}", e)))?;

                let response_json = serde_json::to_string(&response)
                    .map_err(|e| AppError::Ipc(format!("Failed to serialize response: {}", e)))?;

                stream
                    .write_all(response_json.as_bytes())
                    .await
                    .map_err(|e| AppError::Ipc(format!("Failed to write response: {}", e)))?;
                stream
                    .write_all(b"\n")
                    .await
                    .map_err(|e| AppError::Ipc(format!("Failed to write newline: {}", e)))?;
            }
            Err(e) => {
                return Err(AppError::Ipc(format!("Failed to read from client: {}", e)));
            }
        }

        Ok(())
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }
    }
}
