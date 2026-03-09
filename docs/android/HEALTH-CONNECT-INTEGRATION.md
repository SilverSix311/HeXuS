# Health Connect Integration for HeXuS (Android)

**Document Version:** 1.0  
**Last Updated:** 2026-03-09  
**Target Android:** API 34+ (Android 14+)  
**Health Connect SDK:** androidx.health.connect.client 1.1.0-alpha10+

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Required Permissions](#required-permissions)
4. [Manifest Configuration](#manifest-configuration)
5. [Health Connect Data Types](#health-connect-data-types)
6. [Background Sync Strategy](#background-sync-strategy)
7. [Reading Health Data](#reading-health-data)
8. [Data Flow Architecture](#data-flow-architecture)
9. [Code Examples](#code-examples)
10. [Error Handling](#error-handling)
11. [Testing](#testing)
12. [Google Fit Fallback](#google-fit-fallback)

---

## Overview

Health Connect is Android's unified health and fitness data platform (Android 14+). It provides a **device-agnostic**, **privacy-first** API for reading and writing health data from multiple sources (wearables, apps, manual entry).

**HeXuS uses Health Connect to:**
- Read **heart rate** samples (continuous monitoring)
- Read **HRV (Heart Rate Variability)** metrics (RMSSD/SDNN)
- Read **sleep sessions** with stage breakdowns (light/deep/REM)
- Read **steps** and **active calories** (activity patterns)
- Sync data to HeXuS Rust core for biometric analysis and switch detection

**Key Principles:**
- **Foreground-only access** — reads happen in active app or WorkManager background tasks
- **User-granted permissions** — granular per-data-type consent via Health Connect app
- **Privacy-first** — data never leaves device unless user syncs to HeXuS server
- **Battery-efficient** — periodic sync (1-4 hour intervals) with WorkManager constraints

---

## Prerequisites

### Minimum Requirements
- **Android API 34+** (Android 14 Upside Down Cake)
- **Health Connect app installed** (pre-installed on Pixel, downloadable on other devices)
- **HeXuS mobile app** (Tauri 2.0 with Kotlin native layer)

### Dependencies (build.gradle.kts)

```kotlin
dependencies {
    // Health Connect SDK
    implementation("androidx.health.connect:connect-client:1.1.0-alpha10")
    
    // WorkManager for background sync
    implementation("androidx.work:work-runtime-ktx:2.9.0")
    
    // Coroutines for async operations
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.7.3")
    
    // Lifecycle components
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.6.2")
    implementation("androidx.lifecycle:lifecycle-viewmodel-ktx:2.6.2")
    
    // Optional: Room for local caching (if not using Rust SQLite)
    implementation("androidx.room:room-runtime:2.6.1")
    kapt("androidx.room:room-compiler:2.6.1")
    implementation("androidx.room:room-ktx:2.6.1")
}
```

### Health Connect Availability Check

Not all devices support Health Connect. Check at runtime:

```kotlin
suspend fun isHealthConnectAvailable(context: Context): Boolean {
    val availabilityStatus = HealthConnectClient.getSdkStatus(context)
    return availabilityStatus == HealthConnectClient.SDK_AVAILABLE
}
```

**Fallback:** If unavailable, use Google Fit API (see section 12).

---

## Required Permissions

### Permission Types

Health Connect uses **granular runtime permissions** per data type. Apps must declare permissions in manifest and request at runtime.

**HeXuS Required Permissions:**

| Data Type | Permission (Read) | Permission (Write) |
|-----------|-------------------|-------------------|
| Heart Rate | `android.permission.health.READ_HEART_RATE` | `android.permission.health.WRITE_HEART_RATE` |
| HRV (RMSSD) | `android.permission.health.READ_HEART_RATE_VARIABILITY_RMSSD` | `android.permission.health.WRITE_HEART_RATE_VARIABILITY_RMSSD` |
| Sleep Session | `android.permission.health.READ_SLEEP` | `android.permission.health.WRITE_SLEEP` |
| Steps | `android.permission.health.READ_STEPS` | `android.permission.health.WRITE_STEPS` |
| Active Calories | `android.permission.health.READ_ACTIVE_CALORIES_BURNED` | `android.permission.health.WRITE_ACTIVE_CALORIES_BURNED` |

**Note:** HeXuS only needs **read permissions** (no write — we don't modify Health Connect data).

---

## Manifest Configuration

### AndroidManifest.xml

```xml
<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="com.hexus.mobile">

    <!-- Health Connect permissions (read-only) -->
    <uses-permission android:name="android.permission.health.READ_HEART_RATE" />
    <uses-permission android:name="android.permission.health.READ_HEART_RATE_VARIABILITY_RMSSD" />
    <uses-permission android:name="android.permission.health.READ_SLEEP" />
    <uses-permission android:name="android.permission.health.READ_STEPS" />
    <uses-permission android:name="android.permission.health.READ_ACTIVE_CALORIES_BURNED" />

    <!-- Internet for syncing to desktop server (optional) -->
    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />

    <!-- Query Health Connect package -->
    <queries>
        <package android:name="com.google.android.apps.healthdata" />
    </queries>

    <application
        android:name=".HeXuSApplication"
        android:allowBackup="false"
        android:icon="@mipmap/ic_launcher"
        android:label="@string/app_name"
        android:roundIcon="@mipmap/ic_launcher_round"
        android:supportsRtl="true"
        android:theme="@style/Theme.HeXuS">

        <!-- Main activity -->
        <activity
            android:name=".MainActivity"
            android:exported="true"
            android:launchMode="singleTop">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>

        <!-- Health Connect permission rationale activity (Android 14+) -->
        <activity-alias
            android:name=".ViewPermissionUsageActivity"
            android:exported="true"
            android:targetActivity=".MainActivity"
            android:permission="android.permission.START_VIEW_PERMISSION_USAGE">
            <intent-filter>
                <action android:name="android.intent.action.VIEW_PERMISSION_USAGE" />
                <category android:name="android.health.category.HEALTH_PERMISSIONS" />
            </intent-filter>
        </activity-alias>

        <!-- WorkManager sync worker (background) -->
        <provider
            android:name="androidx.startup.InitializationProvider"
            android:authorities="${applicationId}.androidx-startup"
            android:exported="false"
            tools:node="merge">
            <meta-data
                android:name="androidx.work.WorkManagerInitializer"
                android:value="androidx.startup" />
        </provider>

    </application>
</manifest>
```

### health_permissions.xml (res/values/)

Define permissions as a resource array for runtime requests:

```xml
<resources>
    <string-array name="health_permissions">
        <item>android.permission.health.READ_HEART_RATE</item>
        <item>android.permission.health.READ_HEART_RATE_VARIABILITY_RMSSD</item>
        <item>android.permission.health.READ_SLEEP</item>
        <item>android.permission.health.READ_STEPS</item>
        <item>android.permission.health.READ_ACTIVE_CALORIES_BURNED</item>
    </string-array>
</resources>
```

---

## Health Connect Data Types

### Data Types HeXuS Uses

| Record Type | Kotlin Class | Fields |
|-------------|-------------|--------|
| **Heart Rate** | `HeartRateRecord` | `beatsPerMinute: Long`, `time: Instant`, `zoneOffset: ZoneOffset` |
| **HRV** | `HeartRateVariabilityRmssdRecord` | `heartRateVariabilityMillis: Double`, `time: Instant` |
| **Sleep Session** | `SleepSessionRecord` | `startTime`, `endTime`, `stages: List<Stage>`, `title`, `notes` |
| **Steps** | `StepsRecord` | `count: Long`, `startTime`, `endTime` |
| **Active Calories** | `ActiveCaloriesBurnedRecord` | `energy: Energy`, `startTime`, `endTime` |

### Data Structure Details

#### HeartRateRecord
```kotlin
HeartRateRecord(
    time = Instant.parse("2026-03-09T15:30:00Z"),
    zoneOffset = ZoneOffset.UTC,
    beatsPerMinute = 72,
    metadata = Metadata(
        dataOrigin = DataOrigin(packageName = "com.google.android.apps.fitness"),
        device = Device(manufacturer = "Samsung", model = "Galaxy Watch 6")
    )
)
```

#### HeartRateVariabilityRmssdRecord
```kotlin
HeartRateVariabilityRmssdRecord(
    time = Instant.parse("2026-03-09T15:30:00Z"),
    zoneOffset = ZoneOffset.UTC,
    heartRateVariabilityMillis = 45.2,  // RMSSD in milliseconds
    metadata = Metadata(...)
)
```

#### SleepSessionRecord
```kotlin
SleepSessionRecord(
    startTime = Instant.parse("2026-03-08T23:00:00Z"),
    endTime = Instant.parse("2026-03-09T07:00:00Z"),
    zoneOffset = ZoneOffset.UTC,
    title = "Sleep",
    notes = null,
    stages = listOf(
        SleepSessionRecord.Stage(
            startTime = Instant.parse("2026-03-08T23:00:00Z"),
            endTime = Instant.parse("2026-03-09T00:30:00Z"),
            stage = SleepSessionRecord.STAGE_TYPE_LIGHT
        ),
        SleepSessionRecord.Stage(
            startTime = Instant.parse("2026-03-09T00:30:00Z"),
            endTime = Instant.parse("2026-03-09T02:00:00Z"),
            stage = SleepSessionRecord.STAGE_TYPE_DEEP
        ),
        // ... more stages (STAGE_TYPE_REM, STAGE_TYPE_AWAKE, STAGE_TYPE_UNKNOWN)
    ),
    metadata = Metadata(...)
)
```

**Sleep Stage Constants:**
- `STAGE_TYPE_AWAKE` (1)
- `STAGE_TYPE_SLEEP` (2) — generic sleep
- `STAGE_TYPE_OUT_OF_BED` (3)
- `STAGE_TYPE_LIGHT` (4)
- `STAGE_TYPE_DEEP` (5)
- `STAGE_TYPE_REM` (6)
- `STAGE_TYPE_AWAKE_IN_BED` (7)
- `STAGE_TYPE_UNKNOWN` (0)

#### StepsRecord
```kotlin
StepsRecord(
    count = 1024,
    startTime = Instant.parse("2026-03-09T14:00:00Z"),
    endTime = Instant.parse("2026-03-09T15:00:00Z"),
    zoneOffset = ZoneOffset.UTC,
    metadata = Metadata(...)
)
```

#### ActiveCaloriesBurnedRecord
```kotlin
ActiveCaloriesBurnedRecord(
    energy = Energy.kilocalories(250.0),
    startTime = Instant.parse("2026-03-09T14:00:00Z"),
    endTime = Instant.parse("2026-03-09T15:00:00Z"),
    zoneOffset = ZoneOffset.UTC,
    metadata = Metadata(...)
)
```

---

## Background Sync Strategy

### Why WorkManager?

- **Battery-efficient** — respects Doze and App Standby
- **Constraints-based** — only runs when conditions met (Wi-Fi, charging, etc.)
- **Guaranteed execution** — survives app restarts and device reboots
- **Periodic scheduling** — built-in support for recurring tasks

### Sync Schedule

**HeXuS Background Sync:**
- **Interval:** 1-4 hours (user-configurable in settings)
- **Flex window:** 15 minutes (allows system batching)
- **Constraints:**
  - Network: Unmetered (Wi-Fi only) — optional, user choice
  - Battery: Not low
  - Storage: Not low
- **Retry policy:** Exponential backoff (min 10s, max 5min)

### WorkManager Implementation

#### 1. Worker Class (HealthSyncWorker.kt)

```kotlin
package com.hexus.mobile.sync

import android.content.Context
import androidx.health.connect.client.HealthConnectClient
import androidx.health.connect.client.records.*
import androidx.health.connect.client.request.ReadRecordsRequest
import androidx.health.connect.client.time.TimeRangeFilter
import androidx.work.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.time.Instant
import java.time.temporal.ChronoUnit

class HealthSyncWorker(
    context: Context,
    params: WorkerParameters
) : CoroutineWorker(context, params) {

    override suspend fun doWork(): Result = withContext(Dispatchers.IO) {
        return@withContext try {
            // Get Health Connect client
            val client = HealthConnectClient.getOrCreate(applicationContext)

            // Get last sync timestamp from local storage
            val lastSyncTime = getLastSyncTimestamp()
            val now = Instant.now()

            // Sync each data type
            syncHeartRate(client, lastSyncTime, now)
            syncHRV(client, lastSyncTime, now)
            syncSleep(client, lastSyncTime, now)
            syncSteps(client, lastSyncTime, now)
            syncActiveCalories(client, lastSyncTime, now)

            // Update last sync timestamp
            saveLastSyncTimestamp(now)

            // Send data to Rust core via uniffi
            sendToRustCore()

            Result.success()
        } catch (e: Exception) {
            // Log error
            android.util.Log.e("HealthSyncWorker", "Sync failed", e)
            
            // Retry with backoff
            if (runAttemptCount < 3) {
                Result.retry()
            } else {
                Result.failure()
            }
        }
    }

    private suspend fun syncHeartRate(
        client: HealthConnectClient,
        startTime: Instant,
        endTime: Instant
    ) {
        val request = ReadRecordsRequest(
            recordType = HeartRateRecord::class,
            timeRangeFilter = TimeRangeFilter.between(startTime, endTime)
        )
        val response = client.readRecords(request)
        
        // Store in local database
        response.records.forEach { record ->
            storeHeartRateSample(
                timestamp = record.time.epochSecond,
                bpm = record.beatsPerMinute,
                source = record.metadata.dataOrigin.packageName
            )
        }
    }

    private suspend fun syncHRV(
        client: HealthConnectClient,
        startTime: Instant,
        endTime: Instant
    ) {
        val request = ReadRecordsRequest(
            recordType = HeartRateVariabilityRmssdRecord::class,
            timeRangeFilter = TimeRangeFilter.between(startTime, endTime)
        )
        val response = client.readRecords(request)
        
        response.records.forEach { record ->
            storeHRVSample(
                timestamp = record.time.epochSecond,
                rmssd = record.heartRateVariabilityMillis,
                source = record.metadata.dataOrigin.packageName
            )
        }
    }

    private suspend fun syncSleep(
        client: HealthConnectClient,
        startTime: Instant,
        endTime: Instant
    ) {
        val request = ReadRecordsRequest(
            recordType = SleepSessionRecord::class,
            timeRangeFilter = TimeRangeFilter.between(startTime, endTime)
        )
        val response = client.readRecords(request)
        
        response.records.forEach { session ->
            storeSleepSession(
                startTime = session.startTime.epochSecond,
                endTime = session.endTime.epochSecond,
                stages = session.stages.map { stage ->
                    SleepStage(
                        startTime = stage.startTime.epochSecond,
                        endTime = stage.endTime.epochSecond,
                        stageType = stage.stage
                    )
                },
                source = session.metadata.dataOrigin.packageName
            )
        }
    }

    private suspend fun syncSteps(
        client: HealthConnectClient,
        startTime: Instant,
        endTime: Instant
    ) {
        val request = ReadRecordsRequest(
            recordType = StepsRecord::class,
            timeRangeFilter = TimeRangeFilter.between(startTime, endTime)
        )
        val response = client.readRecords(request)
        
        response.records.forEach { record ->
            storeStepsSample(
                startTime = record.startTime.epochSecond,
                endTime = record.endTime.epochSecond,
                count = record.count,
                source = record.metadata.dataOrigin.packageName
            )
        }
    }

    private suspend fun syncActiveCalories(
        client: HealthConnectClient,
        startTime: Instant,
        endTime: Instant
    ) {
        val request = ReadRecordsRequest(
            recordType = ActiveCaloriesBurnedRecord::class,
            timeRangeFilter = TimeRangeFilter.between(startTime, endTime)
        )
        val response = client.readRecords(request)
        
        response.records.forEach { record ->
            storeCaloriesSample(
                startTime = record.startTime.epochSecond,
                endTime = record.endTime.epochSecond,
                kilocalories = record.energy.inKilocalories,
                source = record.metadata.dataOrigin.packageName
            )
        }
    }

    // Database operations (stub - implement with Room or Rust SQLite)
    private fun storeHeartRateSample(timestamp: Long, bpm: Long, source: String) {
        // TODO: Insert into local SQLite via Rust FFI
    }

    private fun storeHRVSample(timestamp: Long, rmssd: Double, source: String) {
        // TODO: Insert into local SQLite via Rust FFI
    }

    private fun storeSleepSession(
        startTime: Long,
        endTime: Long,
        stages: List<SleepStage>,
        source: String
    ) {
        // TODO: Insert into local SQLite via Rust FFI
    }

    private fun storeStepsSample(
        startTime: Long,
        endTime: Long,
        count: Long,
        source: String
    ) {
        // TODO: Insert into local SQLite via Rust FFI
    }

    private fun storeCaloriesSample(
        startTime: Long,
        endTime: Long,
        kilocalories: Double,
        source: String
    ) {
        // TODO: Insert into local SQLite via Rust FFI
    }

    private fun getLastSyncTimestamp(): Instant {
        // TODO: Read from local storage (SharedPreferences or Rust DB)
        return Instant.now().minus(4, ChronoUnit.HOURS) // Default: 4 hours ago
    }

    private fun saveLastSyncTimestamp(timestamp: Instant) {
        // TODO: Save to local storage
    }

    private fun sendToRustCore() {
        // TODO: Call Rust uniffi function to process biometric data
        // Example: RustBridge.processBiometricBatch(samples)
    }

    data class SleepStage(
        val startTime: Long,
        val endTime: Long,
        val stageType: Int
    )
}
```

#### 2. Scheduling the Worker (HeXuSApplication.kt)

```kotlin
package com.hexus.mobile

import android.app.Application
import androidx.work.*
import com.hexus.mobile.sync.HealthSyncWorker
import java.util.concurrent.TimeUnit

class HeXuSApplication : Application() {

    override fun onCreate() {
        super.onCreate()
        
        // Initialize WorkManager
        initializeWorkManager()
        
        // Schedule periodic health sync
        scheduleHealthSync()
    }

    private fun initializeWorkManager() {
        val config = Configuration.Builder()
            .setMinimumLoggingLevel(android.util.Log.INFO)
            .build()
        WorkManager.initialize(this, config)
    }

    private fun scheduleHealthSync() {
        val constraints = Constraints.Builder()
            .setRequiredNetworkType(NetworkType.UNMETERED) // Wi-Fi only (user configurable)
            .setRequiresBatteryNotLow(true)
            .setRequiresStorageNotLow(true)
            .build()

        val syncWork = PeriodicWorkRequestBuilder<HealthSyncWorker>(
            repeatInterval = 1, // 1 hour (user configurable: 1-4 hours)
            repeatIntervalTimeUnit = TimeUnit.HOURS,
            flexTimeInterval = 15, // 15-minute flex window
            flexTimeIntervalUnit = TimeUnit.MINUTES
        )
            .setConstraints(constraints)
            .setBackoffCriteria(
                BackoffPolicy.EXPONENTIAL,
                WorkRequest.MIN_BACKOFF_MILLIS,
                TimeUnit.MILLISECONDS
            )
            .addTag("health_sync")
            .build()

        WorkManager.getInstance(this).enqueueUniquePeriodicWork(
            "health_sync_unique",
            ExistingPeriodicWorkPolicy.KEEP, // Don't replace if already scheduled
            syncWork
        )
    }

    fun cancelHealthSync() {
        WorkManager.getInstance(this).cancelUniqueWork("health_sync_unique")
    }

    fun updateSyncInterval(hours: Long) {
        cancelHealthSync()
        // Re-schedule with new interval (requires re-enqueue)
        scheduleHealthSync()
    }
}
```

---

## Reading Health Data

### Permission Request Flow

#### 1. Check and Request Permissions (MainActivity.kt)

```kotlin
package com.hexus.mobile

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.health.connect.client.HealthConnectClient
import androidx.health.connect.client.PermissionController
import androidx.health.connect.client.permission.HealthPermission
import androidx.health.connect.client.records.*
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {

    private lateinit var healthConnectClient: HealthConnectClient

    private val requestPermissions = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { permissions ->
        if (permissions.all { it.value }) {
            // All permissions granted
            lifecycleScope.launch {
                loadHealthData()
            }
        } else {
            // Show error or explain why permissions are needed
            showPermissionDeniedDialog()
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Initialize Health Connect client
        healthConnectClient = HealthConnectClient.getOrCreate(this)

        setContent {
            HeXuSTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    PermissionsScreen()
                }
            }
        }

        // Check permissions on launch
        lifecycleScope.launch {
            checkPermissionsAndRequest()
        }
    }

    private suspend fun checkPermissionsAndRequest() {
        val permissions = setOf(
            HealthPermission.getReadPermission(HeartRateRecord::class),
            HealthPermission.getReadPermission(HeartRateVariabilityRmssdRecord::class),
            HealthPermission.getReadPermission(SleepSessionRecord::class),
            HealthPermission.getReadPermission(StepsRecord::class),
            HealthPermission.getReadPermission(ActiveCaloriesBurnedRecord::class)
        )

        val grantedPermissions = healthConnectClient.permissionController
            .getGrantedPermissions()

        if (!grantedPermissions.containsAll(permissions)) {
            // Request missing permissions
            requestPermissions.launch(
                permissions.map { it.permissionType }.toTypedArray()
            )
        } else {
            // Already have all permissions
            loadHealthData()
        }
    }

    private suspend fun loadHealthData() {
        // Trigger initial sync
        // (WorkManager handles periodic sync)
        // This can also manually fetch recent data for immediate display
    }

    private fun showPermissionDeniedDialog() {
        // Show AlertDialog explaining why permissions are needed
    }

    @Composable
    fun PermissionsScreen() {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(16.dp),
            verticalArrangement = Arrangement.Center
        ) {
            Text(
                text = "HeXuS needs access to health data to monitor biometric patterns.",
                style = MaterialTheme.typography.bodyLarge
            )
            Spacer(modifier = Modifier.height(16.dp))
            Button(onClick = {
                lifecycleScope.launch {
                    checkPermissionsAndRequest()
                }
            }) {
                Text("Grant Permissions")
            }
        }
    }
}
```

### Manual Read Example (On-Demand)

```kotlin
suspend fun readRecentHeartRate(
    client: HealthConnectClient,
    hours: Int = 24
): List<HeartRateRecord> {
    val startTime = Instant.now().minus(hours.toLong(), ChronoUnit.HOURS)
    val endTime = Instant.now()

    val request = ReadRecordsRequest(
        recordType = HeartRateRecord::class,
        timeRangeFilter = TimeRangeFilter.between(startTime, endTime)
    )

    val response = client.readRecords(request)
    return response.records
}
```

---

## Data Flow Architecture

### HeXuS Android Data Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│                  Health Connect (System)                     │
│  • Heart Rate, HRV, Sleep, Steps, Calories                   │
│  • Data from: Wear OS, Samsung Health, Fitbit, Google Fit   │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼ (Read via HealthConnectClient)
┌─────────────────────────────────────────────────────────────┐
│              HealthSyncWorker (WorkManager)                  │
│  • Periodic background sync (1-4 hour intervals)             │
│  • Reads delta since last sync                               │
│  • Applies constraints (Wi-Fi, battery, storage)            │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼ (Store locally)
┌─────────────────────────────────────────────────────────────┐
│                SQLite Database (Android)                     │
│  • Biometric samples (heart rate, HRV, sleep stages)        │
│  • Cached for offline access                                 │
│  • Managed by Rust core (via uniffi FFI)                    │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼ (Process via FFI)
┌─────────────────────────────────────────────────────────────┐
│                Rust Core (uniffi Bridge)                     │
│  • Normalize biometric data to common format                │
│  • Calculate HRV metrics (cardio-rs)                        │
│  • Detect outliers and patterns (augurs)                    │
│  • Store in unified schema                                  │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼ (Optional sync)
┌─────────────────────────────────────────────────────────────┐
│              HeXuS Desktop Server (REST API)                 │
│  • Receive biometric batch from mobile                       │
│  • Merge with other data sources (Oura, Whoop, etc.)       │
│  • Run ML models for switch detection                       │
│  • Send insights back to mobile                              │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow Steps

1. **Background Sync Trigger:**
   - WorkManager schedules `HealthSyncWorker` every 1-4 hours
   - Constraints checked: Wi-Fi (optional), battery not low, storage not low

2. **Read from Health Connect:**
   - `HealthSyncWorker.doWork()` calls `HealthConnectClient.readRecords()`
   - Time range filter: last sync timestamp → now
   - Reads: `HeartRateRecord`, `HeartRateVariabilityRmssdRecord`, `SleepSessionRecord`, etc.

3. **Store in Local Database:**
   - Kotlin stores records in SQLite (via Rust FFI or Room)
   - Schema: `biometric_samples` table with `timestamp`, `metric_type`, `value`, `unit`, `source`

4. **Process via Rust Core:**
   - Kotlin calls Rust uniffi function: `processBiometricBatch(samples)`
   - Rust normalizes data (convert units, timestamps, etc.)
   - Rust calculates HRV metrics (if raw RR intervals available)
   - Rust detects outliers (augurs outlier detection)

5. **Sync to Desktop (Optional):**
   - Batch upload to HeXuS desktop server via REST API
   - Desktop merges mobile data with wearable API data (Oura, Whoop)
   - Desktop runs advanced ML models (switch prediction, pattern correlation)

6. **Display in UI:**
   - Tauri frontend queries local SQLite
   - Render charts: heart rate timeline, HRV trends, sleep stages
   - Overlay switch logs (manual or auto-detected)

---

## Code Examples

### Complete Kotlin → Rust Bridge Example

#### Kotlin Data Classes (HealthModels.kt)

```kotlin
package com.hexus.mobile.models

data class BiometricSample(
    val timestamp: Long,      // Unix epoch seconds
    val metricType: String,   // "heart_rate", "hrv_rmssd", "sleep_stage", "steps"
    val value: Double,        // Numeric value
    val unit: String,         // "bpm", "ms", "count", etc.
    val source: String        // Package name (e.g., "com.google.android.apps.fitness")
)

data class SleepStageData(
    val startTime: Long,
    val endTime: Long,
    val stageType: Int  // SleepSessionRecord.STAGE_TYPE_*
)
```

#### Rust uniffi Bridge (lib.rs)

```rust
// HeXuS Rust Core (uniffi FFI)

use uniffi;

#[derive(uniffi::Record)]
pub struct BiometricSample {
    pub timestamp: i64,
    pub metric_type: String,
    pub value: f64,
    pub unit: String,
    pub source: String,
}

#[derive(uniffi::Record)]
pub struct SleepStageData {
    pub start_time: i64,
    pub end_time: i64,
    pub stage_type: i32,
}

#[uniffi::export]
pub fn process_biometric_batch(samples: Vec<BiometricSample>) -> Result<String, String> {
    // Normalize and store in SQLite
    for sample in samples {
        store_biometric_sample(&sample)?;
    }

    // Calculate HRV metrics (if heart rate samples available)
    if samples.iter().any(|s| s.metric_type == "heart_rate") {
        calculate_hrv_metrics()?;
    }

    // Detect outliers
    detect_outliers()?;

    Ok("Batch processed successfully".to_string())
}

#[uniffi::export]
pub fn process_sleep_session(
    start_time: i64,
    end_time: i64,
    stages: Vec<SleepStageData>,
    source: String,
) -> Result<String, String> {
    // Store sleep session in database
    store_sleep_session(start_time, end_time, &stages, &source)?;

    // Analyze sleep quality
    analyze_sleep_quality(start_time, end_time, &stages)?;

    Ok("Sleep session processed".to_string())
}

fn store_biometric_sample(sample: &BiometricSample) -> Result<(), String> {
    // TODO: Insert into SQLite database
    Ok(())
}

fn store_sleep_session(
    start_time: i64,
    end_time: i64,
    stages: &[SleepStageData],
    source: &str,
) -> Result<(), String> {
    // TODO: Insert sleep session and stages into database
    Ok(())
}

fn calculate_hrv_metrics() -> Result<(), String> {
    // TODO: Use cardio-rs to calculate HRV from RR intervals
    Ok(())
}

fn detect_outliers() -> Result<(), String> {
    // TODO: Use augurs for outlier detection
    Ok(())
}

fn analyze_sleep_quality(
    start_time: i64,
    end_time: i64,
    stages: &[SleepStageData],
) -> Result<(), String> {
    // TODO: Calculate sleep efficiency, deep sleep %, etc.
    Ok(())
}

uniffi::setup_scaffolding!();
```

#### Kotlin Calls Rust (HealthDataProcessor.kt)

```kotlin
package com.hexus.mobile.bridge

import com.hexus.mobile.models.BiometricSample
import com.hexus.mobile.models.SleepStageData
import uniffi.hexus_core.*  // Generated by uniffi

object HealthDataProcessor {

    fun processBatch(samples: List<BiometricSample>) {
        try {
            val result = processBiometricBatch(samples)
            android.util.Log.i("HealthDataProcessor", "Batch processed: $result")
        } catch (e: Exception) {
            android.util.Log.e("HealthDataProcessor", "Failed to process batch", e)
        }
    }

    fun processSleep(
        startTime: Long,
        endTime: Long,
        stages: List<SleepStageData>,
        source: String
    ) {
        try {
            val result = processSleepSession(startTime, endTime, stages, source)
            android.util.Log.i("HealthDataProcessor", "Sleep processed: $result")
        } catch (e: Exception) {
            android.util.Log.e("HealthDataProcessor", "Failed to process sleep", e)
        }
    }
}
```

#### Usage in HealthSyncWorker

```kotlin
private suspend fun syncHeartRate(
    client: HealthConnectClient,
    startTime: Instant,
    endTime: Instant
) {
    val request = ReadRecordsRequest(
        recordType = HeartRateRecord::class,
        timeRangeFilter = TimeRangeFilter.between(startTime, endTime)
    )
    val response = client.readRecords(request)
    
    // Convert to BiometricSample
    val samples = response.records.map { record ->
        BiometricSample(
            timestamp = record.time.epochSecond,
            metricType = "heart_rate",
            value = record.beatsPerMinute.toDouble(),
            unit = "bpm",
            source = record.metadata.dataOrigin.packageName
        )
    }

    // Send to Rust core
    HealthDataProcessor.processBatch(samples)
}
```

---

## Error Handling

### Common Errors and Mitigations

| Error | Cause | Mitigation |
|-------|-------|------------|
| `SDK_UNAVAILABLE` | Health Connect not installed | Show install prompt or fallback to Google Fit |
| `PermissionDeniedException` | User denied permissions | Re-prompt with explanation, graceful degradation |
| `SecurityException` | Missing manifest permissions | Check `AndroidManifest.xml` declarations |
| `IllegalArgumentException` | Invalid time range filter | Validate start < end, both in past |
| `NetworkException` | No internet for sync to desktop | Queue locally, retry when connected |
| `DatabaseException` | SQLite write failure | Retry with backoff, alert user if persistent |

### Error Handling Pattern

```kotlin
suspend fun syncWithErrorHandling(
    client: HealthConnectClient,
    startTime: Instant,
    endTime: Instant
): Result<Unit> {
    return try {
        // Attempt sync
        syncHeartRate(client, startTime, endTime)
        syncHRV(client, startTime, endTime)
        syncSleep(client, startTime, endTime)
        Result.success(Unit)
    } catch (e: PermissionDeniedException) {
        // User revoked permissions
        android.util.Log.w("HealthSync", "Permissions revoked", e)
        Result.failure(e)
    } catch (e: IllegalArgumentException) {
        // Invalid parameters
        android.util.Log.e("HealthSync", "Invalid sync parameters", e)
        Result.failure(e)
    } catch (e: Exception) {
        // Generic error (network, database, etc.)
        android.util.Log.e("HealthSync", "Sync failed", e)
        Result.failure(e)
    }
}
```

---

## Testing

### Unit Testing (HealthSyncWorkerTest.kt)

```kotlin
package com.hexus.mobile.sync

import android.content.Context
import androidx.health.connect.client.HealthConnectClient
import androidx.test.core.app.ApplicationProvider
import androidx.work.ListenableWorker
import androidx.work.testing.TestListenableWorkerBuilder
import kotlinx.coroutines.runBlocking
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.mockito.Mock
import org.mockito.Mockito.*
import org.mockito.junit.MockitoJUnitRunner

@RunWith(MockitoJUnitRunner::class)
class HealthSyncWorkerTest {

    private lateinit var context: Context

    @Mock
    private lateinit var healthConnectClient: HealthConnectClient

    @Before
    fun setUp() {
        context = ApplicationProvider.getApplicationContext()
    }

    @Test
    fun testSyncSuccess() = runBlocking {
        val worker = TestListenableWorkerBuilder<HealthSyncWorker>(context).build()

        val result = worker.doWork()

        assert(result is ListenableWorker.Result.Success)
    }

    @Test
    fun testSyncRetryOnFailure() = runBlocking {
        // Mock Health Connect to throw exception
        `when`(healthConnectClient.readRecords(any())).thenThrow(RuntimeException("Network error"))

        val worker = TestListenableWorkerBuilder<HealthSyncWorker>(context).build()

        val result = worker.doWork()

        assert(result is ListenableWorker.Result.Retry)
    }
}
```

### Integration Testing

1. **Health Connect Emulator:**
   - Use Android Studio emulator with Health Connect pre-installed
   - Populate test data via adb:
     ```bash
     adb shell am start -a android.health.connect.action.HEALTH_HOME_SETTINGS
     ```
   - Manually add heart rate, sleep, steps samples

2. **WorkManager Testing:**
   - Use `WorkManagerTestInitHelper.initializeTestWorkManager(context)`
   - Trigger immediate execution: `testDriver.setAllConstraintsMet(workRequest.id)`

3. **Rust FFI Testing:**
   - Test uniffi bindings with mock data
   - Verify Kotlin → Rust data conversion

---

## Google Fit Fallback

### When to Use Google Fit

- **Android < 14** (API < 34) — Health Connect unavailable
- **Health Connect not installed** — some manufacturers don't pre-install
- **User preference** — some users may prefer Google Fit ecosystem

### Google Fit API Implementation

#### Dependencies (build.gradle.kts)

```kotlin
dependencies {
    // Google Fit (legacy, being phased out)
    implementation("com.google.android.gms:play-services-fitness:21.1.0")
    implementation("com.google.android.gms:play-services-auth:20.7.0")
}
```

#### Permissions (AndroidManifest.xml)

```xml
<uses-permission android:name="android.permission.ACTIVITY_RECOGNITION" />
<uses-permission android:name="com.google.android.gms.permission.ACTIVITY_RECOGNITION" />
```

#### Google Fit Authorization

```kotlin
import com.google.android.gms.auth.api.signin.GoogleSignIn
import com.google.android.gms.fitness.Fitness
import com.google.android.gms.fitness.FitnessOptions
import com.google.android.gms.fitness.data.DataType

val fitnessOptions = FitnessOptions.builder()
    .addDataType(DataType.TYPE_HEART_RATE_BPM, FitnessOptions.ACCESS_READ)
    .addDataType(DataType.TYPE_SLEEP_SEGMENT, FitnessOptions.ACCESS_READ)
    .addDataType(DataType.TYPE_STEP_COUNT_DELTA, FitnessOptions.ACCESS_READ)
    .addDataType(DataType.TYPE_CALORIES_EXPENDED, FitnessOptions.ACCESS_READ)
    .build()

val account = GoogleSignIn.getAccountForExtension(context, fitnessOptions)

if (!GoogleSignIn.hasPermissions(account, fitnessOptions)) {
    GoogleSignIn.requestPermissions(
        this, // Activity
        GOOGLE_FIT_PERMISSIONS_REQUEST_CODE,
        account,
        fitnessOptions
    )
} else {
    // Already authorized, read data
    readGoogleFitData(account)
}
```

#### Reading Google Fit Data

```kotlin
import com.google.android.gms.fitness.Fitness
import com.google.android.gms.fitness.data.DataType
import com.google.android.gms.fitness.request.DataReadRequest
import java.util.concurrent.TimeUnit

suspend fun readGoogleFitHeartRate(account: GoogleSignInAccount): List<HeartRateSample> {
    val endTime = System.currentTimeMillis()
    val startTime = endTime - TimeUnit.HOURS.toMillis(24)

    val readRequest = DataReadRequest.Builder()
        .read(DataType.TYPE_HEART_RATE_BPM)
        .setTimeRange(startTime, endTime, TimeUnit.MILLISECONDS)
        .build()

    val response = Fitness.getHistoryClient(context, account)
        .readData(readRequest)
        .await()

    return response.dataSets.flatMap { dataSet ->
        dataSet.dataPoints.map { dataPoint ->
            HeartRateSample(
                timestamp = dataPoint.getTimestamp(TimeUnit.SECONDS),
                bpm = dataPoint.getValue(Field.FIELD_BPM).asFloat().toLong(),
                source = dataPoint.originalDataSource.appPackageName ?: "unknown"
            )
        }
    }
}
```

#### Fallback Strategy in HeXuS

```kotlin
suspend fun initializeHealthDataSource(context: Context): HealthDataSource {
    return when {
        // Prefer Health Connect (Android 14+)
        isHealthConnectAvailable(context) -> {
            HealthConnectDataSource(HealthConnectClient.getOrCreate(context))
        }
        // Fallback to Google Fit
        else -> {
            GoogleFitDataSource(context)
        }
    }
}

interface HealthDataSource {
    suspend fun syncHeartRate(startTime: Long, endTime: Long): List<HeartRateSample>
    suspend fun syncHRV(startTime: Long, endTime: Long): List<HRVSample>
    suspend fun syncSleep(startTime: Long, endTime: Long): List<SleepSession>
}
```

**Note:** Google Fit API is being deprecated in favor of Health Connect. Prioritize Health Connect and use Google Fit only for legacy device support.

---

## Summary

**HeXuS Health Connect Integration:**

✅ **Permissions:** Granular read-only access to heart rate, HRV, sleep, steps, calories  
✅ **Background Sync:** WorkManager with 1-4 hour intervals, constraints for battery/network  
✅ **Data Flow:** Health Connect → Kotlin → Rust (uniffi) → SQLite → Desktop Server  
✅ **Privacy:** All processing on-device, optional sync to desktop  
✅ **Fallback:** Google Fit API for Android < 14

**Next Steps:**
1. Implement `HealthSyncWorker` with full Health Connect integration
2. Build uniffi bridge for Kotlin → Rust biometric data transfer
3. Test on Android 14+ devices (Pixel, Samsung Galaxy)
4. Implement Google Fit fallback for older devices
5. Integrate with HeXuS desktop server API for cross-device sync

**Developer Checklist:**
- [ ] Add Health Connect dependencies to `build.gradle.kts`
- [ ] Declare permissions in `AndroidManifest.xml`
- [ ] Implement `HealthSyncWorker` with all data types
- [ ] Create Rust uniffi bridge for biometric processing
- [ ] Schedule WorkManager periodic sync
- [ ] Test permission flow on real device
- [ ] Implement Google Fit fallback
- [ ] Add error handling and retry logic
- [ ] Write unit tests for worker and Rust FFI
- [ ] Document setup for future developers

---

**Document Status:** Complete  
**Author:** HeXuS Android Sub-Agent  
**Date:** 2026-03-09  
**License:** MIT
