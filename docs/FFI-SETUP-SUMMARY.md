# HeXuS uniffi FFI Setup - Summary

**Completed:** 2026-03-09  
**Commit:** b3924d1

## What Was Done

### 1. Dependencies Added
- **uniffi 0.28** added to `crates/hexus-core/Cargo.toml`
- Build dependency for uniffi code generation
- Changed crate type to support FFI: `["lib", "cdylib", "staticlib"]`

### 2. FFI Layer Created (`src/ffi.rs`)

**Key Features:**
- Thin FFI wrapper around core Rust types
- Thread-safe Database wrapper (`Arc<Mutex<CoreDatabase>>`) 
  - Required because `rusqlite::Connection` is not `Sync`
  - Ensures thread-safe access from mobile platforms
- Automatic type conversion between FFI types and core types
- Comprehensive error handling (HeXuSError → mobile exceptions)

**Exposed API:**
- **Namespace functions:**
  - `open_database(path: String) -> Database`
  - `create_in_memory_database() -> Database`

- **Database methods:**
  - Alter CRUD: create, get, list, update, delete
  - Fronting log CRUD: create, get, query, update, delete
  - Biometric sample CRUD: create, create_batch, get, query, delete
  - Baseline operations: upsert, get, list, delete

### 3. uniffi Interface Definition (`src/hexus.udl`)

UDL (uniffi Definition Language) file defining:
- Data structures (dictionaries)
- Enums (BiometricMetric, FrontingSourceType)
- Error types
- Database interface
- Namespace functions

This file generates Swift and Kotlin bindings automatically.

### 4. Build Configuration

**`build.rs`:**
```rust
fn main() {
    uniffi::generate_scaffolding("src/hexus.udl").unwrap();
}
```

**`uniffi.toml`:**
- Swift module name: `HeXuSCore`
- Kotlin package: `dev.hexus.core`

### 5. Documentation (`docs/MOBILE-FFI.md`)

Comprehensive guide covering:
- Architecture overview
- Build instructions for iOS and Android
- Complete API reference with Swift/Kotlin examples
- Platform-specific integration (HealthKit, Health Connect)
- Best practices and performance tips
- Troubleshooting guide

### 6. Supporting Files

**Desktop stub** (`crates/hexus-desktop/`):
- Created to satisfy workspace dependencies
- Placeholder for future desktop application

**Mobile integration examples:**
- `mobile/ios/HeXuS/HealthKitManager.swift` — iOS HealthKit integration
- `mobile/android/HealthConnectManager.kt` — Android Health Connect integration

## How to Use (Quick Start)

### For iOS:

```bash
# Add iOS targets
rustup target add aarch64-apple-ios aarch64-apple-ios-sim

# Build for iOS
cd crates/hexus-core
cargo build --release --target aarch64-apple-ios

# Generate Swift bindings
uniffi-bindgen generate src/hexus.udl --language swift --out-dir ../../mobile/ios/Generated
```

### For Android:

```bash
# Add Android targets
rustup target add aarch64-linux-android

# Build for Android
cargo build --release --target aarch64-linux-android

# Generate Kotlin bindings
uniffi-bindgen generate src/hexus.udl --language kotlin --out-dir ../../mobile/android/Generated
```

## Architecture Benefits

1. **Single Source of Truth:**
   - All business logic lives in Rust
   - Mobile apps are thin clients calling Rust functions
   - No logic duplication across platforms

2. **Type Safety:**
   - Strong typing enforced at FFI boundary
   - Compile-time guarantees (no runtime type errors)

3. **Performance:**
   - Rust handles heavy database operations
   - Efficient batch operations for biometric imports
   - Minimal overhead crossing FFI boundary

4. **Maintainability:**
   - One codebase for storage/logic (Rust)
   - Platform-specific code only for UI and native APIs
   - Easy to add new platforms (uniffi supports more languages)

## Thread Safety Note

The FFI Database wrapper uses `Arc<Mutex<CoreDatabase>>` because:
- `rusqlite::Connection` is not `Sync` (uses internal `RefCell`)
- uniffi requires `Send + Sync` for Arc-wrapped objects
- Mutex ensures safe concurrent access from mobile threads
- Performance impact is minimal (SQLite serializes writes anyway)

## Testing

```bash
# Verify build
cd /home/node/.openclaw/workspace/HeXuS
cargo check

# Run tests
cargo test

# Build for specific mobile target
cargo build --target aarch64-apple-ios
```

## Next Steps

1. **Mobile App Implementation:**
   - iOS: SwiftUI app using generated `HeXuSCore` module
   - Android: Jetpack Compose app using generated Kotlin bindings

2. **Desktop Application:**
   - Implement `hexus-desktop` using Tauri 2.0
   - Web dashboard for data visualization
   - Wearable API integration (Oura, Whoop, etc.)

3. **Sync Protocol:**
   - Implement mobile ↔ desktop sync
   - WebSocket for real-time updates
   - Conflict resolution for offline changes

4. **ML Integration:**
   - Add analysis module FFI bindings
   - Expose HRV calculations to mobile
   - Switch prediction via ONNX models

## Files Created/Modified

**New Files:**
- `crates/hexus-core/src/ffi.rs` (363 lines)
- `crates/hexus-core/src/hexus.udl` (149 lines)
- `crates/hexus-core/build.rs`
- `crates/hexus-core/uniffi.toml`
- `docs/MOBILE-FFI.md` (800+ lines)
- `crates/hexus-desktop/` (stub)

**Modified:**
- `crates/hexus-core/Cargo.toml` (added uniffi deps)
- `crates/hexus-core/src/lib.rs` (added FFI module, scaffolding)

**Generated (not committed):**
- `Cargo.lock` (dependency lockfile)

## Resources

- [uniffi-rs Documentation](https://mozilla.github.io/uniffi-rs/)
- [uniffi UDL Guide](https://mozilla.github.io/uniffi-rs/udl_file_spec.html)
- [HeXuS Architecture Docs](./ARCHITECTURE.md)
- [Mobile FFI API Reference](./MOBILE-FFI.md)

---

**Status:** ✅ Complete and tested  
**Ready for:** Mobile app development
