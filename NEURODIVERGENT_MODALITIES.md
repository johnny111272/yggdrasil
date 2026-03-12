# Neurodivergent Alert Modalities

## Purpose

Design specification for Hlidskjalf's multi-modal alert system. This system is designed for a neurodivergent user whose sensory thresholds and attention needs change throughout the day and from day to day. The core principle: **awareness without overload**. Multiple alert channels at different intrusiveness levels, independently controllable, switchable with one click as sensory state changes.

## The Problem

Audio alerts are helpful — to a point. In hyperfocus mode, speech is the only thing that can break through. But near sensory overload, a spoken alert can be the thing that tips the balance. Muting audio doesn't mean you don't need to know what's happening — it means you need a different channel.

A single alert threshold is wrong by design. It assumes your sensory capacity is constant. It isn't. The system must adapt to you, not the other way around.

---

## Alert Modalities

Five modalities ordered from least to most intrusive. Each serves a different attention state and sensory channel.

### 1. LED Indicator

**Intrusiveness: minimal. Channel: peripheral vision.**

A small dot or indicator area in the header bar, one per active session. Blinks in the severity color of the most recent event from that session.

| Severity | LED Color |
|----------|-----------|
| critical | Red blink |
| high | Amber blink |
| normal | Blue blink |
| low | Dim pulse |

Always active — it's so subtle it doesn't need a threshold. This is the ambient "something is happening" channel. You're focused on your terminal, but in the corner of your eye a red dot starts blinking. You know without looking.

### 2. Click / Tap Sound

**Intrusiveness: low. Channel: ambient audio.**

A brief, non-verbal audio cue. A single click, tap, or soft tone. Not speech — no words, no semantic content to process. Just enough to pull attention from another window or task.

**Geiger counter mode**: In monitoring profile, clicks fire at the rate of incoming activity across all sessions. The rate IS the information:

- Steady slow ticking → sessions active, progressing normally
- Rapid ticking → burst of activity, something ramping up
- Silence → nothing happening, or sessions finished

Each workspace gets a distinct click timbre — a slightly different pitch or tone color. You can hear "that's bragi ticking, phoenix is quiet" without looking. The audio signal matches the visual pulse rate of the workspace chips — same information, two channels. Your brain picks up whichever one it's tuned to right now.

The geiger counter is an **activity rate** channel, separate from severity-gated alerts. It tells you sessions are alive and how busy they are. It runs continuously in monitoring mode regardless of event priority.

### 3. Color Bar Pulse

**Intrusiveness: medium. Channel: peripheral vision (larger stimulus).**

A thin bar at the top or bottom of the Hlidskjalf window that briefly pulses in the severity color. One pulse, then fade. Not persistent blinking — a single attention-getting flash that subsides.

More visually demanding than the LED but still ambient. Effective when you're near Hlidskjalf but focused on adjacent content. The pulse enters your visual field, you glance over, you see the event.

Severity colors match the LED: red for critical, amber for high, blue for normal.

### 4. Screen Flash

**Intrusiveness: high. Channel: direct vision.**

Brief flash of the entire Hlidskjalf window background. Hard to miss even in peripheral vision. The "something just went very wrong" signal.

Reserved for critical events by default. This is the visual equivalent of someone tapping your shoulder. Use sparingly — if this fires often, the threshold is wrong.

### 5. Speech

**Intrusiveness: highest. Channel: direct audio with semantic content.**

macOS `say` command with voice synthesis. Full verbal announcement of the event. The most demanding modality — it requires auditory processing of language, which competes with whatever you're thinking about.

In hyperfocus mode, this is exactly what you need — nothing else breaks through deep concentration. In any other state, it can be overwhelming. That's why it has its own independent threshold.

The `speech` field on the datagram controls what gets spoken. If absent, Hlidskjalf can generate from the event kind and detail.

---

## Independent Thresholds

Each modality has its own severity threshold. They are completely independent of each other and independent of the display filter (what you see in the feed).

| Control | What it gates | Independent of |
|---------|--------------|----------------|
| Priority minimum (filter bar) | What events appear in the feed | All alert modalities |
| LED threshold | What events blink the LED | Everything else |
| Click threshold | What events produce a click sound | Everything else |
| Pulse bar threshold | What events pulse the color bar | Everything else |
| Flash threshold | What events flash the screen | Everything else |
| Speech threshold | What events trigger spoken alerts | Everything else |

**"Just because I am in sensory overload does not mean I don't want to see everything important."** The display filter and the alert modalities are separate concerns. You might see all normal+ events in the feed but hear nothing. Or see only alerts but have the geiger counter ticking for ambient awareness.

---

## Alert Profiles

Adjusting five individual thresholds every time your sensory state changes is too much friction. Profiles bundle the settings into named presets switchable with one click.

### Default Profiles

| Profile | LED | Click | Pulse bar | Flash | Speech | When to use |
|---------|-----|-------|-----------|-------|--------|-------------|
| **hyperfocus** | always | off | off | off | high+ | Deep work. Only speech breaks through. |
| **sensitive** | always | off | critical | off | off | Near overload. Minimal stimulation. A brief pulse for critical only. |
| **monitoring** | always | geiger | critical | off | off | Tracking side processes. Ambient ticking for activity, pulse for critical. |
| **active** | always | high+ | high+ | critical | critical | Actively watching. Full awareness at moderate intrusiveness. |
| **silent** | always | off | off | off | off | LED only. Complete quiet. |

### Profile Switching

The filter bar shows a bell icon with the current profile name:

```
[🔔 monitoring ▾]
```

Click to open a popover:

```
┌──────────────────────────────────┐
│  ● hyperfocus                    │
│  ● sensitive                     │
│  ● monitoring          ← active  │
│  ● active                        │
│  ● silent                        │
│  ─────────────────────           │
│  [customize...]                  │
│                                  │
│  [mute all]    [restore]         │
└──────────────────────────────────┘
```

- **Click a profile** → switches immediately, popover closes
- **Customize** → expands to show the five modality thresholds for the current profile, editable
- **Mute all** → instant silence. All audio stops, all visual alerts except LED stop. One-click panic button. The previous profile is remembered.
- **Restore** → undoes mute, returns to the previous profile and all its settings.

### Custom Profiles

The four defaults are starting points. Customize opens the individual threshold controls:

```
┌──────────────────────────────────┐
│  Editing: monitoring             │
│                                  │
│  LED        [always]             │
│  Click      [geiger ▾]          │
│  Pulse bar  [critical ▾]        │
│  Flash      [off ▾]             │
│  Speech     [off ▾]             │
│                                  │
│  [save]  [save as new...]        │
└──────────────────────────────────┘
```

Threshold options for each modality: `off | critical | high | normal | low | trace`

Click has an additional option: `geiger` — enables ambient activity-rate ticking instead of severity-gated clicks.

Profiles persist across Hlidskjalf sessions (saved to a local config file).

---

## Ambient Awareness System

Beyond severity-gated alerts, there are two ambient awareness channels that run continuously.

### Workspace Chip Pulsing

The session chips in the filter bar are living indicators. Their pulse rate corresponds to the datagram arrival rate from that session:

| Chip state | Visual | What it means |
|-----------|--------|---------------|
| Busy | Pulsing/breathing, rate ~ activity | Tool uses, exchanges, events flowing |
| Quiet | Solid, still | Session exists, nothing happening |
| Critical | Brief bright flash | Critical event just fired |
| Inactive | Dimmed | No activity for sustained period |

Every datagram kind contributes to the pulse rate: tool use events, exchange diffs, alerts, reports. Canaries confirm the session is alive but don't contribute to pulse rate (they're infrastructure, not activity).

The visual pulse rate and the geiger tick rate express the **same signal in two modalities**. Glance at the bar — bragi's chip is pulsing quickly, phoenix is still. Listen without looking — bragi is ticking steadily, phoenix is silent. Same information, two sensory channels. Your brain uses whichever one it's tuned to.

### Geiger Counter Audio

In profiles where click is set to "geiger", a soft tick sound fires when datagrams arrive. The tick rate is the activity rate.

- Each workspace gets a **distinct timbre** — different pitch or tone color
- You can identify which session is active by sound alone
- Rate modulation: ticks are rate-limited to avoid overwhelming (e.g. at most N ticks per second even during burst activity)
- The ticking stops when no datagrams arrive — silence means quiet sessions

The geiger counter is specifically for **background monitoring**. You're working in your terminal. You hear bragi ticking along steadily. Phoenix starts ticking faster. You know something is happening without context-switching. If something critical occurs, the severity-gated alert (pulse bar, speech, whatever your profile has) fires on top of the ambient ticking.

---

## Layered Awareness Summary

The full system provides awareness at four layers, each serving a different attention state:

| Layer | What it tells you | Modality | Intrusiveness |
|-------|-------------------|----------|---------------|
| **Ambient visual** | Which sessions are active, how busy | Chip pulse rate + LED dots | Minimal — peripheral vision |
| **Ambient audio** | Same, without looking | Geiger ticking, timbre per workspace | Low — background audio |
| **Event alerts** | Something specific needs attention | Pulse bar, flash, click | Medium — profile-gated |
| **Speech alerts** | Something critical, break through focus | macOS `say` | High — direct audio |

Each layer is independently controllable. All four can be active simultaneously or any subset. The profile system bundles them into one-click presets that match your current sensory state.

The design principle: **you are always aware, at the level of intrusiveness you can handle right now.**

---

## Implementation Notes

### Audio Backend

- Speech: existing `hlidskjalf_core::speak()` using macOS `say`
- Click/tap sounds: short audio samples played via a lightweight audio API (Web Audio API in the Svelte frontend, or a Tauri command that plays a system sound)
- Geiger ticking: same mechanism as click, but rate-controlled by datagram arrival

### Visual Effects

- LED dots: CSS animations (blink keyframes) on small indicator elements in the header
- Chip pulsing: CSS animation with dynamic animation-duration based on activity rate
- Color bar pulse: CSS animation on a thin bar element, triggered by JS, single pulse then fade
- Screen flash: brief CSS class toggle on the root element with background-color transition

### Profile Storage

Profiles saved as JSON in `~/.ai/hlidskjalf/alert_profiles.json` or equivalent. Loaded on startup. The active profile name persists across sessions.

### Tauri Commands

New commands needed:
- `hlid_play_sound(sound_type: String)` — play a click, tap, or custom sound
- `hlid_set_alert_profile(profile: String)` — switch profile, return current settings
- `hlid_get_alert_profiles()` — load all profiles

Or these could be purely frontend (Web Audio + localStorage) with no Rust backend needed. The speech command already exists.
