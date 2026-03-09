<script lang="ts">
  import { onMount } from 'svelte';

  let privacyMode = $state(true);
  let notifications = $state(true);
  let autoBackup = $state(false);
  let backupLocation = $state('');
  let biometricDevices = $state<string[]>([]);
  let selectedDevice = $state<string | null>(null);

  onMount(() => {
    // Load settings from storage
    const savedSettings = localStorage.getItem('hexus-settings');
    if (savedSettings) {
      const settings = JSON.parse(savedSettings);
      privacyMode = settings.privacyMode ?? true;
      notifications = settings.notifications ?? true;
      autoBackup = settings.autoBackup ?? false;
      backupLocation = settings.backupLocation ?? '';
    }

    // Mock biometric devices (will be replaced with actual device detection)
    biometricDevices = ['No devices connected'];
  });

  function saveSettings() {
    const settings = {
      privacyMode,
      notifications,
      autoBackup,
      backupLocation
    };
    localStorage.setItem('hexus-settings', JSON.stringify(settings));
    alert('Settings saved!');
  }

  function selectBackupLocation() {
    // Placeholder - will use Tauri file dialog
    alert('File dialog will be implemented with Tauri API');
  }

  function exportData() {
    alert('Data export will be implemented');
  }

  function clearData() {
    if (confirm('Are you sure you want to clear all data? This cannot be undone.')) {
      localStorage.clear();
      alert('Data cleared! Please restart the application.');
    }
  }
</script>

<div class="settings-page">
  <header class="page-header">
    <h1>Settings</h1>
    <p class="text-muted">Configure HeXuS preferences and privacy</p>
  </header>

  <!-- Privacy Settings -->
  <section class="settings-section">
    <h2>🔒 Privacy & Security</h2>
    <div class="card">
      <div class="setting-item">
        <div class="setting-info">
          <h3>Privacy Mode</h3>
          <p class="text-muted">Encrypt all stored data and require authentication on startup</p>
        </div>
        <label class="toggle">
          <input type="checkbox" bind:checked={privacyMode} />
          <span class="slider"></span>
        </label>
      </div>

      <div class="setting-item">
        <div class="setting-info">
          <h3>Local-Only Storage</h3>
          <p class="text-muted">All data stays on this device — no cloud sync</p>
        </div>
        <span class="status-badge enabled">Always Enabled</span>
      </div>
    </div>
  </section>

  <!-- Biometric Devices -->
  <section class="settings-section">
    <h2>📡 Biometric Devices</h2>
    <div class="card">
      <div class="setting-item">
        <div class="setting-info">
          <h3>Connected Devices</h3>
          <p class="text-muted">Manage wearables and biometric sensors</p>
        </div>
        <button class="btn btn-secondary">Scan for Devices</button>
      </div>

      <div class="device-list">
        {#if biometricDevices.length === 0}
          <p class="text-muted">No devices connected</p>
        {:else}
          {#each biometricDevices as device}
            <div class="device-item">
              <span class="device-icon">📱</span>
              <span class="device-name">{device}</span>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </section>

  <!-- Notifications -->
  <section class="settings-section">
    <h2>🔔 Notifications</h2>
    <div class="card">
      <div class="setting-item">
        <div class="setting-info">
          <h3>Enable Notifications</h3>
          <p class="text-muted">Receive alerts for switch events and biometric anomalies</p>
        </div>
        <label class="toggle">
          <input type="checkbox" bind:checked={notifications} />
          <span class="slider"></span>
        </label>
      </div>
    </div>
  </section>

  <!-- Backup & Data -->
  <section class="settings-section">
    <h2>💾 Backup & Data</h2>
    <div class="card">
      <div class="setting-item">
        <div class="setting-info">
          <h3>Automatic Backups</h3>
          <p class="text-muted">Automatically backup data to a secure location</p>
        </div>
        <label class="toggle">
          <input type="checkbox" bind:checked={autoBackup} />
          <span class="slider"></span>
        </label>
      </div>

      {#if autoBackup}
        <div class="setting-item">
          <div class="setting-info">
            <h3>Backup Location</h3>
            <p class="text-muted">{backupLocation || 'Not set'}</p>
          </div>
          <button class="btn btn-secondary" onclick={selectBackupLocation}>
            Choose Folder
          </button>
        </div>
      {/if}

      <div class="setting-actions">
        <button class="btn" onclick={exportData}>Export All Data</button>
        <button class="btn btn-danger" onclick={clearData}>Clear All Data</button>
      </div>
    </div>
  </section>

  <!-- About -->
  <section class="settings-section">
    <h2>ℹ️ About</h2>
    <div class="card">
      <div class="about-info">
        <h3>HeXuS</h3>
        <p class="text-muted">Version 0.1.0</p>
        <p class="text-muted">Privacy-first biometric monitoring for plural systems</p>
        <p class="text-muted mt-2">
          <strong>Privacy Promise:</strong> All your data stays local. No telemetry, 
          no cloud storage, no third parties. Ever.
        </p>
      </div>
    </div>
  </section>

  <!-- Save Button -->
  <div class="save-container">
    <button class="btn btn-primary" onclick={saveSettings}>
      Save Settings
    </button>
  </div>
</div>

<style>
  .settings-page {
    max-width: 900px;
    margin: 0 auto;
  }

  .page-header h1 {
    margin: 0;
    font-size: 2.5rem;
  }

  .settings-section {
    margin-top: 2rem;
  }

  .settings-section h2 {
    margin: 0 0 1rem 0;
    font-size: 1.5rem;
    color: var(--text-primary);
  }

  .setting-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 0;
    border-bottom: 1px solid var(--border);
  }

  .setting-item:last-child {
    border-bottom: none;
  }

  .setting-info h3 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .setting-info p {
    margin: 0.25rem 0 0 0;
  }

  /* Toggle Switch */
  .toggle {
    position: relative;
    display: inline-block;
    width: 52px;
    height: 28px;
  }

  .toggle input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--border);
    transition: 0.3s;
    border-radius: 28px;
  }

  .slider:before {
    position: absolute;
    content: "";
    height: 20px;
    width: 20px;
    left: 4px;
    bottom: 4px;
    background-color: white;
    transition: 0.3s;
    border-radius: 50%;
  }

  input:checked + .slider {
    background-color: var(--accent);
  }

  input:checked + .slider:before {
    transform: translateX(24px);
  }

  /* Status Badge */
  .status-badge {
    padding: 0.5rem 1rem;
    border-radius: 8px;
    font-size: 0.85rem;
    font-weight: 600;
  }

  .status-badge.enabled {
    background: var(--accent);
    color: white;
  }

  /* Device List */
  .device-list {
    margin-top: 1rem;
  }

  .device-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--bg-primary);
    border-radius: 8px;
    margin-bottom: 0.5rem;
  }

  .device-icon {
    font-size: 1.25rem;
  }

  .device-name {
    color: var(--text-primary);
  }

  /* Setting Actions */
  .setting-actions {
    display: flex;
    gap: 1rem;
    margin-top: 1.5rem;
    padding-top: 1.5rem;
    border-top: 1px solid var(--border);
  }

  /* About Info */
  .about-info h3 {
    margin: 0;
    font-size: 1.5rem;
    color: var(--accent);
  }

  .about-info p {
    margin: 0.5rem 0;
  }

  /* Save Button */
  .save-container {
    margin-top: 2rem;
    padding-top: 2rem;
    border-top: 2px solid var(--border);
    text-align: right;
  }

  .btn-primary {
    background: var(--accent);
    color: white;
    padding: 0.75rem 2rem;
    font-size: 1.1rem;
  }

  .btn-primary:hover {
    background: var(--accent-hover);
  }

  .btn-danger {
    background: #ff6b6b;
    color: white;
  }

  .btn-danger:hover {
    background: #ff5252;
  }
</style>
