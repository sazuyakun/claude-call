use std::{net::TcpListener, sync::Arc, thread};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::Serialize;

use crate::app::{actions::run_actions, config::ActionConfig, transcript::TranscriptEvent};

const CONTROL_ADDR: &str = "127.0.0.1:8765";

#[derive(Serialize)]
struct StatusResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct TriggerResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct TranscriptResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Clone)]
struct ControlState {
    actions: Arc<Vec<ActionConfig>>,
}

pub fn start_control_server(actions: Vec<ActionConfig>) -> Result<()> {
    let listener = TcpListener::bind(CONTROL_ADDR)
        .with_context(|| format!("failed to bind daemon control server to {CONTROL_ADDR}"))?;
    listener
        .set_nonblocking(true)
        .context("failed to configure daemon control server socket")?;

    let state = ControlState {
        actions: Arc::new(actions),
    };

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

        if let Err(error) = runtime.block_on(serve_control(listener, state)) {
            tracing::error!(%error, "daemon control server stopped");
        }
    });

    Ok(())
}

pub fn request_status() -> Result<()> {
    tracing::debug!(addr = CONTROL_ADDR, "sending daemon status request");

    let response = reqwest::blocking::get(format!("http://{CONTROL_ADDR}/status"))
        .with_context(|| format!("failed to connect to daemon at {CONTROL_ADDR}"))?
        .error_for_status()
        .context("daemon status request failed")?
        .text()
        .context("failed to read status response from daemon")?;

    tracing::debug!("daemon status response received");
    println!("{}", response.trim());

    Ok(())
}

pub fn request_trigger() -> Result<()> {
    tracing::debug!(addr = CONTROL_ADDR, "sending daemon trigger request");

    let response = reqwest::blocking::Client::new()
        .post(format!("http://{CONTROL_ADDR}/trigger"))
        .send()
        .with_context(|| format!("failed to connect to daemon at {CONTROL_ADDR}"))?
        .error_for_status()
        .context("daemon trigger request failed")?
        .text()
        .context("failed to read trigger response from daemon")?;

    tracing::debug!("daemon trigger response received");
    println!("{}", response.trim());

    Ok(())
}

pub fn request_transcript(transcript: &TranscriptEvent) -> Result<()> {
    tracing::debug!(addr = CONTROL_ADDR, "sending daemon transcript request");

    let response = reqwest::blocking::Client::new()
        .post(format!("http://{CONTROL_ADDR}/transcript"))
        .json(transcript)
        .send()
        .with_context(|| format!("failed to connect to daemon at {CONTROL_ADDR}"))?
        .error_for_status()
        .context("daemon transcript request failed")?
        .text()
        .context("failed to read transcript response from daemon")?;

    tracing::debug!("daemon transcript response received");
    println!("{}", response.trim());

    Ok(())
}

async fn serve_control(listener: TcpListener, state: ControlState) -> Result<()> {
    let listener = tokio::net::TcpListener::from_std(listener)
        .context("failed to create async daemon control listener")?;
    let app = Router::new()
        .route("/status", get(status))
        .route("/trigger", post(trigger))
        .route("/transcript", post(transcript))
        .with_state(state);

    tracing::info!(addr = CONTROL_ADDR, "daemon control server listening");

    axum::serve(listener, app)
        .await
        .context("daemon control server failed")
}

async fn status() -> Json<StatusResponse> {
    tracing::info!("daemon status request received");

    Json(StatusResponse { status: "ok" })
}

async fn trigger(
    State(state): State<ControlState>,
) -> Result<Json<TriggerResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!(
        actions = state.actions.len(),
        "daemon trigger request received"
    );

    run_actions(&state.actions).map_err(|error| {
        tracing::warn!(%error, "daemon trigger failed");

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: error.to_string(),
            }),
        )
    })?;

    tracing::info!("daemon trigger completed");

    Ok(Json(TriggerResponse {
        status: "triggered",
    }))
}

async fn transcript(
    Json(transcript): Json<TranscriptEvent>,
) -> Result<Json<TranscriptResponse>, (StatusCode, Json<ErrorResponse>)> {
    let transcript = TranscriptEvent::new(transcript.text).map_err(|error| {
        tracing::warn!(%error, "daemon transcript rejected");

        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: error.to_string(),
            }),
        )
    })?;

    tracing::info!("daemon transcript request received");
    tracing::info!(text = %transcript.text, "transcript received");
    println!("{}", transcript.text);

    Ok(Json(TranscriptResponse { status: "received" }))
}
