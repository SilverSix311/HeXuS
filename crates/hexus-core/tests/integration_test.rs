//! Integration tests for hexus-core storage layer
//!
//! These tests verify the complete storage workflow end-to-end.

use chrono::Utc;
use hexus_core::{
    Alter, AlterBaseline, BiometricMetric, BiometricSample, Database, FrontingLog,
    FrontingSource,
};

#[test]
fn test_complete_workflow() {
    // Create in-memory database
    let db = Database::in_memory().expect("Failed to create database");

    // 1. Create two alters
    let alice = Alter {
        id: "alice".to_string(),
        name: "Alice".to_string(),
        color: Some("#FF5733".to_string()),
        emoji: Some("🌸".to_string()),
        notes: Some("Primary host".to_string()),
        created_at: Utc::now(),
    };

    let bob = Alter {
        id: "bob".to_string(),
        name: "Bob".to_string(),
        color: Some("#3357FF".to_string()),
        emoji: Some("⚡".to_string()),
        notes: Some("Protector".to_string()),
        created_at: Utc::now(),
    };

    db.create_alter(&alice).unwrap();
    db.create_alter(&bob).unwrap();

    // 2. Log a fronting event for Alice
    let fronting_alice = FrontingLog {
        id: "fronting-1".to_string(),
        alter_id: Some("alice".to_string()),
        confidence: Some(0.95),
        source: FrontingSource::Manual,
        started_at: Utc::now(),
        ended_at: None,
        notes: Some("Morning fronting".to_string()),
    };

    db.create_fronting_log(&fronting_alice).unwrap();

    // 3. Add biometric samples during Alice's fronting
    let samples = vec![
        BiometricSample {
            id: "sample-1".to_string(),
            timestamp: Utc::now(),
            metric: BiometricMetric::HeartRate,
            value: 68.0,
            unit: "bpm".to_string(),
            source: "apple_watch".to_string(),
            fronting_id: Some("fronting-1".to_string()),
        },
        BiometricSample {
            id: "sample-2".to_string(),
            timestamp: Utc::now(),
            metric: BiometricMetric::HeartRateVariability,
            value: 52.0,
            unit: "ms".to_string(),
            source: "apple_watch".to_string(),
            fronting_id: Some("fronting-1".to_string()),
        },
        BiometricSample {
            id: "sample-3".to_string(),
            timestamp: Utc::now(),
            metric: BiometricMetric::SkinTemperature,
            value: 36.4,
            unit: "°C".to_string(),
            source: "oura_ring".to_string(),
            fronting_id: Some("fronting-1".to_string()),
        },
    ];

    db.create_biometric_samples_batch(&samples).unwrap();

    // 4. Create baselines for Alice
    let alice_hr_baseline = AlterBaseline {
        alter_id: "alice".to_string(),
        metric: BiometricMetric::HeartRate,
        mean: 70.0,
        std_dev: 5.0,
        min: 55.0,
        max: 90.0,
        sample_count: 500,
        updated_at: Utc::now(),
    };

    let alice_hrv_baseline = AlterBaseline {
        alter_id: "alice".to_string(),
        metric: BiometricMetric::HeartRateVariability,
        mean: 50.0,
        std_dev: 10.0,
        min: 30.0,
        max: 80.0,
        sample_count: 500,
        updated_at: Utc::now(),
    };

    db.upsert_baseline(&alice_hr_baseline).unwrap();
    db.upsert_baseline(&alice_hrv_baseline).unwrap();

    // 5. Create baseline for Bob
    let bob_hr_baseline = AlterBaseline {
        alter_id: "bob".to_string(),
        metric: BiometricMetric::HeartRate,
        mean: 85.0,
        std_dev: 7.0,
        min: 70.0,
        max: 110.0,
        sample_count: 300,
        updated_at: Utc::now(),
    };

    db.upsert_baseline(&bob_hr_baseline).unwrap();

    // --- Verification ---

    // Verify alters exist
    let alters = db.list_alters().unwrap();
    assert_eq!(alters.len(), 2);
    assert_eq!(alters[0].name, "Alice"); // Sorted by name
    assert_eq!(alters[1].name, "Bob");

    // Verify fronting log
    let logs = db.list_fronting_logs(None, None, None, None).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].alter_id, Some("alice".to_string()));

    // Verify biometric samples
    let hr_samples = db
        .query_biometric_samples(Some(&BiometricMetric::HeartRate), None, None, None, None)
        .unwrap();
    assert_eq!(hr_samples.len(), 1);
    assert_eq!(hr_samples[0].value, 68.0);

    // Verify all samples
    let all_samples = db
        .query_biometric_samples(None, None, None, None, None)
        .unwrap();
    assert_eq!(all_samples.len(), 3);

    // Verify Alice's baselines
    let alice_baselines = db.list_baselines_for_alter("alice").unwrap();
    assert_eq!(alice_baselines.len(), 2);

    // Verify Bob's baselines
    let bob_baselines = db.list_baselines_for_alter("bob").unwrap();
    assert_eq!(bob_baselines.len(), 1);
    assert_eq!(bob_baselines[0].mean, 85.0);

    // Verify specific baseline lookup
    let alice_hr = db
        .get_baseline("alice", &BiometricMetric::HeartRate)
        .unwrap();
    assert_eq!(alice_hr.mean, 70.0);
    assert_eq!(alice_hr.sample_count, 500);

    println!("✅ All integration tests passed!");
}

#[test]
fn test_cascade_deletion() {
    let db = Database::in_memory().unwrap();

    // Create alter with fronting log and baseline
    let alter = Alter {
        id: "test".to_string(),
        name: "Test".to_string(),
        color: None,
        emoji: None,
        notes: None,
        created_at: Utc::now(),
    };
    db.create_alter(&alter).unwrap();

    let log = FrontingLog {
        id: "log-1".to_string(),
        alter_id: Some("test".to_string()),
        confidence: Some(0.8),
        source: FrontingSource::Manual,
        started_at: Utc::now(),
        ended_at: None,
        notes: None,
    };
    db.create_fronting_log(&log).unwrap();

    let baseline = AlterBaseline {
        alter_id: "test".to_string(),
        metric: BiometricMetric::HeartRate,
        mean: 75.0,
        std_dev: 5.0,
        min: 60.0,
        max: 95.0,
        sample_count: 100,
        updated_at: Utc::now(),
    };
    db.upsert_baseline(&baseline).unwrap();

    // Create biometric sample linked to fronting log
    let sample = BiometricSample {
        id: "sample-1".to_string(),
        timestamp: Utc::now(),
        metric: BiometricMetric::HeartRate,
        value: 72.0,
        unit: "bpm".to_string(),
        source: "test".to_string(),
        fronting_id: Some("log-1".to_string()),
    };
    db.create_biometric_sample(&sample).unwrap();

    // Verify everything exists
    assert!(db.get_alter("test").is_ok());
    assert!(db.get_fronting_log("log-1").is_ok());
    assert!(db.get_baseline("test", &BiometricMetric::HeartRate).is_ok());
    assert!(db.get_biometric_sample("sample-1").is_ok());

    // Delete the alter
    db.delete_alter("test").unwrap();

    // Verify cascades
    assert!(db.get_alter("test").is_err());
    assert!(db.get_fronting_log("log-1").is_err()); // CASCADE delete
    assert!(db.get_baseline("test", &BiometricMetric::HeartRate).is_err()); // CASCADE delete

    // Biometric sample should still exist but with NULL fronting_id
    let sample = db.get_biometric_sample("sample-1").unwrap();
    assert_eq!(sample.fronting_id, None); // SET NULL on cascade

    println!("✅ CASCADE deletion works correctly!");
}

#[test]
fn test_query_filters() {
    let db = Database::in_memory().unwrap();

    // Create samples with different metrics and timestamps
    use chrono::Duration;

    let now = Utc::now();
    let one_hour_ago = now - Duration::hours(1);
    let two_hours_ago = now - Duration::hours(2);

    let samples = vec![
        BiometricSample {
            id: "s1".to_string(),
            timestamp: two_hours_ago,
            metric: BiometricMetric::HeartRate,
            value: 70.0,
            unit: "bpm".to_string(),
            source: "watch".to_string(),
            fronting_id: None,
        },
        BiometricSample {
            id: "s2".to_string(),
            timestamp: one_hour_ago,
            metric: BiometricMetric::HeartRate,
            value: 75.0,
            unit: "bpm".to_string(),
            source: "watch".to_string(),
            fronting_id: None,
        },
        BiometricSample {
            id: "s3".to_string(),
            timestamp: now,
            metric: BiometricMetric::SkinTemperature,
            value: 36.5,
            unit: "°C".to_string(),
            source: "ring".to_string(),
            fronting_id: None,
        },
    ];

    db.create_biometric_samples_batch(&samples).unwrap();

    // Query by metric
    let hr_samples = db
        .query_biometric_samples(Some(&BiometricMetric::HeartRate), None, None, None, None)
        .unwrap();
    assert_eq!(hr_samples.len(), 2);

    // Query by time range
    let recent = db
        .query_biometric_samples(None, Some(one_hour_ago), None, None, None)
        .unwrap();
    assert_eq!(recent.len(), 2); // one_hour_ago and now

    // Query by source
    let watch_samples = db
        .query_biometric_samples(None, None, None, Some("watch"), None)
        .unwrap();
    assert_eq!(watch_samples.len(), 2);

    // Query with limit
    let limited = db
        .query_biometric_samples(None, None, None, None, Some(1))
        .unwrap();
    assert_eq!(limited.len(), 1);

    println!("✅ Query filters work correctly!");
}
