//
//  HealthKitManager.swift
//  HeXuS
//
//  Created: 2026-03-09
//  Purpose: HealthKit integration for biometric data collection
//

import Foundation
import HealthKit
import Combine

/// Manages HealthKit authorization, queries, and background data sync
@MainActor
class HealthKitManager: ObservableObject {
    
    // MARK: - Properties
    
    private let healthStore = HKHealthStore()
    
    /// Query anchors for incremental sync (persisted to UserDefaults)
    private var heartRateAnchor: HKQueryAnchor?
    private var hrvAnchor: HKQueryAnchor?
    private var sleepAnchor: HKQueryAnchor?
    private var stepsAnchor: HKQueryAnchor?
    
    /// Published state for UI binding
    @Published var authorizationStatus: AuthStatus = .notDetermined
    @Published var latestHeartRate: Double?
    @Published var latestHRV: Double?
    @Published var latestSteps: Int?
    @Published var lastSyncDate: Date?
    
    /// Active queries (kept in memory to receive updates)
    private var activeQueries: [HKQuery] = []
    
    // MARK: - Initialization
    
    init() {
        loadAnchors()
    }
    
    // MARK: - Authorization
    
    enum AuthStatus: String {
        case notDetermined = "Not Requested"
        case denied = "Denied"
        case authorized = "Authorized"
        case restricted = "Restricted"
    }
    
    /// Request HealthKit authorization for biometric data types
    func requestAuthorization() async throws {
        guard HKHealthStore.isHealthDataAvailable() else {
            throw HealthKitError.notAvailable
        }
        
        let typesToRead: Set<HKObjectType> = [
            // Vitals (MVP)
            HKObjectType.quantityType(forIdentifier: .heartRate)!,
            HKObjectType.quantityType(forIdentifier: .heartRateVariabilitySDNN)!,
            HKObjectType.categoryType(forIdentifier: .sleepAnalysis)!,
            HKObjectType.quantityType(forIdentifier: .stepCount)!,
            
            // Additional vitals (future)
            HKObjectType.quantityType(forIdentifier: .oxygenSaturation)!,
            HKObjectType.quantityType(forIdentifier: .bodyTemperature)!,
            HKObjectType.quantityType(forIdentifier: .respiratoryRate)!,
            
            // Activity (future)
            HKObjectType.quantityType(forIdentifier: .distanceWalkingRunning)!,
            HKObjectType.quantityType(forIdentifier: .activeEnergyBurned)!,
            HKObjectType.quantityType(forIdentifier: .appleExerciseTime)!,
        ]
        
        try await healthStore.requestAuthorization(toShare: nil, read: typesToRead)
        
        // Update status
        let heartRateType = HKQuantityType(.heartRate())!
        let status = healthStore.authorizationStatus(for: heartRateType)
        authorizationStatus = status.toAuthStatus()
    }
    
    /// Check current authorization status for a specific data type
    func checkAuthorizationStatus(for identifier: HKQuantityTypeIdentifier) -> HKAuthorizationStatus {
        guard let sampleType = HKQuantityType.quantityType(forIdentifier: identifier) else {
            return .notDetermined
        }
        return healthStore.authorizationStatus(for: sampleType)
    }
    
    // MARK: - Background Observer Setup
    
    /// Set up background observer queries for all data types
    /// Call this in AppDelegate.application(_:didFinishLaunchingWithOptions:)
    func setupBackgroundObservers() {
        print("Setting up background observers...")
        
        setupObserver(for: .heartRate, frequency: .immediate)
        setupObserver(for: .heartRateVariabilitySDNN, frequency: .hourly)
        setupObserver(for: .stepCount, frequency: .hourly)
        setupCategoryObserver(for: .sleepAnalysis, frequency: .daily)
    }
    
    private func setupObserver(for identifier: HKQuantityTypeIdentifier, frequency: HKUpdateFrequency) {
        guard let sampleType = HKObjectType.quantityType(forIdentifier: identifier) else { return }
        
        let query = HKObserverQuery(sampleType: sampleType, predicate: nil) { [weak self] query, completionHandler, error in
            guard let self = self else {
                completionHandler()
                return
            }
            
            if let error = error {
                print("❌ Observer query error for \(identifier): \(error.localizedDescription)")
                completionHandler()
                return
            }
            
            print("✅ New data detected for \(identifier)")
            
            // Trigger incremental sync
            Task {
                await self.handleObserverUpdate(for: identifier)
                completionHandler()
            }
        }
        
        healthStore.execute(query)
        activeQueries.append(query)
        
        // Enable background delivery
        healthStore.enableBackgroundDelivery(for: sampleType, frequency: frequency) { success, error in
            if success {
                print("✅ Background delivery enabled for \(identifier)")
            } else if let error = error {
                print("❌ Background delivery error for \(identifier): \(error.localizedDescription)")
            }
        }
    }
    
    private func setupCategoryObserver(for identifier: HKCategoryTypeIdentifier, frequency: HKUpdateFrequency) {
        guard let sampleType = HKObjectType.categoryType(forIdentifier: identifier) else { return }
        
        let query = HKObserverQuery(sampleType: sampleType, predicate: nil) { [weak self] query, completionHandler, error in
            guard let self = self else {
                completionHandler()
                return
            }
            
            if let error = error {
                print("❌ Observer query error for \(identifier): \(error.localizedDescription)")
                completionHandler()
                return
            }
            
            print("✅ New data detected for \(identifier)")
            
            Task {
                if identifier == .sleepAnalysis {
                    await self.fetchNewSleep()
                }
                completionHandler()
            }
        }
        
        healthStore.execute(query)
        activeQueries.append(query)
        
        healthStore.enableBackgroundDelivery(for: sampleType, frequency: frequency) { success, error in
            if success {
                print("✅ Background delivery enabled for \(identifier)")
            }
        }
    }
    
    private func handleObserverUpdate(for identifier: HKQuantityTypeIdentifier) async {
        switch identifier {
        case .heartRate:
            await fetchNewHeartRate()
        case .heartRateVariabilitySDNN:
            await fetchNewHRV()
        case .stepCount:
            await fetchNewSteps()
        default:
            break
        }
    }
    
    // MARK: - Anchored Queries (Incremental Sync)
    
    /// Fetch new heart rate samples since last anchor
    func fetchNewHeartRate() async {
        let heartRateType = HKQuantityType(.heartRate())!
        
        await withCheckedContinuation { continuation in
            let query = HKAnchoredObjectQuery(
                type: heartRateType,
                predicate: nil,
                anchor: heartRateAnchor,
                limit: HKObjectQueryNoLimit
            ) { [weak self] query, newSamples, deletedSamples, newAnchor, error in
                guard let self = self else {
                    continuation.resume()
                    return
                }
                
                if let error = error {
                    print("❌ Heart rate query error: \(error.localizedDescription)")
                    continuation.resume()
                    return
                }
                
                guard let samples = newSamples as? [HKQuantitySample], !samples.isEmpty else {
                    continuation.resume()
                    return
                }
                
                print("📊 Fetched \(samples.count) new heart rate samples")
                
                // Update anchor
                Task { @MainActor in
                    self.heartRateAnchor = newAnchor
                    self.saveAnchor(newAnchor, forKey: "heartRateAnchor")
                }
                
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
                Task {
                    await self.processInRust(samples: rustSamples, type: "heart_rate")
                    
                    // Update UI
                    if let latest = samples.last {
                        await MainActor.run {
                            self.latestHeartRate = latest.quantity.doubleValue(for: HKUnit(from: "count/min"))
                            self.lastSyncDate = Date()
                        }
                    }
                }
                
                continuation.resume()
            }
            
            healthStore.execute(query)
        }
    }
    
    /// Fetch new HRV samples since last anchor
    func fetchNewHRV() async {
        let hrvType = HKQuantityType(.heartRateVariabilitySDNN())!
        
        await withCheckedContinuation { continuation in
            let query = HKAnchoredObjectQuery(
                type: hrvType,
                predicate: nil,
                anchor: hrvAnchor,
                limit: HKObjectQueryNoLimit
            ) { [weak self] query, newSamples, deletedSamples, newAnchor, error in
                guard let self = self else {
                    continuation.resume()
                    return
                }
                
                if let error = error {
                    print("❌ HRV query error: \(error.localizedDescription)")
                    continuation.resume()
                    return
                }
                
                guard let samples = newSamples as? [HKQuantitySample], !samples.isEmpty else {
                    continuation.resume()
                    return
                }
                
                print("📊 Fetched \(samples.count) new HRV samples")
                
                Task { @MainActor in
                    self.hrvAnchor = newAnchor
                    self.saveAnchor(newAnchor, forKey: "hrvAnchor")
                }
                
                let rustSamples = samples.map { sample in
                    HealthSample(
                        sampleType: "HKQuantityTypeIdentifierHeartRateVariabilitySDNN",
                        value: sample.quantity.doubleValue(for: HKUnit.secondUnit(with: .milli)),
                        startDate: ISO8601DateFormatter().string(from: sample.startDate),
                        endDate: ISO8601DateFormatter().string(from: sample.endDate),
                        source: sample.sourceRevision.source.name,
                        device: sample.device?.name
                    )
                }
                
                Task {
                    await self.processInRust(samples: rustSamples, type: "hrv")
                    
                    if let latest = samples.last {
                        await MainActor.run {
                            self.latestHRV = latest.quantity.doubleValue(for: HKUnit.secondUnit(with: .milli))
                            self.lastSyncDate = Date()
                        }
                    }
                }
                
                continuation.resume()
            }
            
            healthStore.execute(query)
        }
    }
    
    /// Fetch new step count samples since last anchor
    func fetchNewSteps() async {
        let stepsType = HKQuantityType(.stepCount())!
        
        await withCheckedContinuation { continuation in
            let query = HKAnchoredObjectQuery(
                type: stepsType,
                predicate: nil,
                anchor: stepsAnchor,
                limit: HKObjectQueryNoLimit
            ) { [weak self] query, newSamples, deletedSamples, newAnchor, error in
                guard let self = self else {
                    continuation.resume()
                    return
                }
                
                if let error = error {
                    print("❌ Steps query error: \(error.localizedDescription)")
                    continuation.resume()
                    return
                }
                
                guard let samples = newSamples as? [HKQuantitySample], !samples.isEmpty else {
                    continuation.resume()
                    return
                }
                
                print("📊 Fetched \(samples.count) new step count samples")
                
                Task { @MainActor in
                    self.stepsAnchor = newAnchor
                    self.saveAnchor(newAnchor, forKey: "stepsAnchor")
                }
                
                let rustSamples = samples.map { sample in
                    HealthSample(
                        sampleType: "HKQuantityTypeIdentifierStepCount",
                        value: sample.quantity.doubleValue(for: HKUnit.count()),
                        startDate: ISO8601DateFormatter().string(from: sample.startDate),
                        endDate: ISO8601DateFormatter().string(from: sample.endDate),
                        source: sample.sourceRevision.source.name,
                        device: sample.device?.name
                    )
                }
                
                Task {
                    await self.processInRust(samples: rustSamples, type: "steps")
                    
                    let totalSteps = samples.reduce(0.0) { sum, sample in
                        sum + sample.quantity.doubleValue(for: HKUnit.count())
                    }
                    
                    await MainActor.run {
                        self.latestSteps = Int(totalSteps)
                        self.lastSyncDate = Date()
                    }
                }
                
                continuation.resume()
            }
            
            healthStore.execute(query)
        }
    }
    
    /// Fetch new sleep analysis samples since last anchor
    func fetchNewSleep() async {
        let sleepType = HKCategoryType(.sleepAnalysis())!
        
        await withCheckedContinuation { continuation in
            let query = HKAnchoredObjectQuery(
                type: sleepType,
                predicate: nil,
                anchor: sleepAnchor,
                limit: HKObjectQueryNoLimit
            ) { [weak self] query, newSamples, deletedSamples, newAnchor, error in
                guard let self = self else {
                    continuation.resume()
                    return
                }
                
                if let error = error {
                    print("❌ Sleep query error: \(error.localizedDescription)")
                    continuation.resume()
                    return
                }
                
                guard let samples = newSamples as? [HKCategorySample], !samples.isEmpty else {
                    continuation.resume()
                    return
                }
                
                print("📊 Fetched \(samples.count) new sleep samples")
                
                Task { @MainActor in
                    self.sleepAnchor = newAnchor
                    self.saveAnchor(newAnchor, forKey: "sleepAnchor")
                }
                
                // Convert sleep stages to Rust-compatible struct
                let rustSamples = samples.map { sample in
                    let sleepValue = sample.value
                    let stage = self.sleepStageString(from: sleepValue)
                    
                    return SleepSample(
                        stage: stage,
                        startDate: ISO8601DateFormatter().string(from: sample.startDate),
                        endDate: ISO8601DateFormatter().string(from: sample.endDate),
                        durationSeconds: sample.endDate.timeIntervalSince(sample.startDate),
                        source: sample.sourceRevision.source.name
                    )
                }
                
                Task {
                    await self.processSleepInRust(samples: rustSamples)
                    await MainActor.run {
                        self.lastSyncDate = Date()
                    }
                }
                
                continuation.resume()
            }
            
            healthStore.execute(query)
        }
    }
    
    // MARK: - Historical Data Queries
    
    /// Fetch historical heart rate data for a date range
    func fetchHistoricalHeartRate(from startDate: Date, to endDate: Date) async -> [HKQuantitySample] {
        let heartRateType = HKQuantityType(.heartRate())!
        let predicate = HKQuery.predicateForSamples(withStart: startDate, end: endDate, options: .strictStartDate)
        let sortDescriptor = NSSortDescriptor(key: HKSampleSortIdentifierStartDate, ascending: true)
        
        return await withCheckedContinuation { continuation in
            let query = HKSampleQuery(
                sampleType: heartRateType,
                predicate: predicate,
                limit: HKObjectQueryNoLimit,
                sortDescriptors: [sortDescriptor]
            ) { query, samples, error in
                if let error = error {
                    print("❌ Historical heart rate query error: \(error.localizedDescription)")
                    continuation.resume(returning: [])
                    return
                }
                
                guard let samples = samples as? [HKQuantitySample] else {
                    continuation.resume(returning: [])
                    return
                }
                
                print("📊 Fetched \(samples.count) historical heart rate samples")
                continuation.resume(returning: samples)
            }
            
            healthStore.execute(query)
        }
    }
    
    // MARK: - Rust FFI Integration
    
    /// Process health samples in Rust core
    private func processInRust(samples: [HealthSample], type: String) async {
        // TODO: Call uniffi-generated Rust function
        // Example:
        // do {
        //     let result = try processHeartRateSamples(samples: samples)
        //     print("✅ Rust processing result: \(result)")
        // } catch {
        //     print("❌ Rust processing error: \(error)")
        // }
        
        print("📤 Would send \(samples.count) \(type) samples to Rust")
        // Stub: Log sample data for now
        for sample in samples.prefix(3) {
            print("  - \(sample.value) @ \(sample.startDate)")
        }
    }
    
    private func processSleepInRust(samples: [SleepSample]) async {
        // TODO: Call uniffi-generated Rust function
        // Example:
        // do {
        //     let result = try processSleepSamples(samples: samples)
        //     print("✅ Rust sleep processing result: \(result)")
        // } catch {
        //     print("❌ Rust sleep processing error: \(error)")
        // }
        
        print("📤 Would send \(samples.count) sleep samples to Rust")
        for sample in samples.prefix(3) {
            print("  - \(sample.stage): \(sample.durationSeconds/60) min")
        }
    }
    
    // MARK: - Anchor Persistence
    
    private func saveAnchor(_ anchor: HKQueryAnchor?, forKey key: String) {
        guard let anchor = anchor else { return }
        do {
            let data = try NSKeyedArchiver.archivedData(withRootObject: anchor, requiringSecureCoding: true)
            UserDefaults.standard.set(data, forKey: key)
            print("💾 Saved anchor for \(key)")
        } catch {
            print("❌ Failed to save anchor for \(key): \(error)")
        }
    }
    
    private func loadAnchor(forKey key: String) -> HKQueryAnchor? {
        guard let data = UserDefaults.standard.data(forKey: key) else { return nil }
        do {
            let anchor = try NSKeyedUnarchiver.unarchivedObject(ofClass: HKQueryAnchor.self, from: data)
            print("📂 Loaded anchor for \(key)")
            return anchor
        } catch {
            print("❌ Failed to load anchor for \(key): \(error)")
            return nil
        }
    }
    
    private func loadAnchors() {
        heartRateAnchor = loadAnchor(forKey: "heartRateAnchor")
        hrvAnchor = loadAnchor(forKey: "hrvAnchor")
        sleepAnchor = loadAnchor(forKey: "sleepAnchor")
        stepsAnchor = loadAnchor(forKey: "stepsAnchor")
    }
    
    // MARK: - Helpers
    
    private func sleepStageString(from value: Int) -> String {
        switch value {
        case HKCategoryValueSleepAnalysis.inBed.rawValue:
            return "in_bed"
        case HKCategoryValueSleepAnalysis.asleepUnspecified.rawValue:
            return "asleep"
        case HKCategoryValueSleepAnalysis.awake.rawValue:
            return "awake"
        case HKCategoryValueSleepAnalysis.asleepCore.rawValue:
            return "core"
        case HKCategoryValueSleepAnalysis.asleepDeep.rawValue:
            return "deep"
        case HKCategoryValueSleepAnalysis.asleepREM.rawValue:
            return "rem"
        default:
            return "unknown"
        }
    }
}

// MARK: - Extensions

extension HKAuthorizationStatus {
    func toAuthStatus() -> HealthKitManager.AuthStatus {
        switch self {
        case .notDetermined:
            return .notDetermined
        case .sharingDenied:
            return .denied
        case .sharingAuthorized:
            return .authorized
        @unknown default:
            return .restricted
        }
    }
}

// MARK: - Data Structures (uniffi-compatible)

/// HealthKit sample converted to Rust-compatible struct
/// Matches Rust definition in mobile/rust-core/src/lib.rs
struct HealthSample {
    let sampleType: String
    let value: Double
    let startDate: String      // ISO8601
    let endDate: String        // ISO8601
    let source: String
    let device: String?
}

/// Sleep sample converted to Rust-compatible struct
struct SleepSample {
    let stage: String
    let startDate: String      // ISO8601
    let endDate: String        // ISO8601
    let durationSeconds: TimeInterval
    let source: String
}

// MARK: - Errors

enum HealthKitError: Error, LocalizedError {
    case notAvailable
    case authorizationDenied
    case queryFailed(String)
    
    var errorDescription: String? {
        switch self {
        case .notAvailable:
            return "HealthKit is not available on this device"
        case .authorizationDenied:
            return "User denied HealthKit access"
        case .queryFailed(let message):
            return "HealthKit query failed: \(message)"
        }
    }
}
