use serde::Deserialize;
use std::io::Write;
use std::path::PathBuf;
use tokio::io::AsyncBufReadExt;

pub use socket_emit::{Datagram, DatagramKind, Priority};

// ── Legacy HookEvent (backward compat) ───────────────────────────

#[derive(Debug, Clone, Deserialize)]
struct HookEvent {
    timestamp: f64,
    category: String,
    decision: String,
    event_name: String,
    workspace: String,
    detail: String,
    #[allow(dead_code)]
    context_injected: String,
    #[serde(default)]
    speech: Option<String>,
    #[serde(default)]
    payload: Option<serde_json::Value>,
}

fn priority_from_decision(decision: &str) -> Priority {
    match decision {
        "deny" => Priority::High,
        "warn" => Priority::Normal,
        _ => Priority::Low,
    }
}

impl From<HookEvent> for Datagram {
    fn from(ev: HookEvent) -> Self {
        let source = ev
            .event_name
            .split(':')
            .next()
            .unwrap_or("unknown")
            .to_string();

        let kind = if ev.category == "quality" {
            DatagramKind::Report
        } else {
            DatagramKind::Alert
        };

        Datagram {
            timestamp: ev.timestamp,
            source,
            kind,
            priority: priority_from_decision(&ev.decision),
            workspace: ev.workspace,
            detail: Some(ev.detail),
            speech: ev.speech,
            payload: ev.payload,
        }
    }
}

/// Parse a JSON line as either a Datagram or legacy HookEvent (converted to Datagram).
fn parse_event(line: &str) -> Option<Datagram> {
    // Try new format first
    if let Ok(dg) = serde_json::from_str::<Datagram>(line) {
        return Some(dg);
    }
    // Fallback to legacy format
    if let Ok(ev) = serde_json::from_str::<HookEvent>(line) {
        return Some(ev.into());
    }
    None
}

/// Start listening on the Unix socket, forwarding parsed events to the channel.
/// Runs until the sender is dropped.
pub async fn start_listener(
    sender: tokio::sync::mpsc::UnboundedSender<Datagram>,
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
                            match parse_event(&line) {
                                Some(datagram) => {
                                    append_to_log(&datagram);
                                    let _ = tx.send(datagram);
                                }
                                None => {
                                    eprintln!("Failed to parse event: {}", line);
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

// ── Rolling event log ─────────────────────────────────────────────

fn hlidskjalf_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".ai").join("hlidskjalf")
}

fn log_path() -> PathBuf {
    hlidskjalf_dir().join("events.jsonl")
}

fn append_to_log(datagram: &Datagram) {
    let dir = hlidskjalf_dir();
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    if let Ok(json) = serde_json::to_string(datagram) {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path())
        {
            let _ = writeln!(f, "{}", json);
        }
    }
}

/// Rotate the event log on startup — keep only last 24h of events.
pub fn rotate_log() {
    let path = log_path();
    if !path.exists() {
        return;
    }
    let cutoff = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
        - 86400.0;

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let kept: Vec<&str> = content
        .lines()
        .filter(|line| {
            serde_json::from_str::<Datagram>(line)
                .map(|d| d.timestamp >= cutoff)
                .unwrap_or(false)
        })
        .collect();

    let _ = std::fs::write(&path, kept.join("\n") + if kept.is_empty() { "" } else { "\n" });
}

// ── Lockfile system ──────────────────────────────────────────────

fn keep_alive_path() -> PathBuf {
    hlidskjalf_dir().join("KEEP_ALIVE.lock")
}

fn kill_path() -> PathBuf {
    hlidskjalf_dir().join("KILL.lock")
}

/// Ensure the data directory and KEEP_ALIVE.lock exist.
pub fn init_lockfiles() {
    let dir = hlidskjalf_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = keep_alive_path();
    if !path.exists() {
        let _ = std::fs::File::create(&path);
    }
}

/// Start monitoring lockfiles. Emits critical alerts to the channel.
/// Checks every 5 seconds for:
///   - KILL.lock present → critical alert
///   - KEEP_ALIVE.lock missing → critical alert
pub async fn start_lockfile_monitor(
    sender: tokio::sync::mpsc::UnboundedSender<Datagram>,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            interval.tick().await;

            if kill_path().exists() {
                let dg = Datagram {
                    timestamp: now(),
                    source: "lockfile_monitor".to_string(),
                    kind: DatagramKind::Alert,
                    priority: Priority::Critical,
                    workspace: String::new(),
                    detail: Some("KILL.lock detected — kill switch activated".to_string()),
                    speech: Some("CRITICAL: Kill switch activated. Shutting down all operations.".to_string()),
                    payload: None,
                };
                let _ = sender.send(dg);
            }

            if !keep_alive_path().exists() {
                let dg = Datagram {
                    timestamp: now(),
                    source: "lockfile_monitor".to_string(),
                    kind: DatagramKind::Alert,
                    priority: Priority::Critical,
                    workspace: String::new(),
                    detail: Some("KEEP_ALIVE.lock missing — system integrity check failed".to_string()),
                    speech: Some("CRITICAL: Keep alive lock missing. System integrity compromised.".to_string()),
                    payload: None,
                };
                let _ = sender.send(dg);
            }
        }
    });
}

fn now() -> f64 {
    socket_emit::now()
}

// ── Orchestration ────────────────────────────────────────────────

/// Initialize the full Hlidskjalf subsystem: rotate logs, set up lockfiles,
/// start the Unix socket listener, and start the lockfile monitor.
/// All parsed events are forwarded to the provided channel.
pub async fn start_all(
    sender: tokio::sync::mpsc::UnboundedSender<Datagram>,
) -> Result<(), String> {
    rotate_log();
    init_lockfiles();
    start_listener(sender.clone()).await?;
    start_lockfile_monitor(sender).await;
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
