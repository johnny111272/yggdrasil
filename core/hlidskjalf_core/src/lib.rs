use serde::Deserialize;
use std::net::{Ipv4Addr, UdpSocket};
use std::path::PathBuf;

pub use socket_emit::{Datagram, DatagramKind, Priority};

const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(239, 0, 0, 1);
const MULTICAST_PORT: u16 = 9899;

// ── Legacy HookEvent (backward compat) ───────────────────────────

#[derive(Debug, Clone, Deserialize)]
struct HookEvent {
    timestamp: f64,
    category: String,
    decision: String,
    event_name: String,
    workspace: String,
    detail: String,
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
    if let Ok(dg) = serde_json::from_str::<Datagram>(line) {
        return Some(dg);
    }
    if let Ok(ev) = serde_json::from_str::<HookEvent>(line) {
        return Some(ev.into());
    }
    None
}

// ── Multicast listener ──────────────────────────────────────────

/// Join UDP multicast group and forward parsed datagrams to the channel.
/// Receives from socket_emit producers on 239.0.0.1:9899.
pub async fn start_listener(
    sender: tokio::sync::mpsc::UnboundedSender<Datagram>,
) -> Result<(), String> {
    // Bind + join on a std socket, then convert to tokio.
    // Join on loopback — socket_emit sends with TTL=0 (loopback only).
    let std_socket = UdpSocket::bind(("0.0.0.0", MULTICAST_PORT))
        .map_err(|e| format!("Failed to bind multicast port {}: {}", MULTICAST_PORT, e))?;
    std_socket
        .join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::LOCALHOST)
        .map_err(|e| format!("Failed to join multicast group: {}", e))?;
    std_socket
        .set_nonblocking(true)
        .map_err(|e| format!("Failed to set non-blocking: {}", e))?;

    let socket = tokio::net::UdpSocket::from_std(std_socket)
        .map_err(|e| format!("Failed to create async socket: {}", e))?;

    tokio::spawn(async move {
        let mut buf = [0u8; 65535];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, _addr)) => {
                    let data = &buf[..len];
                    let line = match std::str::from_utf8(data) {
                        Ok(s) => s.trim(),
                        Err(_) => continue,
                    };
                    if line.is_empty() {
                        continue;
                    }
                    if let Some(datagram) = parse_event(line) {
                        let _ = sender.send(datagram);
                    }
                }
                Err(e) => {
                    eprintln!("Multicast recv error: {}", e);
                }
            }
        }
    });

    Ok(())
}

// ── Lockfile system ──────────────────────────────────────────────

fn hlidskjalf_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".ai").join("hlidskjalf")
}

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

/// Initialize the full Hlidskjalf subsystem: set up lockfiles,
/// start the multicast listener, and start the lockfile monitor.
/// All parsed events are forwarded to the provided channel.
pub async fn start_all(
    sender: tokio::sync::mpsc::UnboundedSender<Datagram>,
) -> Result<(), String> {
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
