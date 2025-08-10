use std::marker::PhantomData;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::time::Duration;
use std::{io, net::SocketAddr};

use local_ip_address::local_ip;
use log::{debug, info, trace};

use crate::oops;

const MAGIC: [u8; 2] = [0x45, 0x45];

mod message {
    // allowed only before establishing connection
    pub const CONNECT: u8 = 1;
    // allowed only after successfully connecting
    pub const DISCONNECT: u8 = 2;
    pub const DATA: u8 = 4;
    pub const PING: u8 = 8;
}

mod response {
    // sent only before establishing connection
    pub const CONNECTION_SUCCESS: &[u8] = &[100];
    pub const CONNECTION_FAILURE: &[u8] = &[101];
    // sent only after successfully connecting
    pub const DISCONNECTED: &[u8] = &[200];
    pub const UNSUPPORTED: &[u8] = &[201];
    pub const PONG: &[u8] = &[202];
}

pub(crate) struct Listening;
pub(crate) struct Connected;

pub(crate) struct Server<S = Listening> {
    socket: UdpSocket,
    _state: PhantomData<S>,
}

impl Server<Listening> {
    pub fn new(port: u16) -> io::Result<Self> {
        let Ok(local_addr) = local_ip() else {
            return oops!(AddrNotAvailable, "Cannot retrieve local IP address")?;
        };

        Ok(Server {
            socket: UdpSocket::bind((local_addr, port))?,
            _state: PhantomData,
        })
    }

    fn wait_for_client(&self) -> io::Result<SocketAddr> {
        let mut buffer = [0; 3];
        let (_, client_addr) = self.socket.recv_from(&mut buffer)?;
        trace!("Received handshake bytes: {buffer:?}");

        let [message_type, payload @ ..] = buffer;
        if !(message_type == message::CONNECT && payload == MAGIC) {
            self.socket
                .send_to(response::CONNECTION_FAILURE, client_addr)?;
            return oops!(InvalidData, "Invalid connection message received");
        }

        Ok(client_addr)
    }

    pub fn listen(self) -> io::Result<Server<Connected>> {
        info!("Listening at {}", self.socket.local_addr()?);

        let client_addr = self.wait_for_client()?;
        self.socket.connect(client_addr)?;
        self.socket.send(response::CONNECTION_SUCCESS)?;
        info!("Conncted to {client_addr}");

        Ok(Server {
            socket: self.socket,
            _state: PhantomData,
        })
    }
}

impl Server<Connected> {
    pub fn recv_to<const N: usize>(&self, tx: mpsc::Sender<[u8; N]>) -> io::Result<()>
    where
        [u8; N + 1]:,
    {
        let duration = Duration::from_secs(5);
        self.socket.set_read_timeout(Some(duration))?;

        let mut buffer = [0; N + 1];
        while let Ok(length) = self.socket.recv(&mut buffer) {
            trace!("Received {length} bytes");

            match buffer.first() {
                Some(&message::DATA) => {
                    let payload = buffer[1..].try_into().unwrap();
                    trace!("Payload: {payload:?}");

                    let Ok(_) = tx.send(payload) else {
                        return oops!(Other, "Channel is broken, this is bad");
                    };
                }
                Some(&message::DISCONNECT) => {
                    self.socket.send(response::DISCONNECTED)?;
                    return oops!(ConnectionAborted, "Client disconnected");
                }
                Some(&message::PING) => {
                    debug!("Received ping from client");
                    self.socket.send(response::PONG)?;
                }
                _ => {
                    self.socket.send(response::UNSUPPORTED)?;
                    return oops!(Unsupported, "Message format not supported");
                }
            }
        }

        Ok(())
    }
}
