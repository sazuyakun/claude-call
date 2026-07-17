use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use anyhow::{Context, Result, bail};

const CONTROL_ADDR: &str = "127.0.0.1:8765";

pub fn start_status_server() -> Result<()> {
    let listener = TcpListener::bind(CONTROL_ADDR)
        .with_context(|| format!("failed to bind daemon control server to {CONTROL_ADDR}"))?;

    thread::spawn(move || {
        tracing::info!(addr = CONTROL_ADDR, "daemon control server listening");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    if let Err(error) = handle_connection(&mut stream) {
                        tracing::warn!(%error, "failed to handle daemon control request");
                    }
                }
                Err(error) => {
                    tracing::warn!(%error, "failed to accept daemon control connection");
                }
            }
        }
    });

    Ok(())
}

pub fn request_status() -> Result<()> {
    let mut stream = TcpStream::connect(CONTROL_ADDR)
        .with_context(|| format!("failed to connect to daemon at {CONTROL_ADDR}"))?;

    stream
        .write_all(b"GET /status HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .context("failed to send status request to daemon")?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .context("failed to read status response from daemon")?;

    if !response.starts_with("HTTP/1.1 200 OK") {
        bail!(
            "daemon status request failed: {}",
            first_response_line(&response)
        );
    }

    println!("{}", response_body(&response).trim());

    Ok(())
}

fn handle_connection(stream: &mut TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    let bytes_read = stream
        .read(&mut buffer)
        .context("failed to read daemon control request")?;

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    if request.starts_with("GET /status ") {
        write_response(stream, "200 OK", r#"{"status":"ok"}"#)?;
    } else {
        write_response(stream, "404 Not Found", r#"{"error":"not found"}"#)?;
    }

    Ok(())
}

fn write_response(stream: &mut TcpStream, status: &str, body: &str) -> Result<()> {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );

    stream
        .write_all(response.as_bytes())
        .context("failed to write daemon control response")
}

fn first_response_line(response: &str) -> &str {
    response.lines().next().unwrap_or("empty response")
}

fn response_body(response: &str) -> &str {
    response.split_once("\r\n\r\n").map_or("", |(_, body)| body)
}
