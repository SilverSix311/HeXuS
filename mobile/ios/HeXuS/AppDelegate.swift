//
//  AppDelegate.swift
//  HeXuS
//
//  Created: 2026-03-09
//  Purpose: App lifecycle management and background task setup
//

import UIKit
import HealthKit

@main
class AppDelegate: UIResponder, UIApplicationDelegate {
    
    var window: UIWindow?
    let healthKitManager = HealthKitManager()
    
    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        
        print("🚀 HeXuS iOS app launching...")
        
        // Set up background observer queries
        // This allows the app to receive HealthKit updates while in background
        Task {
            do {
                try await healthKitManager.requestAuthorization()
                await MainActor.run {
                    healthKitManager.setupBackgroundObservers()
                }
                print("✅ HealthKit background observers configured")
            } catch {
                print("❌ HealthKit authorization error: \(error)")
            }
        }
        
        // Register for background processing tasks (iOS 13+)
        registerBackgroundTasks()
        
        return true
    }
    
    func applicationDidEnterBackground(_ application: UIApplication) {
        print("📱 App entered background")
        
        // Schedule background refresh if needed
        scheduleBackgroundRefresh()
    }
    
    func applicationWillEnterForeground(_ application: UIApplication) {
        print("📱 App entering foreground")
        
        // Trigger manual sync when app becomes active
        Task {
            await healthKitManager.fetchNewHeartRate()
            await healthKitManager.fetchNewHRV()
            await healthKitManager.fetchNewSteps()
            await healthKitManager.fetchNewSleep()
        }
    }
    
    // MARK: - Background Tasks
    
    private func registerBackgroundTasks() {
        // Note: Background task identifiers must be declared in Info.plist
        // under "Permitted background task scheduler identifiers"
        
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: "com.hexus.app.refresh",
            using: nil
        ) { task in
            self.handleAppRefresh(task: task as! BGAppRefreshTask)
        }
        
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: "com.hexus.app.processing",
            using: nil
        ) { task in
            self.handleProcessing(task: task as! BGProcessingTask)
        }
        
        print("✅ Background tasks registered")
    }
    
    private func scheduleBackgroundRefresh() {
        let request = BGAppRefreshTaskRequest(identifier: "com.hexus.app.refresh")
        request.earliestBeginDate = Date(timeIntervalSinceNow: 15 * 60) // 15 minutes
        
        do {
            try BGTaskScheduler.shared.submit(request)
            print("✅ Scheduled background refresh")
        } catch {
            print("❌ Could not schedule background refresh: \(error)")
        }
    }
    
    private func handleAppRefresh(task: BGAppRefreshTask) {
        print("🔄 Background app refresh triggered")
        
        // Schedule next refresh
        scheduleBackgroundRefresh()
        
        // Set expiration handler
        task.expirationHandler = {
            print("⏱️ Background task expired")
            task.setTaskCompleted(success: false)
        }
        
        // Perform quick sync
        Task {
            await healthKitManager.fetchNewHeartRate()
            await healthKitManager.fetchNewHRV()
            task.setTaskCompleted(success: true)
        }
    }
    
    private func handleProcessing(task: BGProcessingTask) {
        print("⚙️ Background processing task triggered")
        
        task.expirationHandler = {
            print("⏱️ Background processing task expired")
            task.setTaskCompleted(success: false)
        }
        
        // Perform heavy processing (ML inference, baseline updates, etc.)
        Task {
            // TODO: Trigger Rust processing tasks
            // - Update per-alter baselines
            // - Run anomaly detection on recent data
            // - Sync to desktop server if available
            
            task.setTaskCompleted(success: true)
        }
    }
}

import BackgroundTasks
