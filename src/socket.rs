use std::{fs::set_permissions, os::unix::fs::PermissionsExt, path::PathBuf};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

use serde::{Deserialize, Serialize};

use crate::SOCKET_PATH;

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Set { enabled: bool },
    Toggle,
    Status,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Ok,
    Status { enabled: bool },
    Error { message: String },
}

pub struct Socket {
    path: PathBuf,
    listener: UnixListener,
}

#[derive(Debug)]
pub enum SocketError {
    IoError(std::io::Error),
}

impl Socket {
    pub fn create() -> Result<Self, SocketError> {
        let path = &*SOCKET_PATH;

        if path.exists() {
            std::fs::remove_file(&path).map_err(SocketError::IoError)?;
        } else if !path.parent().unwrap().exists() {
            std::fs::create_dir_all(path.parent().unwrap()).map_err(SocketError::IoError)?;
        }

        let listener = UnixListener::bind(&path).map_err(SocketError::IoError)?;

        set_permissions(&path, PermissionsExt::from_mode(0o755)).map_err(SocketError::IoError)?;

        Ok(Socket {
            listener,
            path: path.clone(),
        })
    }

    pub async fn listen<Q: for<'de> Deserialize<'de>, S: Serialize>(
        &self,
        handler: impl Fn(Q) -> S,
    ) {
        loop {
            match self.listener.accept().await {
                Ok((stream, _)) => {
                    let mut client = Client(stream);
                    let request: Q = match client.recv().await {
                        Ok(request) => request,
                        Err(e) => {
                            eprintln!("Received invalid request: {:?}", e);
                            continue;
                        }
                    };
                    let response = handler(request);

                    client.send(response).await.unwrap_or_else(|e| {
                        eprintln!("Failed to send response: {:?}", e);
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

pub struct Client(UnixStream);

#[derive(Debug)]
pub enum ClientError {
    SocketNotFound,
    Io(std::io::Error),
    Postcard(postcard::Error),
}

impl Client {
    pub async fn connect() -> Result<Self, ClientError> {
        let path = &*SOCKET_PATH;
        if !path.exists() {
            return Err(ClientError::SocketNotFound);
        }

        let stream = UnixStream::connect(path).await.map_err(ClientError::Io)?;

        Ok(Client(stream))
    }

    pub async fn send<T>(&mut self, request: T) -> Result<(), ClientError>
    where
        T: Serialize,
    {
        let bytes = postcard::to_stdvec(&request).map_err(ClientError::Postcard)?;
        self.0.write_u16(bytes.len() as u16);
        self.0.write_all(&bytes).await.map_err(ClientError::Io);
        Ok(())
    }

    pub async fn recv<T>(&mut self) -> Result<T, ClientError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let length = self.0.read_u16().await.map_err(ClientError::Io)?;
        let mut bytes = vec![0; length as usize];
        self.0
            .read_exact(&mut bytes)
            .await
            .map_err(ClientError::Io)?;

        postcard::from_bytes(&bytes).map_err(ClientError::Postcard)
    }
}
