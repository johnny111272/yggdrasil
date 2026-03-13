use serde::Deserialize;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::Ipv4Addr;
use std::path::PathBuf;

pub use datagram_io::{Datagram, DatagramKind, Priority};

const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(239, 0, 0, 1);
const MULTICAST_PORT: u16 = 9899;
const FALLBACK_HOME: &str = "/tmp";

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
    fn from(hook_event: HookEvent) -> Self {
        let source = hook_event
            .event_name
            .split(':')
            .next()
            .unwrap_or("unknown")
            .to_string();

        let kind = if hook_event.category == "quality" {
            DatagramKind::Quality
        } else {
            DatagramKind::Alert
        };

        Datagram {
            timestamp: hook_event.timestamp,
            source,
            kind,
            classifier: None,
            priority: priority_from_decision(&hook_event.decision),
            workspace: hook_event.workspace,
            detail: Some(hook_event.detail),
            speech: hook_event.speech,
            payload: hook_event.payload,
        }
    }
}

/// Parse a JSON line as either a Datagram or legacy HookEvent (converted to Datagram).
fn parse_event(line: &str) -> Option<Datagram> {
    if let Ok(datagram) = serde_json::from_str::<Datagram>(line) {
        return Some(datagram);
    }
    if let Ok(hook_event) = serde_json::from_str::<HookEvent>(line) {
        return Some(hook_event.into());
    }
    None
}

// ── Packet processing ───────────────────────────────────────────

/// Try to parse a UDP packet as a datagram and forward it.
fn forward_packet(
    data: &[u8],
    sender: &tokio::sync::mpsc::UnboundedSender<Datagram>,
) {
    let line = match std::str::from_utf8(data) {
        Ok(s) => s.trim(),
        Err(_) => return,
    };
    if line.is_empty() {
        return;
    }
    if let Some(datagram) = parse_event(line) {
        let _ = sender.send(datagram);
    }
}

/// Set up the multicast UDP socket for receiving datagrams.
/// Uses SO_REUSEADDR + SO_REUSEPORT so multiple listeners
/// (Hlidskjalf standalone + Yggdrasil) can bind the same port.
fn setup_multicast_socket() -> Result<tokio::net::UdpSocket, String> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|e| format!("Failed to create socket: {e}"))?;
    socket
        .set_reuse_address(true)
        .map_err(|e| format!("Failed to set SO_REUSEADDR: {e}"))?;
    socket
        .set_reuse_port(true)
        .map_err(|e| format!("Failed to set SO_REUSEPORT: {e}"))?;
    let bind_addr = std::net::SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, MULTICAST_PORT);
    socket
        .bind(&bind_addr.into())
        .map_err(|e| format!("Failed to bind multicast port {MULTICAST_PORT}: {e}"))?;
    socket
        .join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::LOCALHOST)
        .map_err(|e| format!("Failed to join multicast group: {e}"))?;
    socket
        .set_nonblocking(true)
        .map_err(|e| format!("Failed to set non-blocking: {e}"))?;
    let std_socket: std::net::UdpSocket = socket.into();
    tokio::net::UdpSocket::from_std(std_socket)
        .map_err(|e| format!("Failed to create async socket: {e}"))
}

// ── Lockfile system ──────────────────────────────────────────────

fn hlidskjalf_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| FALLBACK_HOME.into());
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

/// Check lockfiles and emit critical alerts for any violations.
fn check_lockfiles(sender: &tokio::sync::mpsc::UnboundedSender<Datagram>) {
    if kill_path().exists() {
        let _ = sender.send(Datagram {
            timestamp: now(),
            source: "lockfile_monitor".to_string(),
            kind: DatagramKind::Alert,
            classifier: None,
            priority: Priority::Critical,
            workspace: String::new(),
            detail: Some("KILL.lock detected — kill switch activated".to_string()),
            speech: Some("CRITICAL: Kill switch activated. Shutting down all operations.".to_string()),
            payload: None,
        });
    }

    if !keep_alive_path().exists() {
        let _ = sender.send(Datagram {
            timestamp: now(),
            source: "lockfile_monitor".to_string(),
            kind: DatagramKind::Alert,
            classifier: None,
            priority: Priority::Critical,
            workspace: String::new(),
            detail: Some("KEEP_ALIVE.lock missing — system integrity check failed".to_string()),
            speech: Some("CRITICAL: Keep alive lock missing. System integrity compromised.".to_string()),
            payload: None,
        });
    }
}

fn now() -> f64 {
    datagram_io::now()
}

// ── Listener task ────────────────────────────────────────────────

/// Spawn a task that receives UDP multicast datagrams and forwards them.
async fn start_listener(
    sender: tokio::sync::mpsc::UnboundedSender<Datagram>,
) -> Result<(), String> {
    let socket = setup_multicast_socket()?;

    tokio::spawn(async move {
        eprintln!("[hlidskjalf_core] listener task started, awaiting multicast on 239.0.0.1:9899");
        let mut buf = [0u8; 65535];
        let mut recv_count: u64 = 0;
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, src)) => {
                    recv_count += 1;
                    eprintln!("[hlidskjalf_core] recv #{recv_count}: {len} bytes from {src}");
                    forward_packet(&buf[..len], &sender);
                }
                Err(e) => eprintln!("[hlidskjalf_core] multicast recv error: {e}"),
            }
        }
    });

    Ok(())
}

// ── Lockfile monitor task ────────────────────────────────────────

/// Spawn a task that checks lockfiles every 5 seconds.
async fn start_lockfile_monitor(
    sender: tokio::sync::mpsc::UnboundedSender<Datagram>,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            check_lockfiles(&sender);
        }
    });
}

// ── Orchestration ────────────────────────────────────────────────

/// Initialize the full Hlidskjalf subsystem: set up lockfiles,
/// start the multicast listener, and start the lockfile monitor.
/// All parsed events are forwarded to the provided channel.
///
/// Two independent tasks share the channel via sender.clone().
/// UnboundedSender::clone is an Arc refcount bump — this is the
/// intended use pattern for mpsc with multiple producers.
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
