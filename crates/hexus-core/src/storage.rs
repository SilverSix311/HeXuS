//! Local storage layer for HeXuS
//!
//! All biometric data is stored locally in SQLite.
//! Privacy-first: no cloud sync required.

use anyhow::Result;
use rusqlite::Connection;
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
                FOREIGN KEY (alter_id) REFERENCES alters(id)
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
                FOREIGN KEY (fronting_id) REFERENCES fronting_log(id)
            );

            -- Indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_samples_timestamp 
                ON biometric_samples(timestamp);
            CREATE INDEX IF NOT EXISTS idx_samples_metric 
                ON biometric_samples(metric);
            CREATE INDEX IF NOT EXISTS idx_fronting_started 
                ON fronting_log(started_at);

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
                FOREIGN KEY (alter_id) REFERENCES alters(id)
            );
            "#,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::in_memory().expect("Failed to create in-memory database");
        // If we got here, schema initialization worked
        drop(db);
    }
}
