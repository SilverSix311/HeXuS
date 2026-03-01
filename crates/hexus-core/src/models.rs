//! Data models for HeXuS
//!
//! Defines the core types for biometric data, alters, and fronting states.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a member of the plural system (an alter)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alter {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Optional color for UI representation
    pub color: Option<String>,
    /// Optional emoji/icon
    pub emoji: Option<String>,
    /// Notes about this alter
    pub notes: Option<String>,
    /// When this alter was added to the system
    pub created_at: DateTime<Utc>,
}

/// A logged fronting state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontingLog {
    /// Unique identifier
    pub id: String,
    /// Who is fronting (alter ID, or "unknown")
    pub alter_id: Option<String>,
    /// Confidence level (0.0 - 1.0)
    /// 1.0 = definitely this alter, 0.5 = unsure, None = manual entry
    pub confidence: Option<f32>,
    /// How this was determined
    pub source: FrontingSource,
    /// When this fronting state started
    pub started_at: DateTime<Utc>,
    /// When this fronting state ended (None = still fronting)
    pub ended_at: Option<DateTime<Utc>>,
    /// Optional notes
    pub notes: Option<String>,
}

/// How a fronting state was determined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrontingSource {
    /// Manually logged by user
    Manual,
    /// Inferred from biometric patterns
    Biometric { model_version: String },
    /// Imported from another app (e.g., PluralKit, Simply Plural)
    Import { source: String },
}

/// A single biometric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricSample {
    /// Unique identifier
    pub id: String,
    /// Timestamp of the measurement
    pub timestamp: DateTime<Utc>,
    /// Type of measurement
    pub metric: BiometricMetric,
    /// The measured value
    pub value: f64,
    /// Unit of measurement
    pub unit: String,
    /// Source device/app
    pub source: String,
    /// Optional associated fronting state (for labeled data)
    pub fronting_id: Option<String>,
}

/// Types of biometric measurements we track
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BiometricMetric {
    // Cardiac
    HeartRate,
    HeartRateVariability,
    RestingHeartRate,
    
    // Electrodermal
    GalvanicSkinResponse,
    SkinConductance,
    
    // Metabolic
    BloodGlucose,
    
    // Sleep
    SleepDuration,
    SleepStage, // REM, Deep, Light, Awake
    
    // Activity
    Steps,
    ActiveCalories,
    
    // Environmental
    SkinTemperature,
    
    // Respiratory
    RespiratoryRate,
    BloodOxygen,
    
    // Custom/Other
    Custom(String),
}

/// Baseline profile for an alter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlterBaseline {
    /// Which alter this baseline is for
    pub alter_id: String,
    /// Metric being baselined
    pub metric: BiometricMetric,
    /// Average value
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Minimum observed
    pub min: f64,
    /// Maximum observed
    pub max: f64,
    /// Number of samples used to calculate
    pub sample_count: u32,
    /// When this baseline was last updated
    pub updated_at: DateTime<Utc>,
}
