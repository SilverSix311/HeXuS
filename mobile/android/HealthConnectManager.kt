package com.hexus.mobile.health

import android.content.Context
import androidx.health.connect.client.HealthConnectClient
import androidx.health.connect.client.permission.HealthPermission
import androidx.health.connect.client.records.*
import androidx.health.connect.client.request.ReadRecordsRequest
import androidx.health.connect.client.time.TimeRangeFilter
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.time.Instant
import java.time.temporal.ChronoUnit

/**
 * HealthConnectManager - Centralized Health Connect API wrapper for HeXuS
 * 
 * Responsibilities:
 * - Check Health Connect availability
 * - Request and verify permissions
 * - Read biometric data (heart rate, HRV, sleep, steps, calories)
 * - Convert Health Connect records to HeXuS unified format
 * 
 * Usage:
 *   val manager = HealthConnectManager(context)
 *   if (manager.isAvailable()) {
 *       manager.requestPermissions(activity)
 *       val heartRate = manager.readHeartRate(startTime, endTime)
 *   }
 */
class HealthConnectManager(private val context: Context) {

    private val healthConnectClient by lazy {
        HealthConnectClient.getOrCreate(context)
    }

    companion object {
        // Required permissions for HeXuS
        val REQUIRED_PERMISSIONS = setOf(
            HealthPermission.getReadPermission(HeartRateRecord::class),
            HealthPermission.getReadPermission(HeartRateVariabilityRmssdRecord::class),
            HealthPermission.getReadPermission(SleepSessionRecord::class),
            HealthPermission.getReadPermission(StepsRecord::class),
            HealthPermission.getReadPermission(ActiveCaloriesBurnedRecord::class)
        )
    }

    /**
     * Check if Health Connect is available on this device
     */
    suspend fun isAvailable(): Boolean {
        return HealthConnectClient.getSdkStatus(context) == HealthConnectClient.SDK_AVAILABLE
    }

    /**
     * Check if all required permissions are granted
     */
    suspend fun hasAllPermissions(): Boolean = withContext(Dispatchers.IO) {
        val grantedPermissions = healthConnectClient.permissionController.getGrantedPermissions()
        return@withContext grantedPermissions.containsAll(REQUIRED_PERMISSIONS)
    }

    /**
     * Get currently granted permissions
     */
    suspend fun getGrantedPermissions(): Set<String> = withContext(Dispatchers.IO) {
        healthConnectClient.permissionController.getGrantedPermissions()
    }

    /**
     * Read heart rate samples within time range
     * 
     * @param startTime Start of time range (Instant)
     * @param endTime End of time range (Instant)
     * @return List of HeartRateSample (internal model)
     */
    suspend fun readHeartRate(
        startTime: Instant,
        endTime: Instant
    ): List<HeartRateSample> = withContext(Dispatchers.IO) {
        try {
            val request = ReadRecordsRequest(
                recordType = HeartRateRecord::class,
                timeRangeFilter = TimeRangeFilter.between(startTime, endTime)
            )
            val response = healthConnectClient.readRecords(request)
            
            response.records.map { record ->
                HeartRateSample(
                    timestamp = record.time.epochSecond,
                    beatsPerMinute = record.beatsPerMinute,
                    source = record.metadata.dataOrigin.packageName,
                    device = record.metadata.device?.let { "${it.manufacturer} ${it.model}" }
                )
            }
        } catch (e: Exception) {
            android.util.Log.e("HealthConnectManager", "Failed to read heart rate", e)
            emptyList()
        }
    }

    /**
     * Read HRV (Heart Rate Variability) samples within time range
     * 
     * @param startTime Start of time range
     * @param endTime End of time range
     * @return List of HRVSample (RMSSD values in milliseconds)
     */
    suspend fun readHRV(
        startTime: Instant,
        endTime: Instant
    ): List<HRVSample> = withContext(Dispatchers.IO) {
        try {
            val request = ReadRecordsRequest(
                recordType = HeartRateVariabilityRmssdRecord::class,
                timeRangeFilter = TimeRangeFilter.between(startTime, endTime)
            )
            val response = healthConnectClient.readRecords(request)
            
            response.records.map { record ->
                HRVSample(
                    timestamp = record.time.epochSecond,
                    rmssdMillis = record.heartRateVariabilityMillis,
                    source = record.metadata.dataOrigin.packageName,
                    device = record.metadata.device?.let { "${it.manufacturer} ${it.model}" }
                )
            }
        } catch (e: Exception) {
            android.util.Log.e("HealthConnectManager", "Failed to read HRV", e)
            emptyList()
        }
    }

    /**
     * Read sleep sessions within time range
     * 
     * @param startTime Start of time range
     * @param endTime End of time range
     * @return List of SleepSession (with stages)
     */
    suspend fun readSleep(
        startTime: Instant,
        endTime: Instant
    ): List<SleepSession> = withContext(Dispatchers.IO) {
        try {
            val request = ReadRecordsRequest(
                recordType = SleepSessionRecord::class,
                timeRangeFilter = TimeRangeFilter.between(startTime, endTime)
            )
            val response = healthConnectClient.readRecords(request)
            
            response.records.map { record ->
                SleepSession(
                    startTime = record.startTime.epochSecond,
                    endTime = record.endTime.epochSecond,
                    title = record.title ?: "Sleep",
                    notes = record.notes,
                    stages = record.stages.map { stage ->
                        SleepStage(
                            startTime = stage.startTime.epochSecond,
                            endTime = stage.endTime.epochSecond,
                            stageType = mapSleepStageType(stage.stage),
                            durationMinutes = ChronoUnit.MINUTES.between(
                                stage.startTime,
                                stage.endTime
                            )
                        )
                    },
                    source = record.metadata.dataOrigin.packageName,
                    device = record.metadata.device?.let { "${it.manufacturer} ${it.model}" }
                )
            }
        } catch (e: Exception) {
            android.util.Log.e("HealthConnectManager", "Failed to read sleep", e)
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
        try {
            val request = ReadRecordsRequest(
                recordType = StepsRecord::class,
                timeRangeFilter = TimeRangeFilter.between(startTime, endTime)
            )
            val response = healthConnectClient.readRecords(request)
            
            response.records.map { record ->
                StepsSample(
                    startTime = record.startTime.epochSecond,
                    endTime = record.endTime.epochSecond,
                    count = record.count,
                    source = record.metadata.dataOrigin.packageName,
                    device = record.metadata.device?.let { "${it.manufacturer} ${it.model}" }
                )
            }
        } catch (e: Exception) {
            android.util.Log.e("HealthConnectManager", "Failed to read steps", e)
            emptyList()
        }
    }

    /**
     * Read active calories burned within time range
     * 
     * @param startTime Start of time range
     * @param endTime End of time range
     * @return List of CaloriesSample
     */
    suspend fun readActiveCalories(
        startTime: Instant,
        endTime: Instant
    ): List<CaloriesSample> = withContext(Dispatchers.IO) {
        try {
            val request = ReadRecordsRequest(
                recordType = ActiveCaloriesBurnedRecord::class,
                timeRangeFilter = TimeRangeFilter.between(startTime, endTime)
            )
            val response = healthConnectClient.readRecords(request)
            
            response.records.map { record ->
                CaloriesSample(
                    startTime = record.startTime.epochSecond,
                    endTime = record.endTime.epochSecond,
                    kilocalories = record.energy.inKilocalories,
                    source = record.metadata.dataOrigin.packageName,
                    device = record.metadata.device?.let { "${it.manufacturer} ${it.model}" }
                )
            }
        } catch (e: Exception) {
            android.util.Log.e("HealthConnectManager", "Failed to read calories", e)
            emptyList()
        }
    }

    /**
     * Map Health Connect sleep stage constants to HeXuS internal format
     */
    private fun mapSleepStageType(stage: Int): String {
        return when (stage) {
            SleepSessionRecord.STAGE_TYPE_AWAKE -> "awake"
            SleepSessionRecord.STAGE_TYPE_SLEEP -> "sleep"
            SleepSessionRecord.STAGE_TYPE_OUT_OF_BED -> "out_of_bed"
            SleepSessionRecord.STAGE_TYPE_LIGHT -> "light"
            SleepSessionRecord.STAGE_TYPE_DEEP -> "deep"
            SleepSessionRecord.STAGE_TYPE_REM -> "rem"
            SleepSessionRecord.STAGE_TYPE_AWAKE_IN_BED -> "awake_in_bed"
            SleepSessionRecord.STAGE_TYPE_UNKNOWN -> "unknown"
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
        val startTime = lastSyncTime ?: Instant.now().minus(7, ChronoUnit.DAYS)
        val endTime = Instant.now()

        return BiometricBatch(
            heartRate = readHeartRate(startTime, endTime),
            hrv = readHRV(startTime, endTime),
            sleep = readSleep(startTime, endTime),
            steps = readSteps(startTime, endTime),
            calories = readActiveCalories(startTime, endTime),
            syncTimestamp = endTime.epochSecond
        )
    }
}

// Internal data models (HeXuS unified format)

data class HeartRateSample(
    val timestamp: Long,        // Unix epoch seconds
    val beatsPerMinute: Long,   // BPM
    val source: String,         // Package name
    val device: String?         // Device info
)

data class HRVSample(
    val timestamp: Long,
    val rmssdMillis: Double,    // RMSSD in milliseconds
    val source: String,
    val device: String?
)

data class SleepSession(
    val startTime: Long,
    val endTime: Long,
    val title: String,
    val notes: String?,
    val stages: List<SleepStage>,
    val source: String,
    val device: String?
)

data class SleepStage(
    val startTime: Long,
    val endTime: Long,
    val stageType: String,      // "light", "deep", "rem", "awake", etc.
    val durationMinutes: Long
)

data class StepsSample(
    val startTime: Long,
    val endTime: Long,
    val count: Long,
    val source: String,
    val device: String?
)

data class CaloriesSample(
    val startTime: Long,
    val endTime: Long,
    val kilocalories: Double,
    val source: String,
    val device: String?
)

data class BiometricBatch(
    val heartRate: List<HeartRateSample>,
    val hrv: List<HRVSample>,
    val sleep: List<SleepSession>,
    val steps: List<StepsSample>,
    val calories: List<CaloriesSample>,
    val syncTimestamp: Long
)
