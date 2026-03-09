package com.hexus.mobile.sync

import android.content.Context
import androidx.work.*
import com.hexus.mobile.bridge.RustBridge
import com.hexus.mobile.health.HealthConnectManager
import com.hexus.mobile.storage.SyncPreferences
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.time.Instant
import java.util.concurrent.TimeUnit

/**
 * HealthSyncWorker - Background worker for periodic Health Connect data sync
 * 
 * Responsibilities:
 * - Run periodically (1-4 hours, configurable)
 * - Read new data from Health Connect since last sync
 * - Convert to HeXuS unified format
 * - Send to Rust core via uniffi bridge
 * - Update last sync timestamp on success
 * 
 * Constraints:
 * - Wi-Fi only (configurable)
 * - Battery not low
 * - Storage not low
 * 
 * Retry Strategy:
 * - Exponential backoff on failure
 * - Max 3 attempts before giving up
 * 
 * Usage:
 *   Scheduled automatically by HeXuSApplication.onCreate()
 *   Manual trigger: WorkManager.getInstance(context).enqueueUniqueWork(...)
 */
class HealthSyncWorker(
    context: Context,
    params: WorkerParameters
) : CoroutineWorker(context, params) {

    private val healthConnectManager = HealthConnectManager(context)
    private val syncPreferences = SyncPreferences(context)

    companion object {
        const val WORK_NAME = "health_sync_periodic"
        const val TAG = "health_sync"

        /**
         * Schedule periodic sync
         * 
         * @param context Application context
         * @param intervalHours Sync interval in hours (1-4 recommended)
         * @param requireUnmeteredNetwork Require Wi-Fi (default: true)
         */
        fun schedule(
            context: Context,
            intervalHours: Long = 1,
            requireUnmeteredNetwork: Boolean = true
        ) {
            val constraints = Constraints.Builder()
                .setRequiredNetworkType(
                    if (requireUnmeteredNetwork) NetworkType.UNMETERED else NetworkType.CONNECTED
                )
                .setRequiresBatteryNotLow(true)
                .setRequiresStorageNotLow(true)
                .build()

            val syncWork = PeriodicWorkRequestBuilder<HealthSyncWorker>(
                repeatInterval = intervalHours,
                repeatIntervalTimeUnit = TimeUnit.HOURS,
                flexTimeInterval = 15,
                flexTimeIntervalUnit = TimeUnit.MINUTES
            )
                .setConstraints(constraints)
                .setBackoffCriteria(
                    BackoffPolicy.EXPONENTIAL,
                    WorkRequest.MIN_BACKOFF_MILLIS,
                    TimeUnit.MILLISECONDS
                )
                .addTag(TAG)
                .build()

            WorkManager.getInstance(context).enqueueUniquePeriodicWork(
                WORK_NAME,
                ExistingPeriodicWorkPolicy.KEEP,
                syncWork
            )

            android.util.Log.i("HealthSyncWorker", "Scheduled sync every $intervalHours hours")
        }

        /**
         * Cancel scheduled sync
         */
        fun cancel(context: Context) {
            WorkManager.getInstance(context).cancelUniqueWork(WORK_NAME)
            android.util.Log.i("HealthSyncWorker", "Cancelled scheduled sync")
        }

        /**
         * Trigger one-time sync immediately (for testing or manual trigger)
         */
        fun runOnce(context: Context) {
            val syncWork = OneTimeWorkRequestBuilder<HealthSyncWorker>()
                .addTag(TAG)
                .build()

            WorkManager.getInstance(context).enqueue(syncWork)
            android.util.Log.i("HealthSyncWorker", "Triggered one-time sync")
        }
    }

    override suspend fun doWork(): Result = withContext(Dispatchers.IO) {
        android.util.Log.i("HealthSyncWorker", "Starting sync (attempt ${runAttemptCount + 1})")

        return@withContext try {
            // Check Health Connect availability
            if (!healthConnectManager.isAvailable()) {
                android.util.Log.w("HealthSyncWorker", "Health Connect not available")
                return@withContext Result.failure(
                    workDataOf("error" to "Health Connect not available")
                )
            }

            // Check permissions
            if (!healthConnectManager.hasAllPermissions()) {
                android.util.Log.w("HealthSyncWorker", "Missing permissions")
                return@withContext Result.failure(
                    workDataOf("error" to "Missing Health Connect permissions")
                )
            }

            // Get last sync timestamp
            val lastSyncTime = syncPreferences.getLastSyncTimestamp()
            android.util.Log.d(
                "HealthSyncWorker",
                "Last sync: ${lastSyncTime ?: "never"}"
            )

            // Sync all data types
            val batch = healthConnectManager.syncAllData(lastSyncTime)

            android.util.Log.i(
                "HealthSyncWorker",
                "Synced: ${batch.heartRate.size} HR, ${batch.hrv.size} HRV, " +
                "${batch.sleep.size} sleep, ${batch.steps.size} steps, " +
                "${batch.calories.size} calories"
            )

            // Send to Rust core for processing
            val processResult = RustBridge.processBiometricBatch(batch)
            
            if (processResult.success) {
                // Update last sync timestamp
                syncPreferences.saveLastSyncTimestamp(Instant.now())
                
                android.util.Log.i("HealthSyncWorker", "Sync completed successfully")
                
                Result.success(
                    workDataOf(
                        "heart_rate_count" to batch.heartRate.size,
                        "hrv_count" to batch.hrv.size,
                        "sleep_count" to batch.sleep.size,
                        "steps_count" to batch.steps.size,
                        "calories_count" to batch.calories.size,
                        "sync_timestamp" to batch.syncTimestamp
                    )
                )
            } else {
                android.util.Log.e(
                    "HealthSyncWorker",
                    "Rust processing failed: ${processResult.error}"
                )
                
                // Retry if under max attempts
                if (runAttemptCount < 3) {
                    Result.retry()
                } else {
                    Result.failure(workDataOf("error" to processResult.error))
                }
            }

        } catch (e: SecurityException) {
            // Permissions revoked during sync
            android.util.Log.e("HealthSyncWorker", "Permissions error", e)
            Result.failure(workDataOf("error" to "Permissions revoked"))
            
        } catch (e: Exception) {
            // Generic error (network, database, etc.)
            android.util.Log.e("HealthSyncWorker", "Sync failed", e)
            
            // Retry with exponential backoff
            if (runAttemptCount < 3) {
                android.util.Log.w("HealthSyncWorker", "Retrying (attempt ${runAttemptCount + 1})")
                Result.retry()
            } else {
                android.util.Log.e("HealthSyncWorker", "Max retries exceeded")
                Result.failure(workDataOf("error" to e.message))
            }
        }
    }

    /**
     * Called when worker is stopped (e.g., constraints no longer met)
     */
    override suspend fun onStopped() {
        android.util.Log.w("HealthSyncWorker", "Worker stopped (constraints not met)")
        super.onStopped()
    }
}

/**
 * SyncPreferences - Persist last sync timestamp
 */
class SyncPreferences(context: Context) {
    
    private val prefs = context.getSharedPreferences("hexus_sync", Context.MODE_PRIVATE)

    fun getLastSyncTimestamp(): Instant? {
        val epochSeconds = prefs.getLong("last_sync_timestamp", -1)
        return if (epochSeconds > 0) {
            Instant.ofEpochSecond(epochSeconds)
        } else {
            null
        }
    }

    fun saveLastSyncTimestamp(timestamp: Instant) {
        prefs.edit()
            .putLong("last_sync_timestamp", timestamp.epochSecond)
            .apply()
    }

    fun clearLastSyncTimestamp() {
        prefs.edit()
            .remove("last_sync_timestamp")
            .apply()
    }
}
