use std::{net::TcpListener, thread};

use anyhow::{Context, Result};
use axum::{Json, Router, routing::get};
use serde::Serialize;

const CONTROL_ADDR: &str = "127.0.0.1:8765";

#[derive(Serialize)]
struct StatusResponse {
    status: &'static str,
}

pub fn start_status_server() -> Result<()> {
    let listener = TcpListener::bind(CONTROL_ADDR)
        .with_context(|| format!("failed to bind daemon control server to {CONTROL_ADDR}"))?;
    listener
        .set_nonblocking(true)
        .context("failed to configure daemon control server socket")?;

    thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                tracing::error!(%error, "failed to start daemon control runtime");
                return;
            }
        };

        if let Err(error) = runtime.block_on(serve_status(listener)) {
            tracing::error!(%error, "daemon control server stopped");
        }
    });

    Ok(())
}

pub fn request_status() -> Result<()> {
    let response = reqwest::blocking::get(format!("http://{CONTROL_ADDR}/status"))
        .with_context(|| format!("failed to connect to daemon at {CONTROL_ADDR}"))?
        .error_for_status()
        .context("daemon status request failed")?
        .text()
        .context("failed to read status response from daemon")?;

    println!("{}", response.trim());

    Ok(())
}

async fn serve_status(listener: TcpListener) -> Result<()> {
    let listener = tokio::net::TcpListener::from_std(listener)
        .context("failed to create async daemon control listener")?;
    let app = Router::new().route("/status", get(status));

    tracing::info!(addr = CONTROL_ADDR, "daemon control server listening");

    axum::serve(listener, app)
        .await
        .context("daemon control server failed")
}

async fn status() -> Json<StatusResponse> {
    Json(StatusResponse { status: "ok" })
}
