# HeXuS Android Integration

Kotlin source files for Health Connect integration on Android 14+.

## Overview

This directory contains stub Kotlin files demonstrating the Health Connect integration pattern for HeXuS. These files are **design templates** — they show the architecture and data flow but are not yet connected to a compiled Rust core.

## Files

### Core Integration

- **`HealthConnectManager.kt`** — Health Connect API wrapper
  - Permissions management
  - Read heart rate, HRV, sleep, steps, calories
  - Convert to HeXuS unified format
  - Time range filtering

- **`HealthSyncWorker.kt`** — WorkManager background sync
  - Periodic sync (1-4 hour intervals)
  - Constraints (Wi-Fi, battery, storage)
  - Exponential backoff on failure
  - Sync preferences (last sync timestamp)

- **`RustBridge.kt`** — Kotlin → Rust FFI bridge
  - Stub for uniffi-generated bindings
  - Process biometric batches
  - Calculate HRV metrics
  - Detect outliers
  - Sync to desktop server

- **`GoogleFitManager.kt`** — Fallback for Android < 14
  - Google Fit API wrapper
  - Legacy device support
  - Same unified format as Health Connect
  - Unified data source abstraction

## Usage

### 1. Check Health Connect Availability

```kotlin
val manager = HealthConnectManager(context)
if (manager.isAvailable()) {
    // Proceed with Health Connect
} else {
    // Fallback to Google Fit
}
```

### 2. Request Permissions

```kotlin
// In Activity
val requestPermissions = registerForActivityResult(
    ActivityResultContracts.RequestMultiplePermissions()
) { permissions ->
    if (permissions.all { it.value }) {
        // All permissions granted
        lifecycleScope.launch {
            syncData()
        }
    }
}

lifecycleScope.launch {
    if (!manager.hasAllPermissions()) {
        requestPermissions.launch(
            HealthConnectManager.REQUIRED_PERMISSIONS
                .map { it.permissionType }
                .toTypedArray()
        )
    }
}
```

### 3. Schedule Background Sync

```kotlin
// In Application.onCreate()
HealthSyncWorker.schedule(
    context = this,
    intervalHours = 1,  // Sync every 1 hour
    requireUnmeteredNetwork = true  // Wi-Fi only
)
```

### 4. Manual Sync (One-Time)

```kotlin
HealthSyncWorker.runOnce(context)
```

### 5. Read Data Manually

```kotlin
val manager = HealthConnectManager(context)
val startTime = Instant.now().minus(24, ChronoUnit.HOURS)
val endTime = Instant.now()

val heartRate = manager.readHeartRate(startTime, endTime)
val hrv = manager.readHRV(startTime, endTime)
val sleep = manager.readSleep(startTime, endTime)
```

### 6. Unified Data Source (Auto-Fallback)

```kotlin
val dataSource = UnifiedHealthDataSource.create(context)
val batch = dataSource.syncAllData(lastSyncTime = null)

// Automatically uses Health Connect if available, Google Fit otherwise
```

## Data Flow

```
Health Connect (System)
    │
    ▼ Read via HealthConnectClient
HealthConnectManager (Kotlin)
    │
    ▼ Convert to unified format
BiometricBatch (heartRate, hrv, sleep, steps, calories)
    │
    ▼ Background sync via WorkManager
HealthSyncWorker
    │
    ▼ FFI call via uniffi
RustBridge
    │
    ▼ Process and store
Rust Core (HRV calculation, outlier detection, SQLite storage)
    │
    ▼ Optional sync
HeXuS Desktop Server
```

## Build Integration

### Dependencies (build.gradle.kts)

```kotlin
dependencies {
    // Health Connect
    implementation("androidx.health.connect:connect-client:1.1.0-alpha10")
    
    // WorkManager
    implementation("androidx.work:work-runtime-ktx:2.9.0")
    
    // Coroutines
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.7.3")
    
    // Google Fit (fallback)
    implementation("com.google.android.gms:play-services-fitness:21.1.0")
    implementation("com.google.android.gms:play-services-auth:20.7.0")
}
```

### Rust Core Integration (uniffi)

1. **Build Rust core for Android:**
   ```bash
   cd ../../rust-core
   rustup target add aarch64-linux-android
   cargo build --target aarch64-linux-android --release
   ```

2. **Generate Kotlin bindings:**
   ```bash
   cargo install uniffi-bindgen
   uniffi-bindgen generate src/lib.rs --language kotlin --out-dir ../mobile/android/uniffi
   ```

3. **Copy native library:**
   ```bash
   mkdir -p ../mobile/android/app/src/main/jniLibs/arm64-v8a
   cp target/aarch64-linux-android/release/libhexus_core.so \
      ../mobile/android/app/src/main/jniLibs/arm64-v8a/
   ```

4. **Import generated bindings in RustBridge.kt:**
   ```kotlin
   import uniffi.hexus_core.*
   
   // Uncomment FFI calls in RustBridge.kt
   ```

## Permissions

### Required in AndroidManifest.xml

```xml
<!-- Health Connect -->
<uses-permission android:name="android.permission.health.READ_HEART_RATE" />
<uses-permission android:name="android.permission.health.READ_HEART_RATE_VARIABILITY_RMSSD" />
<uses-permission android:name="android.permission.health.READ_SLEEP" />
<uses-permission android:name="android.permission.health.READ_STEPS" />
<uses-permission android:name="android.permission.health.READ_ACTIVE_CALORIES_BURNED" />

<!-- Network for desktop sync -->
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />

<!-- Google Fit (fallback) -->
<uses-permission android:name="android.permission.ACTIVITY_RECOGNITION" />
```

## Testing

### Unit Tests

```kotlin
@Test
fun testHeartRateReading() = runBlocking {
    val manager = HealthConnectManager(context)
    val samples = manager.readHeartRate(startTime, endTime)
    
    assertTrue(samples.isNotEmpty())
    assertTrue(samples.all { it.beatsPerMinute > 0 })
}
```

### Integration Tests

1. **Install Health Connect app** on test device/emulator
2. **Add test data** via Health Connect settings
3. **Run HealthSyncWorker** manually: `HealthSyncWorker.runOnce(context)`
4. **Verify** data appears in local database

## Troubleshooting

### "Health Connect not available"
- Check Android version (14+ required)
- Install Health Connect app from Play Store
- Use Google Fit fallback: `UnifiedHealthDataSource.create(context)`

### "Missing permissions"
- Verify `AndroidManifest.xml` declarations
- Request runtime permissions via `PermissionController`
- Check user granted permissions in Health Connect app

### "WorkManager not running"
- Check constraints (Wi-Fi, battery, storage)
- View logs: `adb logcat | grep HealthSyncWorker`
- Test immediately: `HealthSyncWorker.runOnce(context)`

### "Rust FFI errors"
- Verify `libhexus_core.so` in `jniLibs/arm64-v8a/`
- Regenerate uniffi bindings if Rust API changed
- Check Rust panic logs: `adb logcat | grep RUST`

## Next Steps

1. **Implement Rust core** (`../../rust-core/lib.rs`)
   - Define uniffi interface
   - Implement biometric processing
   - Integrate cardio-rs for HRV
   - Integrate augurs for outlier detection

2. **Generate uniffi bindings** and integrate with Kotlin

3. **Test on real Android 14+ device** (Pixel, Samsung Galaxy)

4. **Implement Google Fit fallback** for older devices

5. **Build Tauri mobile app** with this Kotlin layer

6. **Integrate with desktop server** sync API

## Documentation

Full documentation: `../../docs/android/HEALTH-CONNECT-INTEGRATION.md`

## License

MIT
