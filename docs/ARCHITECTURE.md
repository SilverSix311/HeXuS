# HeXuS Architecture v2.0

## Overview

HeXuS is a privacy-first, local-first biometric monitoring system for plural/DID systems. Unlike typical cloud-based health apps, HeXuS operates on a **desktop-as-server, mobile-as-client** model where all data lives on the user's personal hardware with zero cloud dependency.

**Core Principles:**
- **Privacy First:** Your data never leaves your hardware unless you explicitly choose to export it
- **Local First:** Desktop app acts as personal server; mobile apps are clients
- **Zero Cloud:** No HeXuS cloud services, no remote servers holding your data
- **User Control:** Complete ownership of data, sync, and access

---

## Table of Contents

1. [System Architecture](#system-architecture)
2. [Component Details](#component-details)
3. [Data Flow](#data-flow)
4. [Network Architecture](#network-architecture)
5. [Security Architecture](#security-architecture)
6. [Sync Protocol](#sync-protocol)
7. [Platform-Specific Integration](#platform-specific-integration)
8. [Machine Learning Pipeline](#machine-learning-pipeline)
9. [Privacy Architecture](#privacy-architecture)
10. [Technology Stack](#technology-stack)
11. [Deployment Model](#deployment-model)

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Home Network                                  │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │           HeXuS Desktop Server (Linux/Mac/Windows)            │  │
│  │                                                                │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌──────────────────────┐  │  │
│  │  │  Ingest     │  │  Analysis   │  │   Storage            │  │  │
│  │  │             │  │             │  │                      │  │  │
│  │  │ • Wearables │  │ • HRV Calc  │  │ • SQLite/DuckDB      │  │  │
│  │  │ • CGM       │  │ • Baseline  │  │ • Time Series Data   │  │  │
│  │  │ • Manual    │  │ • Pattern   │  │ • Encrypted at Rest  │  │  │
│  │  │ • Mobile    │  │ • ML Model  │  │ • Per-Alter Profiles │  │  │
│  │  │   Sync      │  │ • Anomaly   │  │                      │  │  │
│  │  └──────┬──────┘  └──────┬──────┘  └──────────┬───────────┘  │  │
│  │         │                │                    │              │  │
│  │         └────────────────┼────────────────────┘              │  │
│  │                          │                                   │  │
│  │                    ┌─────┴──────┐                            │  │
│  │                    │ Local API  │                            │  │
│  │                    │ (REST/WS)  │                            │  │
│  │                    └─────┬──────┘                            │  │
│  └──────────────────────────┼────────────────────────────────────┘  │
│                             │                                        │
│              ┌──────────────┼──────────────┐                         │
│              ▼              ▼              ▼                         │
│        ┌──────────┐   ┌──────────┐   ┌──────────┐                   │
│        │ iPhone   │   │  iPad    │   │  Android │                   │
│        │  Client  │   │  Client  │   │  Client  │                   │
│        └──────────┘   └──────────┘   └──────────┘                   │
└─────────────────────────────────────────────────────────────────────┘
                             │
                             │ (Optional Remote Access)
                             │
                    ┌────────┴─────────┐
                    │  Tailscale/      │
                    │  WireGuard VPN   │
                    └────────┬─────────┘
                             │
                    ┌────────┴─────────┐
                    │  Mobile Away     │
                    │  From Home       │
                    └──────────────────┘
```

### Architecture Philosophy

1. **Desktop as Server:**
   - Always-on personal server running on user's computer
   - Owns all data, runs all processing
   - No external dependencies for core functionality
   - Acts as single source of truth

2. **Mobile as Client:**
   - Lightweight apps for iOS/Android
   - Connect directly to desktop server over local network
   - Cache recent data for offline viewing
   - Sync bidirectionally when connected

3. **Network Topology:**
   - **At Home:** Direct LAN connection (mDNS discovery)
   - **Away:** Tailscale/WireGuard VPN (peer-to-peer encrypted tunnel)
   - **Offline:** Mobile caches show last synced state (read-only)

4. **Zero Cloud Dependency:**
   - No HeXuS-operated servers
   - No user accounts or authentication with HeXuS
   - No telemetry, no tracking, no analytics sent to us
   - User may optionally use their own cloud storage for backups (encrypted)

---

## Component Details

### Desktop Server Components

#### 1. **Ingest Layer**

**Wearable Integration:**
- **Direct API Polling:**
  - Oura Ring API (OAuth2, 5-min intervals)
  - Whoop API (OAuth2 + webhooks)
  - Dexcom CGM API (OAuth2, continuous glucose)
  - Garmin Connect API (if business approval obtained)

- **Local Data Import:**
  - Apple Health export (XML/JSON)
  - Google Fit export (JSON)
  - Fitbit export (JSON)
  - CSV/JSON manual import

**Mobile Sync Receiver:**
- REST API endpoint for mobile clients to push:
  - HealthKit data (iOS)
  - Health Connect data (Android)
  - Manual switch logs
  - Alter state updates

**Manual Entry Interface:**
- Web UI (served by desktop server)
- Switch logging form
- Fronter identification
- Mood/trigger notes
- Timestamped events

**Implementation:**
```rust
// Rust modules
mod ingest {
    pub mod wearables;    // API clients for Oura/Whoop/etc
    pub mod mobile_sync;  // REST endpoints for mobile push
    pub mod manual;       // Web form handlers
    pub mod importers;    // CSV/JSON/XML parsers
}
```

---

#### 2. **Analysis Layer**

**Real-Time Processing:**
- **HRV Calculation:** `cardio-rs` crate for time-domain and frequency-domain HRV metrics
- **Changepoint Detection:** CUSUM algorithm for detecting physiological shifts
- **Outlier Detection:** `augurs-outlier` for anomaly flagging

**Pattern Recognition:**
- **Time Series Analysis:** `augurs` and `oxidiviner` crates
  - Seasonal decomposition (daily/weekly patterns)
  - ARIMA forecasting (switch prediction)
  - Correlation analysis (biometrics ↔ switches)

**Machine Learning Pipeline:**
- **Phase 1 (MVP):** Rule-based + CUSUM changepoint detection
- **Phase 2:** Semi-supervised SVM with active learning (`linfa`)
- **Phase 3:** Few-shot prototypical networks (PyTorch → ONNX → `candle`)

**Baseline Management:**
- Per-alter physiological profiles
- Statistical baselines (mean, std, HRV ranges)
- Adaptive baselines (rolling window, exponential smoothing)

**Implementation:**
```rust
mod analysis {
    pub mod hrv;          // cardio-rs wrapper
    pub mod changepoint;  // CUSUM implementation
    pub mod patterns;     // augurs/oxidiviner time series
    pub mod ml;           // Inference engine (candle/linfa)
    pub mod baselines;    // Per-alter profile management
}
```

---

#### 3. **Storage Layer**

**Database:**
- **Primary:** SQLite with SQLCipher (encrypted at rest)
- **Alternative:** DuckDB for analytical queries (embedded, fast)

**Schema Design:**

```sql
-- Alters/Members
CREATE TABLE alters (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    pronouns TEXT,
    description TEXT,
    avatar_url TEXT,
    color TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Fronting History
CREATE TABLE switches (
    id TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    alter_ids TEXT NOT NULL, -- JSON array for co-fronting
    co_conscious BOOLEAN DEFAULT FALSE,
    notes TEXT,
    trigger TEXT,
    created_at INTEGER NOT NULL
);

-- Biometric Time Series (normalized schema)
CREATE TABLE biometric_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    source TEXT NOT NULL,        -- 'healthkit', 'oura', 'whoop', 'manual'
    metric_type TEXT NOT NULL,    -- 'heart_rate', 'hrv', 'sleep', 'glucose'
    value REAL NOT NULL,
    unit TEXT NOT NULL,           -- 'bpm', 'ms', 'mg/dL'
    metadata TEXT,                -- JSON (device info, confidence, etc.)
    INDEX idx_timestamp (timestamp),
    INDEX idx_metric_type (metric_type)
);

-- ML Model Metadata
CREATE TABLE ml_models (
    id TEXT PRIMARY KEY,
    model_type TEXT NOT NULL,     -- 'svm', 'prototypical', 'changepoint'
    version TEXT NOT NULL,
    trained_at INTEGER NOT NULL,
    accuracy REAL,
    hyperparameters TEXT,         -- JSON
    file_path TEXT NOT NULL       -- Path to serialized model
);

-- Per-Alter Baselines
CREATE TABLE baselines (
    id TEXT PRIMARY KEY,
    alter_id TEXT NOT NULL,
    metric_type TEXT NOT NULL,
    mean REAL,
    std REAL,
    percentile_5 REAL,
    percentile_95 REAL,
    sample_count INTEGER,
    last_updated INTEGER NOT NULL,
    FOREIGN KEY (alter_id) REFERENCES alters(id),
    UNIQUE(alter_id, metric_type)
);
```

**File Storage:**
- ML models: `~/.hexus/models/` (ONNX, pickle, custom formats)
- Exports: `~/.hexus/exports/` (JSON, CSV, Parquet)
- Logs: `~/.hexus/logs/` (application logs)

**Encryption:**
- SQLite database encrypted with SQLCipher (AES-256)
- Key derived from user's master password (scrypt KDF)
- Alternative: OS keychain integration (macOS Keychain, Windows Credential Manager, Linux Secret Service)

**Implementation:**
```rust
mod storage {
    pub mod db;           // sqlx wrapper for SQLite/DuckDB
    pub mod encryption;   // SQLCipher integration
    pub mod migrations;   // Schema versioning
    pub mod exports;      // JSON/CSV/Parquet exporters
}
```

---

#### 4. **Local API Server**

**Technology:** Rust `axum` web framework

**Endpoints:**

**Alters Management:**
- `GET /api/v1/alters` — List all alters
- `POST /api/v1/alters` — Create new alter
- `GET /api/v1/alters/:id` — Get alter details
- `PATCH /api/v1/alters/:id` — Update alter
- `DELETE /api/v1/alters/:id` — Delete alter

**Switch Tracking:**
- `GET /api/v1/switches` — List switch history (with pagination)
- `POST /api/v1/switches` — Log new switch
- `GET /api/v1/switches/:id` — Get switch details
- `PATCH /api/v1/switches/:id` — Update switch
- `DELETE /api/v1/switches/:id` — Delete switch

**Biometric Data:**
- `POST /api/v1/biometrics/batch` — Bulk upload (from mobile)
- `GET /api/v1/biometrics/query` — Query time series (filters: time range, metric type)
- `GET /api/v1/biometrics/latest` — Latest readings

**Patterns & Insights:**
- `GET /api/v1/patterns/correlations` — Biometric ↔ switch correlations
- `GET /api/v1/patterns/outliers` — Recent anomalies
- `GET /api/v1/patterns/forecast` — Switch likelihood forecast

**Sync:**
- `GET /api/v1/sync/status` — Server status, last sync time
- `POST /api/v1/sync/mobile` — Mobile client sync (bidirectional)

**WebSocket:**
- `WS /api/v1/ws` — Real-time updates (switch events, biometric alerts)

**Authentication:**
- **Local network:** Optional (trust LAN devices)
- **Remote (VPN):** API key or JWT token
- **Alternative:** mTLS (mutual TLS with client certificates)

**Implementation:**
```rust
mod api {
    pub mod routes;       // axum route handlers
    pub mod auth;         // API key / JWT middleware
    pub mod websocket;    // WS connection manager
    pub mod models;       // Request/response DTOs
}
```

---

#### 5. **Web Dashboard (Desktop)**

**Technology:** Tauri 2.0 (Rust backend + web frontend)

**Frontend:**
- **Framework:** Svelte or React
- **Charts:** Chart.js or D3.js for time series visualization
- **UI Library:** shadcn/ui or Tailwind CSS

**Views:**
- **Dashboard:** Current fronter, recent switches, latest biometrics
- **Timeline:** Combined view of switches + HRV + sleep + other metrics
- **Alters:** Manage alter profiles, view per-alter statistics
- **Patterns:** Correlation heatmaps, outlier alerts, forecasts
- **Settings:** Configure data sources, ML models, sync, export
- **Sync:** Mobile device pairing, status, logs

**Capabilities:**
- Configure wearable API connections (OAuth flows)
- View/edit switch logs
- Export data (JSON, CSV, PDF reports)
- Train/update ML models
- Pair mobile devices (generate API keys)

**Implementation:**
```
desktop/
├── src-tauri/         # Rust backend (Tauri)
│   ├── main.rs        # Tauri app initialization
│   └── commands.rs    # IPC commands (called from frontend)
├── src/               # Frontend (Svelte/React)
│   ├── App.svelte
│   ├── routes/
│   ├── components/
│   └── lib/
└── tauri.conf.json    # Tauri configuration
```

---

### Mobile Client Components (iOS/Android)

#### 1. **Platform Data Collection**

**iOS (Swift):**
- HealthKit integration:
  - Request permissions for heart rate, HRV, sleep, activity
  - Background queries with HKAnchoredObjectQuery
  - Real-time updates via HKObserverQuery
- Data export to Rust core via `uniffi` FFI

**Android (Kotlin):**
- Health Connect integration:
  - Request permissions per data type
  - Read records with time range filters
  - Background sync with WorkManager
- Data export to Rust core via `uniffi` FFI

**Implementation:**
```
mobile/
├── ios/
│   ├── HeXuS/
│   │   ├── HealthKitManager.swift      # HealthKit API wrapper
│   │   └── RustBridge.swift            # uniffi-generated bindings
│   └── HeXuS.xcodeproj
├── android/
│   ├── app/
│   │   └── src/main/java/
│   │       ├── HealthConnectManager.kt # Health Connect wrapper
│   │       └── RustBridge.kt           # uniffi-generated bindings
│   └── build.gradle
└── rust-core/                          # Shared Rust logic
    ├── lib.rs
    └── uniffi.udl                      # FFI interface definition
```

---

#### 2. **Rust Core (Shared Logic)**

**Via `uniffi` FFI:**
- Biometric data normalization
- Local caching (SQLite embedded in mobile app)
- Sync queue management (pending uploads)
- Encryption/decryption for sync payloads

**Capabilities:**
- Process HealthKit/Health Connect data into common format
- Store recent data locally (SQLite)
- Batch uploads to desktop server when connected
- Handle offline mode gracefully

**Example uniffi API:**
```rust
#[uniffi::export]
fn process_heart_rate_samples(samples: Vec<HeartRateSample>) -> HrvMetrics {
    // Rust HRV calculation logic
}

#[uniffi::export]
fn sync_to_desktop(server_url: String, api_key: String, data: SyncPayload) -> Result<SyncStatus> {
    // REST API call to desktop server
}
```

---

#### 3. **Mobile UI**

**Technology:** Tauri 2.0 mobile (same codebase as desktop)

**Views:**
- **Dashboard:** Current fronter, quick switch logging
- **Timeline:** Recent switches + biometrics (cached from desktop)
- **Switch Log:** Log new switch with timestamp, alter, notes
- **Sync Status:** Connection to desktop server, pending uploads

**Offline Capabilities:**
- View cached timeline (last synced data)
- Log switches locally (queue for sync)
- Browse alter profiles
- Read-only mode (no biometric data without desktop connection)

---

## Data Flow

### 1. Biometric Data Collection

```
[Wearable Device]
       │
       ▼
[Device API] ────── (polling/webhooks) ────── [Desktop Server Ingest]
       │                                              │
       │                                              ▼
       │                                      [Normalize & Store]
       │                                              │
       │                                              ▼
       │                                      [SQLite Database]
       │
       ▼
[HealthKit (iOS)]  ────── (background query) ────── [Mobile App]
       │                                                  │
       │                                                  ▼
       │                                          [Rust Core (uniffi)]
       │                                                  │
       │                                                  ▼
       │                                          [Batch to Desktop]
       │
       ▼
[Health Connect (Android)] ── (read records) ── [Mobile App] ── [Desktop]
```

**Flow Steps:**

1. **Desktop Direct Collection:**
   - Desktop server polls Oura/Whoop APIs every 5-15 minutes
   - Data normalized to common schema
   - Stored in SQLite with timestamp, source, metadata

2. **Mobile Collection:**
   - iOS/Android background tasks fetch HealthKit/Health Connect data
   - Rust core (via uniffi) processes samples
   - Stored in mobile SQLite cache
   - Batched upload to desktop server when connected

3. **Manual Entry:**
   - User logs switch via mobile app or desktop dashboard
   - Immediately synced to desktop (if connected)
   - Queued for later sync (if offline)

---

### 2. Analysis Pipeline

```
[Biometric Database]
       │
       ▼
[Analysis Engine]
       │
       ├─── [HRV Calculation] ─── (cardio-rs)
       │
       ├─── [Changepoint Detection] ─── (CUSUM)
       │
       ├─── [Pattern Analysis] ─── (augurs, oxidiviner)
       │
       └─── [ML Inference] ─── (candle/linfa)
       │
       ▼
[Insights Database]
       │
       ├─── Outliers
       ├─── Correlations
       ├─── Forecasts
       └─── Switch Predictions
       │
       ▼
[Dashboard / Mobile UI]
```

**Triggered Events:**

- **Continuous:** HRV calculation on new heart rate data
- **Every 15 min:** Changepoint detection scan
- **Hourly:** Pattern analysis updates
- **Daily:** Baseline recalculation
- **On-demand:** ML model retraining (user-initiated)

---

### 3. Sync Flow (Mobile ↔ Desktop)

```
[Mobile Device]                           [Desktop Server]
       │                                          │
       │─── (1) Discovery (mDNS/Tailscale) ──────│
       │◄────── (2) Server Info Response ────────│
       │                                          │
       │─── (3) Authenticate (API Key) ───────────│
       │◄────── (4) Auth Success + Capabilities ─│
       │                                          │
       │─── (5) Request Sync ─────────────────────│
       │         (last_sync_timestamp)            │
       │◄────── (6) Delta Response ───────────────│
       │         (new switches, biometrics)       │
       │                                          │
       │─── (7) Upload Mobile Data ───────────────│
       │         (HealthKit/Health Connect batch) │
       │◄────── (8) ACK + Conflicts ──────────────│
       │                                          │
       │─── (9) Resolve Conflicts ────────────────│
       │◄────── (10) Sync Complete ───────────────│
       │                                          │
       │◄══════ (11) WebSocket (Real-Time) ══════►│
               (live updates while connected)
```

**Conflict Resolution:**

- **Last-write-wins** for switch logs (timestamped)
- **Merge** for biometric data (append-only)
- **User prompt** for alter profile conflicts

---

## Network Architecture

### Local Network (At Home)

**Discovery:**
- **mDNS (Bonjour/Avahi):** Desktop server advertises as `_hexus._tcp.local`
- Mobile apps auto-discover server on LAN
- No configuration needed

**Connection:**
- Direct HTTP/HTTPS connection to desktop server IP
- Optional TLS with self-signed certificate (pin on first use)
- API key authentication

**Ports:**
- `8080` — HTTP API (local only, optional)
- `8443` — HTTPS API (recommended)
- `8444` — WebSocket (real-time updates)

---

### Remote Access (Away from Home)

**Tailscale/WireGuard VPN:**

**Option 1: Tailscale (Recommended for Ease)**
- Install Tailscale on desktop + mobile devices
- Desktop joins Tailscale network
- Mobile devices access desktop via Tailscale IP (e.g., `100.64.x.x`)
- Encrypted tunnel, NAT traversal, zero-config

**Option 2: WireGuard (Advanced Users)**
- User self-hosts WireGuard server on desktop
- Mobile devices connect via WireGuard client
- Full control, no third-party (but requires setup)

**Benefits:**
- End-to-end encrypted tunnel
- Works behind NATs/firewalls
- No port forwarding needed
- No HeXuS relay servers

**Limitations:**
- Desktop must be powered on
- Requires VPN configuration (Tailscale simplifies)
- Mobile battery usage (VPN overhead)

---

### No Cloud Fallback

**What if desktop is offline?**
- Mobile shows cached data (read-only)
- Switch logs queue locally (sync when desktop returns)
- No biometric updates (source of truth is desktop)
- User can export mobile cache as backup

**What if mobile is offline?**
- Desktop continues collecting wearable data
- Switch logs only via desktop dashboard or later mobile sync
- No impact on desktop functionality

---

## Security Architecture

### Threat Model

**Actors:**
1. **Curious Household Member:** Physical access to desktop computer
2. **Network Snooper:** LAN eavesdropper or public WiFi attacker
3. **Malicious App:** Compromised mobile app trying to access HeXuS data
4. **Remote Attacker:** Internet-based attacker (minimal surface area)

**Assets to Protect:**
- Alter identity information (names, descriptions, avatars)
- Switch logs (who, when, triggers)
- Biometric data (heart rate, HRV, sleep, glucose)
- ML models (trained on user's patterns)

---

### Defense Layers

#### 1. **Encryption at Rest**

**Desktop Database:**
- SQLCipher for SQLite (AES-256-CBC)
- Key derived from user's master password (scrypt KDF: N=16384, r=8, p=1)
- Alternative: Key stored in OS keychain (encrypted by OS)

**Mobile Cache:**
- SQLCipher for mobile SQLite
- Key derived from device biometric unlock (iOS Keychain/Android KeyStore)

**ML Models:**
- Stored as binary files (not sensitive, but can be encrypted)
- Optional: Encrypt with same master key as database

**File Exports:**
- User can encrypt exports (GPG, age, or ZIP password)
- Plaintext exports clearly labeled (user responsibility)

**Implementation:**
```rust
use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions};
use sqlcipher::Connection;

// SQLCipher connection
let opts = SqliteConnectOptions::new()
    .filename("~/.hexus/data.db")
    .pragma("key", user_derived_key)
    .pragma("cipher_page_size", "4096")
    .create_if_missing(true);
```

---

#### 2. **Encryption in Transit**

**Local Network:**
- **Option A:** TLS with self-signed certificate (pin on first pairing)
- **Option B:** Plaintext HTTP (trust LAN) + encrypt sync payloads (NaCl/libsodium)

**Remote (VPN):**
- Tailscale/WireGuard provide encrypted tunnel (no additional TLS needed)
- API payloads encrypted at application layer (defense in depth)

**Sync Payload Encryption:**
```rust
use sodiumoxide::crypto::secretbox;

// Encrypt sync data
let nonce = secretbox::gen_nonce();
let ciphertext = secretbox::seal(&plaintext_data, &nonce, &shared_key);
```

**Shared Key Derivation:**
- ECDH key exchange during pairing
- Shared secret stored in encrypted database
- Rotate keys periodically (user-initiated)

---

#### 3. **Authentication & Authorization**

**Desktop Dashboard (Local UI):**
- Optional master password (unlock database)
- OS-level user account protection (rely on OS login)

**API Access (Mobile Clients):**
- **Pairing Flow:**
  1. User initiates pairing in desktop dashboard
  2. Desktop generates pairing code (6-digit, expires in 5 minutes)
  3. User enters code in mobile app
  4. Desktop issues long-lived API key (stored in mobile keychain)
  5. Mobile uses API key in `Authorization: Bearer <key>` header

- **API Key Properties:**
  - Cryptographically random (32 bytes, base64-encoded)
  - Scoped to specific device (revocable)
  - Stored in SQLite with device name, first_seen, last_seen

**Revocation:**
- User can revoke API keys in desktop dashboard
- Immediate invalidation (no grace period)

---

#### 4. **Access Control**

**Per-Alter Data Isolation (Future):**
- Some alters may want private data (not shared with system)
- Implement per-alter encryption keys (opt-in)
- UI requires alter-specific password to view their data

**Therapist/Supporter Access (Future):**
- Export read-only reports (PDF, JSON)
- No direct API access
- User controls what data is included

---

#### 5. **Audit Logging**

**Security Events:**
- API key creation/revocation
- Failed authentication attempts
- Database encryption key changes
- Data exports

**Storage:**
- Append-only log file (`~/.hexus/audit.log`)
- Tamper-evident (cryptographic hash chain)
- User can review in desktop dashboard

---

### Attack Mitigations

| Attack | Mitigation |
|--------|------------|
| **Physical access to desktop** | Database encrypted at rest, requires master password |
| **Network sniffing (LAN)** | TLS or encrypted payloads (NaCl) |
| **Network sniffing (remote)** | Tailscale/WireGuard tunnel (encrypted) |
| **Stolen mobile device** | Biometric lock, API key in keychain (encrypted), cached data encrypted |
| **Compromised API key** | User revokes in dashboard, keys are scoped per-device |
| **Malicious mobile app** | API keys not stored in app code, sandbox protections |
| **Desktop malware** | OS-level protections (antivirus, firewall), encrypted database |

---

## Sync Protocol

### Protocol Design

**Goals:**
- Efficient (minimize bandwidth, especially on mobile data)
- Conflict-free (eventual consistency)
- Resumable (handle network interruptions)
- Secure (encrypted, authenticated)

**Approach:** Incremental sync with vector clocks

---

### Data Model

Each record (switch, biometric sample, alter profile) has:
- `id` — UUID (globally unique)
- `created_at` — Unix timestamp
- `updated_at` — Unix timestamp (for mutable records)
- `version` — Vector clock (device_id → logical_clock)
- `deleted` — Boolean (soft deletes for sync)

**Vector Clock Example:**
```json
{
  "id": "switch-12345",
  "timestamp": 1678886400,
  "alter_ids": ["alice"],
  "version": {
    "desktop-abc123": 5,
    "mobile-iphone-xyz": 3
  }
}
```

---

### Sync Algorithm

**Step 1: Mobile requests sync**
```
POST /api/v1/sync/request
{
  "device_id": "mobile-iphone-xyz",
  "last_sync_at": 1678880000,
  "vector_clock": {
    "desktop-abc123": 4,
    "mobile-iphone-xyz": 10
  }
}
```

**Step 2: Desktop calculates delta**
- Find records where `updated_at > last_sync_at` OR `version > mobile_vector_clock`
- Exclude records already seen by mobile (vector clock comparison)
- Return delta + current server vector clock

**Step 3: Desktop responds**
```
200 OK
{
  "delta": {
    "switches": [...],
    "biometrics": [...],
    "alters": [...]
  },
  "server_vector_clock": {
    "desktop-abc123": 7,
    "mobile-iphone-xyz": 3
  },
  "conflicts": []  // If any
}
```

**Step 4: Mobile applies delta**
- Merge new records into local SQLite
- Update vector clock
- Detect conflicts (same record updated on both sides)

**Step 5: Mobile uploads local changes**
```
POST /api/v1/sync/push
{
  "device_id": "mobile-iphone-xyz",
  "changes": {
    "switches": [...],  // New switch logs from mobile
    "biometrics": [...]  // HealthKit data
  },
  "vector_clock": {
    "desktop-abc123": 7,
    "mobile-iphone-xyz": 12
  }
}
```

**Step 6: Desktop merges and ACKs**
```
200 OK
{
  "ack": true,
  "new_server_vector_clock": {
    "desktop-abc123": 8,
    "mobile-iphone-xyz": 12
  }
}
```

---

### Conflict Resolution

**Switch Logs:**
- Last-write-wins (most recent `updated_at`)
- No conflicts expected (append-only, unique timestamps)

**Alter Profiles:**
- Merge fields independently (e.g., name changed on desktop, avatar on mobile → both apply)
- Prompt user for conflicts (e.g., name changed differently on both sides)

**Biometric Data:**
- Append-only (no updates, only inserts)
- Duplicates detected by `(timestamp, source, metric_type)` tuple
- Idempotent (re-uploading same data is safe)

---

### Offline Queue

**Mobile Behavior When Offline:**
1. User logs switch → stored in local SQLite with `sync_pending = true`
2. Periodic sync attempts (every 5 minutes if VPN connected)
3. When desktop reachable, upload queued changes
4. Mark as synced, clear `sync_pending` flag

**Desktop Behavior When Mobile Offline:**
- Continue collecting wearable data
- Increment vector clock
- Mobile will catch up on next sync

---

## Platform-Specific Integration

### iOS (HealthKit)

**Permissions:**
```swift
let healthStore = HKHealthStore()
let typesToRead: Set = [
    HKObjectType.quantityType(forIdentifier: .heartRate)!,
    HKObjectType.quantityType(forIdentifier: .heartRateVariabilitySDNN)!,
    HKObjectType.categoryType(forIdentifier: .sleepAnalysis)!,
    // ... more types
]
healthStore.requestAuthorization(toShare: nil, read: typesToRead) { success, error in
    // Handle
}
```

**Background Queries:**
```swift
let query = HKAnchoredObjectQuery(
    type: heartRateType,
    predicate: nil,
    anchor: lastAnchor,
    limit: HKObjectQueryNoLimit
) { query, samples, deletedObjects, newAnchor, error in
    // Process samples, send to Rust core via uniffi
}
healthStore.execute(query)
```

**Data Export to Rust:**
```swift
import HeXuSCore  // uniffi-generated module

let rustSamples = samples.map { sample in
    HeartRateSample(
        timestamp: Int64(sample.startDate.timeIntervalSince1970),
        value: sample.quantity.doubleValue(for: HKUnit.count().unitDivided(by: .minute())),
        unit: "bpm"
    )
}
let metrics = processHeartRateSamples(samples: rustSamples)
```

---

### Android (Health Connect)

**Permissions:**
```kotlin
val permissions = setOf(
    HealthPermission.getReadPermission(HeartRateRecord::class),
    HealthPermission.getReadPermission(HeartRateVariabilityRmssdRecord::class),
    HealthPermission.getReadPermission(SleepSessionRecord::class)
)
val permissionContract = PermissionController.createRequestPermissionResultContract()
requestPermissionLauncher.launch(permissions)
```

**Data Retrieval:**
```kotlin
val response = healthConnectClient.readRecords(
    ReadRecordsRequest(
        recordType = HeartRateRecord::class,
        timeRangeFilter = TimeRangeFilter.after(lastSyncTime)
    )
)
val records = response.records
```

**Data Export to Rust:**
```kotlin
import uniffi.hexus_core.*  // uniffi-generated bindings

val rustSamples = records.map { record ->
    HeartRateSample(
        timestamp = record.time.epochSecond,
        value = record.beatsPerMinute.toDouble(),
        unit = "bpm"
    )
}
val metrics = processHeartRateSamples(rustSamples)
```

---

### Desktop (Wearable APIs)

**OAuth2 Flow (Oura Ring Example):**
```rust
use reqwest::Client;
use oauth2::{AuthorizationCode, TokenResponse};

// Step 1: Redirect user to Oura authorization URL
let auth_url = oauth_client
    .authorize_url(CsrfToken::new_random)
    .url();
// Open in browser

// Step 2: User grants access, Oura redirects to localhost callback
// Extract authorization code from callback URL

// Step 3: Exchange code for access token
let token = oauth_client
    .exchange_code(AuthorizationCode::new(code))
    .request_async(async_http_client)
    .await?;

// Step 4: Store access token (encrypted in database)
store_token("oura", token.access_token());

// Step 5: Poll API for data
let client = Client::new();
let response = client
    .get("https://api.ouraring.com/v2/usercollection/daily_sleep")
    .bearer_auth(token.access_token())
    .send()
    .await?
    .json::<SleepData>()
    .await?;
```

**Polling Schedule:**
- Oura: Every 15 minutes (data updates after ring syncs)
- Whoop: Webhook subscriptions (real-time) + fallback polling
- Dexcom: Every 5 minutes (near real-time glucose)

---

## Machine Learning Pipeline

### Phase 1: Changepoint Detection (MVP)

**Goal:** Detect when biometric patterns shift (potential alter switches)

**Algorithm:** CUSUM (Cumulative Sum)

**Implementation:**
```rust
struct CUSUMDetector {
    baseline_mean: f64,
    baseline_std: f64,
    threshold: f64,
    cusum_pos: f64,
    cusum_neg: f64,
}

impl CUSUMDetector {
    fn process_sample(&mut self, value: f64) -> Option<ChangePoint> {
        let z_score = (value - self.baseline_mean) / self.baseline_std;
        
        self.cusum_pos = (self.cusum_pos + z_score - 0.5).max(0.0);
        self.cusum_neg = (self.cusum_neg - z_score - 0.5).max(0.0);
        
        if self.cusum_pos > self.threshold || self.cusum_neg > self.threshold {
            Some(ChangePoint { timestamp: now(), direction: ... })
        } else {
            None
        }
    }
}
```

**User Interaction:**
1. Desktop detects changepoint at 14:35
2. Notification: "Unusual biometric pattern detected. Did a switch occur?"
3. User labels: "Yes, Alice switched to Bob"
4. Store labeled changepoint → training data

---

### Phase 2: Semi-Supervised Learning

**Goal:** Classify which alter is fronting based on biometric patterns

**Algorithm:** SVM with self-training

**Implementation:**
```rust
use linfa::prelude::*;
use linfa_svm::Svm;

// Train initial model on 50 labeled examples
let dataset = Dataset::new(features, labels);
let model = Svm::<f64, Pr>::params()
    .gaussian_kernel(0.5)
    .fit(&dataset)?;

// Self-training loop
loop {
    // Predict on unlabeled data
    let predictions = model.predict(&unlabeled_features);
    
    // Add high-confidence predictions (>90%) to training set
    let high_confidence = predictions.iter()
        .filter(|p| p.confidence > 0.9)
        .collect();
    
    dataset.extend(high_confidence);
    
    // Retrain
    model = Svm::params().fit(&dataset)?;
}
```

**Active Learning:**
```rust
// Query user for ambiguous samples
let ambiguous = predictions.iter()
    .filter(|p| p.confidence < 0.6)
    .take(5);  // Ask user to label 5 samples

for sample in ambiguous {
    notify_user("Who was fronting at this time?", sample.timestamp);
    let label = await_user_response();
    dataset.add(sample.features, label);
}
```

---

### Phase 3: Few-Shot Learning

**Goal:** Classify alters with minimal labeled examples (1-5 per alter)

**Algorithm:** Prototypical Networks

**Training (Python → ONNX):**
```python
import torch
import torch.nn as nn

class PrototypicalNetwork(nn.Module):
    def __init__(self):
        super().__init__()
        self.encoder = nn.Sequential(
            nn.Linear(10, 64),  # 10 biometric features
            nn.ReLU(),
            nn.Linear(64, 32),
            nn.ReLU(),
            nn.Linear(32, 16)   # 16-dim embedding
        )
    
    def forward(self, x):
        return self.encoder(x)

# Train on support set
model = PrototypicalNetwork()
# ... training loop ...

# Export to ONNX
torch.onnx.export(model, dummy_input, "prototypical.onnx")
```

**Inference (Rust with Candle):**
```rust
use candle_core::{Device, Tensor};
use candle_onnx::read_file;

// Load ONNX model
let model = read_file("prototypical.onnx")?;

// Compute prototypes (mean embedding per alter)
let alice_prototype = compute_prototype(alice_examples);
let bob_prototype = compute_prototype(bob_examples);

// Classify query sample
let query_embedding = model.forward(&query_features)?;
let distances = vec![
    (alice_prototype - &query_embedding).sqr().sum(),
    (bob_prototype - &query_embedding).sqr().sum(),
];
let predicted_alter = distances.argmin();
```

---

### Model Management

**Storage:**
- Models stored in `~/.hexus/models/`
- Versioned (e.g., `svm_v1.pickle`, `prototypical_v3.onnx`)
- Metadata in SQLite `ml_models` table

**Retraining:**
- User-initiated (button in dashboard)
- Automatic (weekly, if >100 new labeled samples)
- Background task (doesn't block UI)

**Evaluation:**
- Cross-validation on labeled dataset
- Display accuracy, precision, recall in dashboard
- User can revert to previous model if accuracy drops

---

## Privacy Architecture

### Data Minimization

**Collect Only What's Needed:**
- Don't collect raw wearable data if summary metrics suffice
- Don't store intermediate ML features (only final predictions)
- Delete old biometric data after aggregation (user-configurable retention policy)

**Retention Policies:**
- **Raw biometric samples:** 90 days (default, configurable)
- **Aggregated metrics:** 1 year (daily/weekly averages)
- **Switch logs:** Indefinite (user can manually delete)
- **ML models:** Keep last 3 versions (auto-delete older)

---

### User Control

**Transparency:**
- Dashboard shows all data collection sources
- Display what data is stored locally
- Show API keys and paired devices

**Data Deletion:**
- "Delete All Data" button (wipe database, models, exports)
- Per-alter deletion (remove specific alter's data)
- Per-source deletion (remove Oura data, keep HealthKit)

**Exports:**
- JSON (full database dump)
- CSV (time series for spreadsheets)
- Parquet (for data science tools)
- PDF (human-readable reports)

---

### No Telemetry

**What HeXuS NEVER Collects:**
- Your biometric data
- Your alter names or identities
- Your switch logs
- Your location
- Usage analytics
- Crash reports (unless you opt-in to send manually)

**Open Source:**
- Full source code available (verify no telemetry)
- Reproducible builds (verify binary matches source)
- Community audits welcome

---

### Threat: Subpoenaed Data

**Risk:** Legal demands to turn over user data

**Mitigation:**
- **We don't have your data** (it's on your desktop, not our servers)
- Zero-knowledge architecture (even if we wanted to, we can't access your data)
- Recommend full-disk encryption on desktop (LUKS, FileVault, BitLocker)
- Recommend strong master password (or keychain integration)

---

## Technology Stack

### Desktop Server

| Component | Technology | Rationale |
|-----------|------------|-----------|
| **Core Language** | Rust | Memory safety, performance, concurrency |
| **Web Framework** | `axum` | Async, ergonomic, modular |
| **Database** | SQLite + SQLCipher | Embedded, encrypted, zero-config |
| **Time Series** | DuckDB (optional) | Fast analytics on biometric data |
| **ML Inference** | `candle` (ONNX) | On-device, no Python runtime |
| **Traditional ML** | `linfa` | SVM, clustering, PCA in pure Rust |
| **Time Series Analysis** | `augurs`, `oxidiviner` | Forecasting, anomaly detection |
| **HRV Processing** | `cardio-rs` | Heart rate variability metrics |
| **OAuth2** | `oauth2` crate | Wearable API authentication |
| **HTTP Client** | `reqwest` | API polling (Oura, Whoop, etc.) |
| **Encryption** | `sodiumoxide`, `ring` | NaCl crypto, key derivation |
| **Serialization** | `serde`, `serde_json` | JSON API payloads |

---

### Desktop Dashboard

| Component | Technology | Rationale |
|-----------|------------|-----------|
| **Framework** | Tauri 2.0 | Cross-platform, small bundle size |
| **Frontend** | Svelte or React | Reactive UI, rich ecosystem |
| **Styling** | Tailwind CSS | Utility-first, fast prototyping |
| **Charts** | Chart.js or D3.js | Time series visualization |
| **State Management** | Svelte stores / React Context | Simple, built-in |

---

### Mobile Apps

| Component | Technology | Rationale |
|-----------|------------|-----------|
| **Framework** | Tauri 2.0 Mobile | Shared codebase with desktop |
| **FFI** | `uniffi` | Swift/Kotlin ↔ Rust bridges |
| **iOS Data** | HealthKit (Swift) | Native iOS health data |
| **Android Data** | Health Connect (Kotlin) | Native Android health data |
| **Database** | SQLite (embedded) | Local cache for offline mode |
| **HTTP Client** | Native (URLSession/OkHttp) | Sync to desktop server |

---

### Build & Deployment

| Tool | Purpose |
|------|---------|
| **Cargo** | Rust build system |
| **Tauri CLI** | Desktop/mobile app bundling |
| **Xcode** | iOS build and signing |
| **Android Studio** | Android build and signing |
| **GitHub Actions** | CI/CD (automated builds) |
| **Nix** (optional) | Reproducible builds |

---

## Deployment Model

### Distribution Channels

**Desktop:**
- **macOS:** `.dmg` installer, codesigned (notarized by Apple)
- **Windows:** `.msi` installer, codesigned (Authenticode)
- **Linux:** `.deb`, `.rpm`, AppImage, Flatpak

**Mobile:**
- **iOS:** TestFlight (beta) → App Store (production)
- **Android:** Google Play Store (or F-Droid for FOSS version)

---

### Installation Flow

**Desktop Setup:**
1. User downloads installer
2. Run installer → creates `~/.hexus/` directory
3. First launch → master password setup
4. Database initialized (encrypted)
5. Dashboard opens → guide user through wearable API setup

**Mobile Pairing:**
1. User installs mobile app from App Store / Play Store
2. Tap "Pair with Desktop"
3. App scans LAN for mDNS `_hexus._tcp` service (or user enters IP manually)
4. Desktop dashboard shows pairing code
5. User enters code in mobile app
6. Desktop issues API key, stored in mobile keychain
7. Mobile app syncs initial data (alter profiles, recent switches)

---

### Updates

**Desktop:**
- Auto-update via Tauri updater (downloads delta patches)
- User can opt-out (manual downloads from website)
- Changelog displayed before update

**Mobile:**
- Standard App Store / Play Store update flow
- User controls when to update

**Database Migrations:**
- Automatic schema migrations on update (backward-compatible)
- Backup database before migration
- Rollback mechanism if migration fails

---

### Backup & Recovery

**User Backup Options:**

1. **Manual Export:**
   - Dashboard → Settings → Export All Data
   - Generates encrypted ZIP with database + models
   - User stores on external drive, cloud storage (encrypted)

2. **Scheduled Backups (Future):**
   - Auto-export to user-specified location (e.g., Dropbox, NAS)
   - Encrypted with user's master password
   - User controls frequency

**Recovery:**
- Reinstall desktop app
- Import backup ZIP during first-launch setup
- Decrypt with master password
- Database and models restored

---

### Multi-Device Desktop (Future)

**Scenario:** User has desktop at home + laptop for travel

**Solution:**
- Same HeXuS install on both devices
- Sync via encrypted file sync (Syncthing, Resilio Sync)
- Vector clocks handle conflict-free merging
- User designates one as "primary" (runs wearable API polling)

---

## Open Questions & Future Work

### Technical Questions

1. **Battery Life on Mobile:**
   - How often can we sync without draining battery?
   - Testing needed with real users

2. **ML Model Accuracy:**
   - What accuracy is "good enough" for switch detection?
   - How many labeled examples per alter?

3. **Conflict Resolution:**
   - Is last-write-wins sufficient for switch logs?
   - Do we need more sophisticated merging?

4. **Desktop Always-On:**
   - What if user's desktop sleeps?
   - Wake-on-LAN for mobile sync?

### Feature Roadmap

**v1.0 (MVP):**
- Desktop server (ingest, storage, basic analysis)
- iOS app (HealthKit sync)
- Android app (Health Connect sync)
- Manual switch logging
- Timeline visualization
- HRV calculation
- Changepoint detection

**v1.1:**
- Oura Ring API integration
- Whoop API integration
- PluralKit/Simply Plural API sync
- Therapist PDF reports

**v1.2:**
- Semi-supervised ML (SVM)
- Active learning (user labeling prompts)
- Pattern insights (correlations, seasonality)

**v2.0:**
- Few-shot prototypical networks
- Switch prediction (forecasting)
- Multi-desktop sync
- Per-alter data isolation
- Advanced privacy features

---

## Conclusion

HeXuS represents a fundamentally different approach to health data: **privacy-first, user-owned, zero-cloud**. By making the desktop the personal server and mobile devices thin clients, we ensure:

- **User retains complete control** over sensitive biometric and identity data
- **No central point of failure** or data breach risk (no HeXuS servers to hack)
- **Works without internet** (core functionality is local-first)
- **Extensible and open** (users can export data, run on their own infrastructure)

The architecture is designed to be:
- **Technically feasible** (mature Rust ecosystem, proven ML techniques)
- **Privacy-respecting** (encryption, zero telemetry, open source)
- **Clinically useful** (correlate biometrics with switches, therapist reports)
- **Community-aligned** (compatible with PluralKit/Simply Plural, trauma-informed design)

This is not just a health app. It's a tool for self-knowledge, built with the plural community's needs and values at its core.

---

**Document Version:** 2.0  
**Last Updated:** 2026-03-01  
**Author:** Silas Vane (Architecture Sub-Agent)  
**Status:** Complete — Ready for Review
