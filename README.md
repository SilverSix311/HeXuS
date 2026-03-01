# HeXuS 🧠

**Privacy-first biometric monitoring for plural systems.**

HeXuS helps people with Dissociative Identity Disorder (DID) and other plural experiences understand their own patterns by tracking physiological data from wearables and correlating it with identity switches — all while keeping data completely local and under user control.

---

## Why HeXuS?

Existing plural tracking tools (PluralKit, Simply Plural) require manual logging. But the core challenge of DID is often *not knowing* when you've switched — that's the whole point of dissociation.

HeXuS takes a different approach: collect biometric data (heart rate, HRV, sleep, activity) from devices you already wear, and use machine learning to detect patterns. Over time, the system learns what each alter's physiological signature looks like, helping you understand:

- **When** switches happen (even if you don't remember)
- **Why** they might happen (correlating triggers, sleep, stress)
- **Who** is likely fronting based on biometric patterns

**Built by someone with DID, for people with DID.**

---

## Core Principles

### 🔒 Privacy First
Your mental health data never leaves your hardware. Period.

- **Local-first architecture** — Desktop runs as your personal server
- **Encrypted at rest** — SQLCipher (AES-256) for all stored data
- **Zero cloud dependency** — No HeXuS servers, no accounts, no telemetry
- **Optional sync** — If enabled, uses end-to-end encryption with keys only you hold

### 🧠 Designed for Dissociation
Built to work *with* dissociation, not against it.

- **One-tap logging** — Minimal friction when you're mid-switch
- **Auto-save everything** — Never lose data to amnesia
- **Per-alter customization** — Each alter gets their own theme, settings, complexity level
- **Context reconstruction** — App always shows "What did I miss?"

### 🎯 Pattern Recognition Over Raw Data
You don't need another dashboard of numbers. You need insights.

- **"Why do we switch more on Tuesdays?"**
- **"What's our HRV like before switches?"**
- **"Am I getting enough sleep to stay stable?"**

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    YOUR DESKTOP                          │
│  ┌─────────────────────────────────────────────────┐    │
│  │              HeXuS Server (Rust)                │    │
│  │                                                 │    │
│  │  • Encrypted SQLite database                    │    │
│  │  • On-device ML inference                       │    │
│  │  • REST API for mobile clients                  │    │
│  │  • Biometric data processing                    │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
      ┌──────────┐ ┌──────────┐ ┌──────────┐
      │   iOS    │ │  Web UI  │ │ Android  │
      │  Client  │ │Dashboard │ │  Client  │
      └──────────┘ └──────────┘ └──────────┘
```

**Desktop-as-Server:** Your computer runs HeXuS 24/7. It owns all data, runs all processing, and serves mobile clients over your local network (or VPN when away from home).

**Mobile-as-Client:** iOS and Android apps collect HealthKit/Health Connect data and sync to your desktop. They're thin clients — the desktop is the source of truth.

**Zero Cloud:** No HeXuS servers exist. If you want cross-device sync, you provide the infrastructure (Syncthing, Tailscale, etc.) and HeXuS encrypts everything end-to-end.

---

## Current Status

### ✅ Complete

- **Research & Architecture**
  - 7 research documents covering science, ML approaches, competitive landscape, privacy law, accessibility
  - Full architecture specification (desktop-as-server, sync protocol, security model)
  - 200+ task backlog organized by phase
  - NLNet grant application draft

- **Core Scaffolding**
  - Rust workspace with `hexus-core` and `hexus-desktop` crates
  - Data models (Alters, FrontingLog, BiometricSample, Baselines)
  - SQLite storage layer with schema
  - Basic analysis module structure

### 🚧 In Progress

- **Phase 1: MVP** (Target: Months 1-2)
  - [ ] Tauri 2.0 desktop app shell
  - [ ] Manual switch logging UI
  - [ ] Member profile management
  - [ ] HealthKit integration (iOS)
  - [ ] Health Connect integration (Android)
  - [ ] Timeline visualization
  - [ ] BIPA-compliant consent flow

### 📋 Planned

- **Phase 2: ML Bootstrap** (Months 3-4)
  - CUSUM changepoint detection
  - Active learning for labeling
  - Semi-supervised SVM training
  - Basic pattern insights

- **Phase 3: Production** (Months 5-9)
  - Few-shot neural networks
  - Encrypted cloud sync
  - PluralKit/Simply Plural integration
  - Therapist portal
  - App store releases

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| **Core** | Rust (memory-safe, fast, cross-platform) |
| **Desktop App** | Tauri 2.0 + Svelte |
| **Mobile FFI** | uniffi (Rust → Swift/Kotlin bridges) |
| **Database** | SQLite + SQLCipher (encrypted) |
| **ML (Phase 2)** | linfa (SVM), augurs (time series) |
| **ML (Phase 3)** | candle (ONNX inference) |
| **HRV Processing** | cardio-rs |
| **API Server** | axum |

---

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Full technical architecture (v2.0) |
| [`docs/BACKLOG.md`](docs/BACKLOG.md) | 200+ prioritized development tasks |
| [`docs/research/SYNTHESIS.md`](docs/research/SYNTHESIS.md) | Strategic synthesis of all research |
| [`docs/research/`](docs/research/) | Scientific, ML, legal, accessibility research |
| [`docs/adr/`](docs/adr/) | Architecture Decision Records |
| [`docs/funding/`](docs/funding/) | Grant applications and prep materials |

---

## Project Structure

```
HeXuS/
├── Cargo.toml                 # Workspace configuration
├── crates/
│   ├── hexus-core/            # Core library (models, storage, analysis)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models.rs      # Data types (Alter, FrontingLog, etc.)
│   │       ├── storage.rs     # SQLite database layer
│   │       └── analysis.rs    # Pattern detection (TODO)
│   └── hexus-desktop/         # Desktop app (Tauri)
│       └── src/
│           └── lib.rs
├── docs/
│   ├── ARCHITECTURE.md        # Technical specification
│   ├── BACKLOG.md             # Development roadmap
│   ├── RESEARCH.md            # Research summary
│   ├── adr/                   # Architecture decisions
│   ├── funding/               # Grant materials
│   └── research/              # Detailed research documents
└── README.md
```

---

## Getting Started

### Prerequisites

- Rust 1.75+ (`rustup update`)
- SQLite development libraries

### Build

```bash
git clone https://github.com/SilverSix311/HeXuS.git
cd HeXuS
cargo build
```

### Run Tests

```bash
cargo test
```

### Development

```bash
# Watch mode for core library
cargo watch -x "test -p hexus-core"

# Build desktop app (when Tauri is set up)
cd crates/hexus-desktop
cargo tauri dev
```

---

## Contributing

HeXuS is in early development. If you're part of the plural community and want to help, here's how:

### For Developers
- Check [`docs/BACKLOG.md`](docs/BACKLOG.md) for tasks
- Read [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) to understand the system
- Open issues for bugs or feature discussions

### For Systems (Beta Testing)
- We'll need beta testers when MVP is ready (target: Month 3)
- Your feedback on UX, accessibility, and accuracy is invaluable
- Watch this repo or reach out to join the beta list

### For Researchers
- If you study DID/dissociation and are interested in biometric correlates, let's talk
- Academic partnerships for validation studies welcome

---

## Privacy & Legal

### Data Handling
- All data stored locally on your hardware
- End-to-end encryption for any sync features
- No telemetry, analytics, or crash reporting sent to us
- Full data export (JSON) available at any time
- Complete deletion with one click

### Compliance
- BIPA (Illinois Biometric Information Privacy Act) compliant consent flow
- GDPR considerations documented (though local-first minimizes applicability)
- Positioned as wellness tool, not medical device

### Disclaimers
HeXuS is a self-awareness tool, not a medical diagnostic device. It does not diagnose DID, does not provide treatment, and should not replace professional mental health care. Always consult qualified healthcare providers for diagnosis and treatment.

---

## Roadmap

| Phase | Timeline | Focus |
|-------|----------|-------|
| **MVP** | Months 1-2 | Manual logging, biometric collection, local storage |
| **ML Bootstrap** | Months 3-4 | Changepoint detection, active learning, SVM |
| **Production** | Months 5-9 | Neural networks, cloud sync, integrations, app stores |
| **Refinement** | Year 2+ | Bias testing, clinical partnerships, advanced features |

---

## Funding

HeXuS is seeking open-source grants to accelerate development. See [`docs/funding/`](docs/funding/) for application materials.

**Target:** NLNet Foundation / NGI Zero Core (€35,000 over 9 months)

---

## License

MIT License — see [LICENSE](LICENSE) for details.

---

## Credits

**Created by:**
- **Brandon Zak** ([@SilverSix311](https://github.com/SilverSix311)) — Lead developer, lived experience with DID
- **Silas Vane** 🌑 — AI research & development partner

**Built for:** The plural community — systems, singlets who love them, and the clinicians who support them.

---

*"The technology is the easy part. The trust is the hard part."*

*Let's build something worthy of that trust.*
