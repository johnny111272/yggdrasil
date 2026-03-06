use serde::{Deserialize, Serialize};
use tokio::io::AsyncBufReadExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEvent {
    pub timestamp: f64,
    pub category: String,
    pub decision: String,
    pub event_name: String,
    pub workspace: String,
    pub detail: String,
    pub context_injected: String,
    #[serde(default)]
    pub speech: Option<String>,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// Start listening on the Unix socket, forwarding parsed events to the channel.
/// Runs until the sender is dropped.
pub async fn start_listener(
    sender: tokio::sync::mpsc::UnboundedSender<HookEvent>,
) -> Result<(), String> {
    let socket_path = "/tmp/hlidskjalf.sock";

    // Remove stale socket file
    let _ = std::fs::remove_file(socket_path);

    let listener = tokio::net::UnixListener::bind(socket_path)
        .map_err(|e| format!("Failed to bind socket: {}", e))?;

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let tx = sender.clone();
                    tokio::spawn(async move {
                        let reader = tokio::io::BufReader::new(stream);
                        let mut lines = reader.lines();
                        while let Ok(Some(line)) = lines.next_line().await {
                            if line.trim().is_empty() {
                                continue;
                            }
                            match serde_json::from_str::<HookEvent>(&line) {
                                Ok(event) => {
                                    let _ = tx.send(event);
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse event: {}", e);
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Socket accept error: {}", e);
                }
            }
        }
    });

    Ok(())
}

// ── Voice alerts ──────────────────────────────────────────────────

/// Speak a message using macOS `say`. Fire-and-forget.
pub fn speak(text: &str) {
    if text.is_empty() {
        return;
    }
    let _ = std::process::Command::new("say")
        .args(["-v", "Fiona", text])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}
