use std::marker::PhantomData;
use std::net::UdpSocket;
use std::sync::mpsc::Sender;
use std::{io::Result, time::Duration};

use local_ip_address::local_ip;
use log::{info, debug, trace};

use crate::oops;

const MAGIC: [u8; 2] = [0x45, 0x45];

mod message {
    pub const CONNECT: u8 = 1;
    pub const DISCONNECT: u8 = 2;
    pub const DATA: u8 = 4;
    pub const PING: u8 = 8;
}

mod response {
    pub const CONNECTION_SUCCESS: &[u8] = &[100];
    pub const CONNECTION_FAILURE: &[u8] = &[101];
    pub const DISCONNECTED: &[u8] = &[200];
    pub const UNSUPPORTED: &[u8] = &[201];
    pub const PONG: &[u8] = &[202];
}

pub(crate) struct Listening;
pub(crate) struct Connected;

pub(crate) struct Server<S = Listening> {
    _state: PhantomData<S>,
    socket: UdpSocket,
}

impl Server<Listening> {
    pub fn new(port: u16) -> Result<Self> {
        let Ok(local_addr) = local_ip() else {
            return oops!(AddrNotAvailable, "cannot retrieve the local IP address")?;
        };

        Ok(Server {
            _state: PhantomData,
            socket: UdpSocket::bind((local_addr, port))?,
        })
    }

    pub fn listen(self) -> Result<Server<Connected>> {
        info!("listening at {}", self.socket.local_addr()?);

        let mut buffer = [0; 3];
        let (_, peer_addr) = self.socket.recv_from(&mut buffer)?;
        trace!("received handshake bytes: {:?}", buffer);

        let [message_type, payload @ ..] = buffer;
        if !(message_type == message::CONNECT && payload == MAGIC) {
            self.socket.send_to(response::CONNECTION_FAILURE, peer_addr)?;
            return oops!(InvalidData, "received invalid connection message");
        }

        self.socket.connect(peer_addr)?;
        self.socket.send(response::CONNECTION_SUCCESS)?;
        info!("conncted to {}", peer_addr);

        Ok(Server {
            _state: PhantomData,
            socket: self.socket,
        })
    }
}

impl Server<Connected> {
    pub fn recv_to<const N: usize>(&self, sender: Sender<[u8; N]>) -> Result<()>
    where
        [u8; N + 1]:,
    {
        let duration = Duration::from_secs(5);
        self.socket.set_read_timeout(Some(duration))?;

        let mut buffer = [0; N + 1];
        while let Ok(length) = self.socket.recv(&mut buffer) {
            trace!("received {} bytes", length);

            match buffer.first() {
                Some(&message::DATA) => {
                    let payload = buffer[1..].try_into().unwrap();
                    trace!("payload: {:?}", payload);

                    let Ok(_) = sender.send(payload) else {
                        return oops!(Other, "data channel is disconnected");
                    };
                }
                Some(&message::DISCONNECT) => {
                    self.socket.send(response::DISCONNECTED)?;
                    return oops!(ConnectionAborted, "peer disconnected");
                }
                Some(&message::PING) => {
                    debug!("received ping from client");
                    self.socket.send(response::PONG)?;
                }
                _ => {
                    self.socket.send(response::UNSUPPORTED)?;
                    return oops!(Unsupported, "message format not supported");
                }
            }
        }

        Ok(())
    }
}
