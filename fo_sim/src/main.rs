//! A Forward Observer (FO) Simulator
//!
//! This crate provides an `async` FO Simulator built on `tokio`.
//! The crate will listen for and talk to any FDC over the [`fo_fdc_comms`] message interface.
use anyhow::Result;
use tokio::{select, sync::mpsc, try_join};
use tracing::info;

mod fo_fdc_commhandler;
mod state_machine;

use fo_fdc_commhandler::fo_fdc_commhandler_loop;
use state_machine::state_machine_loop;

/// Entry function
#[tokio::main]
async fn main() -> Result<()> {
    // tracing_subscriber::fmt::init();
    console_subscriber::ConsoleLayer::builder()
        .server_addr(([127, 0, 0, 1], 6999))
        .init();

    let (message_queue_sender, message_queue) = mpsc::unbounded_channel();
    let (to_fdc, to_fdc_receiver) = mpsc::unbounded_channel();

    info!("Starting the FO-FDC Comm Handler...");
    let fo_fdc_commhandler_handle = tokio::spawn(async move {
        fo_fdc_commhandler_loop(to_fdc_receiver, message_queue_sender).await
    });

    info!("Starting state machine...");
    let state_machine_handle =
        tokio::spawn(async move { state_machine_loop(message_queue, to_fdc).await });

    //TODO: Right now, this select creates a stop on main until ctrl_c. We need to also exit on completion of handles.
    select! {
        _ = tokio::signal::ctrl_c() => {state_machine_handle.abort()}
    }
    let _results = try_join!(fo_fdc_commhandler_handle, state_machine_handle);

    //TODO: Unpack the results to propagate relevant errors.
    Ok(())
}
