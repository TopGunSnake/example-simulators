//! A Forward Observer (FO) Simulator
//!
//! This crate provides an `async` FO Simulator built on `tokio`.
//! The crate will listen for and talk to any FDC over the [`fo_fdc_comms`] message interface.
use anyhow::Result;
use state_machine::state_machine_loop;
use tokio::{select, sync::mpsc, try_join};
use tracing::info;

mod fo_fdc_commhandler;
mod state_machine;

/// Entry function
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let (message_queue_sender, message_queue) = mpsc::unbounded_channel();
    let (to_fdc, to_fdc_receiver) = mpsc::unbounded_channel();

    info!("Starting state machine...");
    let state_machine_handle =
        tokio::spawn(async move { state_machine_loop(message_queue, to_fdc).await });

    //TODO: Right now, this select creates a stop on main until ctrl_c. We need to also exit on completion of handles.
    select! {
        _ = tokio::signal::ctrl_c() => {state_machine_handle.abort()}
    }
    let results = try_join!(state_machine_handle);

    //TODO: Unpack the results to propagate relevant errors.
    Ok(())
}
