# Debugging Hlidskjalf — Agent Reference

**Read this before touching code.** This document captures hard-won debugging knowledge so it doesn't have to be re-taught every session.

---

## Architecture: The Data Path

Datagrams flow through a dual transport system. Understanding both channels is the key to diagnosing any problem.

```
Producer (syn, bifrost, send_alert, loki, etc.)
    │
    ├── Channel 1: Unix stream → /tmp/ai_logger.sock → record_datagrams daemon
    │                                                    └── writes to ~/.ai/intercept/datagrams/datagrams_YYYY-MM-DD.jsonl
    │
    └── Channel 2: UDP multicast → 239.0.0.1:9899 → Hlidskjalf listener
                                                     └── mpsc channel → Tauri emit("datagram") → Svelte frontend
```

Both channels are fire-and-forget. Either can fail silently. The datagram crate (`nornir/capability/datagram`) sends on both in `try_emit()`.

### What this means for debugging

The logging daemon is your **evidence layer**. If datagrams appear in the JSONL log but not in Hlidskjalf, the problem is in Channel 2 (multicast → listener → frontend). If datagrams don't appear in the log either, the problem is in the producer or the datagram crate itself.

```bash
# Check today's datagram log for recent activity
tail -3 ~/.ai/intercept/datagrams/datagrams_$(date +%Y-%m-%d).jsonl | python3 -m json.tool

# Count datagrams in the last N minutes (adjust the timestamp)
python3 -c "
import json, time
cutoff = time.time() - 300  # last 5 minutes
with open('$HOME/.ai/intercept/datagrams/datagrams_$(date +%Y-%m-%d).jsonl') as f:
    recent = [json.loads(l) for l in f if json.loads(l)['timestamp'] > cutoff]
print(f'{len(recent)} datagrams in last 5 minutes')
for d in recent[-5:]:
    print(f'  {d[\"kind\"]:10s} {d[\"source\"]:20s} {d.get(\"detail\",\"\")[:60]}')
"
```

---

## Failure Modes and What They Tell You

### Mode A: Nothing at all — no speech, no display

**Diagnosis:** Datagrams are not reaching `hlidskjalf_core` at all.

**Check:**
1. Is the logging daemon receiving datagrams? → Check the JSONL log (see above)
2. Is Hlidskjalf bound to port 9899? → `lsof -i UDP:9899`
3. Is the multicast group joined? → `netstat -g | grep 239.0.0.1`
4. Is the binary the current build? → `ps -p $(pgrep hlidskjalf) -o lstart=` vs `ls -la /Applications/hlidskjalf.app/Contents/MacOS/hlidskjalf`

**Common causes:**
- Old binary still running after deploy (quit and relaunch)
- `start_all()` failed silently (check if "listening" status shows in UI)
- Multicast socket bind failed (port already in use by stale process)
- Code regression in hlidskjalf_core (e.g., broken import, wrong crate name)

### Mode B: Speech works, display broken

**Diagnosis:** Datagrams ARE arriving and being processed by the backend. The Tauri event bridge is working. The problem is in the Svelte frontend rendering.

**This is critical:** speech is triggered in the frontend's event handler (`HlidskjalfView.svelte` `listen("datagram", ...)` callback). If speech works, it means:
- The multicast listener received the datagram ✓
- It was sent through the mpsc channel ✓
- The Tauri shell emitted the `"datagram"` event ✓
- The frontend received the event payload ✓
- The speech priority check passed ✓

So the bug is ONLY in the rendering path — the part of the same callback that appends to the `datagrams` array and the template that renders it.

**Check:**
- Browser devtools in the webview (right-click → Inspect if enabled)
- Is the `datagrams` array being populated? (add a console.log in the listen callback)
- Is a filter hiding the events? (kind filter, priority threshold)
- Is the CSS hiding rendered elements? (opacity, display, visibility)

### Mode C: Some kinds render, others don't

**Diagnosis:** The event handler and basic rendering work. The problem is kind-specific rendering — likely in a payload renderer component (QualityReport.svelte, TrafficReport.svelte) or in the conditional that selects which renderer to use.

**Check:**
- Does the feed row appear at all? (even without expanded payload)
- Does the expand button work?
- Does the payload renderer component mount without errors?
- Is the payload field present and correctly shaped in the datagram?

---

## The Multicast Transport

### Sender (datagram crate)
```
bind 0.0.0.0:0 (ephemeral port)
IP_MULTICAST_TTL = 1
IP_MULTICAST_IF = 127.0.0.1 (loopback)
sendto (239.0.0.1, 9899)
```

### Receiver (hlidskjalf_core)
```
bind 0.0.0.0:9899
join_multicast_v4(239.0.0.1, 127.0.0.1)
set_nonblocking(true)
convert to tokio::net::UdpSocket
```

### Testing multicast manually

Send a test datagram:
```bash
python3 -c "
import socket, json, time
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 1)
sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_IF, socket.inet_aton('127.0.0.1'))
dg = {
    'timestamp': time.time(),
    'source': 'debug_probe',
    'kind': 'alert',
    'priority': 'high',
    'workspace': 'yggdrasil',
    'detail': 'Test datagram from debug probe',
}
sock.sendto(json.dumps(dg).encode() + b'\n', ('239.0.0.1', 9899))
print('Sent')
"
```

### Architecture constraint: two tasks, not select!

The multicast listener and lockfile monitor run as **two independent `tokio::spawn` tasks** sharing a channel via `sender.clone()`. `UnboundedSender::clone()` is an Arc refcount bump — this is the intended use pattern for mpsc.

**Do NOT refactor these into a single `tokio::select!` loop.** This was tried (2026-03-12) and broke multicast reception entirely. The two-task architecture is proven to work. The clone is correct.

---

## Hlidskjalf Core → Tauri → Frontend Bridge

```
hlidskjalf_core::start_all(sender)
    └── spawns listener task:   UDP recv → parse_event() → sender.send(datagram)
    └── spawns lockfile task:   interval check → sender.send(alert_datagram)

Tauri shell (lib.rs):
    let (sender, mut receiver) = unbounded_channel();
    start_all(sender).await?;
    tokio::spawn(async move {
        while let Some(datagram) = receiver.recv().await {
            handle.emit("datagram", &datagram);   // ← Tauri event to webview
        }
    });

Frontend (HlidskjalfView.svelte):
    listen<Datagram>("datagram", (event) => {
        const ev = event.payload;
        datagrams = [...datagrams.filter(d => d.timestamp > cutoff), ev];
        if (ev.speech && priority >= threshold) invoke("speak", { text: ev.speech });
    });
```

The Tauri event name is `"datagram"` (not `"hook-event"`). The Datagram struct is serialized via serde, so the frontend receives a plain JS object with the same field names.

---

## Yggdrasil vs Standalone Sync

Hlidskjalf runs in two contexts:
1. **Standalone** — `hlidskjalf/src-tauri/src/lib.rs` registers `start_monitor` and `speak`
2. **Yggdrasil** — `yggdrasil/src-tauri/src/lib.rs` registers `hlid_start_monitor` and `hlid_speak`

Both call the same `hlidskjalf_core` functions. The `commands` prop on `HlidskjalfView.svelte` maps bare names to prefixed names. If behavior diverges between standalone and Yggdrasil:
- Check that both Tauri shells call core functions identically
- Check that the `commands` prop mapping in `yggdrasil/src/routes/+page.svelte` is complete
- Check that Yggdrasil's Vite alias (`$hlidskjalf`) resolves to the same source as standalone

---

## Common Regressions

| Symptom | Likely Cause | Where to Look |
|---------|-------------|---------------|
| Nothing at all | Core crate doesn't compile, old binary running | `cargo check`, process start time vs binary mtime |
| Speech but no display | Frontend rendering bug | HlidskjalfView.svelte template/CSS |
| Alerts render, quality doesn't | QualityReport.svelte broken or payload shape mismatch | QualityReport.svelte, datagram payload structure |
| Works standalone, broken in Yggdrasil | Command prefix missing or Vite alias stale | yggdrasil lib.rs, vite.config.js |
| Works in Yggdrasil, broken standalone | Import path divergence ($lib/ vs ./) | View component imports |
| Datagrams in log but not in Hlidskjalf | Multicast transport broken | hlidskjalf_core listener, port binding |
| Stale process after deploy | macOS keeps old binary in memory | Quit app, relaunch from /Applications |

---

## Quick Diagnostic Checklist

```
1. Are datagrams being emitted?
   → tail ~/.ai/intercept/datagrams/datagrams_$(date +%Y-%m-%d).jsonl

2. Is Hlidskjalf listening?
   → lsof -i UDP:9899

3. Is it the current binary?
   → compare: ps start time vs ls -la binary mtime

4. Does the UI say "listening"?
   → yes = start_all() succeeded
   → no = start_all() failed or start_monitor never called

5. Does speech work?
   → yes = full data path works, bug is rendering only
   → no = bug is in core/transport/bridge

6. Which kinds render?
   → alerts but not quality = QualityReport.svelte or payload issue
   → nothing renders = feed rendering or event handler bug
```
