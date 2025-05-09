#![feature(generic_const_exprs)]

mod input;
mod server;
mod utils;

use std::io::{ErrorKind, Result};
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::spawn;

use log::{error, warn};

use crate::input::{Gamepad, GamepadInput};
use crate::server::Server;
use crate::utils::init_logger;

fn start_server(port: u16, sender: Sender<[u8; 14]>) -> Result<()> {
    let server = Server::new(port)?.listen()?;
    server.recv_to(sender)
}

fn recv_inputs(receiver: Receiver<[u8; 14]>) -> Result<()> {
    let mut gamepad = Gamepad::new("Virtual Gamepad")?;
    while let Ok(data) = receiver.recv() {
        gamepad.emit(GamepadInput::try_from(&data)?)?;
    }

    Ok(())
}

fn main() {
    init_logger("info");

    let (sender, receiver) = channel();

    spawn(move || {
        loop {
            let Err(e) = start_server(3000, sender.clone()) else {
                warn!("client timeout, aborting connection");
                continue;
            };

            match e.kind() {
                ErrorKind::AddrNotAvailable | ErrorKind::Other => {
                    error!("fatal: {}", e);
                    exit(e.raw_os_error().unwrap_or(1));
                },
                ErrorKind::ConnectionAborted => warn!("client disconnected"),
                _ => warn!("{}", e),
            }

            warn!("reloading server");
        }
    });

    if let Err(e) = recv_inputs(receiver) {
        error!("fatal: {}", e);
    }
}
