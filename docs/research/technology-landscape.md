# Technology Landscape for HeXuS Project

**Last Updated:** 2026-03-01  
**Research Focus:** Biometric monitoring system for plural systems (DID/OSDD)

---

## Executive Summary

This document maps the technology landscape for HeXuS, focusing on three key areas:
1. **Wearable APIs** for biometric data access (heart rate, HRV, sleep, glucose)
2. **Existing plural system tools** (PluralKit, Simply Plural) and their capabilities
3. **Rust ecosystem** for health data processing, mobile development, and cross-platform deployment

### Key Findings

- **HealthKit (iOS)** and **Health Connect (Android)** provide the most comprehensive device-agnostic data access
- **Oura Ring** and **Whoop** APIs offer high-quality summary data but require active subscriptions
- **PluralKit** and **Simply Plural** both provide REST APIs for switch tracking integration
- **Tauri 2.0** with **uniffi** provides the strongest path for cross-platform mobile deployment
- **Time series analysis** in Rust is well-supported via `augurs` and `oxidiviner` crates

---

## 1. Wearable APIs & Data Access

### 1.1 Apple HealthKit

**Platform:** iOS/watchOS only (native apps required)  
**Access Method:** Native HealthKit framework  
**Authentication:** User consent per data type (granular permissions)

#### Available Data Types

| Category | Data Types | Update Frequency |
|----------|------------|------------------|
| **Vitals** | Heart rate, HRV (SDNN), SpO2, blood pressure, body temperature, respiratory rate | Real-time (watch), periodic (phone) |
| **Activity** | Steps, distance, floors climbed, active energy, exercise minutes, workouts (with pace/splits) | Continuous |
| **Sleep** | Sleep stages (light/deep/REM), sleep duration, time in bed | Per-session |
| **Body** | Weight, height, body fat %, BMI, lean body mass, basal metabolic rate | Manual/device entry |
| **Nutrition** | Calories, macros, hydration | Manual entry |
| **Mindfulness** | Meditation sessions, mindful minutes | Manual/app entry |
| **Cycle Tracking** | Menstruation, symptoms, ovulation | Manual entry |
| **Medications** | User-annotated medications, dose events (iOS 19+) | Manual tracking |

#### Data Schema (Open mHealth JSON Format)

Heart Rate example:
```json
{
  "body": {
    "heart_rate": 72.5,
    "effective_time_frame": {
      "time_interval": {
        "start_date_time": "2023-10-01T10:00:00Z",
        "end_date_time": "2023-10-01T10:00:30Z"
      }
    },
    "unit_id": "bpm"
  },
  "header": {
    "id": "unique-sample-uuid",
    "creation_date_time": "2023-10-01T10:00:00Z",
    "schema_id": {
      "namespace": "omh",
      "name": "heart-rate",
      "version": "1.0"
    }
  }
}
```

HRV example:
```json
{
  "body": {
    "heart_rate_variability": 45.2,
    "effective_time_frame": {
      "time_interval": {
        "start_date_time": "2023-10-01T10:00:00Z",
        "end_date_time": "2023-10-01T10:00:05Z"
      }
    },
    "unit_id": "ms"
  },
  "header": {
    "id": "unique-sample-uuid",
    "schema_id": {
      "namespace": "omh",
      "name": "heart-rate-variability",
      "version": "1.0"
    }
  }
}
```

#### Limitations

- **iOS/watchOS only** — cannot access from Android or web
- **No backend API** — data stays on-device, requires native app
- **No multi-user access** — single-user health data only
- **Granular consent required** — users must approve each data type individually
- **Version fragmentation** — features vary by iOS version (e.g., medications in iOS 19+)

#### Integration Recommendations

- Use Swift native app or React Native with HealthKit bridge
- Implement sync to HeXuS backend via background tasks
- Export to Open mHealth or FHIR JSON for interoperability
- Library: [Granola](https://github.com/CardinalKit/Granola) for HealthKit → OMH/FHIR conversion

---

### 1.2 Android Health Connect

**Platform:** Android 14+ (API level 34+)  
**Access Method:** Kotlin/Java Android SDK  
**Authentication:** User consent per data type via Play Console declaration

#### Available Data Types

| Category | Data Types |
|----------|------------|
| **Activity** | Steps, distance, calories burned, exercise sessions, power, active time |
| **Body Measurement** | Weight, height, body fat %, bone mass, lean body mass, BMR |
| **Cycle Tracking** | Menstruation, ovulation tests, cervical mucus, sexual activity |
| **Nutrition** | Food intake, hydration, macronutrients |
| **Sleep** | Sleep sessions, sleep duration, sleep stages |
| **Vitals** | Heart rate, blood pressure, body temperature, blood glucose, SpO2, respiratory rate |
| **Wellness** | Mindfulness sessions, meditation |

#### Data Structure

All data stored as **Record objects** with:
- **Generic fields:** `time`, `zoneOffset`, Health Connect ID, last modified time
- **Specific fields:** Type-dependent (e.g., `count`, `percentage`, `duration`)
- **Metadata:** Data origin (package name), device (manufacturer/model), client ID, recording method (passive/active)
- **Type system:** Basic types (long, double, string) + complex types (enums, `Instant`, `ZoneOffset`)

#### Limitations

- **Android 14+ required** — older devices unsupported
- **Write access restricted** — requires Google Play approval for certain data types
- **Fragmentation** — not all devices/manufacturers support all data types
- **Permission complexity** — must declare specific read/write permissions per type

#### Integration Recommendations

- Use Kotlin with `android.health.connect.datatypes` package
- Implement sync service with WorkManager for background updates
- Handle permission flow gracefully (some types may be unavailable)
- Support fallback to Google Fit API for older Android versions

---

### 1.3 Oura Ring API v2

**Platform:** Cloud API (device-agnostic)  
**Access Method:** REST API with OAuth2  
**Subscription:** Requires active Oura Membership (Gen3/Oura Ring 4)

#### Available Data & Endpoints

| Endpoint | Data Available | Scope Required |
|----------|----------------|----------------|
| `/v2/usercollection/daily_sleep` | Sleep stages, HRV, heart rate during sleep | `daily` |
| `/v2/usercollection/daily_readiness` | Readiness scores, recovery metrics | `daily` |
| `/v2/usercollection/daily_activity` | Activity summaries, calories, steps | `daily` |
| Heart rate time series | Continuous heart rate (5-min intervals) | `heartrate` (Gen3+) |
| SpO2 | Daily SpO2 averages during sleep | `spo2` |

#### Data Quality

- **HRV:** Nocturnal HRV balance and variability (high accuracy)
- **Heart Rate:** Continuous during sleep, periodic during day (Gen3+)
- **Sleep Staging:** Light/deep/REM with timestamps
- **Readiness:** Proprietary score based on HRV, sleep, activity

#### Limitations

- **Subscription wall** — API access requires active membership (~$6/month)
- **Gen2 sunset** — older rings may lose support
- **Delayed data** — syncs after ring uploads (not real-time)
- **Summary focus** — raw sensor data not available, only processed metrics

#### Integration Recommendations

- Best for users already wearing Oura Ring
- Use `/daily_sleep` for HRV and nocturnal heart rate
- Implement date range queries for historical data backfill
- Cache data locally (API rate limits apply)

---

### 1.4 Whoop API

**Platform:** Cloud API (device-agnostic)  
**Access Method:** REST API with OAuth2  
**Subscription:** Requires active Whoop membership

#### Available Data & Endpoints

| Endpoint | Data Available | Scope Required |
|----------|----------------|----------------|
| `/v1/user/profile/basic` | Name, email | `read:profile` |
| `/v1/user/body_measurement` | Height, weight, max heart rate | `read:body_measurement` |
| `/v1/cycles` | Physiological cycles (sleep + recovery + strain) | `read:cycles` |
| `/v1/recovery` | Recovery scores, HRV, RHR | `read:recovery` |
| `/v1/sleep` | Sleep quality, stages, disturbances | `read:sleep` |
| `/v1/workouts` | Workout details, strain, recovery activities | `read:workout` |

#### Data Quality

- **Recovery Score:** Proprietary metric (0-100%) based on HRV, RHR, sleep
- **Strain:** Daily exertion score (0-21)
- **Sleep Performance:** Sleep quality percentage with stage breakdown
- **HRV:** RMSSD (root mean square of successive differences)

#### Limitations

- **Summary data only** — no raw heart rate time series
- **Subscription required** — ~$30/month for Whoop membership
- **Delayed updates** — syncs after device uploads
- **Rate limits** — webhooks recommended for real-time updates

#### Integration Recommendations

- Best for users already wearing Whoop
- Use webhooks instead of polling (real-time updates)
- Focus on recovery/strain correlations with switches
- Store historical cycles for longitudinal analysis

---

### 1.5 Garmin Connect API

**Platform:** Cloud API (device-agnostic)  
**Access Method:** REST API with OAuth2  
**Cost:** Free for approved business developers

#### Available APIs

- **Health API:** Vitals, wellness data, biometrics
- **Activity API:** Workouts, exercises, fitness data
- **Sleep API:** Sleep stages, duration, quality
- **Training API:** Training plans, goals
- **Women's Health API:** Cycle tracking, symptoms

#### Access Process

1. Apply at developer.garmin.com
2. Describe company and intended use
3. Wait for approval (vetting process)
4. Receive evaluation API key for development
5. Request production key when ready

#### Limitations

- **Application required** — not open to individual developers
- **Business use only** — consumer apps may not qualify
- **Limited documentation** — less detailed than competitors
- **Approval delays** — vetting can take weeks

#### Integration Recommendations

- Only pursue if targeting Garmin users specifically
- Requires business entity and clear use case
- Consider as secondary integration (not primary)

---

### 1.6 Fitbit Web API

**Platform:** Cloud API (device-agnostic)  
**Access Method:** REST API with OAuth2  
**Status:** Being phased out (Google migration in progress)

#### Available Data Types

| Data Type | Endpoints | Scope Required |
|-----------|-----------|----------------|
| **Heart Rate** | Intraday time series, heart rate zones | `heartrate` |
| **Sleep** | Sleep stages (light/deep/REM/awake), duration, efficiency | `sleep` |
| **Activity** | Steps, distance, calories, floors, active minutes | `activity` |
| **Body** | Weight, BMI, body fat % | `weight` |

#### Data Schema Example

Sleep log:
```json
{
  "minutesAsleep": 420,
  "minutesAwake": 45,
  "minutesToFallAsleep": 12,
  "isMainSleep": true,
  "sleepLevelDeep": 120,
  "sleepLevelLight": 240,
  "sleepLevelRem": 60,
  "sleepLevelAwake": 45
}
```

Heart rate zones:
```json
{
  "heartRateZones": {
    "fatBurn": {
      "caloriesOut": 250.0,
      "min": 106,
      "max": 131,
      "minutes": 45
    },
    "cardio": {
      "caloriesOut": 180.0,
      "min": 132,
      "max": 160,
      "minutes": 30
    }
  }
}
```

#### Limitations

- **Uncertain future** — Google migrating to Health Connect
- **Intraday limits** — high-frequency data requires special approval
- **API instability** — deprecation notices common
- **Rate limits** — 150 requests/hour per user

#### Integration Recommendations

- **Avoid as primary integration** — use Health Connect on Android instead
- Consider for legacy Fitbit users only
- Plan migration path to Health Connect
- Monitor Google announcements for deprecation timeline

---

### 1.7 Continuous Glucose Monitors (CGMs)

#### Dexcom CGM API

**Platform:** Cloud API  
**Access Method:** REST API with OAuth2 + HIPAA authorization  
**Cost:** Free (registration required)

**Available Data:**
- Estimated Glucose Values (EGVs) — near real-time (~5 min delay)
- Historical glucose readings with timestamps
- Trend data (rising/falling indicators)

**Access Levels:**
- **Sandbox:** Immediate (mock data for testing)
- **Limited Access:** Up to 5 users (requires Data Licensing Agreement)
- **Full Access:** Commercial use (requires approved app + active users)

**API Example:**
```bash
GET https://api.dexcom.com/v3/users/self/egvs?startDate=2025-02-06T09:12:35&endDate=2025-02-06T09:12:35
Authorization: Bearer <access_token>
```

**Limitations:**
- Production access requires approval process
- User-driven authorization (cannot access without user consent)
- HIPAA compliance required for production
- ~5 minute delay on "real-time" data

#### FreeStyle Libre (Abbott)

**Platform:** No official public API  
**Access Method:** Third-party integrations only

**Third-Party Options:**
- **Terra API:** Webhook streaming, HTTP requests
- **Thryve:** LibreLink/LibreView sync via OAuth2
- **Nightscout Pro:** Open-source sync to Nightscout API
- **Unofficial libraries:** Node.js/Python wrappers (reliability concerns)

**Data Available (via third-party):**
- Glucose measurements (continuous)
- Trends, time-in-range
- Sensor info, calibration data

**Limitations:**
- **No official API** — relies on reverse-engineering LibreLink
- **Legal/reliability risks** — unofficial access may break
- **Third-party costs** — API services charge fees
- **Medical device regulations** — direct sensor access restricted

#### Integration Recommendations for CGMs

- **Dexcom:** Best option if targeting diabetic plural systems
- **Libre:** Only via trusted third-party (Terra, Thryve) for production
- **Use case:** Correlate glucose fluctuations with switches/dissociation
- **Priority:** Low (niche use case, but valuable for diabetic users)

---

## 2. Existing Plural System Tools

### 2.1 PluralKit (Discord Bot)

**Platform:** Discord bot  
**Purpose:** Message proxying, switch tracking, system management

#### Core Features

- **Message Proxying:** Members can "speak as" alters via webhook proxying with custom names/avatars
- **Autoproxy:** Automatic message conversion without typing tags
- **Switch Tracking:** Log current fronters, view switch history, statistics on fronting time
- **System Management:** Create systems, add members to groups, customize profiles (names, avatars, descriptions, pronouns, colors)
- **Privacy Controls:** Per-field visibility, channel blacklists, per-server avatars

#### API Capabilities

**Base URL:** `https://api.pluralkit.me/v2/`  
**Authentication:** `Authorization: <system_token>` header

**Systems Endpoints:**
- `GET /s/{system_id}` — Retrieve system details
- `PATCH /s/{system_id}` — Update system (JSON body)
- `GET /s/{system_id}/s/{server_id}` — Get server-specific settings

**Members Endpoints:**
- `GET /s/{system_id}/m` — List all members
- `GET /s/{system_id}/m/{member_id}` — Get specific member
- `POST /s/{system_id}/m` — Create member
- `PATCH /s/{system_id}/m/{member_id}` — Update member
- `DELETE /s/{system_id}/m/{member_id}` — Delete member

**Switches Endpoints:**
- `GET /s/{system_id}/sw` — List switches
- `GET /s/{system_id}/sw/{switch_id}` — Get specific switch
- `POST /s/{system_id}/sw` — Log new switch
- `PATCH /s/{system_id}/sw/{switch_id}` — Update switch
- `DELETE /s/{system_id}/sw/{switch_id}` — Delete switch

**Data Models:**

System model:
```json
{
  "id": "abcde",
  "name": "The System Name",
  "tag": "| System",
  "description": "Description text",
  "avatar_url": "https://...",
  "banner": "https://...",
  "color": "FF6B6B",
  "created": "2020-01-01T00:00:00.000000Z",
  "proxy_tags": [{"prefix": "[", "suffix": "]"}]
}
```

Member model:
```json
{
  "id": "abcde",
  "name": "Member Name",
  "display_name": "Display",
  "description": "Description",
  "color": "FF6B6B",
  "birthday": "1990-01-01",
  "pronouns": "they/them",
  "avatar_url": "https://...",
  "proxy_tags": [{"prefix": "M:", "suffix": ""}],
  "created": "2020-01-01T00:00:00.000000Z"
}
```

Switch model:
```json
{
  "id": "abcde",
  "timestamp": "2020-01-01T12:00:00.000000Z",
  "members": ["member_id_1", "member_id_2"],
  "created": "2020-01-01T12:00:00.000000Z"
}
```

#### Privacy Behavior

- Private fields return `null` unless authenticated with system token
- Entire endpoints return `403 Forbidden` if privacy settings block access
- Content-Type must be `application/json` for POST/PATCH

#### What PluralKit Does Well

✅ **Message proxying** — seamless Discord integration  
✅ **Low barrier to entry** — simple commands, no app required  
✅ **Community adoption** — widely used in plural Discord servers  
✅ **API access** — full CRUD operations on systems/members/switches  
✅ **Privacy controls** — granular visibility settings

#### What PluralKit Does Poorly

❌ **No biometric integration** — purely manual tracking  
❌ **Discord-only** — locked to Discord platform  
❌ **No analytics** — basic stats only, no pattern analysis  
❌ **Manual switch logging** — requires conscious effort to update  
❌ **No mobile app** — Discord mobile only

---

### 2.2 Simply Plural

**Platform:** Mobile app (Android/iOS) + web  
**Purpose:** System tracking, fronting visualization, internal communication

#### Core Features

- **Member Management:** Profiles with avatars, descriptions, custom fields, notes, stats (avg fronting time, daily patterns)
- **Fronting Tracking:** Manual switch logging with custom statuses (co-conscious, co-front), visual graph of front history
- **Notifications:** Share front changes with selected friends/systems
- **Decision Tools:** Per-member voting/polls for collaborative choices
- **Communication:** Internal chat (system), external chat (friends), Discord community
- **Reports:** Generate shareable user reports (email) for therapists/non-users with members, fronts, history
- **Accessibility:** High-contrast/dark modes, adjustable font sizes
- **Integrations:** Import/export with PluralKit

#### API Capabilities

**Base URLs:**
- Production: `https://api.apparyllis.com`
- Development: `https://devapi.apparyllis.com` (recommended for testing)

**Authentication:** `Authorization: <api_token>` header (generate in Settings > Account > Tokens)

**Available Endpoints:**
- `GET /v1/me` — User ID and basic profile
- `GET /v1/members` — All system members
- `GET /v1/groups` — All groups
- `GET /v1/customfronts` — Custom fronts
- `GET /v1/fronthistory` — Front history
- Full CRUD support (POST/PATCH/DELETE)

**Real-Time Updates:**
- WebSocket: `wss://api.apparyllis.com/v1/socket`
- Recommended over polling for >30min intervals

**SDKs:**
- Dart/Flutter SDK available (handles batch writes, streams, offline persistence)

**Example Request:**
```bash
curl -H "Authorization: your-token-here" \
  https://api.apparyllis.com/v1/members
```

#### What Simply Plural Does Well

✅ **Visual front history** — timeline graphs for pattern recognition  
✅ **Mobile-first** — native apps for Android/iOS  
✅ **Therapist reports** — shareable summaries for clinical use  
✅ **API access** — full CRUD + WebSocket for real-time updates  
✅ **Internal communication** — chat and decision-making tools  
✅ **Privacy-focused** — data stays private unless explicitly shared  
✅ **PluralKit integration** — import/export compatibility

#### What Simply Plural Does Poorly

❌ **No biometric integration** — purely manual tracking  
❌ **No automatic switch detection** — requires conscious logging  
❌ **Limited analytics** — basic stats, no correlation analysis  
❌ **Web version buggy** — mobile apps more reliable  
❌ **Subscription for cosmetics** — Simply Plus for premium features

---

### 2.3 Other Tools in the Community

#### The Bag System
- **Platform:** Mobile app (nonprofit project)
- **Focus:** DID/OSDD-specific with helpline support
- **Features:** Internal communication, tracking assistance
- **Status:** Emerging tool, less adoption than PluralKit/Simply Plural

#### Joplin (Note-Taking)
- **Platform:** Cross-platform note app
- **Use Case:** Community uses for per-alter journals/notebooks
- **Integration:** Manual, no automatic switch tracking
- **Privacy:** Device-only storage, encryption available

#### Bearable (Health Tracker)
- **Platform:** Android/iOS health diary
- **Use Case:** Some plural systems use for mood/symptom tracking
- **Features:** Correlations, timelines, exports, custom tracking
- **Limitation:** Not designed for DID/plural systems specifically

---

### 2.4 Key Gaps in Existing Tools

The plural community currently lacks:

1. **Biometric integration** — no tools correlate switches with heart rate, HRV, sleep data
2. **Automatic detection** — all switch logging is manual (requires conscious awareness)
3. **Pattern analysis** — basic stats only, no ML-based pattern recognition
4. **Cross-platform syncing** — Discord vs mobile app silos
5. **Therapist dashboards** — limited clinical utility beyond basic reports
6. **Trigger correlation** — no linking of switches to environmental/physiological triggers

**HeXuS Opportunity:** First tool to bridge biometric data with plural system tracking.

---

## 3. Rust Libraries & Ecosystem

### 3.1 Health Data Processing

#### FHIR/Medical Data

**nppes** (crate)
- **Purpose:** Parse/query NPPES provider data (~8M US healthcare providers)
- **Features:** Parallel processing, full-text search, type-safe structures, export to JSON/CSV/SQL
- **Use Case:** Provider lookups, healthcare data indexing
- **Limitation:** US-specific, not general health data processing

**codelist-tools** (crate)
- **Purpose:** Medical codelist management (epidemiology)
- **Features:** Metadata handling, code validation, editing, mapping (SNOMED, diabetes codes)
- **Bindings:** Python, R
- **Use Case:** Medical terminology mapping, data validation

**Custom FHIR Processing:**
- High-performance repositories use custom code generation from FHIR StructureDefinitions
- 1,500-8,000 FHIR resources/second throughput
- Type-safe enums for codes (e.g., Patient.gender)
- FHIRPath evaluation, search parameter indexing
- **Recommendation:** Generate FHIR types from schemas if needed

#### Biometric Processing

**cardio-rs** (crate)
- **Purpose:** Heart rate variability (HRV) metrics from RR intervals
- **Features:** Time-domain and frequency-domain HRV analysis
- **Use Case:** HRV calculation, cardiac biometrics
- **Status:** v0.1.1 (early stage)

**Recommendation for HeXuS:**
- Use `cardio-rs` for HRV processing
- Build custom parsers for HealthKit/Health Connect data formats
- Consider code generation for FHIR if exporting to medical systems

---

### 3.2 Time Series Analysis

#### augurs (crate family)

**Latest:** v0.10.0 (May 2025)  
**Repository:** https://github.com/grafana/augurs

**Capabilities:**
- **Forecasting:** ETS (exponential smoothing), MSTL (seasonal decomposition), Prophet
- **Outlier Detection:** Anomaly detection in time series
- **Seasonality Detection:** Identify periodic patterns
- **Changepoint Detection:** Detect regime changes
- **Clustering:** Dynamic Time Warping (DTW)
- **Bindings:** Python, JavaScript/WASM

**Sub-crates:**
- `augurs-ets` — Exponential smoothing state space models
- `augurs-mstl` — Multiple seasonal-trend decomposition
- `augurs-prophet` — Prophet algorithm port from Python
- `augurs-outlier` — Outlier detection
- `augurs-seasons` — Seasonality detection
- `augurs-dtw` — Dynamic time warping

**Use Case for HeXuS:**
- Detect patterns in biometric data correlated with switches
- Forecast switch likelihood based on HRV/sleep trends
- Identify outliers (unusual biometric events)
- Cluster similar switch episodes

#### oxidiviner (crate)

**Repository:** https://github.com/rustic-ml/OxiDiviner

**Capabilities:**
- **ARIMA/SARIMA** — Auto-regressive integrated moving average (with seasonality)
- **GARCH** — Generalized autoregressive conditional heteroskedasticity (volatility modeling)
- **Holt-Winters** — Exponential smoothing with trend/seasonality
- **Kalman Filters** — State space estimation
- **STL Decomposition** — Seasonal-trend decomposition using LOESS
- **Validation:** MAE, RMSE, MAPE, AIC/BIC metrics
- **Cross-Validation:** Time series CV support
- **Builder API:** Ergonomic model construction

**Use Case for HeXuS:**
- Financial-grade forecasting (production-ready)
- Volatility analysis (GARCH for HRV variability)
- Trend decomposition (separate signal from noise)

#### Recommendation

- **Use `augurs`** for:
  - Outlier detection (identify unusual biometric events)
  - Seasonality detection (daily/weekly patterns in switches)
  - Changepoint detection (major system changes)
  - Python bindings if needed for ML pipelines

- **Use `oxidiviner`** for:
  - Production forecasting (switch prediction)
  - ARIMA modeling (time series regression)
  - Validation metrics (model performance)

- **Database Integration:**
  - Use `sqlx` for PostgreSQL/TimescaleDB (time series extension)
  - Use `influxdb-rs` for InfluxDB (purpose-built TSDB)
  - Consider TimescaleDB for SQL compatibility + time series performance

---

### 3.3 Mobile FFI (Foreign Function Interface)

#### uniffi (Mozilla)

**Latest:** v0.29.0  
**Repository:** https://github.com/mozilla/uniffi-rs

**Purpose:** Multi-language bindings generator for Rust → Swift/Kotlin/Python

**Key Features:**
- **Automatic bindings:** Compile Rust to shared libraries + generate type-safe Swift/Kotlin/Python wrappers
- **Type conversion:** Enums, structs, functions auto-convert (e.g., `snake_case` → `camelCase`)
- **Cross-platform:** iOS (Swift), Android (Kotlin/JNI), Python
- **Production-proven:** Powers Firefox mobile apps
- **Safety-focused:** Safe-by-default bindings prevent undefined behavior

**iOS Setup:**
1. Add `uniffi` to Cargo.toml:
   ```toml
   [dependencies]
   uniffi = { version = "0.29", features = ["cli"] }
   ```

2. Mark exports in `lib.rs`:
   ```rust
   uniffi::setup_scaffolding!();

   #[uniffi::export]
   pub fn process_heart_rate(samples: Vec<HeartRateSample>) -> HrvMetrics {
       // Rust logic here
   }
   ```

3. Build bindings:
   ```bash
   cargo uniffi-bindgen generate --language swift
   ```

4. Compile for iOS:
   ```bash
   rustup target add aarch64-apple-ios
   cargo build --target aarch64-apple-ios --release
   ```

5. Integrate:
   - Swift: Embed generated `.swift` in Xcode
   - Android: Use NDK/JNI with generated Kotlin

**Kotlin Example Output:**
```kotlin
// Auto-generated from Rust
fun processHeartRate(samples: List<HeartRateSample>): HrvMetrics {
    // Calls Rust via FFI
}
```

**Swift Example Output:**
```swift
// Auto-generated from Rust
func processHeartRate(samples: [HeartRateSample]) -> HrvMetrics {
    // Calls Rust via FFI
}
```

**Advanced Features:**
- Callbacks (Rust → foreign language)
- Async/await support
- Complex types (nested structs, enums)
- UDL files for interface definitions (alternative to proc-macros)

**Limitations:**
- Requires explicit `#[uniffi::export]` (not all public functions auto-bind)
- Platform-specific compilation (need separate builds for iOS/Android)
- Some Rust types not directly mappable (need wrapper types)

#### Recommendation for HeXuS

- **Use uniffi** for core biometric processing logic
- Write once in Rust, deploy to iOS (Swift) and Android (Kotlin)
- Handle HealthKit (Swift) and Health Connect (Kotlin) in native code, call Rust for processing
- Example architecture:
  - Swift/Kotlin: Fetch data from platform APIs
  - Rust (via uniffi): Process HRV, detect patterns, store to DB
  - Swift/Kotlin: Display results in UI

---

### 3.4 Tauri 2.0 (Cross-Platform Framework)

**Latest:** v2.0 (beta, stable expected soon)  
**Website:** https://v2.tauri.app

**Purpose:** Build cross-platform desktop + mobile apps with web frontend + Rust backend

#### Supported Platforms

- **Desktop:** Linux, macOS, Windows
- **Mobile:** Android, iOS (full support, not experimental)

#### Key Capabilities

**Mobile Features:**
- Native APIs: Notifications, dialogs, NFC, barcode scanning, biometric auth, clipboard, deep links
- Platform integration: Swift (iOS), Kotlin (Android) for native code
- Seamless porting: Desktop apps work on mobile with minimal changes

**Desktop Enhancements:**
- Multiwebview: Multiple webviews per window
- Menus: Native menu bar, tray icons, context menus
- Window controls: Programmatic window management from JS
- Faster startup: Optimized Rust architecture, revamped IPC (custom protocols)

**Security:**
- **ACLs (Access Control Lists):** Granular API access per window/remote URL (replaces v1 allowlists)
- **Capabilities system:** Fine-grained API exposure
- **Stricter defaults:** Security audit in progress

**Plugin Ecosystem:**
- Official: Autostart, global shortcuts, file system, geolocation, HTTP client, logging, process, SQL, more
- Community: In-app updates, WebSockets, etc.

**Architecture:**
- **Frontend:** Any JS framework (React, Vue, Svelte, etc.) or vanilla HTML/CSS/JS
- **Backend:** Rust (handles native APIs, database, business logic)
- **IPC:** Channel API for Rust ↔ JS communication (type-safe, efficient)

**Bundle Sizes:**
- Small (~10MB) — webviews loaded at runtime (no bundled browser like Electron)

#### Tauri vs. Alternatives

| Feature | Tauri 2.0 | Electron | React Native |
|---------|-----------|----------|--------------|
| Mobile support | ✅ iOS/Android | ❌ Desktop only | ✅ iOS/Android |
| Desktop support | ✅ Linux/Mac/Win | ✅ Linux/Mac/Win | ❌ Mobile only |
| Bundle size | ~10MB (webview) | ~100MB (Chromium) | Varies |
| Language | Rust + JS/TS | JS/TS | JS/TS + native |
| Native APIs | Full (via Rust) | Limited | Full (bridges) |
| Security | ACLs, capabilities | Process isolation | Platform defaults |

#### Recommendation for HeXuS

- **Primary framework for cross-platform deployment**
- Build once, deploy to iOS, Android, desktop (Linux/Mac/Windows)
- Use Tauri plugins for:
  - SQL database (local storage)
  - File system (biometric data export)
  - Notifications (switch alerts)
  - HTTP client (API sync with backend)

**Architecture:**
1. **Frontend (Tauri webview):**
   - React/Vue/Svelte for UI
   - Dashboard, charts, settings
   - Call Rust backend via IPC

2. **Backend (Rust):**
   - HealthKit bridge (Swift → uniffi → Rust)
   - Health Connect bridge (Kotlin → uniffi → Rust)
   - Biometric processing (HRV, pattern detection)
   - Local SQLite database (time series storage)
   - API sync (push to HeXuS cloud backend)

3. **Native layers:**
   - iOS: Swift code for HealthKit access
   - Android: Kotlin code for Health Connect access
   - Both call Rust core via uniffi

---

## 4. Technical Recommendations for HeXuS

### 4.1 Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     HeXuS Mobile App                         │
│                     (Tauri 2.0)                              │
├─────────────────────────────────────────────────────────────┤
│  Frontend (Web Tech - React/Svelte)                          │
│  - Dashboard, charts, switch log, settings                   │
│  - Calls Rust backend via Tauri IPC                         │
├─────────────────────────────────────────────────────────────┤
│  Rust Core (Business Logic)                                  │
│  - Biometric processing (cardio-rs for HRV)                 │
│  - Time series analysis (augurs/oxidiviner)                 │
│  - Pattern detection (switch prediction)                    │
│  - Local SQLite/TimescaleDB storage                         │
│  - API sync with HeXuS cloud backend                        │
├─────────────────────────────────────────────────────────────┤
│  Platform Adapters (via uniffi)                              │
│  ┌─────────────────────┐  ┌─────────────────────┐          │
│  │  iOS (Swift)        │  │  Android (Kotlin)   │          │
│  │  - HealthKit fetch  │  │  - Health Connect   │          │
│  │  - Background sync  │  │  - Background sync  │          │
│  └─────────────────────┘  └─────────────────────┘          │
└─────────────────────────────────────────────────────────────┘
                              ▲
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              HeXuS Cloud Backend (Optional)                  │
│  - Cross-device sync                                         │
│  - Therapist portal                                          │
│  - PluralKit/Simply Plural integration                      │
│  - Long-term data storage                                    │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 Data Flow

1. **Biometric Collection:**
   - iOS: HealthKit → Swift → uniffi → Rust
   - Android: Health Connect → Kotlin → uniffi → Rust
   - Wearables: Oura/Whoop API → Rust HTTP client

2. **Processing Pipeline:**
   - Rust receives raw samples (heart rate, HRV, sleep)
   - `cardio-rs` computes HRV metrics
   - Store in local SQLite/TimescaleDB
   - `augurs` analyzes patterns (outliers, seasonality)
   - `oxidiviner` forecasts switch likelihood

3. **Switch Integration:**
   - User logs switch manually (or auto-detected via biometric anomaly)
   - Correlate with recent biometric data
   - Update pattern models
   - Sync to PluralKit/Simply Plural APIs

4. **Visualization:**
   - Tauri frontend renders charts (Chart.js, D3.js)
   - Timeline view: switches overlaid on HRV/sleep/heart rate
   - Insights: "HRV dropped 30% before last switch"

### 4.3 Technology Stack Recommendations

#### Core Framework
- **Tauri 2.0** (primary)
  - Cross-platform (iOS, Android, desktop)
  - Small bundle size
  - Rust backend for performance

#### Frontend
- **Svelte** or **React**
  - Lightweight (Svelte) or ecosystem-rich (React)
  - TypeScript for type safety
  - Chart.js or Recharts for data visualization

#### Backend (Rust)
- **uniffi** — Mobile FFI for Swift/Kotlin bridges
- **cardio-rs** — HRV calculation
- **augurs** — Pattern detection, outlier detection, seasonality
- **oxidiviner** — Forecasting, ARIMA modeling
- **sqlx** — Database access (PostgreSQL/SQLite)
- **reqwest** — HTTP client (API calls to Oura/Whoop/PluralKit/Simply Plural)
- **serde** — JSON serialization
- **tokio** — Async runtime

#### Database
- **Local:** SQLite (built into Tauri)
- **Self-hosted:** TimescaleDB (PostgreSQL + time series extension)
- **Cloud:** PostgreSQL (RDS, Supabase) with TimescaleDB extension

#### API Integrations
- **HealthKit:** Swift → uniffi → Rust
- **Health Connect:** Kotlin → uniffi → Rust
- **Oura Ring API:** OAuth2 + REST (reqwest)
- **Whoop API:** OAuth2 + REST + webhooks (reqwest)
- **Dexcom API:** OAuth2 + REST (optional, for diabetic users)
- **PluralKit API:** REST (switch sync)
- **Simply Plural API:** REST + WebSocket (switch sync)

#### Machine Learning (Future)
- **Linfa** (Rust ML crate) — clustering, regression
- **SmartCore** — Random forests, decision trees
- **ort** (ONNX Runtime) — Deploy ML models from Python (train with scikit-learn/PyTorch, export to ONNX, run in Rust)

### 4.4 MVP Feature Set

**Phase 1: Data Collection**
- HealthKit integration (iOS)
- Health Connect integration (Android)
- Manual switch logging
- Local SQLite storage

**Phase 2: Basic Analysis**
- HRV calculation (cardio-rs)
- Timeline view (switches + heart rate + HRV)
- Basic stats (avg HRV per fronter, switch frequency)

**Phase 3: Pattern Detection**
- Outlier detection (augurs)
- Seasonality analysis (daily/weekly switch patterns)
- Correlation analysis (HRV ↔ switches)

**Phase 4: Integrations**
- PluralKit API sync (import/export switches)
- Simply Plural API sync (import/export fronts)
- Oura Ring API (optional, for Oura users)

**Phase 5: Advanced Features**
- Switch prediction (oxidiviner forecasting)
- Trigger identification (ML clustering)
- Therapist reports (export PDF with biometric correlations)

### 4.5 Privacy & Security Considerations

**On-Device Processing:**
- All biometric processing happens locally (Rust core)
- No cloud requirement for basic functionality
- Optional cloud sync (user choice)

**Data Minimization:**
- Only collect necessary biometric data
- User controls which data types to track
- Granular permissions (HealthKit/Health Connect model)

**Encryption:**
- SQLite database encrypted at rest (SQLCipher)
- API tokens stored in platform keychain (iOS Keychain, Android KeyStore)
- HTTPS for all API calls

**User Control:**
- Export data anytime (JSON, CSV)
- Delete data locally and from cloud
- Opt-out of specific integrations

**Compliance:**
- HIPAA considerations if targeting clinical use
- GDPR compliance for EU users
- Transparent privacy policy

### 4.6 Challenges & Mitigations

#### Challenge: Automatic Switch Detection
- **Problem:** No ground truth for switches in biometric data
- **Mitigation:**
  - Start with manual logging + biometric correlation
  - Build labeled dataset over time
  - Train supervised ML model (switch vs. no-switch)
  - Use outlier detection as "switch candidate" alerts

#### Challenge: Individual Variability
- **Problem:** HRV patterns differ widely between people
- **Mitigation:**
  - Per-user baseline calibration
  - Relative changes (not absolute values)
  - Personalized models (per system, not global)

#### Challenge: Wearable Adoption
- **Problem:** Not all users have wearables
- **Mitigation:**
  - Prioritize HealthKit/Health Connect (phone + watch data)
  - Optional Oura/Whoop integration (power users)
  - Manual entry fallback (heart rate, sleep quality)

#### Challenge: API Reliability
- **Problem:** Third-party APIs may change/break
- **Mitigation:**
  - Graceful degradation (missing data types)
  - Version pinning for API clients
  - Regular monitoring of API deprecation notices

#### Challenge: Battery Life
- **Problem:** Continuous biometric monitoring drains battery
- **Mitigation:**
  - Background sync intervals (not real-time)
  - Batch processing (aggregate hourly/daily)
  - User-configurable sync frequency

### 4.7 Comparison to Existing Tools

| Feature | HeXuS (Proposed) | PluralKit | Simply Plural |
|---------|------------------|-----------|---------------|
| **Biometric Integration** | ✅ HealthKit, Health Connect, Oura, Whoop | ❌ None | ❌ None |
| **Automatic Detection** | ✅ ML-based pattern detection | ❌ Manual only | ❌ Manual only |
| **Pattern Analysis** | ✅ Time series ML (augurs, oxidiviner) | ❌ Basic stats | ❌ Basic stats |
| **Cross-Platform** | ✅ iOS, Android, Desktop (Tauri) | ✅ Discord only | ✅ Mobile + web |
| **API Access** | ✅ REST + WebSocket | ✅ REST | ✅ REST + WebSocket |
| **PluralKit Sync** | ✅ Import/export | — | ✅ Import/export |
| **Simply Plural Sync** | ✅ Import/export | — | — |
| **Therapist Reports** | ✅ Biometric correlations | ❌ None | ✅ Basic reports |
| **Privacy** | ✅ On-device processing | ✅ User-controlled | ✅ Private by default |
| **Open Source** | ✅ Planned | ✅ Yes | ✅ Yes (since 2021) |

**HeXuS Unique Value:**
- Only tool with biometric integration
- Only tool with automatic pattern detection
- Only tool correlating switches with physiological data
- Clinical potential (therapist dashboards)

---

## 5. Next Steps

### Immediate Actions

1. **Prototype Core:**
   - Set up Tauri 2.0 project
   - Implement HealthKit bridge (Swift → uniffi → Rust)
   - Implement Health Connect bridge (Kotlin → uniffi → Rust)
   - Basic SQLite storage

2. **Test Biometric Processing:**
   - Integrate `cardio-rs` for HRV calculation
   - Store sample data locally
   - Visualize in Tauri frontend

3. **Validate APIs:**
   - Test PluralKit API (read/write switches)
   - Test Simply Plural API (read/write fronts)
   - Test Oura Ring API (if device available)

### Research Gaps to Fill

- **ML model selection** — which algorithms work best for switch detection?
- **Baseline calibration** — how to normalize biometric data per user?
- **Real-world testing** — recruit plural systems for beta testing
- **Clinical validation** — consult therapists on useful metrics

### Community Engagement

- Present concept to plural Discord servers
- Survey community on desired features
- Open-source development (GitHub)
- Collaborate with PluralKit/Simply Plural maintainers

---

## 6. Conclusion

HeXuS sits at the intersection of three mature technology domains:

1. **Wearable APIs** — HealthKit and Health Connect provide comprehensive biometric access
2. **Plural tools** — PluralKit and Simply Plural offer established switch tracking with APIs
3. **Rust ecosystem** — Tauri, uniffi, augurs, and cardio-rs enable cross-platform, high-performance implementation

**The technology is ready.** The primary challenge is not technical feasibility, but rather:
- Designing effective pattern detection algorithms
- Building trust with the plural community
- Ensuring privacy and user control
- Validating clinical utility

**Recommended path forward:**
- Build MVP with HealthKit + Health Connect + manual switch logging
- Release to small beta group (plural systems willing to test)
- Iterate based on feedback
- Expand to wearable APIs (Oura/Whoop) and ML features

The plural community needs this tool. The technology exists. Let's build it.

---

**Document Version:** 1.0  
**Author:** Research Sub-Agent  
**Date:** 2026-03-01  
**License:** MIT (recommended for HeXuS project)
