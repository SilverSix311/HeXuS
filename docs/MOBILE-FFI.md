# HeXuS Mobile FFI Documentation

This document describes the Rust → Swift/Kotlin FFI (Foreign Function Interface) for HeXuS mobile applications.

## Overview

HeXuS uses **uniffi-rs** to generate Swift and Kotlin bindings from Rust code. This allows mobile apps to call directly into the Rust core library for all business logic and storage operations.

**Key Design Principle:** The FFI layer is thin. Mobile apps should call Rust functions directly rather than reimplementing logic in Swift/Kotlin. This ensures:

- **Consistency:** Business logic lives in one place (Rust)
- **Performance:** Rust handles heavy operations (database queries, analysis)
- **Maintainability:** One codebase for storage/logic across all platforms

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Mobile App (Swift/Kotlin)            │
│                                                          │
│  ┌────────────────────────────────────────────────────┐ │
│  │  Platform-Specific Code                            │ │
│  │  • HealthKit (iOS) / Health Connect (Android)      │ │
│  │  • UI Components (SwiftUI / Jetpack Compose)       │ │
│  │  • Background Sync Workers                         │ │
│  └──────────────────┬─────────────────────────────────┘ │
│                     │                                    │
│                     ▼                                    │
│  ┌────────────────────────────────────────────────────┐ │
│  │  HeXuS Core FFI (Generated Bindings)               │ │
│  │  • open_database()                                 │ │
│  │  • create_alter()                                  │ │
│  │  • create_biometric_sample()                       │ │
│  │  • query_biometric_samples()                       │ │
│  └──────────────────┬─────────────────────────────────┘ │
└────────────────────┼──────────────────────────────────┘
                     │
                     ▼
         ┌────────────────────────┐
         │  Rust Core Library     │
         │  (hexus-core)          │
         │  • Database (SQLite)   │
         │  • Models              │
         │  • Business Logic      │
         └────────────────────────┘
```

---

## Building for Mobile

### Prerequisites

1. **Rust toolchain** with mobile targets:
   ```bash
   # iOS targets
   rustup target add aarch64-apple-ios      # iOS devices (arm64)
   rustup target add aarch64-apple-ios-sim  # iOS simulator (arm64, M1+ Macs)
   rustup target add x86_64-apple-ios       # iOS simulator (Intel Macs)
   
   # Android targets
   rustup target add aarch64-linux-android  # Android arm64
   rustup target add armv7-linux-androideabi # Android armv7
   rustup target add x86_64-linux-android   # Android x86_64 (emulator)
   rustup target add i686-linux-android     # Android i686
   ```

2. **uniffi-bindgen** CLI tool:
   ```bash
   cargo install uniffi-bindgen
   ```

3. **Android NDK** (for Android builds)
   - Download from [Android NDK Releases](https://developer.android.com/ndk/downloads)
   - Set `ANDROID_NDK_HOME` environment variable

---

### Generate Swift Bindings

```bash
cd crates/hexus-core

# Build for iOS devices
cargo build --release --target aarch64-apple-ios

# Build for iOS simulator (M1+ Macs)
cargo build --release --target aarch64-apple-ios-sim

# Generate Swift bindings
uniffi-bindgen generate src/hexus.udl --language swift --out-dir ../../mobile/ios/HeXuS/Generated

# The generated files will be:
# - HeXuSCore.swift       (Swift API)
# - HeXuSCoreFFI.h        (C header)
# - HeXuSCoreFFI.modulemap (Module map)
```

**Add to Xcode:**
1. Drag `HeXuSCore.swift` into your Xcode project
2. Link the compiled Rust library:
   - Copy `target/aarch64-apple-ios/release/libhexus_core.a` to your project
   - Add to "Link Binary With Libraries" build phase
3. Add `HeXuSCoreFFI.modulemap` to bridging header (if needed)

---

### Generate Kotlin Bindings

```bash
cd crates/hexus-core

# Build for Android arm64 (most devices)
cargo build --release --target aarch64-linux-android

# Build for other Android targets if needed
cargo build --release --target armv7-linux-androideabi
cargo build --release --target x86_64-linux-android

# Generate Kotlin bindings
uniffi-bindgen generate src/hexus.udl --language kotlin --out-dir ../../mobile/android/app/src/main/kotlin/generated

# The generated file will be:
# - dev/hexus/core/HeXuSCore.kt (Kotlin API)
```

**Setup in Android Studio:**

1. **Add JNI libraries:**
   - Copy compiled `.so` files to `app/src/main/jniLibs/<arch>/`
   - Example: `libhexus_core.so` → `jniLibs/arm64-v8a/libhexus_core.so`

2. **Add Kotlin file to project:**
   - Place `HeXuSCore.kt` in `app/src/main/kotlin/dev/hexus/core/`

3. **Load native library:**
   ```kotlin
   // In your Application class or main activity
   companion object {
       init {
           System.loadLibrary("hexus_core")
       }
   }
   ```

---

## FFI API Reference

### Initialization

#### `open_database(path: String) -> Database`

Opens or creates a SQLite database at the given path.

**Swift:**
```swift
import HeXuSCore

do {
    let dbPath = FileManager.default
        .urls(for: .documentDirectory, in: .userDomainMask)[0]
        .appendingPathComponent("hexus.db")
        .path
    
    let db = try openDatabase(path: dbPath)
} catch {
    print("Failed to open database: \(error)")
}
```

**Kotlin:**
```kotlin
import dev.hexus.core.*

val dbPath = context.getDatabasePath("hexus.db").absolutePath
val db = openDatabase(dbPath)
```

#### `create_in_memory_database() -> Database`

Creates an in-memory database (useful for testing).

**Swift:**
```swift
let db = try createInMemoryDatabase()
```

**Kotlin:**
```kotlin
val db = createInMemoryDatabase()
```

---

### Alter Management

#### `Database.create_alter(alter: Alter)`

Creates a new alter in the database.

**Swift:**
```swift
let alter = Alter(
    id: UUID().uuidString,
    name: "Alice",
    color: "#FF5733",
    emoji: "🌸",
    notes: "Primary host",
    createdAt: Int64(Date().timeIntervalSince1970)
)

try db.createAlter(alter: alter)
```

**Kotlin:**
```kotlin
val alter = Alter(
    id = UUID.randomUUID().toString(),
    name = "Alice",
    color = "#FF5733",
    emoji = "🌸",
    notes = "Primary host",
    createdAt = System.currentTimeMillis() / 1000
)

db.createAlter(alter)
```

#### `Database.get_alter(id: String) -> Alter`

Retrieves an alter by ID.

**Swift:**
```swift
let alter = try db.getAlter(id: "alice-id")
print("Alter name: \(alter.name)")
```

**Kotlin:**
```kotlin
val alter = db.getAlter("alice-id")
println("Alter name: ${alter.name}")
```

#### `Database.list_alters() -> [Alter]`

Lists all alters.

**Swift:**
```swift
let alters = try db.listAlters()
for alter in alters {
    print("- \(alter.name) (\(alter.emoji ?? ""))")
}
```

**Kotlin:**
```kotlin
val alters = db.listAlters()
alters.forEach { alter ->
    println("- ${alter.name} (${alter.emoji ?: ""})")
}
```

#### `Database.update_alter(alter: Alter)`

Updates an existing alter.

**Swift:**
```swift
var alter = try db.getAlter(id: "alice-id")
alter.name = "Alice (Host)"
try db.updateAlter(alter: alter)
```

**Kotlin:**
```kotlin
val alter = db.getAlter("alice-id")
val updated = alter.copy(name = "Alice (Host)")
db.updateAlter(updated)
```

#### `Database.delete_alter(id: String)`

Deletes an alter (cascades to fronting logs and baselines).

**Swift:**
```swift
try db.deleteAlter(id: "alice-id")
```

**Kotlin:**
```kotlin
db.deleteAlter("alice-id")
```

---

### Fronting Log Management

#### `Database.create_fronting_log(log: FrontingLog)`

Logs a fronting state.

**Swift:**
```swift
let log = FrontingLog(
    id: UUID().uuidString,
    alterId: "alice-id",
    confidence: 1.0,
    source: FrontingSource(
        sourceType: .manual,
        extraData: nil
    ),
    startedAt: Int64(Date().timeIntervalSince1970),
    endedAt: nil,
    notes: "Morning switch"
)

try db.createFrontingLog(log: log)
```

**Kotlin:**
```kotlin
val log = FrontingLog(
    id = UUID.randomUUID().toString(),
    alterId = "alice-id",
    confidence = 1.0f,
    source = FrontingSource(
        sourceType = FrontingSourceType.MANUAL,
        extraData = null
    ),
    startedAt = System.currentTimeMillis() / 1000,
    endedAt = null,
    notes = "Morning switch"
)

db.createFrontingLog(log)
```

#### `Database.query_fronting_logs(query: FrontingLogQuery) -> [FrontingLog]`

Queries fronting logs with filters.

**Swift:**
```swift
let query = FrontingLogQuery(
    alterId: "alice-id",
    startTime: Int64(Date().timeIntervalSince1970) - 86400, // Last 24 hours
    endTime: nil,
    limit: 50
)

let logs = try db.queryFrontingLogs(query: query)
```

**Kotlin:**
```kotlin
val query = FrontingLogQuery(
    alterId = "alice-id",
    startTime = (System.currentTimeMillis() / 1000) - 86400, // Last 24 hours
    endTime = null,
    limit = 50u
)

val logs = db.queryFrontingLogs(query)
```

---

### Biometric Data Management

#### `Database.create_biometric_sample(sample: BiometricSample)`

Stores a single biometric sample.

**Swift:**
```swift
let sample = BiometricSample(
    id: UUID().uuidString,
    timestamp: Int64(Date().timeIntervalSince1970),
    metric: .heartRate,
    value: 72.5,
    unit: "bpm",
    source: "healthkit",
    frontingId: nil
)

try db.createBiometricSample(sample: sample)
```

**Kotlin:**
```kotlin
val sample = BiometricSample(
    id = UUID.randomUUID().toString(),
    timestamp = System.currentTimeMillis() / 1000,
    metric = BiometricMetric.HEART_RATE,
    value = 72.5,
    unit = "bpm",
    source = "healthconnect",
    frontingId = null
)

db.createBiometricSample(sample)
```

#### `Database.create_biometric_samples_batch(samples: [BiometricSample])`

Stores multiple samples efficiently (use for bulk HealthKit/Health Connect imports).

**Swift:**
```swift
let samples: [BiometricSample] = healthKitData.map { hkSample in
    BiometricSample(
        id: UUID().uuidString,
        timestamp: Int64(hkSample.startDate.timeIntervalSince1970),
        metric: .heartRate,
        value: hkSample.quantity.doubleValue(for: HKUnit.count().unitDivided(by: .minute())),
        unit: "bpm",
        source: "healthkit",
        frontingId: nil
    )
}

try db.createBiometricSamplesBatch(samples: samples)
```

**Kotlin:**
```kotlin
val samples = healthConnectRecords.map { record ->
    BiometricSample(
        id = UUID.randomUUID().toString(),
        timestamp = record.time.epochSecond,
        metric = BiometricMetric.HEART_RATE,
        value = record.beatsPerMinute.toDouble(),
        unit = "bpm",
        source = "healthconnect",
        frontingId = null
    )
}

db.createBiometricSamplesBatch(samples)
```

#### `Database.query_biometric_samples(query: BiometricQuery) -> [BiometricSample]`

Queries biometric samples with filters.

**Swift:**
```swift
let query = BiometricQuery(
    metric: .heartRate,
    startTime: Int64(Date().timeIntervalSince1970) - 3600, // Last hour
    endTime: nil,
    source: "healthkit",
    limit: 100
)

let samples = try db.queryBiometricSamples(query: query)

// Calculate average
let avgHeartRate = samples.map(\.value).reduce(0, +) / Double(samples.count)
print("Average HR: \(avgHeartRate) bpm")
```

**Kotlin:**
```kotlin
val query = BiometricQuery(
    metric = BiometricMetric.HEART_RATE,
    startTime = (System.currentTimeMillis() / 1000) - 3600, // Last hour
    endTime = null,
    source = "healthconnect",
    limit = 100u
)

val samples = db.queryBiometricSamples(query)

// Calculate average
val avgHeartRate = samples.map { it.value }.average()
println("Average HR: $avgHeartRate bpm")
```

---

### Baseline Management

#### `Database.upsert_baseline(baseline: AlterBaseline)`

Creates or updates a baseline profile for an alter.

**Swift:**
```swift
let baseline = AlterBaseline(
    alterId: "alice-id",
    metric: .heartRate,
    mean: 70.0,
    stdDev: 5.0,
    min: 55.0,
    max: 90.0,
    sampleCount: 1000,
    updatedAt: Int64(Date().timeIntervalSince1970)
)

try db.upsertBaseline(baseline: baseline)
```

**Kotlin:**
```kotlin
val baseline = AlterBaseline(
    alterId = "alice-id",
    metric = BiometricMetric.HEART_RATE,
    mean = 70.0,
    stdDev = 5.0,
    min = 55.0,
    max = 90.0,
    sampleCount = 1000u,
    updatedAt = System.currentTimeMillis() / 1000
)

db.upsertBaseline(baseline)
```

#### `Database.get_baseline(alter_id: String, metric: BiometricMetric) -> AlterBaseline`

Retrieves a baseline for a specific alter and metric.

**Swift:**
```swift
let baseline = try db.getBaseline(alterId: "alice-id", metric: .heartRate)
print("Alice's baseline HR: \(baseline.mean) ± \(baseline.stdDev) bpm")
```

**Kotlin:**
```kotlin
val baseline = db.getBaseline("alice-id", BiometricMetric.HEART_RATE)
println("Alice's baseline HR: ${baseline.mean} ± ${baseline.stdDev} bpm")
```

---

## Data Types

### `Alter`

| Field | Type | Description |
|-------|------|-------------|
| `id` | String | Unique identifier (UUID recommended) |
| `name` | String | Display name |
| `color` | String? | Hex color code (e.g., "#FF5733") |
| `emoji` | String? | Emoji representation |
| `notes` | String? | Optional notes |
| `created_at` | i64 | Unix timestamp (seconds) |

### `FrontingLog`

| Field | Type | Description |
|-------|------|-------------|
| `id` | String | Unique identifier |
| `alter_id` | String? | ID of fronting alter (null = unknown) |
| `confidence` | f32? | Confidence level (0.0-1.0) |
| `source` | FrontingSource | How this was determined |
| `started_at` | i64 | Unix timestamp (seconds) |
| `ended_at` | i64? | Unix timestamp (null = still fronting) |
| `notes` | String? | Optional notes |

### `BiometricSample`

| Field | Type | Description |
|-------|------|-------------|
| `id` | String | Unique identifier |
| `timestamp` | i64 | Unix timestamp (seconds) |
| `metric` | BiometricMetric | Type of measurement |
| `value` | f64 | Measured value |
| `unit` | String | Unit (e.g., "bpm", "ms", "mg/dL") |
| `source` | String | Source device/app |
| `fronting_id` | String? | Associated fronting log (for labeled data) |

### `AlterBaseline`

| Field | Type | Description |
|-------|------|-------------|
| `alter_id` | String | Alter ID |
| `metric` | BiometricMetric | Metric type |
| `mean` | f64 | Average value |
| `std_dev` | f64 | Standard deviation |
| `min` | f64 | Minimum observed |
| `max` | f64 | Maximum observed |
| `sample_count` | u32 | Number of samples |
| `updated_at` | i64 | Unix timestamp (seconds) |

### `BiometricMetric` Enum

- `HeartRate`
- `HeartRateVariability`
- `RestingHeartRate`
- `GalvanicSkinResponse`
- `SkinConductance`
- `BloodGlucose`
- `SleepDuration`
- `SleepStage`
- `Steps`
- `ActiveCalories`
- `SkinTemperature`
- `RespiratoryRate`
- `BloodOxygen`

### `FrontingSource`

| Field | Type | Description |
|-------|------|-------------|
| `source_type` | FrontingSourceType | Manual, Biometric, or Import |
| `extra_data` | String? | Additional context (model version, import source) |

### `FrontingSourceType` Enum

- `Manual` — User manually logged the switch
- `Biometric` — Inferred from biometric patterns
- `Import` — Imported from another app

---

## Error Handling

All FFI functions that can fail throw/return a `HeXuSError`.

### Swift Error Handling

```swift
do {
    let alter = try db.getAlter(id: "nonexistent-id")
} catch let error as HeXuSError {
    switch error {
    case .NotFound(let msg):
        print("Not found: \(msg)")
    case .DatabaseError(let msg):
        print("Database error: \(msg)")
    default:
        print("Error: \(error)")
    }
}
```

### Kotlin Error Handling

```kotlin
try {
    val alter = db.getAlter("nonexistent-id")
} catch (e: HeXuSException.NotFound) {
    println("Not found: ${e.message}")
} catch (e: HeXuSException.DatabaseError) {
    println("Database error: ${e.message}")
} catch (e: HeXuSException) {
    println("Error: ${e.message}")
}
```

---

## Best Practices

### 1. Use Batch Operations for Bulk Inserts

When importing HealthKit or Health Connect data, use `create_biometric_samples_batch()` instead of calling `create_biometric_sample()` in a loop. Batch operations use a database transaction and are **much faster**.

**Good:**
```swift
let samples = healthKitData.map { ... }
try db.createBiometricSamplesBatch(samples: samples)
```

**Bad:**
```swift
for hkSample in healthKitData {
    let sample = BiometricSample(...)
    try db.createBiometricSample(sample: sample) // Slow!
}
```

### 2. Use Unix Timestamps (Seconds)

All timestamps in the FFI use **Unix seconds** (not milliseconds). Convert your platform's time format accordingly.

**Swift:**
```swift
let unixSeconds = Int64(Date().timeIntervalSince1970)
```

**Kotlin:**
```swift
val unixSeconds = System.currentTimeMillis() / 1000
```

### 3. Generate UUIDs for IDs

Use platform UUIDs for `id` fields to avoid collisions.

**Swift:**
```swift
let id = UUID().uuidString
```

**Kotlin:**
```kotlin
val id = UUID.randomUUID().toString()
```

### 4. Query with Limits

When querying large datasets, always use `limit` to avoid loading too much data into memory.

```swift
let query = BiometricQuery(
    metric: .heartRate,
    startTime: nil,
    endTime: nil,
    source: nil,
    limit: 1000 // Limit to 1000 most recent
)
```

### 5. Handle Errors Gracefully

Database operations can fail (disk full, corrupted database, etc.). Always catch errors and provide user-friendly messages.

---

## Platform-Specific Integration

### iOS: HealthKit → Rust

```swift
import HealthKit

func importHealthKitData(db: Database) async throws {
    let healthStore = HKHealthStore()
    
    // Request authorization
    let heartRateType = HKObjectType.quantityType(forIdentifier: .heartRate)!
    try await healthStore.requestAuthorization(toShare: [], read: [heartRateType])
    
    // Query heart rate data
    let query = HKSampleQuery(
        sampleType: heartRateType,
        predicate: nil,
        limit: 1000,
        sortDescriptors: [NSSortDescriptor(key: HKSampleSortIdentifierStartDate, ascending: false)]
    ) { _, samples, error in
        guard let samples = samples as? [HKQuantitySample] else { return }
        
        // Convert to HeXuS samples
        let hexusSamples = samples.map { hkSample in
            BiometricSample(
                id: UUID().uuidString,
                timestamp: Int64(hkSample.startDate.timeIntervalSince1970),
                metric: .heartRate,
                value: hkSample.quantity.doubleValue(for: HKUnit.count().unitDivided(by: .minute())),
                unit: "bpm",
                source: "healthkit",
                frontingId: nil
            )
        }
        
        // Batch insert
        try? db.createBiometricSamplesBatch(samples: hexusSamples)
    }
    
    healthStore.execute(query)
}
```

### Android: Health Connect → Rust

```kotlin
import androidx.health.connect.client.HealthConnectClient
import androidx.health.connect.client.records.HeartRateRecord
import androidx.health.connect.client.request.ReadRecordsRequest
import androidx.health.connect.client.time.TimeRangeFilter

suspend fun importHealthConnectData(db: Database, context: Context) {
    val healthConnectClient = HealthConnectClient.getOrCreate(context)
    
    // Read heart rate records
    val request = ReadRecordsRequest(
        recordType = HeartRateRecord::class,
        timeRangeFilter = TimeRangeFilter.none(),
        limit = 1000
    )
    
    val response = healthConnectClient.readRecords(request)
    
    // Convert to HeXuS samples
    val hexusSamples = response.records.map { record ->
        BiometricSample(
            id = UUID.randomUUID().toString(),
            timestamp = record.time.epochSecond,
            metric = BiometricMetric.HEART_RATE,
            value = record.beatsPerMinute.toDouble(),
            unit = "bpm",
            source = "healthconnect",
            frontingId = null
        )
    }
    
    // Batch insert
    db.createBiometricSamplesBatch(hexusSamples)
}
```

---

## Troubleshooting

### Build Errors

**"uniffi scaffolding not found"**
- Ensure `build.rs` exists and is configured correctly
- Run `cargo clean` and rebuild

**"Cannot find module 'HeXuSCore'"**
- Verify `uniffi-bindgen` generated the Swift files
- Check Xcode project includes `HeXuSCore.swift`
- Ensure `.modulemap` is accessible

### Runtime Errors

**"Database is locked"**
- SQLite doesn't support concurrent writes from multiple threads
- Ensure all database operations happen on a single queue/thread

**"No such table: alters"**
- Database schema not initialized
- Verify `open_database()` was called successfully

**"Symbol not found: _uniffi_hexus_core_..."**
- Rust library not linked correctly
- Check Xcode build phases or Android `jniLibs` folder

---

## Performance Tips

1. **Batch inserts:** Always use `create_biometric_samples_batch()` for multiple samples
2. **Query with limits:** Don't load entire database into memory
3. **Background threads:** Run database operations off the main thread (iOS) / use coroutines (Android)
4. **Caching:** Cache frequently accessed data (alter list, baselines) in memory
5. **Indexes:** The database is pre-indexed on timestamp and metric columns

---

## Next Steps

- **iOS:** See `mobile/ios/HeXuS/HealthKitManager.swift` for HealthKit integration
- **Android:** See `mobile/android/app/.../HealthConnectManager.kt` for Health Connect integration
- **Sync Protocol:** See `docs/SYNC-PROTOCOL.md` for mobile ↔ desktop sync
- **Analysis:** See `docs/ANALYSIS.md` for ML model integration

---

**Document Version:** 1.0  
**Last Updated:** 2026-03-09  
**Status:** Complete
