# HeXuS Architecture

## Overview

HeXuS follows a local-first, privacy-focused architecture. All biometric data stays on the user's machine.

```
┌─────────────────────────────────────────────────────────────┐
│                        HeXuS Core                           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Ingest    │  │   Analyze   │  │      Present        │  │
│  │             │  │             │  │                     │  │
│  │ • Wearables │  │ • Baselines │  │ • Dashboard         │  │
│  │ • CGM       │  │ • Anomaly   │  │ • Alerts            │  │
│  │ • Manual    │  │ • ML Model  │  │ • History           │  │
│  │ • Keyboard  │  │             │  │                     │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         │                │                    │             │
│         └────────────────┼────────────────────┘             │
│                          │                                  │
│                    ┌─────┴─────┐                            │
│                    │  Storage  │                            │
│                    │  (Local)  │                            │
│                    └───────────┘                            │
└─────────────────────────────────────────────────────────────┘
```

## Components

### Ingest Layer
Collects biometric data from various sources:
- **Wearable APIs** — Garmin, Fitbit, Apple Health (export), Oura, Whoop
- **CGM Integration** — Dexcom, Libre via local export
- **Manual Logging** — "Who's fronting right now?" journal entries
- **Passive Collection** — Typing patterns, mouse movement (opt-in)

### Analysis Layer
Processes collected data:
- **Baseline Establishment** — Per-alter physiological profiles
- **Anomaly Detection** — "This doesn't match current reported fronter"
- **Pattern Recognition** — Trends, triggers, time-of-day correlations
- **ML Classification** — After sufficient labeled data, predict fronting state

### Presentation Layer
Surfaces insights:
- **Dashboard** — Current state, recent history, system overview
- **Alerts** — Optional notifications for detected switches
- **History** — Timeline view, pattern exploration

### Storage
- **Local SQLite/DuckDB** — All data stays on-device
- **Encrypted at rest** — Biometric data is sensitive
- **Export format** — JSON/Parquet for backup and portability

## Data Flow

1. **Collection** — Wearable syncs or manual entry
2. **Normalization** — Convert to common schema
3. **Storage** — Persist with timestamp + optional fronter label
4. **Analysis** — Background processing for patterns
5. **Presentation** — Dashboard updates, alerts fire

## Privacy Principles

1. **Local-first** — No cloud services required
2. **Opt-in everything** — User chooses what to collect
3. **Encrypted storage** — Data protected at rest
4. **No telemetry** — What happens in HeXuS stays in HeXuS
5. **Exportable** — Your data, your control

## Future Considerations

- **Multi-device sync** — Local network only, encrypted
- **Knurv integration** — Share insights with AI embodiment layer
- **Community baselines** — Opt-in anonymized research contribution
