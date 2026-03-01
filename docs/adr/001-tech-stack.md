# ADR-001: Technology Stack

## Status
Accepted

## Date
2026-03-01

## Context
HeXuS needs to run on multiple platforms (iOS, Android, Desktop) while maintaining a single source of truth for core logic. The project involves biometric data processing, local storage, pattern analysis, and eventually ML-based classification.

## Decision

### Core: Rust
- **Why:** Performance, memory safety, single codebase for core logic
- **Alignment:** Matches Knurv architecture, skills transfer between projects
- **Libraries:**
  - `rusqlite` — Local SQLite storage
  - `serde` — Serialization for data schemas
  - `tokio` — Async runtime for API calls
  - `burn` or `candle` — ML inference (when ready)

### Desktop: Tauri 2.0
- **Why:** Rust backend + web frontend, small binary size, cross-platform
- **Frontend:** Likely Svelte or React for dashboard UI
- **Target:** Windows, macOS, Linux

### Mobile: Native + Rust FFI
- **iOS:** Swift/SwiftUI with Rust core via `uniffi` or manual FFI
- **Android:** Kotlin/Compose with Rust core via JNI/`uniffi`
- **Why:** HealthKit and Health Connect require native platform access

### Wearable Integration
- **Apple Watch:** HealthKit (iOS native, bridged to Rust)
- **Wear OS/Samsung:** Health Connect API (Android native, bridged to Rust)
- **Third-party (Oura, Whoop, Garmin):** REST APIs called directly from Rust
- **CGM (Dexcom, Libre):** REST APIs + local file export parsing

## Consequences

### Positive
- Single Rust core = one place for business logic
- Type safety across the entire data pipeline
- Performance for real-time analysis
- Skills transfer with Knurv development

### Negative
- Steeper learning curve for mobile FFI
- Two thin native layers to maintain (iOS + Android)
- Rust ML ecosystem less mature than Python (mitigated by inference-only on device)

### Mitigations
- Use `uniffi` to generate bindings automatically
- Keep native layers minimal — just HealthKit/Health Connect bridges
- Heavy ML training happens on desktop, mobile only runs inference

## Alternatives Considered

### Python
- **Pro:** Faster prototyping, rich ML ecosystem
- **Con:** No good mobile story, performance concerns for real-time
- **Verdict:** Could use for desktop-only prototype, but doesn't scale to mobile vision

### Flutter + Rust
- **Pro:** Single UI codebase for mobile
- **Con:** Dart is another language to maintain, Flutter → Rust FFI adds complexity
- **Verdict:** Maybe revisit if native iOS/Android becomes painful

### React Native + Rust
- **Pro:** JavaScript UI, large ecosystem
- **Con:** Performance overhead, bridge complexity
- **Verdict:** Not worth it when we want Rust anyway

## References
- [Tauri 2.0 Mobile Support](https://tauri.app/)
- [UniFFI for Rust bindings](https://mozilla.github.io/uniffi-rs/)
- [Apple HealthKit](https://developer.apple.com/documentation/healthkit)
- [Android Health Connect](https://developer.android.com/health-and-fitness/guides/health-connect)
