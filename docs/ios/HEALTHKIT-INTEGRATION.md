# HealthKit Integration for HeXuS iOS

**Version:** 1.0  
**Last Updated:** 2026-03-09  
**Status:** Design Specification (Pre-Implementation)

---

## Table of Contents

1. [Overview](#overview)
2. [Required HealthKit Permissions](#required-healthkit-permissions)
3. [Data Types & Identifiers](#data-types--identifiers)
4. [Background Fetch Strategy](#background-fetch-strategy)
5. [Swift Code Examples](#swift-code-examples)
6. [Data Flow: HealthKit → Swift → uniffi → Rust](#data-flow-healthkit--swift--uniffi--rust)
7. [Info.plist Configuration](#infoplist-configuration)
8. [Implementation Checklist](#implementation-checklist)
9. [Testing & Validation](#testing--validation)
10. [Troubleshooting](#troubleshooting)

---

## Overview

The HeXuS iOS app integrates with **Apple HealthKit** to collect biometric data that may correlate with alter switches in plural systems. This document provides a complete technical specification for implementing HealthKit integration using:

- **Swift** for HealthKit API access (native iOS framework)
- **uniffi** for Swift ↔ Rust FFI bridging
- **Rust core** for biometric processing, storage, and analysis

### Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    iOS Device (iPhone/Apple Watch)            │
│  ┌────────────────────────────────────────────────────────┐  │
│  │                     HealthKit                           │  │
│  │  • Heart Rate (HKQuantityTypeIdentifierHeartRate)      │  │
│  │  • HRV (HKQuantityTypeIdentifierHeartRateVariabilitySDNN) │
│  │  • Sleep Analysis (HKCategoryTypeIdentifierSleepAnalysis) │
│  │  • Steps (HKQuantityTypeIdentifierStepCount)           │  │
│  └────────────────┬───────────────────────────────────────┘  │
│                   │                                           │
│                   ▼                                           │
│  ┌────────────────────────────────────────────────────────┐  │
│  │           HealthKitManager.swift                        │  │
│  │  • HKObserverQuery (background updates)                │  │
│  │  • HKAnchoredObjectQuery (incremental sync)            │  │
│  │  • HKSampleQuery (historical data)                     │  │
│  └────────────────┬───────────────────────────────────────┘  │
│                   │                                           │
│                   ▼                                           │
│  ┌────────────────────────────────────────────────────────┐  │
│  │           RustBridge.swift (uniffi-generated)           │  │
│  │  • Convert HKQuantitySample → HealthSample struct      │  │
│  │  • Call Rust functions via FFI                         │  │
│  └────────────────┬───────────────────────────────────────┘  │
│                   │                                           │
│                   ▼                                           │
│  ┌────────────────────────────────────────────────────────┐  │
│  │              Rust Core (lib.rs)                         │  │
│  │  • process_heart_rate_samples()                        │  │
│  │  • calculate_hrv_metrics()                             │  │
│  │  • store_biometric_data()                              │  │
│  │  • detect_anomalies()                                  │  │
│  └────────────────┬───────────────────────────────────────┘  │
│                   │                                           │
│                   ▼                                           │
│  ┌────────────────────────────────────────────────────────┐  │
│  │           SQLite Database (encrypted)                   │  │
│  │  • biometric_samples table                             │  │
│  │  • processed_metrics table                             │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

---

## Required HealthKit Permissions

### Read-Only Permissions

HeXuS **only reads** health data—it does not write any data back to HealthKit. This minimizes privacy concerns and simplifies authorization.

**Required `HKObjectType` permissions:**

```swift
import HealthKit

let typesToRead: Set<HKObjectType> = [
    // Vitals
    HKObjectType.quantityType(forIdentifier: .heartRate)!,
    HKObjectType.quantityType(forIdentifier: .heartRateVariabilitySDNN)!,
    HKObjectType.quantityType(forIdentifier: .oxygenSaturation)!,
    HKObjectType.quantityType(forIdentifier: .bodyTemperature)!,
    HKObjectType.quantityType(forIdentifier: .respiratoryRate)!,
    
    // Activity
    HKObjectType.quantityType(forIdentifier: .stepCount)!,
    HKObjectType.quantityType(forIdentifier: .distanceWalkingRunning)!,
    HKObjectType.quantityType(forIdentifier: .activeEnergyBurned)!,
    HKObjectType.quantityType(forIdentifier: .appleExerciseTime)!,
    
    // Sleep
    HKObjectType.categoryType(forIdentifier: .sleepAnalysis)!,
    
    // Body Measurements (optional)
    HKObjectType.quantityType(forIdentifier: .bodyMass)!,
    HKObjectType.quantityType(forIdentifier: .height)!,
]
```

**MVP (Minimum Viable Product) permissions:**
- Heart Rate (`.heartRate`)
- Heart Rate Variability (`.heartRateVariabilitySDNN`)
- Sleep Analysis (`.sleepAnalysis`)
- Step Count (`.stepCount`)

### Permission Request Flow

1. **Check availability:**
   ```swift
   guard HKHealthStore.isHealthDataAvailable() else {
       print("HealthKit not available on this device")
       return
   }
   ```

2. **Request authorization:**
   ```swift
   let healthStore = HKHealthStore()
   healthStore.requestAuthorization(toShare: nil, read: typesToRead) { success, error in
       if success {
           print("HealthKit authorization granted")
       } else if let error = error {
           print("HealthKit authorization error: \(error.localizedDescription)")
       }
   }
   ```

3. **Handle granular consent:**
   - Users may deny specific data types
   - Always check authorization status before queries:
     ```swift
     let heartRateType = HKQuantityType(.heartRate())!
     let status = healthStore.authorizationStatus(for: heartRateType)
     switch status {
     case .notDetermined:
         // Request permission
     case .sharingDenied:
         // User denied access
     case .sharingAuthorized:
         // Proceed with query
     @unknown default:
         break
     }
     ```

---

## Data Types & Identifiers

### Heart Rate

**Identifier:** `HKQuantityTypeIdentifierHeartRate`  
**Unit:** Beats per minute (bpm) — `HKUnit.count().unitDivided(by: .minute())`  
**Frequency:** Continuous (Apple Watch: ~5-10 second intervals during activity, ~5 minutes at rest)

**Sample structure:**
```swift
// HKQuantitySample
sample.quantity.doubleValue(for: HKUnit(from: "count/min")) // e.g., 72.5
sample.startDate                                              // 2026-03-09T14:32:45Z
sample.endDate                                                // 2026-03-09T14:32:50Z
sample.device?.name                                           // "Apple Watch"
sample.sourceRevision.source.name                             // "Apple Watch" or "HeartWatch"
```

**Use case:** Baseline fronter heart rate, detect sudden changes during switches.

---

### Heart Rate Variability (HRV)

**Identifier:** `HKQuantityTypeIdentifierHeartRateVariabilitySDNN`  
**Unit:** Milliseconds (ms) — `HKUnit.secondUnit(with: .milli)`  
**Frequency:** Periodic (Apple Watch: ~1-2 readings per day during rest/sleep)

**Metric definition:** SDNN (Standard Deviation of NN intervals) — measures beat-to-beat variability in heart rhythm. Higher values typically indicate better autonomic nervous system regulation.

**Sample structure:**
```swift
sample.quantity.doubleValue(for: HKUnit.secondUnit(with: .milli)) // e.g., 45.2 ms
sample.startDate                                                    // 2026-03-09T03:15:00Z
sample.endDate                                                      // 2026-03-09T03:20:00Z
```

**Use case:** Primary biomarker for alter switches. Changes in HRV may correlate with fronting changes.

---

### Sleep Analysis

**Identifier:** `HKCategoryTypeIdentifierSleepAnalysis`  
**Type:** `HKCategoryValue` (enum, not quantity)  
**Values:**
- `.inBed` — User is in bed but not necessarily asleep
- `.asleepUnspecified` — Asleep (no stage data)
- `.awake` — Awake during sleep session
- `.asleepCore` — Core sleep (light + deep combined)
- `.asleepDeep` — Deep sleep (SWS)
- `.asleepREM` — REM sleep

**Sample structure:**
```swift
// HKCategorySample
let sleepValue = sample.value // HKCategoryValueSleepAnalysis enum
let duration = sample.endDate.timeIntervalSince(sample.startDate) // seconds
sample.startDate // Sleep stage start time
sample.endDate   // Sleep stage end time
```

**Use case:** Correlate sleep quality (deep/REM stages) with switch frequency. Poor sleep may increase dissociative episodes.

---

### Step Count

**Identifier:** `HKQuantityTypeIdentifierStepCount`  
**Unit:** Count — `HKUnit.count()`  
**Frequency:** Continuous (iPhone/Apple Watch)

**Sample structure:**
```swift
sample.quantity.doubleValue(for: HKUnit.count()) // e.g., 1250 steps
sample.startDate                                  // 2026-03-09T08:00:00Z
sample.endDate                                    // 2026-03-09T09:00:00Z
```

**Use case:** Activity level as a secondary indicator. Some systems report energy level changes during switches.

---

### Additional Data Types (Future Expansion)

| Identifier | Unit | Use Case |
|------------|------|----------|
| `.oxygenSaturation` | `%` | Detect stress/dissociation (SpO2 drops) |
| `.bodyTemperature` | `degC` / `degF` | Some systems report temperature changes |
| `.respiratoryRate` | `count/min` | Breathing rate correlates with anxiety |
| `.activeEnergyBurned` | `kcal` | Energy expenditure patterns |
| `.appleExerciseTime` | `min` | Distinguish activity from resting state |

---

## Background Fetch Strategy

### Challenge

iOS apps have limited background execution time. HealthKit queries must be efficient to:
1. Avoid battery drain
2. Respect iOS background task limits (~30 seconds)
3. Handle app termination gracefully

### Solution: Three-Tier Query Strategy

#### 1. **Observer Queries** (Real-Time Notifications)

**Purpose:** Get notified when new data arrives, wake app in background.

**Implementation:**
```swift
func setupObserverQueries() {
    let heartRateType = HKObjectType.quantityType(forIdentifier: .heartRate)!
    
    let query = HKObserverQuery(sampleType: heartRateType, predicate: nil) { 
        query, completionHandler, error in
        
        if let error = error {
            print("Observer query error: \(error)")
            completionHandler()
            return
        }
        
        // New heart rate data detected
        print("New heart rate data available")
        
        // Trigger anchored query to fetch new samples
        self.fetchNewHeartRateSamples()
        
        // CRITICAL: Call completion handler to avoid timeout
        completionHandler()
    }
    
    healthStore.execute(query)
    
    // Enable background delivery (requires entitlement)
    healthStore.enableBackgroundDelivery(for: heartRateType, frequency: .immediate) { success, error in
        if success {
            print("Background delivery enabled for heart rate")
        }
    }
}
```

**Key requirements:**
- App must have `com.apple.developer.healthkit.background-delivery` entitlement
- Call `completionHandler()` within 30 seconds or iOS kills the task
- Register observer queries in `AppDelegate.application(_:didFinishLaunchingWithOptions:)`

---

#### 2. **Anchored Object Queries** (Incremental Sync)

**Purpose:** Fetch only new/changed samples since last query (efficient, no duplicates).

**How it works:**
1. First query: `anchor = nil` → returns all historical data
2. Save anchor from query result
3. Subsequent queries: pass saved anchor → returns only new samples since that anchor

**Implementation:**
```swift
class HealthKitManager {
    private var heartRateAnchor: HKQueryAnchor?
    
    func fetchNewHeartRateSamples() {
        let heartRateType = HKQuantityType(.heartRate())!
        
        let query = HKAnchoredObjectQuery(
            type: heartRateType,
            predicate: nil,
            anchor: heartRateAnchor, // nil on first run
            limit: HKObjectQueryNoLimit
        ) { query, newSamples, deletedSamples, newAnchor, error in
            
            guard error == nil else {
                print("Anchored query error: \(error!)")
                return
            }
            
            // Save anchor for next query
            self.heartRateAnchor = newAnchor
            
            // Process new samples
            if let samples = newSamples as? [HKQuantitySample] {
                self.processHeartRateSamples(samples)
            }
            
            // Handle deletions (rare, but possible if user deletes data)
            if let deleted = deletedSamples {
                print("Deleted samples: \(deleted.count)")
            }
        }
        
        query.updateHandler = { query, newSamples, deletedSamples, newAnchor, error in
            // Called automatically when new data arrives (long-running query)
            self.heartRateAnchor = newAnchor
            if let samples = newSamples as? [HKQuantitySample] {
                self.processHeartRateSamples(samples)
            }
        }
        
        healthStore.execute(query)
    }
}
```

**Persistence strategy:**
```swift
// Save anchor to UserDefaults or Keychain
func saveAnchor(_ anchor: HKQueryAnchor?, forKey key: String) {
    guard let anchor = anchor else { return }
    if let data = try? NSKeyedArchiver.archivedData(withRootObject: anchor, requiringSecureCoding: true) {
        UserDefaults.standard.set(data, forKey: key)
    }
}

func loadAnchor(forKey key: String) -> HKQueryAnchor? {
    guard let data = UserDefaults.standard.data(forKey: key) else { return nil }
    return try? NSKeyedUnarchiver.unarchivedObject(ofClass: HKQueryAnchor.self, from: data)
}
```

---

#### 3. **Sample Queries** (Historical Backfill)

**Purpose:** Fetch historical data when app first installs or user changes date range.

**Implementation:**
```swift
func fetchHistoricalHeartRate(startDate: Date, endDate: Date, completion: @escaping ([HKQuantitySample]) -> Void) {
    let heartRateType = HKQuantityType(.heartRate())!
    let predicate = HKQuery.predicateForSamples(withStart: startDate, end: endDate, options: .strictStartDate)
    let sortDescriptor = NSSortDescriptor(key: HKSampleSortIdentifierStartDate, ascending: true)
    
    let query = HKSampleQuery(
        sampleType: heartRateType,
        predicate: predicate,
        limit: HKObjectQueryNoLimit,
        sortDescriptors: [sortDescriptor]
    ) { query, samples, error in
        
        guard let samples = samples as? [HKQuantitySample], error == nil else {
            print("Sample query error: \(error!)")
            completion([])
            return
        }
        
        completion(samples)
    }
    
    healthStore.execute(query)
}
```

**Use case:**
- First launch: backfill last 30 days of data
- User requests export: fetch all historical data
- Visualization: load specific date range for charts

---

### Background Task Workflow

```
┌──────────────────────────────────────────────────────────┐
│  1. New HealthKit Data Arrives (e.g., Apple Watch sync)  │
└────────────────────┬─────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────┐
│  2. HKObserverQuery fires → wakes app in background      │
└────────────────────┬─────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────┐
│  3. App executes HKAnchoredObjectQuery with saved anchor │
└────────────────────┬─────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────┐
│  4. Receive only new samples (since last anchor)         │
└────────────────────┬─────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────┐
│  5. Convert HKQuantitySample → Rust HealthSample struct │
└────────────────────┬─────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────┐
│  6. Call Rust via uniffi: process_heart_rate_samples()  │
└────────────────────┬─────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────┐
│  7. Rust processes data, stores to SQLite, detects      │
│     anomalies, updates baselines                         │
└────────────────────┬─────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────┐
│  8. Update UI (if app in foreground) or queue            │
│     notification (if background)                         │
└────────────────────┬─────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────┐
│  9. Call observer query completionHandler() (critical!)  │
└────────────────────┬─────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────┐
│ 10. App suspends until next observer query notification  │
└──────────────────────────────────────────────────────────┘
```

---

## Swift Code Examples

### Complete HealthKitManager Implementation

See `mobile/ios/HeXuS/HealthKitManager.swift` for full code. Key excerpts:

#### Setup & Authorization

```swift
import HealthKit

class HealthKitManager: ObservableObject {
    private let healthStore = HKHealthStore()
    
    // Anchor storage for incremental queries
    private var heartRateAnchor: HKQueryAnchor?
    private var hrvAnchor: HKQueryAnchor?
    private var sleepAnchor: HKQueryAnchor?
    private var stepsAnchor: HKQueryAnchor?
    
    // Published state for UI
    @Published var authorizationStatus: String = "Not Requested"
    @Published var latestHeartRate: Double?
    @Published var latestHRV: Double?
    
    init() {
        loadAnchors()
    }
    
    /// Request HealthKit authorization
    func requestAuthorization(completion: @escaping (Bool, Error?) -> Void) {
        guard HKHealthStore.isHealthDataAvailable() else {
            completion(false, NSError(domain: "HeXuS", code: 1, userInfo: [
                NSLocalizedDescriptionKey: "HealthKit not available on this device"
            ]))
            return
        }
        
        let typesToRead: Set<HKObjectType> = [
            HKObjectType.quantityType(forIdentifier: .heartRate)!,
            HKObjectType.quantityType(forIdentifier: .heartRateVariabilitySDNN)!,
            HKObjectType.categoryType(forIdentifier: .sleepAnalysis)!,
            HKObjectType.quantityType(forIdentifier: .stepCount)!,
        ]
        
        healthStore.requestAuthorization(toShare: nil, read: typesToRead) { success, error in
            DispatchQueue.main.async {
                self.authorizationStatus = success ? "Authorized" : "Denied"
                completion(success, error)
            }
        }
    }
}
```

---

#### Observer Query Setup

```swift
extension HealthKitManager {
    func setupBackgroundObservers() {
        setupObserver(for: .heartRate, frequency: .immediate)
        setupObserver(for: .heartRateVariabilitySDNN, frequency: .hourly)
        setupObserver(for: .stepCount, frequency: .hourly)
        setupObserver(for: .sleepAnalysis, frequency: .daily)
    }
    
    private func setupObserver(for identifier: HKQuantityTypeIdentifier, frequency: HKUpdateFrequency) {
        guard let sampleType = HKObjectType.quantityType(forIdentifier: identifier) else { return }
        
        let query = HKObserverQuery(sampleType: sampleType, predicate: nil) { query, completionHandler, error in
            if let error = error {
                print("Observer query error for \(identifier): \(error)")
                completionHandler()
                return
            }
            
            print("New data detected for \(identifier)")
            
            // Trigger incremental sync
            switch identifier {
            case .heartRate:
                self.fetchNewHeartRate()
            case .heartRateVariabilitySDNN:
                self.fetchNewHRV()
            case .stepCount:
                self.fetchNewSteps()
            default:
                break
            }
            
            completionHandler()
        }
        
        healthStore.execute(query)
        
        // Enable background delivery
        healthStore.enableBackgroundDelivery(for: sampleType, frequency: frequency) { success, error in
            if success {
                print("Background delivery enabled for \(identifier)")
            } else if let error = error {
                print("Background delivery error: \(error)")
            }
        }
    }
    
    private func setupObserver(for identifier: HKCategoryTypeIdentifier, frequency: HKUpdateFrequency) {
        guard let sampleType = HKObjectType.categoryType(forIdentifier: identifier) else { return }
        
        let query = HKObserverQuery(sampleType: sampleType, predicate: nil) { query, completionHandler, error in
            if let error = error {
                print("Observer query error for \(identifier): \(error)")
                completionHandler()
                return
            }
            
            print("New data detected for \(identifier)")
            
            if identifier == .sleepAnalysis {
                self.fetchNewSleep()
            }
            
            completionHandler()
        }
        
        healthStore.execute(query)
        healthStore.enableBackgroundDelivery(for: sampleType, frequency: frequency) { success, _ in
            if success {
                print("Background delivery enabled for \(identifier)")
            }
        }
    }
}
```

---

#### Anchored Queries

```swift
extension HealthKitManager {
    func fetchNewHeartRate() {
        let heartRateType = HKQuantityType(.heartRate())!
        
        let query = HKAnchoredObjectQuery(
            type: heartRateType,
            predicate: nil,
            anchor: heartRateAnchor,
            limit: HKObjectQueryNoLimit
        ) { query, newSamples, deletedSamples, newAnchor, error in
            
            guard error == nil, let samples = newSamples as? [HKQuantitySample] else {
                print("Heart rate query error: \(error?.localizedDescription ?? "unknown")")
                return
            }
            
            // Update anchor
            self.heartRateAnchor = newAnchor
            self.saveAnchor(newAnchor, forKey: "heartRateAnchor")
            
            // Convert to Rust-compatible struct
            let rustSamples = samples.map { sample in
                HealthSample(
                    sampleType: "HKQuantityTypeIdentifierHeartRate",
                    value: sample.quantity.doubleValue(for: HKUnit(from: "count/min")),
                    startDate: ISO8601DateFormatter().string(from: sample.startDate),
                    endDate: ISO8601DateFormatter().string(from: sample.endDate),
                    source: sample.sourceRevision.source.name,
                    device: sample.device?.name
                )
            }
            
            // Send to Rust for processing
            do {
                let result = try processHeartRateSamples(samples: rustSamples)
                print("Processed \(samples.count) heart rate samples: \(result)")
                
                // Update UI
                if let latest = samples.last {
                    DispatchQueue.main.async {
                        self.latestHeartRate = latest.quantity.doubleValue(for: HKUnit(from: "count/min"))
                    }
                }
            } catch {
                print("Rust processing error: \(error)")
            }
        }
        
        healthStore.execute(query)
    }
}
```

---

## Data Flow: HealthKit → Swift → uniffi → Rust

### Step-by-Step Flow

#### 1. Swift Layer (HealthKitManager.swift)

**Input:** `HKQuantitySample` from HealthKit  
**Output:** `HealthSample` struct (Swift, uniffi-compatible)

```swift
// HKQuantitySample properties
sample.quantity.doubleValue(for: unit) // → value: Double
sample.startDate                        // → startDate: Date → ISO8601 String
sample.endDate                          // → endDate: Date → ISO8601 String
sample.sourceRevision.source.name       // → source: String (e.g., "Apple Watch")
sample.device?.name                     // → device: String? (e.g., "Apple Watch Series 9")
```

**Conversion:**
```swift
let rustSample = HealthSample(
    sampleType: "HKQuantityTypeIdentifierHeartRate",
    value: sample.quantity.doubleValue(for: HKUnit(from: "count/min")),
    startDate: ISO8601DateFormatter().string(from: sample.startDate),
    endDate: ISO8601DateFormatter().string(from: sample.endDate),
    source: sample.sourceRevision.source.name,
    device: sample.device?.name
)
```

---

#### 2. uniffi Bridge Layer (RustBridge.swift - auto-generated)

**Purpose:** Type-safe FFI between Swift and Rust.

**Generated from `mobile/rust-core/src/lib.rs`:**

```rust
// Rust side
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
    // Process samples
    let total = samples.len();
    let avg_hr = samples.iter().map(|s| s.value).sum::<f64>() / total as f64;
    
    // Store to database
    for sample in samples {
        storage::insert_biometric_sample(&sample)?;
    }
    
    // Detect anomalies
    let anomalies = analysis::detect_heart_rate_anomalies(&samples);
    
    Ok(format!("Processed {} samples, avg HR: {:.1} bpm, {} anomalies", total, avg_hr, anomalies.len()))
}
```

**Swift call (auto-generated bindings):**
```swift
import HeXuSCore  // uniffi-generated module

let result = try processHeartRateSamples(samples: rustSamples)
print(result) // "Processed 150 samples, avg HR: 72.3 bpm, 2 anomalies"
```

**Type conversions (automatic):**
- Swift `[HealthSample]` → Rust `Vec<HealthSample>`
- Swift `String` → Rust `String`
- Swift `Double` → Rust `f64`
- Swift `String?` → Rust `Option<String>`
- Swift `throws` → Rust `Result<T, E>`

---

#### 3. Rust Core Layer (lib.rs, processing modules)

**Modules:**

```
mobile/rust-core/
├── lib.rs               # uniffi exports
├── storage.rs           # SQLite database operations
├── analysis.rs          # HRV calculation, anomaly detection
├── baselines.rs         # Per-alter baseline management
└── sync.rs              # Desktop server sync
```

**Example: HRV Calculation**

```rust
// analysis.rs
use cardio_rs::hrv::{self, TimedomainMetrics};

pub fn calculate_hrv_from_samples(samples: &[HealthSample]) -> Result<HrvMetrics, String> {
    // Extract RR intervals (time between heartbeats)
    let rr_intervals: Vec<f64> = samples
        .windows(2)
        .map(|pair| {
            let t1 = parse_iso8601(&pair[0].start_date)?;
            let t2 = parse_iso8601(&pair[1].start_date)?;
            Ok((t2 - t1) * 1000.0) // Convert to milliseconds
        })
        .collect::<Result<Vec<_>, _>>()?;
    
    // Calculate HRV metrics using cardio-rs
    let metrics = hrv::timedomain(&rr_intervals);
    
    Ok(HrvMetrics {
        sdnn: metrics.sdnn,
        rmssd: metrics.rmssd,
        pnn50: metrics.pnn50,
        mean_rr: metrics.mean_rr,
        timestamp: samples.last().unwrap().start_date.clone(),
    })
}
```

**Example: Anomaly Detection**

```rust
// analysis.rs
use augurs::outlier::{OutlierDetector, MADDetector};

pub fn detect_heart_rate_anomalies(samples: &[HealthSample]) -> Vec<AnomalyEvent> {
    let values: Vec<f64> = samples.iter().map(|s| s.value).collect();
    
    // Use MAD (Median Absolute Deviation) outlier detector
    let detector = MADDetector::new(2.5); // 2.5 MADs from median
    let outliers = detector.detect(&values);
    
    outliers
        .into_iter()
        .enumerate()
        .filter_map(|(i, is_outlier)| {
            if is_outlier {
                Some(AnomalyEvent {
                    metric_type: "heart_rate".to_string(),
                    value: samples[i].value,
                    timestamp: samples[i].start_date.clone(),
                    severity: calculate_severity(samples[i].value),
                })
            } else {
                None
            }
        })
        .collect()
}
```

---

#### 4. Storage Layer (SQLite)

```rust
// storage.rs
use sqlx::{SqlitePool, Row};

pub async fn insert_biometric_sample(sample: &HealthSample) -> Result<(), String> {
    let pool = get_db_pool()?;
    
    sqlx::query(
        r#"
        INSERT INTO biometric_samples (timestamp, source, metric_type, value, unit, metadata)
        VALUES (?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&sample.start_date)
    .bind(&sample.source)
    .bind(&sample.sample_type)
    .bind(sample.value)
    .bind("bpm") // Unit derived from sample_type
    .bind(serde_json::json!({
        "device": sample.device,
        "end_date": sample.end_date,
    }).to_string())
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(())
}
```

---

### Complete Data Flow Diagram

```
[HealthKit Database]
        │
        │ HKObserverQuery triggers update
        ▼
[HealthKitManager.swift]
        │
        │ HKAnchoredObjectQuery fetches new samples
        ▼
[HKQuantitySample]
  ├─ quantity.doubleValue(for: unit) → value
  ├─ startDate → ISO8601 string
  ├─ endDate → ISO8601 string
  ├─ sourceRevision.source.name → source
  └─ device?.name → device
        │
        ▼
[HealthSample struct] (Swift)
        │
        │ uniffi FFI call
        ▼
[process_heart_rate_samples()] (Rust)
        │
        ├─→ [storage::insert_biometric_sample()] → SQLite
        ├─→ [analysis::calculate_hrv()]          → cardio-rs
        ├─→ [analysis::detect_anomalies()]       → augurs
        └─→ [baselines::update_baseline()]       → Per-alter stats
        │
        ▼
[Return Result<String>] ← Back to Swift
        │
        ▼
[Update UI / Queue Notification]
```

---

## Info.plist Configuration

### Required Keys

Add these keys to `mobile/ios/HeXuS/Info.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- HealthKit Capability -->
    <key>UIRequiredDeviceCapabilities</key>
    <array>
        <string>healthkit</string>
    </array>
    
    <!-- Privacy Usage Descriptions (REQUIRED) -->
    <key>NSHealthShareUsageDescription</key>
    <string>HeXuS reads your heart rate, heart rate variability, sleep data, and activity metrics to detect patterns that may correlate with fronter switches in plural systems. This data is processed locally on your device and never shared with third parties.</string>
    
    <key>NSHealthUpdateUsageDescription</key>
    <string>HeXuS does not write any data to HealthKit. This permission is not used.</string>
    
    <!-- Background Modes -->
    <key>UIBackgroundModes</key>
    <array>
        <string>processing</string>
        <string>fetch</string>
    </array>
    
    <!-- App Name -->
    <key>CFBundleDisplayName</key>
    <string>HeXuS</string>
    
    <!-- Bundle Identifier -->
    <key>CFBundleIdentifier</key>
    <string>com.hexus.app</string>
    
    <!-- App Version -->
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
</dict>
</plist>
```

### Xcode Configuration

1. **Enable HealthKit Capability:**
   - Open Xcode project
   - Select target → Signing & Capabilities
   - Click "+ Capability"
   - Add "HealthKit"
   - Enable "Background Delivery" checkbox

2. **Add Background Modes:**
   - In Signing & Capabilities → Background Modes
   - Enable:
     - ✅ Background fetch
     - ✅ Background processing

3. **Privacy Manifest (iOS 17+):**
   - Create `PrivacyInfo.xcprivacy` in project root:
   ```xml
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
   <dict>
       <key>NSPrivacyCollectedDataTypes</key>
       <array>
           <dict>
               <key>NSPrivacyCollectedDataType</key>
               <string>NSPrivacyCollectedDataTypeHealthAndFitness</string>
               <key>NSPrivacyCollectedDataTypeLinked</key>
               <false/>
               <key>NSPrivacyCollectedDataTypeTracking</key>
               <false/>
               <key>NSPrivacyCollectedDataTypePurposes</key>
               <array>
                   <string>NSPrivacyCollectedDataTypePurposeAnalytics</string>
               </array>
           </dict>
       </array>
       <key>NSPrivacyAccessedAPITypes</key>
       <array>
           <dict>
               <key>NSPrivacyAccessedAPIType</key>
               <string>NSPrivacyAccessedAPICategoryFileTimestamp</string>
               <key>NSPrivacyAccessedAPITypeReasons</key>
               <array>
                   <string>C617.1</string>
               </array>
           </dict>
       </array>
   </dict>
   </plist>
   ```

---

## Implementation Checklist

### Phase 1: Basic HealthKit Integration (MVP)

- [ ] Create `HealthKitManager.swift` class
- [ ] Request authorization for heart rate, HRV, sleep, steps
- [ ] Implement `HKSampleQuery` for historical backfill (last 30 days)
- [ ] Display latest heart rate in UI
- [ ] Add Info.plist privacy strings
- [ ] Enable HealthKit capability in Xcode
- [ ] Test on physical device (HealthKit unavailable in Simulator)

### Phase 2: Background Sync

- [ ] Implement `HKObserverQuery` for real-time notifications
- [ ] Implement `HKAnchoredObjectQuery` for incremental sync
- [ ] Persist query anchors to UserDefaults
- [ ] Enable background delivery entitlement
- [ ] Test background delivery (leave app in background, generate new data from Health app)

### Phase 3: Rust FFI Integration

- [ ] Set up Rust crate with uniffi in `mobile/rust-core/`
- [ ] Define `HealthSample` struct with `#[derive(uniffi::Record)]`
- [ ] Implement `process_heart_rate_samples()` in Rust
- [ ] Generate Swift bindings: `cargo run --bin uniffi-bindgen generate --language swift`
- [ ] Build Rust library for iOS targets: `cargo build --target aarch64-apple-ios --release`
- [ ] Create XCFramework from compiled `.a` files
- [ ] Import XCFramework into Xcode project
- [ ] Call Rust functions from Swift

### Phase 4: Data Processing

- [ ] Implement SQLite storage in Rust
- [ ] Integrate `cardio-rs` for HRV calculation
- [ ] Integrate `augurs` for anomaly detection
- [ ] Create per-alter baseline tables
- [ ] Implement anomaly alerting (local notifications)

### Phase 5: Sleep & Activity Integration

- [ ] Implement sleep analysis query (`HKCategoryTypeIdentifier.sleepAnalysis`)
- [ ] Parse sleep stages (deep/REM/light/awake)
- [ ] Implement step count query
- [ ] Correlate sleep quality with switch frequency
- [ ] Add activity level to baseline calculations

### Phase 6: Optimization & Polish

- [ ] Batch process samples (reduce FFI calls)
- [ ] Implement rate limiting (avoid excessive queries)
- [ ] Add error handling and retry logic
- [ ] Monitor battery usage (Instruments → Energy Log)
- [ ] Test on multiple iOS versions (iOS 16+)
- [ ] Test with Apple Watch vs iPhone-only data
- [ ] Write unit tests for Rust processing logic

---

## Testing & Validation

### Manual Testing

**Device Requirements:**
- Physical iPhone (iOS 16+) — Simulator does not support HealthKit
- Apple Watch (optional, but recommended for continuous heart rate)

**Test Cases:**

1. **Authorization Flow:**
   - Fresh install → request permissions
   - Grant all permissions → verify queries run
   - Deny specific permission → verify graceful degradation
   - Revoke permission in Settings → verify app handles re-auth

2. **Background Sync:**
   - Install app, grant permissions
   - Close app completely (swipe up in App Switcher)
   - Generate new health data:
     - Go for a walk (step count)
     - Use Breathe app (heart rate/HRV)
     - Log sleep in Health app
   - Check app logs (Xcode Console) for observer query triggers

3. **Data Accuracy:**
   - Compare HeXuS-displayed heart rate with Health app
   - Verify timestamps match
   - Check for duplicate samples (should be zero with anchored queries)

4. **Edge Cases:**
   - App killed by iOS (force quit)
   - Low battery mode enabled
   - Network offline (Rust processing should work locally)
   - HealthKit database corrupted (rare, but handle gracefully)

---

### Automated Testing

**Swift Unit Tests:**
```swift
// HeXuSTests/HealthKitManagerTests.swift
import XCTest
@testable import HeXuS

class HealthKitManagerTests: XCTestCase {
    func testConvertHKSampleToRustStruct() {
        // Create mock HKQuantitySample
        let heartRateType = HKQuantityType(.heartRate())!
        let quantity = HKQuantity(unit: HKUnit(from: "count/min"), doubleValue: 72.5)
        let now = Date()
        let sample = HKQuantitySample(
            type: heartRateType,
            quantity: quantity,
            start: now,
            end: now
        )
        
        // Convert
        let rustSample = HealthKitManager.convertToHealthSample(sample)
        
        // Assert
        XCTAssertEqual(rustSample.sampleType, "HKQuantityTypeIdentifierHeartRate")
        XCTAssertEqual(rustSample.value, 72.5, accuracy: 0.01)
        XCTAssertNotNil(rustSample.startDate)
    }
}
```

**Rust Integration Tests:**
```rust
// mobile/rust-core/tests/integration_test.rs
#[test]
fn test_process_heart_rate_samples() {
    let samples = vec![
        HealthSample {
            sample_type: "HKQuantityTypeIdentifierHeartRate".to_string(),
            value: 72.0,
            start_date: "2026-03-09T14:00:00Z".to_string(),
            end_date: "2026-03-09T14:00:05Z".to_string(),
            source: "Apple Watch".to_string(),
            device: Some("Apple Watch Series 9".to_string()),
        },
        // ... more samples
    ];
    
    let result = process_heart_rate_samples(samples).unwrap();
    assert!(result.contains("Processed 1 samples"));
}
```

---

## Troubleshooting

### Common Issues

#### 1. **"HealthKit not available on this device"**

**Cause:** Running on iOS Simulator  
**Solution:** Test on physical device only

---

#### 2. **Observer query not firing in background**

**Causes:**
- Missing `com.apple.developer.healthkit.background-delivery` entitlement
- Forgot to call `completionHandler()` in query handler
- Background delivery not enabled for sample type

**Solutions:**
```swift
// 1. Enable entitlement in Xcode (Signing & Capabilities → HealthKit → Background Delivery)

// 2. Always call completion handler
let query = HKObserverQuery(...) { query, completionHandler, error in
    // Your code here
    completionHandler() // CRITICAL!
}

// 3. Enable background delivery
healthStore.enableBackgroundDelivery(for: sampleType, frequency: .immediate) { success, error in
    print("Background delivery: \(success)")
}
```

---

#### 3. **Duplicate samples in database**

**Cause:** Not using anchored queries, or not persisting anchors correctly  
**Solution:**
```swift
// Save anchor after every query
self.heartRateAnchor = newAnchor
saveAnchor(newAnchor, forKey: "heartRateAnchor")

// Load anchor on init
self.heartRateAnchor = loadAnchor(forKey: "heartRateAnchor")
```

---

#### 4. **App crashes when requesting authorization**

**Cause:** Missing `NSHealthShareUsageDescription` or `NSHealthUpdateUsageDescription` in Info.plist  
**Solution:** Add required privacy strings (see Info.plist section above)

---

#### 5. **Rust FFI errors**

**Symptoms:** "Symbol not found" or "Library not loaded"  
**Causes:**
- XCFramework not built for correct architecture
- Missing uniffi-generated bindings

**Solutions:**
```bash
# Rebuild for iOS targets
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
cargo build --release --target aarch64-apple-ios         # Device
cargo build --release --target aarch64-apple-ios-sim     # M1 Simulator
cargo build --release --target x86_64-apple-ios          # Intel Simulator

# Create XCFramework with all architectures
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libhexus_core.a \
  -library target/aarch64-apple-ios-sim/release/libhexus_core.a \
  -library target/x86_64-apple-ios/release/libhexus_core.a \
  -output HeXuSCore.xcframework
```

---

#### 6. **HRV data missing or infrequent**

**Cause:** Apple Watch measures HRV sporadically (1-2 times per day during rest)  
**Solution:**
- Use Breathe app to trigger HRV measurement
- Check Health app to verify HRV data exists
- Use heart rate time series to calculate custom HRV (via `cardio-rs`)

---

## Next Steps

After implementing this HealthKit integration:

1. **Android Health Connect integration** — parallel implementation for Android
2. **Desktop sync protocol** — upload biometric data to HeXuS desktop server
3. **Machine learning pipeline** — train models on biometric + switch data
4. **Visualization** — build iOS UI with charts (Chart.js or SwiftUI Charts)
5. **Notifications** — alert user when anomalies detected

---

## References

- [Apple HealthKit Documentation](https://developer.apple.com/documentation/healthkit)
- [HKObserverQuery](https://developer.apple.com/documentation/healthkit/hkobserverquery)
- [HKAnchoredObjectQuery](https://developer.apple.com/documentation/healthkit/hkanchoredobjectquery)
- [uniffi User Guide](https://mozilla.github.io/uniffi-rs/)
- [cardio-rs Crate](https://crates.io/crates/cardio-rs)
- [augurs Crate](https://crates.io/crates/augurs)

---

**Document Status:** Complete design specification  
**Next Action:** Implement `HealthKitManager.swift` stub file  
**Assigned To:** Swift developer  
**Estimated Effort:** 2-3 days (Phase 1-3), 1 week (full integration)
