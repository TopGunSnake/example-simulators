use anyhow::Result;
use tokio::{select, sync::mpsc, try_join};
use tracing::info;

use crate::{fo_fdc_commhandler::fo_fdc_commhandler_loop, state_machine::state_machine_loop};

mod fo_fdc_commhandler;
mod state_machine;

#[tokio::main]
async fn main() -> Result<()> {
    // tracing_subscriber::fmt::init();
    console_subscriber::ConsoleLayer::builder()
        .server_addr(([127, 0, 0, 1], 7000))
        .init();

    let (from_fo_tx, from_fo_rx) = mpsc::unbounded_channel();
    let (to_fo_tx, to_fo_rx) = mpsc::unbounded_channel();

    info!("Starting the FO-FDC Comm Handler...");
    let fo_fdc_commhandler_handle =
        tokio::spawn(async move { fo_fdc_commhandler_loop(to_fo_rx, from_fo_tx).await });

    info!("Starting the FDC State Machine...");
    let state_machine_handle =
        tokio::spawn(async move { state_machine_loop(from_fo_rx, to_fo_tx).await });

    // //TODO: Right now, this select creates a stop on main until ctrl_c. We need to also exit on completion of handles.
    // select! {
    //     _ = tokio::signal::ctrl_c() => {state_machine_handle.abort()}
    // }

    let _results = try_join!(fo_fdc_commhandler_handle, state_machine_handle)?;
    _results.0?;
    _results.1?;
    Ok(())
}
