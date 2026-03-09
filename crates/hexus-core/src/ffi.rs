//! FFI (Foreign Function Interface) layer for mobile platforms
//!
//! This module provides uniffi bindings for Swift (iOS) and Kotlin (Android).
//! The FFI layer is intentionally thin - mobile apps call directly into Rust
//! for all business logic and storage.

use crate::error::HeXuSError as CoreHeXuSError;
use crate::models::{
    Alter as CoreAlter, AlterBaseline as CoreAlterBaseline,
    BiometricMetric as CoreBiometricMetric, BiometricSample as CoreBiometricSample,
    FrontingLog as CoreFrontingLog, FrontingSource as CoreFrontingSource,
};
use crate::storage::Database as CoreDatabase;
use chrono::{TimeZone, Utc};
use std::sync::Arc;

// ============================================================================
// FFI Error Type
// ============================================================================

/// FFI-compatible error type
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum HeXuSError {
    #[error("Database error: {msg}")]
    DatabaseError { msg: String },

    #[error("Not found: {msg}")]
    NotFound { msg: String },

    #[error("Validation error: {msg}")]
    ValidationError { msg: String },

    #[error("Serialization error: {msg}")]
    SerializationError { msg: String },

    #[error("Invalid time range: {msg}")]
    InvalidTimeRange { msg: String },

    #[error("Other error: {msg}")]
    Other { msg: String },
}

impl From<CoreHeXuSError> for HeXuSError {
    fn from(err: CoreHeXuSError) -> Self {
        match err {
            CoreHeXuSError::Database(e) => HeXuSError::DatabaseError {
                msg: e.to_string(),
            },
            CoreHeXuSError::NotFound { entity_type, id } => HeXuSError::NotFound {
                msg: format!("{} with id {}", entity_type, id),
            },
            CoreHeXuSError::Validation(msg) => HeXuSError::ValidationError { msg },
            CoreHeXuSError::Serialization(e) => HeXuSError::SerializationError {
                msg: e.to_string(),
            },
            CoreHeXuSError::InvalidTimeRange { start, end } => HeXuSError::InvalidTimeRange {
                msg: format!("start {} is after end {}", start, end),
            },
            CoreHeXuSError::Other(msg) => HeXuSError::Other { msg },
        }
    }
}

// ============================================================================
// FFI Data Types
// ============================================================================

/// FFI-compatible biometric metric enum
#[derive(Debug, Clone, uniffi::Enum)]
pub enum BiometricMetric {
    HeartRate,
    HeartRateVariability,
    RestingHeartRate,
    GalvanicSkinResponse,
    SkinConductance,
    BloodGlucose,
    SleepDuration,
    SleepStage,
    Steps,
    ActiveCalories,
    SkinTemperature,
    RespiratoryRate,
    BloodOxygen,
}

impl From<BiometricMetric> for CoreBiometricMetric {
    fn from(metric: BiometricMetric) -> Self {
        match metric {
            BiometricMetric::HeartRate => CoreBiometricMetric::HeartRate,
            BiometricMetric::HeartRateVariability => CoreBiometricMetric::HeartRateVariability,
            BiometricMetric::RestingHeartRate => CoreBiometricMetric::RestingHeartRate,
            BiometricMetric::GalvanicSkinResponse => CoreBiometricMetric::GalvanicSkinResponse,
            BiometricMetric::SkinConductance => CoreBiometricMetric::SkinConductance,
            BiometricMetric::BloodGlucose => CoreBiometricMetric::BloodGlucose,
            BiometricMetric::SleepDuration => CoreBiometricMetric::SleepDuration,
            BiometricMetric::SleepStage => CoreBiometricMetric::SleepStage,
            BiometricMetric::Steps => CoreBiometricMetric::Steps,
            BiometricMetric::ActiveCalories => CoreBiometricMetric::ActiveCalories,
            BiometricMetric::SkinTemperature => CoreBiometricMetric::SkinTemperature,
            BiometricMetric::RespiratoryRate => CoreBiometricMetric::RespiratoryRate,
            BiometricMetric::BloodOxygen => CoreBiometricMetric::BloodOxygen,
        }
    }
}

impl From<CoreBiometricMetric> for BiometricMetric {
    fn from(metric: CoreBiometricMetric) -> Self {
        match metric {
            CoreBiometricMetric::HeartRate => BiometricMetric::HeartRate,
            CoreBiometricMetric::HeartRateVariability => BiometricMetric::HeartRateVariability,
            CoreBiometricMetric::RestingHeartRate => BiometricMetric::RestingHeartRate,
            CoreBiometricMetric::GalvanicSkinResponse => BiometricMetric::GalvanicSkinResponse,
            CoreBiometricMetric::SkinConductance => BiometricMetric::SkinConductance,
            CoreBiometricMetric::BloodGlucose => BiometricMetric::BloodGlucose,
            CoreBiometricMetric::SleepDuration => BiometricMetric::SleepDuration,
            CoreBiometricMetric::SleepStage => BiometricMetric::SleepStage,
            CoreBiometricMetric::Steps => BiometricMetric::Steps,
            CoreBiometricMetric::ActiveCalories => BiometricMetric::ActiveCalories,
            CoreBiometricMetric::SkinTemperature => BiometricMetric::SkinTemperature,
            CoreBiometricMetric::RespiratoryRate => BiometricMetric::RespiratoryRate,
            CoreBiometricMetric::BloodOxygen => BiometricMetric::BloodOxygen,
            CoreBiometricMetric::Custom(_) => BiometricMetric::HeartRate, // Fallback for now
        }
    }
}

/// FFI-compatible fronting source type
#[derive(Debug, Clone, uniffi::Enum)]
pub enum FrontingSourceType {
    Manual,
    Biometric,
    Import,
}

/// FFI-compatible fronting source
#[derive(Debug, Clone, uniffi::Record)]
pub struct FrontingSource {
    pub source_type: FrontingSourceType,
    pub extra_data: Option<String>,
}

impl From<FrontingSource> for CoreFrontingSource {
    fn from(source: FrontingSource) -> Self {
        match source.source_type {
            FrontingSourceType::Manual => CoreFrontingSource::Manual,
            FrontingSourceType::Biometric => CoreFrontingSource::Biometric {
                model_version: source.extra_data.unwrap_or_default(),
            },
            FrontingSourceType::Import => CoreFrontingSource::Import {
                source: source.extra_data.unwrap_or_default(),
            },
        }
    }
}

impl From<CoreFrontingSource> for FrontingSource {
    fn from(source: CoreFrontingSource) -> Self {
        match source {
            CoreFrontingSource::Manual => FrontingSource {
                source_type: FrontingSourceType::Manual,
                extra_data: None,
            },
            CoreFrontingSource::Biometric { model_version } => FrontingSource {
                source_type: FrontingSourceType::Biometric,
                extra_data: Some(model_version),
            },
            CoreFrontingSource::Import { source } => FrontingSource {
                source_type: FrontingSourceType::Import,
                extra_data: Some(source),
            },
        }
    }
}

/// FFI-compatible Alter record
#[derive(Debug, Clone, uniffi::Record)]
pub struct Alter {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub emoji: Option<String>,
    pub notes: Option<String>,
    pub created_at: i64, // Unix timestamp
}

impl From<Alter> for CoreAlter {
    fn from(alter: Alter) -> Self {
        CoreAlter {
            id: alter.id,
            name: alter.name,
            color: alter.color,
            emoji: alter.emoji,
            notes: alter.notes,
            created_at: Utc.timestamp_opt(alter.created_at, 0).unwrap(),
        }
    }
}

impl From<CoreAlter> for Alter {
    fn from(alter: CoreAlter) -> Self {
        Alter {
            id: alter.id,
            name: alter.name,
            color: alter.color,
            emoji: alter.emoji,
            notes: alter.notes,
            created_at: alter.created_at.timestamp(),
        }
    }
}

/// FFI-compatible FrontingLog record
#[derive(Debug, Clone, uniffi::Record)]
pub struct FrontingLog {
    pub id: String,
    pub alter_id: Option<String>,
    pub confidence: Option<f32>,
    pub source: FrontingSource,
    pub started_at: i64, // Unix timestamp
    pub ended_at: Option<i64>,
    pub notes: Option<String>,
}

impl From<FrontingLog> for CoreFrontingLog {
    fn from(log: FrontingLog) -> Self {
        CoreFrontingLog {
            id: log.id,
            alter_id: log.alter_id,
            confidence: log.confidence,
            source: log.source.into(),
            started_at: Utc.timestamp_opt(log.started_at, 0).unwrap(),
            ended_at: log.ended_at.map(|ts| Utc.timestamp_opt(ts, 0).unwrap()),
            notes: log.notes,
        }
    }
}

impl From<CoreFrontingLog> for FrontingLog {
    fn from(log: CoreFrontingLog) -> Self {
        FrontingLog {
            id: log.id,
            alter_id: log.alter_id,
            confidence: log.confidence,
            source: log.source.into(),
            started_at: log.started_at.timestamp(),
            ended_at: log.ended_at.map(|dt| dt.timestamp()),
            notes: log.notes,
        }
    }
}

/// FFI-compatible BiometricSample record
#[derive(Debug, Clone, uniffi::Record)]
pub struct BiometricSample {
    pub id: String,
    pub timestamp: i64, // Unix timestamp
    pub metric: BiometricMetric,
    pub value: f64,
    pub unit: String,
    pub source: String,
    pub fronting_id: Option<String>,
}

impl From<BiometricSample> for CoreBiometricSample {
    fn from(sample: BiometricSample) -> Self {
        CoreBiometricSample {
            id: sample.id,
            timestamp: Utc.timestamp_opt(sample.timestamp, 0).unwrap(),
            metric: sample.metric.into(),
            value: sample.value,
            unit: sample.unit,
            source: sample.source,
            fronting_id: sample.fronting_id,
        }
    }
}

impl From<CoreBiometricSample> for BiometricSample {
    fn from(sample: CoreBiometricSample) -> Self {
        BiometricSample {
            id: sample.id,
            timestamp: sample.timestamp.timestamp(),
            metric: sample.metric.into(),
            value: sample.value,
            unit: sample.unit,
            source: sample.source,
            fronting_id: sample.fronting_id,
        }
    }
}

/// FFI-compatible AlterBaseline record
#[derive(Debug, Clone, uniffi::Record)]
pub struct AlterBaseline {
    pub alter_id: String,
    pub metric: BiometricMetric,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sample_count: u32,
    pub updated_at: i64, // Unix timestamp
}

impl From<AlterBaseline> for CoreAlterBaseline {
    fn from(baseline: AlterBaseline) -> Self {
        CoreAlterBaseline {
            alter_id: baseline.alter_id,
            metric: baseline.metric.into(),
            mean: baseline.mean,
            std_dev: baseline.std_dev,
            min: baseline.min,
            max: baseline.max,
            sample_count: baseline.sample_count,
            updated_at: Utc.timestamp_opt(baseline.updated_at, 0).unwrap(),
        }
    }
}

impl From<CoreAlterBaseline> for AlterBaseline {
    fn from(baseline: CoreAlterBaseline) -> Self {
        AlterBaseline {
            alter_id: baseline.alter_id,
            metric: baseline.metric.into(),
            mean: baseline.mean,
            std_dev: baseline.std_dev,
            min: baseline.min,
            max: baseline.max,
            sample_count: baseline.sample_count,
            updated_at: baseline.updated_at.timestamp(),
        }
    }
}

/// Query filter for fronting logs
#[derive(Debug, Clone, uniffi::Record)]
pub struct FrontingLogQuery {
    pub alter_id: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: Option<u32>,
}

/// Query filter for biometric samples
#[derive(Debug, Clone, uniffi::Record)]
pub struct BiometricQuery {
    pub metric: Option<BiometricMetric>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub source: Option<String>,
    pub limit: Option<u32>,
}

// ============================================================================
// FFI Database Interface
// ============================================================================

/// FFI wrapper around the core Database
/// 
/// Uses Arc<Mutex> for thread-safety since rusqlite::Connection is not Sync.
/// This is required by uniffi's Arc handling.
#[derive(uniffi::Object)]
pub struct Database {
    inner: std::sync::Arc<std::sync::Mutex<CoreDatabase>>,
}

#[uniffi::export]
impl Database {
    // ------------------------------------------------------------------------
    // Alter Operations
    // ------------------------------------------------------------------------

    pub fn create_alter(&self, alter: Alter) -> Result<(), HeXuSError> {
        self.inner.lock().unwrap().create_alter(&alter.into())?;
        Ok(())
    }

    pub fn get_alter(&self, id: String) -> Result<Alter, HeXuSError> {
        let alter = self.inner.lock().unwrap().get_alter(&id)?;
        Ok(alter.into())
    }

    pub fn list_alters(&self) -> Result<Vec<Alter>, HeXuSError> {
        let alters = self.inner.lock().unwrap().list_alters()?;
        Ok(alters.into_iter().map(Into::into).collect())
    }

    pub fn update_alter(&self, alter: Alter) -> Result<(), HeXuSError> {
        self.inner.lock().unwrap().update_alter(&alter.into())?;
        Ok(())
    }

    pub fn delete_alter(&self, id: String) -> Result<(), HeXuSError> {
        self.inner.lock().unwrap().delete_alter(&id)?;
        Ok(())
    }

    // ------------------------------------------------------------------------
    // Fronting Log Operations
    // ------------------------------------------------------------------------

    pub fn create_fronting_log(&self, log: FrontingLog) -> Result<(), HeXuSError> {
        self.inner.lock().unwrap().create_fronting_log(&log.into())?;
        Ok(())
    }

    pub fn get_fronting_log(&self, id: String) -> Result<FrontingLog, HeXuSError> {
        let log = self.inner.lock().unwrap().get_fronting_log(&id)?;
        Ok(log.into())
    }

    pub fn query_fronting_logs(
        &self,
        query: FrontingLogQuery,
    ) -> Result<Vec<FrontingLog>, HeXuSError> {
        let logs = self.inner.lock().unwrap().list_fronting_logs(
            query.alter_id.as_deref(),
            query.start_time.map(|ts| Utc.timestamp_opt(ts, 0).unwrap()),
            query.end_time.map(|ts| Utc.timestamp_opt(ts, 0).unwrap()),
            query.limit.map(|l| l as usize),
        )?;
        Ok(logs.into_iter().map(Into::into).collect())
    }

    pub fn update_fronting_log(&self, log: FrontingLog) -> Result<(), HeXuSError> {
        self.inner.lock().unwrap().update_fronting_log(&log.into())?;
        Ok(())
    }

    pub fn delete_fronting_log(&self, id: String) -> Result<(), HeXuSError> {
        self.inner.lock().unwrap().delete_fronting_log(&id)?;
        Ok(())
    }

    // ------------------------------------------------------------------------
    // Biometric Sample Operations
    // ------------------------------------------------------------------------

    pub fn create_biometric_sample(&self, sample: BiometricSample) -> Result<(), HeXuSError> {
        self.inner.lock().unwrap().create_biometric_sample(&sample.into())?;
        Ok(())
    }

    pub fn create_biometric_samples_batch(
        &self,
        samples: Vec<BiometricSample>,
    ) -> Result<(), HeXuSError> {
        let core_samples: Vec<CoreBiometricSample> =
            samples.into_iter().map(Into::into).collect();
        self.inner.lock().unwrap().create_biometric_samples_batch(&core_samples)?;
        Ok(())
    }

    pub fn get_biometric_sample(&self, id: String) -> Result<BiometricSample, HeXuSError> {
        let sample = self.inner.lock().unwrap().get_biometric_sample(&id)?;
        Ok(sample.into())
    }

    pub fn query_biometric_samples(
        &self,
        query: BiometricQuery,
    ) -> Result<Vec<BiometricSample>, HeXuSError> {
        let samples = self.inner.lock().unwrap().query_biometric_samples(
            query.metric.as_ref().map(|m| {
                let core_metric: CoreBiometricMetric = m.clone().into();
                core_metric
            }).as_ref(),
            query.start_time.map(|ts| Utc.timestamp_opt(ts, 0).unwrap()),
            query.end_time.map(|ts| Utc.timestamp_opt(ts, 0).unwrap()),
            query.source.as_deref(),
            query.limit.map(|l| l as usize),
        )?;
        Ok(samples.into_iter().map(Into::into).collect())
    }

    pub fn delete_biometric_samples_before(&self, timestamp: i64) -> Result<u32, HeXuSError> {
        let dt = Utc.timestamp_opt(timestamp, 0).unwrap();
        let count = self.inner.lock().unwrap().delete_biometric_samples_before(dt)?;
        Ok(count as u32)
    }

    // ------------------------------------------------------------------------
    // Baseline Operations
    // ------------------------------------------------------------------------

    pub fn upsert_baseline(&self, baseline: AlterBaseline) -> Result<(), HeXuSError> {
        self.inner.lock().unwrap().upsert_baseline(&baseline.into())?;
        Ok(())
    }

    pub fn get_baseline(
        &self,
        alter_id: String,
        metric: BiometricMetric,
    ) -> Result<AlterBaseline, HeXuSError> {
        let baseline = self.inner.lock().unwrap().get_baseline(&alter_id, &metric.into())?;
        Ok(baseline.into())
    }

    pub fn list_baselines_for_alter(
        &self,
        alter_id: String,
    ) -> Result<Vec<AlterBaseline>, HeXuSError> {
        let baselines = self.inner.lock().unwrap().list_baselines_for_alter(&alter_id)?;
        Ok(baselines.into_iter().map(Into::into).collect())
    }

    pub fn delete_baselines_for_alter(&self, alter_id: String) -> Result<u32, HeXuSError> {
        let count = self.inner.lock().unwrap().delete_baselines_for_alter(&alter_id)?;
        Ok(count as u32)
    }
}

// ============================================================================
// FFI Namespace Functions
// ============================================================================

/// Open database at given path
#[uniffi::export]
pub fn open_database(path: String) -> Result<Arc<Database>, HeXuSError> {
    let inner = CoreDatabase::open(path)?;
    Ok(Arc::new(Database {
        inner: Arc::new(std::sync::Mutex::new(inner)),
    }))
}

/// Create in-memory database (for testing)
#[uniffi::export]
pub fn create_in_memory_database() -> Result<Arc<Database>, HeXuSError> {
    let inner = CoreDatabase::in_memory()?;
    Ok(Arc::new(Database {
        inner: Arc::new(std::sync::Mutex::new(inner)),
    }))
}

// Note: uniffi::setup_scaffolding!() is called in lib.rs
