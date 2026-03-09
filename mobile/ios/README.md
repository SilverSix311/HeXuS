# HeXuS iOS Integration

This directory contains the iOS-specific implementation for HeXuS biometric monitoring.

## File Structure

```
mobile/ios/
├── HeXuS/
│   ├── HealthKitManager.swift    # HealthKit integration (queries, background sync)
│   ├── RustBridge.swift           # Swift ↔ Rust FFI bridge (uniffi-generated)
│   ├── AppDelegate.swift          # App lifecycle, background tasks
│   ├── Info.plist                 # App configuration, HealthKit permissions
│   └── Generated/                 # uniffi-generated bindings (not yet created)
└── README.md                      # This file
```

## Implementation Status

✅ **Completed:**
- HealthKit authorization flow
- Background observer queries (heart rate, HRV, sleep, steps)
- Anchored incremental sync (no duplicates)
- Data structure definitions (Swift ↔ Rust compatibility)
- Info.plist privacy strings
- Background task registration

⚠️ **Stub/TODO:**
- Rust core FFI bindings (needs `mobile/rust-core/` implementation)
- XCFramework build (requires Rust compilation for iOS targets)
- Actual uniffi code generation
- SQLite integration (Rust side)
- Desktop server sync

## Next Steps

### 1. Implement Rust Core

Create `mobile/rust-core/` with:

```rust
// lib.rs
uniffi::setup_scaffolding!();

#[derive(uniffi::Record)]
pub struct HealthSample {
    pub sample_type: String,
    pub value: f64,
    pub start_date: String,
    pub end_date: String,
    pub source: String,
    pub device: Option<String>,
}

#[uniffi::export]
pub fn process_heart_rate_samples(samples: Vec<HealthSample>) -> Result<String, String> {
    // Implement processing logic
    Ok(format!("Processed {} samples", samples.len()))
}
```

### 2. Generate Bindings

```bash
cd mobile/rust-core
cargo build --release --target aarch64-apple-ios
cargo run --bin uniffi-bindgen generate \
  --library target/debug/libhexus_core.dylib \
  --language swift \
  --out-dir ../ios/HeXuS/Generated
```

### 3. Build XCFramework

```bash
# Build for all iOS targets
cargo build --release --target aarch64-apple-ios         # Device
cargo build --release --target aarch64-apple-ios-sim     # M1 Simulator
cargo build --release --target x86_64-apple-ios          # Intel Simulator

# Create XCFramework
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libhexus_core.a \
  -library target/aarch64-apple-ios-sim/release/libhexus_core.a \
  -library target/x86_64-apple-ios/release/libhexus_core.a \
  -output ../ios/Frameworks/HeXuSCore.xcframework
```

### 4. Xcode Project Setup

1. Create new iOS app in Xcode
2. Add Swift files from `HeXuS/` directory
3. Add XCFramework to project (Embed & Sign)
4. Enable HealthKit capability (Signing & Capabilities)
5. Enable Background Modes: `fetch`, `processing`
6. Test on physical device (HealthKit unavailable in Simulator)

## Testing

### Manual Testing

1. Install on iPhone (requires physical device)
2. Grant HealthKit permissions when prompted
3. Generate test data:
   - Use Breathe app (triggers HRV measurement)
   - Go for a walk (step count + heart rate)
   - Log sleep in Health app
4. Check Xcode Console for observer query logs
5. Verify data in SQLite database (Rust side)

### Automated Testing

```swift
// HeXuSTests/HealthKitManagerTests.swift
import XCTest
@testable import HeXuS

class HealthKitManagerTests: XCTestCase {
    func testConvertHKSampleToRustStruct() {
        // Test HKQuantitySample → HealthSample conversion
    }
}
```

## Privacy Considerations

- HealthKit data never leaves the device except for:
  - Optional sync to user's desktop server (local network only)
  - User-initiated exports
- No telemetry, no third-party analytics
- All processing happens locally (Rust core)
- SQLite database encrypted at rest (SQLCipher)

## Documentation

See `docs/ios/HEALTHKIT-INTEGRATION.md` for complete technical specification including:
- HealthKit API reference
- Background fetch strategy
- Data flow diagrams
- Troubleshooting guide
- Complete code examples

## License

MIT (same as HeXuS project)
