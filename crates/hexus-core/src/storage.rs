//! Local storage layer for HeXuS
//!
//! All biometric data is stored locally in SQLite.
//! Privacy-first: no cloud sync required.

use crate::error::{HeXuSError, Result};
use crate::models::*;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

/// Database manager for HeXuS
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create a database at the given path
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Initialize database schema
    fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            -- Alters table
            CREATE TABLE IF NOT EXISTS alters (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                color TEXT,
                emoji TEXT,
                notes TEXT,
                created_at TEXT NOT NULL
            );

            -- Fronting log
            CREATE TABLE IF NOT EXISTS fronting_log (
                id TEXT PRIMARY KEY,
                alter_id TEXT,
                confidence REAL,
                source_type TEXT NOT NULL,
                source_data TEXT,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                notes TEXT,
                FOREIGN KEY (alter_id) REFERENCES alters(id) ON DELETE CASCADE
            );

            -- Biometric samples
            CREATE TABLE IF NOT EXISTS biometric_samples (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                metric TEXT NOT NULL,
                value REAL NOT NULL,
                unit TEXT NOT NULL,
                source TEXT NOT NULL,
                fronting_id TEXT,
                FOREIGN KEY (fronting_id) REFERENCES fronting_log(id) ON DELETE SET NULL
            );

            -- Indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_samples_timestamp 
                ON biometric_samples(timestamp);
            CREATE INDEX IF NOT EXISTS idx_samples_metric 
                ON biometric_samples(metric);
            CREATE INDEX IF NOT EXISTS idx_fronting_started 
                ON fronting_log(started_at);
            CREATE INDEX IF NOT EXISTS idx_fronting_alter
                ON fronting_log(alter_id);

            -- Alter baselines
            CREATE TABLE IF NOT EXISTS alter_baselines (
                alter_id TEXT NOT NULL,
                metric TEXT NOT NULL,
                mean REAL NOT NULL,
                std_dev REAL NOT NULL,
                min REAL NOT NULL,
                max REAL NOT NULL,
                sample_count INTEGER NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (alter_id, metric),
                FOREIGN KEY (alter_id) REFERENCES alters(id) ON DELETE CASCADE
            );
            "#,
        )?;
        Ok(())
    }

    // ========================================================================
    // Alter CRUD Operations
    // ========================================================================

    /// Create a new alter
    pub fn create_alter(&self, alter: &Alter) -> Result<()> {
        let created_at = alter.created_at.to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO alters (id, name, color, emoji, notes, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &alter.id,
                &alter.name,
                &alter.color,
                &alter.emoji,
                &alter.notes,
                &created_at,
            ],
        )?;
        
        Ok(())
    }

    /// Get an alter by ID
    pub fn get_alter(&self, id: &str) -> Result<Alter> {
        self.conn
            .query_row(
                "SELECT id, name, color, emoji, notes, created_at 
                 FROM alters WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Alter {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        color: row.get(2)?,
                        emoji: row.get(3)?,
                        notes: row.get(4)?,
                        created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                                5,
                                rusqlite::types::Type::Text,
                                Box::new(e)
                            ))?
                            .with_timezone(&Utc),
                    })
                },
            )
            .optional()?
            .ok_or_else(|| HeXuSError::not_found("Alter", id))
    }

    /// List all alters
    pub fn list_alters(&self) -> Result<Vec<Alter>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, color, emoji, notes, created_at 
             FROM alters 
             ORDER BY name ASC"
        )?;

        let alters = stmt
            .query_map([], |row| {
                Ok(Alter {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    emoji: row.get(3)?,
                    notes: row.get(4)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(e)
                        ))?
                        .with_timezone(&Utc),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(alters)
    }

    /// Update an alter
    pub fn update_alter(&self, alter: &Alter) -> Result<()> {
        let rows_affected = self.conn.execute(
            "UPDATE alters 
             SET name = ?1, color = ?2, emoji = ?3, notes = ?4 
             WHERE id = ?5",
            params![
                &alter.name,
                &alter.color,
                &alter.emoji,
                &alter.notes,
                &alter.id,
            ],
        )?;

        if rows_affected == 0 {
            return Err(HeXuSError::not_found("Alter", &alter.id));
        }

        Ok(())
    }

    /// Delete an alter (cascades to fronting logs and baselines)
    pub fn delete_alter(&self, id: &str) -> Result<()> {
        let rows_affected = self.conn.execute(
            "DELETE FROM alters WHERE id = ?1",
            params![id],
        )?;

        if rows_affected == 0 {
            return Err(HeXuSError::not_found("Alter", id));
        }

        Ok(())
    }

    // ========================================================================
    // FrontingLog CRUD Operations
    // ========================================================================

    /// Create a new fronting log entry
    pub fn create_fronting_log(&self, log: &FrontingLog) -> Result<()> {
        let started_at = log.started_at.to_rfc3339();
        let ended_at = log.ended_at.as_ref().map(|dt| dt.to_rfc3339());
        
        let (source_type, source_data) = match &log.source {
            FrontingSource::Manual => ("Manual".to_string(), None),
            FrontingSource::Biometric { model_version } => {
                ("Biometric".to_string(), Some(model_version.clone()))
            }
            FrontingSource::Import { source } => {
                ("Import".to_string(), Some(source.clone()))
            }
        };

        self.conn.execute(
            "INSERT INTO fronting_log 
             (id, alter_id, confidence, source_type, source_data, started_at, ended_at, notes) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &log.id,
                &log.alter_id,
                &log.confidence,
                &source_type,
                &source_data,
                &started_at,
                &ended_at,
                &log.notes,
            ],
        )?;

        Ok(())
    }

    /// Get a fronting log entry by ID
    pub fn get_fronting_log(&self, id: &str) -> Result<FrontingLog> {
        self.conn
            .query_row(
                "SELECT id, alter_id, confidence, source_type, source_data, 
                        started_at, ended_at, notes 
                 FROM fronting_log WHERE id = ?1",
                params![id],
                |row| Self::fronting_log_from_row(row),
            )
            .optional()?
            .ok_or_else(|| HeXuSError::not_found("FrontingLog", id))
    }

    /// List fronting logs with optional filters
    pub fn list_fronting_logs(
        &self,
        alter_id: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<FrontingLog>> {
        let mut query = String::from(
            "SELECT id, alter_id, confidence, source_type, source_data, 
                    started_at, ended_at, notes 
             FROM fronting_log WHERE 1=1"
        );
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(aid) = alter_id {
            query.push_str(" AND alter_id = ?");
            params.push(Box::new(aid.to_string()));
        }

        if let Some(start) = start_time {
            query.push_str(" AND started_at >= ?");
            params.push(Box::new(start.to_rfc3339()));
        }

        if let Some(end) = end_time {
            query.push_str(" AND started_at <= ?");
            params.push(Box::new(end.to_rfc3339()));
        }

        query.push_str(" ORDER BY started_at DESC");

        if let Some(lim) = limit {
            query.push_str(" LIMIT ?");
            params.push(Box::new(lim));
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&query)?;
        
        let logs = stmt
            .query_map(param_refs.as_slice(), |row| Self::fronting_log_from_row(row))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    /// Update a fronting log entry
    pub fn update_fronting_log(&self, log: &FrontingLog) -> Result<()> {
        let ended_at = log.ended_at.as_ref().map(|dt| dt.to_rfc3339());

        let rows_affected = self.conn.execute(
            "UPDATE fronting_log 
             SET alter_id = ?1, confidence = ?2, ended_at = ?3, notes = ?4 
             WHERE id = ?5",
            params![
                &log.alter_id,
                &log.confidence,
                &ended_at,
                &log.notes,
                &log.id,
            ],
        )?;

        if rows_affected == 0 {
            return Err(HeXuSError::not_found("FrontingLog", &log.id));
        }

        Ok(())
    }

    /// Delete a fronting log entry
    pub fn delete_fronting_log(&self, id: &str) -> Result<()> {
        let rows_affected = self.conn.execute(
            "DELETE FROM fronting_log WHERE id = ?1",
            params![id],
        )?;

        if rows_affected == 0 {
            return Err(HeXuSError::not_found("FrontingLog", id));
        }

        Ok(())
    }

    /// Helper to parse fronting log from database row
    fn fronting_log_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<FrontingLog> {
        let source_type: String = row.get(3)?;
        let source_data: Option<String> = row.get(4)?;

        let source = match source_type.as_str() {
            "Manual" => FrontingSource::Manual,
            "Biometric" => FrontingSource::Biometric {
                model_version: source_data.unwrap_or_default(),
            },
            "Import" => FrontingSource::Import {
                source: source_data.unwrap_or_default(),
            },
            _ => FrontingSource::Manual, // Fallback
        };

        Ok(FrontingLog {
            id: row.get(0)?,
            alter_id: row.get(1)?,
            confidence: row.get(2)?,
            source,
            started_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Text,
                    Box::new(e)
                ))?
                .with_timezone(&Utc),
            ended_at: row.get::<_, Option<String>>(6)?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            notes: row.get(7)?,
        })
    }

    // ========================================================================
    // BiometricSample CRUD Operations
    // ========================================================================

    /// Create a new biometric sample
    pub fn create_biometric_sample(&self, sample: &BiometricSample) -> Result<()> {
        let timestamp = sample.timestamp.to_rfc3339();
        let metric = Self::metric_to_string(&sample.metric);

        self.conn.execute(
            "INSERT INTO biometric_samples 
             (id, timestamp, metric, value, unit, source, fronting_id) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &sample.id,
                &timestamp,
                &metric,
                &sample.value,
                &sample.unit,
                &sample.source,
                &sample.fronting_id,
            ],
        )?;

        Ok(())
    }

    /// Batch insert biometric samples (more efficient for bulk imports)
    pub fn create_biometric_samples_batch(&self, samples: &[BiometricSample]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        for sample in samples {
            let timestamp = sample.timestamp.to_rfc3339();
            let metric = Self::metric_to_string(&sample.metric);

            tx.execute(
                "INSERT INTO biometric_samples 
                 (id, timestamp, metric, value, unit, source, fronting_id) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    &sample.id,
                    &timestamp,
                    &metric,
                    &sample.value,
                    &sample.unit,
                    &sample.source,
                    &sample.fronting_id,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Get a biometric sample by ID
    pub fn get_biometric_sample(&self, id: &str) -> Result<BiometricSample> {
        self.conn
            .query_row(
                "SELECT id, timestamp, metric, value, unit, source, fronting_id 
                 FROM biometric_samples WHERE id = ?1",
                params![id],
                |row| Self::biometric_sample_from_row(row),
            )
            .optional()?
            .ok_or_else(|| HeXuSError::not_found("BiometricSample", id))
    }

    /// Query biometric samples with filters
    pub fn query_biometric_samples(
        &self,
        metric: Option<&BiometricMetric>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        source: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<BiometricSample>> {
        let mut query = String::from(
            "SELECT id, timestamp, metric, value, unit, source, fronting_id 
             FROM biometric_samples WHERE 1=1"
        );
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(m) = metric {
            query.push_str(" AND metric = ?");
            params.push(Box::new(Self::metric_to_string(m)));
        }

        if let Some(start) = start_time {
            query.push_str(" AND timestamp >= ?");
            params.push(Box::new(start.to_rfc3339()));
        }

        if let Some(end) = end_time {
            query.push_str(" AND timestamp <= ?");
            params.push(Box::new(end.to_rfc3339()));
        }

        if let Some(src) = source {
            query.push_str(" AND source = ?");
            params.push(Box::new(src.to_string()));
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(lim) = limit {
            query.push_str(" LIMIT ?");
            params.push(Box::new(lim));
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&query)?;
        
        let samples = stmt
            .query_map(param_refs.as_slice(), |row| Self::biometric_sample_from_row(row))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(samples)
    }

    /// Delete biometric samples (typically for cleanup/retention policies)
    pub fn delete_biometric_samples_before(&self, timestamp: DateTime<Utc>) -> Result<usize> {
        let timestamp_str = timestamp.to_rfc3339();
        let count = self.conn.execute(
            "DELETE FROM biometric_samples WHERE timestamp < ?1",
            params![timestamp_str],
        )?;
        Ok(count)
    }

    /// Helper to convert BiometricMetric to string for storage
    fn metric_to_string(metric: &BiometricMetric) -> String {
        match metric {
            BiometricMetric::HeartRate => "HeartRate".to_string(),
            BiometricMetric::HeartRateVariability => "HeartRateVariability".to_string(),
            BiometricMetric::RestingHeartRate => "RestingHeartRate".to_string(),
            BiometricMetric::GalvanicSkinResponse => "GalvanicSkinResponse".to_string(),
            BiometricMetric::SkinConductance => "SkinConductance".to_string(),
            BiometricMetric::BloodGlucose => "BloodGlucose".to_string(),
            BiometricMetric::SleepDuration => "SleepDuration".to_string(),
            BiometricMetric::SleepStage => "SleepStage".to_string(),
            BiometricMetric::Steps => "Steps".to_string(),
            BiometricMetric::ActiveCalories => "ActiveCalories".to_string(),
            BiometricMetric::SkinTemperature => "SkinTemperature".to_string(),
            BiometricMetric::RespiratoryRate => "RespiratoryRate".to_string(),
            BiometricMetric::BloodOxygen => "BloodOxygen".to_string(),
            BiometricMetric::Custom(s) => format!("Custom:{}", s),
        }
    }

    /// Helper to parse BiometricMetric from string
    fn string_to_metric(s: &str) -> BiometricMetric {
        match s {
            "HeartRate" => BiometricMetric::HeartRate,
            "HeartRateVariability" => BiometricMetric::HeartRateVariability,
            "RestingHeartRate" => BiometricMetric::RestingHeartRate,
            "GalvanicSkinResponse" => BiometricMetric::GalvanicSkinResponse,
            "SkinConductance" => BiometricMetric::SkinConductance,
            "BloodGlucose" => BiometricMetric::BloodGlucose,
            "SleepDuration" => BiometricMetric::SleepDuration,
            "SleepStage" => BiometricMetric::SleepStage,
            "Steps" => BiometricMetric::Steps,
            "ActiveCalories" => BiometricMetric::ActiveCalories,
            "SkinTemperature" => BiometricMetric::SkinTemperature,
            "RespiratoryRate" => BiometricMetric::RespiratoryRate,
            "BloodOxygen" => BiometricMetric::BloodOxygen,
            s if s.starts_with("Custom:") => {
                BiometricMetric::Custom(s.strip_prefix("Custom:").unwrap().to_string())
            }
            _ => BiometricMetric::Custom(s.to_string()),
        }
    }

    /// Helper to parse BiometricSample from database row
    fn biometric_sample_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<BiometricSample> {
        Ok(BiometricSample {
            id: row.get(0)?,
            timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e)
                ))?
                .with_timezone(&Utc),
            metric: Self::string_to_metric(&row.get::<_, String>(2)?),
            value: row.get(3)?,
            unit: row.get(4)?,
            source: row.get(5)?,
            fronting_id: row.get(6)?,
        })
    }

    // ========================================================================
    // AlterBaseline CRUD Operations
    // ========================================================================

    /// Create or update an alter baseline
    pub fn upsert_baseline(&self, baseline: &AlterBaseline) -> Result<()> {
        let metric = Self::metric_to_string(&baseline.metric);
        let updated_at = baseline.updated_at.to_rfc3339();

        self.conn.execute(
            "INSERT INTO alter_baselines 
             (alter_id, metric, mean, std_dev, min, max, sample_count, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(alter_id, metric) DO UPDATE SET
                mean = excluded.mean,
                std_dev = excluded.std_dev,
                min = excluded.min,
                max = excluded.max,
                sample_count = excluded.sample_count,
                updated_at = excluded.updated_at",
            params![
                &baseline.alter_id,
                &metric,
                &baseline.mean,
                &baseline.std_dev,
                &baseline.min,
                &baseline.max,
                &baseline.sample_count,
                &updated_at,
            ],
        )?;

        Ok(())
    }

    /// Get a baseline for an alter and metric
    pub fn get_baseline(&self, alter_id: &str, metric: &BiometricMetric) -> Result<AlterBaseline> {
        let metric_str = Self::metric_to_string(metric);
        
        self.conn
            .query_row(
                "SELECT alter_id, metric, mean, std_dev, min, max, sample_count, updated_at 
                 FROM alter_baselines 
                 WHERE alter_id = ?1 AND metric = ?2",
                params![alter_id, metric_str],
                |row| {
                    Ok(AlterBaseline {
                        alter_id: row.get(0)?,
                        metric: Self::string_to_metric(&row.get::<_, String>(1)?),
                        mean: row.get(2)?,
                        std_dev: row.get(3)?,
                        min: row.get(4)?,
                        max: row.get(5)?,
                        sample_count: row.get(6)?,
                        updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                                7,
                                rusqlite::types::Type::Text,
                                Box::new(e)
                            ))?
                            .with_timezone(&Utc),
                    })
                },
            )
            .optional()?
            .ok_or_else(|| HeXuSError::not_found(
                "AlterBaseline",
                format!("{}/{:?}", alter_id, metric)
            ))
    }

    /// List all baselines for an alter
    pub fn list_baselines_for_alter(&self, alter_id: &str) -> Result<Vec<AlterBaseline>> {
        let mut stmt = self.conn.prepare(
            "SELECT alter_id, metric, mean, std_dev, min, max, sample_count, updated_at 
             FROM alter_baselines 
             WHERE alter_id = ?1 
             ORDER BY metric ASC"
        )?;

        let baselines = stmt
            .query_map(params![alter_id], |row| {
                Ok(AlterBaseline {
                    alter_id: row.get(0)?,
                    metric: Self::string_to_metric(&row.get::<_, String>(1)?),
                    mean: row.get(2)?,
                    std_dev: row.get(3)?,
                    min: row.get(4)?,
                    max: row.get(5)?,
                    sample_count: row.get(6)?,
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            7,
                            rusqlite::types::Type::Text,
                            Box::new(e)
                        ))?
                        .with_timezone(&Utc),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(baselines)
    }

    /// Delete all baselines for an alter (happens on alter deletion via CASCADE)
    pub fn delete_baselines_for_alter(&self, alter_id: &str) -> Result<usize> {
        let count = self.conn.execute(
            "DELETE FROM alter_baselines WHERE alter_id = ?1",
            params![alter_id],
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::in_memory().expect("Failed to create in-memory database");
        drop(db);
    }

    #[test]
    fn test_alter_crud() {
        let db = Database::in_memory().unwrap();

        // Create
        let alter = Alter {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            color: Some("#FF5733".to_string()),
            emoji: Some("🌸".to_string()),
            notes: Some("Primary host".to_string()),
            created_at: Utc::now(),
        };
        db.create_alter(&alter).unwrap();

        // Read
        let retrieved = db.get_alter("alice").unwrap();
        assert_eq!(retrieved.name, "Alice");
        assert_eq!(retrieved.emoji, Some("🌸".to_string()));

        // List
        let alters = db.list_alters().unwrap();
        assert_eq!(alters.len(), 1);

        // Update
        let mut updated = retrieved.clone();
        updated.name = "Alice (Host)".to_string();
        db.update_alter(&updated).unwrap();
        
        let retrieved = db.get_alter("alice").unwrap();
        assert_eq!(retrieved.name, "Alice (Host)");

        // Delete
        db.delete_alter("alice").unwrap();
        assert!(db.get_alter("alice").is_err());
    }

    #[test]
    fn test_fronting_log_crud() {
        let db = Database::in_memory().unwrap();

        // Create alter first
        let alter = Alter {
            id: "bob".to_string(),
            name: "Bob".to_string(),
            color: None,
            emoji: None,
            notes: None,
            created_at: Utc::now(),
        };
        db.create_alter(&alter).unwrap();

        // Create fronting log
        let log = FrontingLog {
            id: "log-1".to_string(),
            alter_id: Some("bob".to_string()),
            confidence: Some(0.95),
            source: FrontingSource::Manual,
            started_at: Utc::now(),
            ended_at: None,
            notes: Some("Felt a switch".to_string()),
        };
        db.create_fronting_log(&log).unwrap();

        // Read
        let retrieved = db.get_fronting_log("log-1").unwrap();
        assert_eq!(retrieved.alter_id, Some("bob".to_string()));
        assert_eq!(retrieved.confidence, Some(0.95));

        // List
        let logs = db.list_fronting_logs(Some("bob"), None, None, None).unwrap();
        assert_eq!(logs.len(), 1);

        // Update (end the fronting)
        let mut updated = retrieved.clone();
        updated.ended_at = Some(Utc::now());
        db.update_fronting_log(&updated).unwrap();

        let retrieved = db.get_fronting_log("log-1").unwrap();
        assert!(retrieved.ended_at.is_some());

        // Delete
        db.delete_fronting_log("log-1").unwrap();
        assert!(db.get_fronting_log("log-1").is_err());
    }

    #[test]
    fn test_biometric_sample_crud() {
        let db = Database::in_memory().unwrap();

        // Create sample
        let sample = BiometricSample {
            id: "sample-1".to_string(),
            timestamp: Utc::now(),
            metric: BiometricMetric::HeartRate,
            value: 72.5,
            unit: "bpm".to_string(),
            source: "healthkit".to_string(),
            fronting_id: None,
        };
        db.create_biometric_sample(&sample).unwrap();

        // Read
        let retrieved = db.get_biometric_sample("sample-1").unwrap();
        assert_eq!(retrieved.value, 72.5);
        assert_eq!(retrieved.metric, BiometricMetric::HeartRate);

        // Query
        let samples = db.query_biometric_samples(
            Some(&BiometricMetric::HeartRate),
            None,
            None,
            None,
            None,
        ).unwrap();
        assert_eq!(samples.len(), 1);

        // Batch insert
        let batch = vec![
            BiometricSample {
                id: "sample-2".to_string(),
                timestamp: Utc::now(),
                metric: BiometricMetric::HeartRateVariability,
                value: 45.0,
                unit: "ms".to_string(),
                source: "oura".to_string(),
                fronting_id: None,
            },
            BiometricSample {
                id: "sample-3".to_string(),
                timestamp: Utc::now(),
                metric: BiometricMetric::HeartRate,
                value: 68.0,
                unit: "bpm".to_string(),
                source: "healthkit".to_string(),
                fronting_id: None,
            },
        ];
        db.create_biometric_samples_batch(&batch).unwrap();

        // Query all
        let all_samples = db.query_biometric_samples(None, None, None, None, None).unwrap();
        assert_eq!(all_samples.len(), 3);
    }

    #[test]
    fn test_baseline_crud() {
        let db = Database::in_memory().unwrap();

        // Create alter
        let alter = Alter {
            id: "charlie".to_string(),
            name: "Charlie".to_string(),
            color: None,
            emoji: None,
            notes: None,
            created_at: Utc::now(),
        };
        db.create_alter(&alter).unwrap();

        // Create baseline
        let baseline = AlterBaseline {
            alter_id: "charlie".to_string(),
            metric: BiometricMetric::HeartRate,
            mean: 70.0,
            std_dev: 5.0,
            min: 55.0,
            max: 90.0,
            sample_count: 1000,
            updated_at: Utc::now(),
        };
        db.upsert_baseline(&baseline).unwrap();

        // Read
        let retrieved = db.get_baseline("charlie", &BiometricMetric::HeartRate).unwrap();
        assert_eq!(retrieved.mean, 70.0);
        assert_eq!(retrieved.sample_count, 1000);

        // Update (upsert with new values)
        let mut updated = baseline.clone();
        updated.mean = 72.0;
        updated.sample_count = 1500;
        db.upsert_baseline(&updated).unwrap();

        let retrieved = db.get_baseline("charlie", &BiometricMetric::HeartRate).unwrap();
        assert_eq!(retrieved.mean, 72.0);
        assert_eq!(retrieved.sample_count, 1500);

        // List all baselines for alter
        let baselines = db.list_baselines_for_alter("charlie").unwrap();
        assert_eq!(baselines.len(), 1);
    }

    #[test]
    fn test_cascade_delete() {
        let db = Database::in_memory().unwrap();

        // Create alter with baseline and fronting log
        let alter = Alter {
            id: "diana".to_string(),
            name: "Diana".to_string(),
            color: None,
            emoji: None,
            notes: None,
            created_at: Utc::now(),
        };
        db.create_alter(&alter).unwrap();

        let baseline = AlterBaseline {
            alter_id: "diana".to_string(),
            metric: BiometricMetric::HeartRate,
            mean: 65.0,
            std_dev: 4.0,
            min: 50.0,
            max: 85.0,
            sample_count: 500,
            updated_at: Utc::now(),
        };
        db.upsert_baseline(&baseline).unwrap();

        let log = FrontingLog {
            id: "log-diana".to_string(),
            alter_id: Some("diana".to_string()),
            confidence: Some(0.9),
            source: FrontingSource::Manual,
            started_at: Utc::now(),
            ended_at: None,
            notes: None,
        };
        db.create_fronting_log(&log).unwrap();

        // Delete alter (should cascade)
        db.delete_alter("diana").unwrap();

        // Verify cascades
        assert!(db.get_baseline("diana", &BiometricMetric::HeartRate).is_err());
        assert!(db.get_fronting_log("log-diana").is_err());
    }
}
