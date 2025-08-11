#![feature(generic_const_exprs)]

mod input;
mod server;
mod utils;

use std::env;
use std::io;
use std::process;
use std::sync::mpsc;
use std::thread;

use log::{error, warn};

use crate::input::{Gamepad, GamepadInput};
use crate::server::Server;

fn get_port_from_args() -> Option<u16> {
    env::args().nth(1)?.parse().ok()
}

fn start_server(port: u16, tx: mpsc::Sender<[u8; 14]>) -> io::Result<()> {
    let server = Server::new(port)?.listen()?;
    server.recv_to(tx)
}

fn recv_inputs(rx: mpsc::Receiver<[u8; 14]>) -> io::Result<()> {
    let mut gamepad = Gamepad::new("Virtual Gamepad")?;
    while let Ok(data) = rx.recv() {
        gamepad.emit(GamepadInput::try_from(&data)?)?;
    }

    Ok(())
}

fn terminate_with_error(e: io::Error) {
    error!("{e}");
    let code = e.raw_os_error().unwrap_or(1);
    process::exit(code);
}

fn main() {
    utils::init_logger("info");
    let port = get_port_from_args().unwrap_or_else(|| {
        warn!("No valid port provided, using the default port");
        11096
    });
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        loop {
            match start_server(port, tx.clone()) {
                Ok(_) => warn!("Client timeout, disconnecting"),
                Err(e) => match e.kind() {
                    io::ErrorKind::ConnectionAborted => warn!("{e}"),
                    _ => terminate_with_error(e),
                },
            }
            warn!("Reloading server");
            if port == 0 {
                warn!("Selecting a new port");
            }
        }
    });

    if let Err(e) = recv_inputs(rx) {
        terminate_with_error(e);
    }
}
