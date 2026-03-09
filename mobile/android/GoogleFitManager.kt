package com.hexus.mobile.health

import android.content.Context
import com.google.android.gms.auth.api.signin.GoogleSignIn
import com.google.android.gms.auth.api.signin.GoogleSignInAccount
import com.google.android.gms.fitness.Fitness
import com.google.android.gms.fitness.FitnessOptions
import com.google.android.gms.fitness.data.DataType
import com.google.android.gms.fitness.data.Field
import com.google.android.gms.fitness.request.DataReadRequest
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.tasks.await
import kotlinx.coroutines.withContext
import java.time.Instant
import java.util.concurrent.TimeUnit

/**
 * GoogleFitManager - Fallback for devices without Health Connect
 * 
 * Used when:
 * - Android < 14 (Health Connect unavailable)
 * - Health Connect app not installed
 * - User preference (some users prefer Google Fit ecosystem)
 * 
 * NOTE: Google Fit API is being deprecated in favor of Health Connect.
 * This should only be used for legacy device support.
 * 
 * Responsibilities:
 * - Authenticate with Google Fit
 * - Read historical biometric data
 * - Convert Google Fit data to HeXuS unified format
 * 
 * Data Types Supported:
 * - Heart rate (TYPE_HEART_RATE_BPM)
 * - Sleep (TYPE_SLEEP_SEGMENT)
 * - Steps (TYPE_STEP_COUNT_DELTA)
 * - Active calories (TYPE_CALORIES_EXPENDED)
 * 
 * Limitations:
 * - No native HRV support (must calculate from RR intervals if available)
 * - Sleep stage granularity varies by device
 * - Rate limits: 150 requests/hour per user
 * - Uncertain future (migration to Health Connect ongoing)
 */
class GoogleFitManager(private val context: Context) {

    companion object {
        private const val TAG = "GoogleFitManager"
        
        /**
         * Fitness options (permissions) required by HeXuS
         */
        val FITNESS_OPTIONS = FitnessOptions.builder()
            .addDataType(DataType.TYPE_HEART_RATE_BPM, FitnessOptions.ACCESS_READ)
            .addDataType(DataType.TYPE_SLEEP_SEGMENT, FitnessOptions.ACCESS_READ)
            .addDataType(DataType.TYPE_STEP_COUNT_DELTA, FitnessOptions.ACCESS_READ)
            .addDataType(DataType.TYPE_CALORIES_EXPENDED, FitnessOptions.ACCESS_READ)
            .build()
    }

    /**
     * Check if user is authenticated with Google Fit
     */
    fun isAuthenticated(): Boolean {
        val account = GoogleSignIn.getAccountForExtension(context, FITNESS_OPTIONS)
        return GoogleSignIn.hasPermissions(account, FITNESS_OPTIONS)
    }

    /**
     * Get Google Sign-In account for Fitness API
     */
    fun getAccount(): GoogleSignInAccount? {
        return GoogleSignIn.getAccountForExtension(context, FITNESS_OPTIONS)
    }

    /**
     * Read heart rate samples within time range
     * 
     * @param startTime Start of time range (Instant)
     * @param endTime End of time range (Instant)
     * @return List of HeartRateSample
     */
    suspend fun readHeartRate(
        startTime: Instant,
        endTime: Instant
    ): List<HeartRateSample> = withContext(Dispatchers.IO) {
        val account = getAccount() ?: run {
            android.util.Log.w(TAG, "Not authenticated with Google Fit")
            return@withContext emptyList()
        }

        return@withContext try {
            val readRequest = DataReadRequest.Builder()
                .read(DataType.TYPE_HEART_RATE_BPM)
                .setTimeRange(
                    startTime.toEpochMilli(),
                    endTime.toEpochMilli(),
                    TimeUnit.MILLISECONDS
                )
                .build()

            val response = Fitness.getHistoryClient(context, account)
                .readData(readRequest)
                .await()

            response.dataSets.flatMap { dataSet ->
                dataSet.dataPoints.map { dataPoint ->
                    HeartRateSample(
                        timestamp = dataPoint.getTimestamp(TimeUnit.SECONDS),
                        beatsPerMinute = dataPoint.getValue(Field.FIELD_BPM).asFloat().toLong(),
                        source = dataPoint.originalDataSource.appPackageName ?: "google_fit",
                        device = dataPoint.originalDataSource.device?.model
                    )
                }
            }
        } catch (e: Exception) {
            android.util.Log.e(TAG, "Failed to read heart rate", e)
            emptyList()
        }
    }

    /**
     * Read sleep segments within time range
     * 
     * NOTE: Google Fit sleep data is less granular than Health Connect.
     * Stage types may be limited.
     * 
     * @param startTime Start of time range
     * @param endTime End of time range
     * @return List of SleepSession
     */
    suspend fun readSleep(
        startTime: Instant,
        endTime: Instant
    ): List<SleepSession> = withContext(Dispatchers.IO) {
        val account = getAccount() ?: run {
            android.util.Log.w(TAG, "Not authenticated with Google Fit")
            return@withContext emptyList()
        }

        return@withContext try {
            val readRequest = DataReadRequest.Builder()
                .read(DataType.TYPE_SLEEP_SEGMENT)
                .setTimeRange(
                    startTime.toEpochMilli(),
                    endTime.toEpochMilli(),
                    TimeUnit.MILLISECONDS
                )
                .build()

            val response = Fitness.getHistoryClient(context, account)
                .readData(readRequest)
                .await()

            // Group sleep segments into sessions
            val sessions = mutableListOf<SleepSession>()
            
            response.dataSets.forEach { dataSet ->
                dataSet.dataPoints.forEach { dataPoint ->
                    val startTimeEpoch = dataPoint.getStartTime(TimeUnit.SECONDS)
                    val endTimeEpoch = dataPoint.getEndTime(TimeUnit.SECONDS)
                    val sleepType = dataPoint.getValue(Field.FIELD_SLEEP_SEGMENT_TYPE).asInt()

                    // Convert Google Fit sleep type to HeXuS format
                    val stageType = mapGoogleFitSleepType(sleepType)

                    sessions.add(
                        SleepSession(
                            startTime = startTimeEpoch,
                            endTime = endTimeEpoch,
                            title = "Sleep",
                            notes = null,
                            stages = listOf(
                                SleepStage(
                                    startTime = startTimeEpoch,
                                    endTime = endTimeEpoch,
                                    stageType = stageType,
                                    durationMinutes = (endTimeEpoch - startTimeEpoch) / 60
                                )
                            ),
                            source = dataPoint.originalDataSource.appPackageName ?: "google_fit",
                            device = dataPoint.originalDataSource.device?.model
                        )
                    )
                }
            }

            sessions
        } catch (e: Exception) {
            android.util.Log.e(TAG, "Failed to read sleep", e)
            emptyList()
        }
    }

    /**
     * Read steps count within time range
     * 
     * @param startTime Start of time range
     * @param endTime End of time range
     * @return List of StepsSample
     */
    suspend fun readSteps(
        startTime: Instant,
        endTime: Instant
    ): List<StepsSample> = withContext(Dispatchers.IO) {
        val account = getAccount() ?: run {
            android.util.Log.w(TAG, "Not authenticated with Google Fit")
            return@withContext emptyList()
        }

        return@withContext try {
            val readRequest = DataReadRequest.Builder()
                .read(DataType.TYPE_STEP_COUNT_DELTA)
                .setTimeRange(
                    startTime.toEpochMilli(),
                    endTime.toEpochMilli(),
                    TimeUnit.MILLISECONDS
                )
                .build()

            val response = Fitness.getHistoryClient(context, account)
                .readData(readRequest)
                .await()

            response.dataSets.flatMap { dataSet ->
                dataSet.dataPoints.map { dataPoint ->
                    StepsSample(
                        startTime = dataPoint.getStartTime(TimeUnit.SECONDS),
                        endTime = dataPoint.getEndTime(TimeUnit.SECONDS),
                        count = dataPoint.getValue(Field.FIELD_STEPS).asInt().toLong(),
                        source = dataPoint.originalDataSource.appPackageName ?: "google_fit",
                        device = dataPoint.originalDataSource.device?.model
                    )
                }
            }
        } catch (e: Exception) {
            android.util.Log.e(TAG, "Failed to read steps", e)
            emptyList()
        }
    }

    /**
     * Read active calories within time range
     * 
     * @param startTime Start of time range
     * @param endTime End of time range
     * @return List of CaloriesSample
     */
    suspend fun readActiveCalories(
        startTime: Instant,
        endTime: Instant
    ): List<CaloriesSample> = withContext(Dispatchers.IO) {
        val account = getAccount() ?: run {
            android.util.Log.w(TAG, "Not authenticated with Google Fit")
            return@withContext emptyList()
        }

        return@withContext try {
            val readRequest = DataReadRequest.Builder()
                .read(DataType.TYPE_CALORIES_EXPENDED)
                .setTimeRange(
                    startTime.toEpochMilli(),
                    endTime.toEpochMilli(),
                    TimeUnit.MILLISECONDS
                )
                .build()

            val response = Fitness.getHistoryClient(context, account)
                .readData(readRequest)
                .await()

            response.dataSets.flatMap { dataSet ->
                dataSet.dataPoints.map { dataPoint ->
                    CaloriesSample(
                        startTime = dataPoint.getStartTime(TimeUnit.SECONDS),
                        endTime = dataPoint.getEndTime(TimeUnit.SECONDS),
                        kilocalories = dataPoint.getValue(Field.FIELD_CALORIES).asFloat().toDouble(),
                        source = dataPoint.originalDataSource.appPackageName ?: "google_fit",
                        device = dataPoint.originalDataSource.device?.model
                    )
                }
            }
        } catch (e: Exception) {
            android.util.Log.e(TAG, "Failed to read calories", e)
            emptyList()
        }
    }

    /**
     * Map Google Fit sleep segment types to HeXuS format
     * 
     * Google Fit sleep types:
     * - 1: Awake (during sleep)
     * - 2: Sleep (generic)
     * - 3: Out of bed
     * - 4: Light sleep
     * - 5: Deep sleep
     * - 6: REM sleep
     */
    private fun mapGoogleFitSleepType(sleepType: Int): String {
        return when (sleepType) {
            1 -> "awake"
            2 -> "sleep"
            3 -> "out_of_bed"
            4 -> "light"
            5 -> "deep"
            6 -> "rem"
            else -> "unknown"
        }
    }

    /**
     * Sync all data types since last sync
     * 
     * @param lastSyncTime Timestamp of last successful sync (or null for initial sync)
     * @return BiometricBatch containing all synced data
     */
    suspend fun syncAllData(lastSyncTime: Instant?): BiometricBatch {
        val startTime = lastSyncTime ?: Instant.now().minusSeconds(7 * 24 * 60 * 60) // 7 days
        val endTime = Instant.now()

        return BiometricBatch(
            heartRate = readHeartRate(startTime, endTime),
            hrv = emptyList(), // Google Fit doesn't directly support HRV
            sleep = readSleep(startTime, endTime),
            steps = readSteps(startTime, endTime),
            calories = readActiveCalories(startTime, endTime),
            syncTimestamp = endTime.epochSecond
        )
    }
}

/**
 * UnifiedHealthDataSource - Abstraction layer for Health Connect vs Google Fit
 * 
 * Allows seamless switching between data sources based on device capabilities.
 * 
 * Usage:
 *   val dataSource = UnifiedHealthDataSource.create(context)
 *   val batch = dataSource.syncAllData(lastSyncTime)
 */
interface HealthDataSource {
    suspend fun isAvailable(): Boolean
    suspend fun hasPermissions(): Boolean
    suspend fun syncAllData(lastSyncTime: Instant?): BiometricBatch
}

class HealthConnectDataSource(
    private val manager: HealthConnectManager
) : HealthDataSource {
    override suspend fun isAvailable() = manager.isAvailable()
    override suspend fun hasPermissions() = manager.hasAllPermissions()
    override suspend fun syncAllData(lastSyncTime: Instant?) = manager.syncAllData(lastSyncTime)
}

class GoogleFitDataSource(
    private val manager: GoogleFitManager
) : HealthDataSource {
    override suspend fun isAvailable() = manager.isAuthenticated()
    override suspend fun hasPermissions() = manager.isAuthenticated()
    override suspend fun syncAllData(lastSyncTime: Instant?) = manager.syncAllData(lastSyncTime)
}

object UnifiedHealthDataSource {
    /**
     * Factory method to create appropriate data source based on device capabilities
     * 
     * Preference order:
     * 1. Health Connect (Android 14+, modern)
     * 2. Google Fit (legacy fallback)
     */
    suspend fun create(context: Context): HealthDataSource {
        val healthConnect = HealthConnectManager(context)
        
        return if (healthConnect.isAvailable()) {
            android.util.Log.i("UnifiedHealthDataSource", "Using Health Connect")
            HealthConnectDataSource(healthConnect)
        } else {
            android.util.Log.i("UnifiedHealthDataSource", "Using Google Fit (fallback)")
            GoogleFitDataSource(GoogleFitManager(context))
        }
    }
}
