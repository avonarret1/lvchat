use std::{net::TcpListener, thread::spawn};

use crate::state::State;

pub mod client;
pub mod config;
pub mod error;
pub mod event;
pub mod handler;
pub mod state;

pub fn run(config: crate::config::Config) -> Result<(), crate::error::Error> {
    let state = State::new(config);

    let listener = TcpListener::bind(("0.0.0.0", state.config.port))?;
    let mut incoming = listener.incoming();

    let (client_queue_tx, client_queue_rx) = flume::bounded(1);

    log::info!("Listening on port {}", state.config.port);

    {
        let _server_state = state.clone();
        let _ = spawn(move || handler::server::handle(_server_state, client_queue_rx));
    }

    while let Some(client) = incoming.next() {
        let client = client?;

        client_queue_tx.send(client).expect("Incoming client in queue");
    }

    log::info!("Shutting down");

    Ok(())
}
