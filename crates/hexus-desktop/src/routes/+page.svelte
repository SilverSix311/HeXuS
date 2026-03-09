<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface SystemStatus {
    total_alters: number;
    total_switches: number;
    current_fronter: string | null;
    last_switch: string | null;
  }

  interface Switch {
    id: string;
    timestamp: string;
    alter_ids: string[];
    co_conscious: boolean;
    notes: string | null;
    trigger: string | null;
  }

  let status = $state<SystemStatus | null>(null);
  let recentSwitches = $state<Switch[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      [status, recentSwitches] = await Promise.all([
        invoke<SystemStatus>('get_status'),
        invoke<Switch[]>('get_switches', { limit: 5 })
      ]);
      loading = false;
    } catch (e) {
      error = String(e);
      loading = false;
    }
  });

  function formatTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    
    if (diffHours < 1) {
      const diffMins = Math.floor(diffMs / (1000 * 60));
      return `${diffMins} min ago`;
    } else if (diffHours < 24) {
      return `${diffHours} hours ago`;
    } else {
      return date.toLocaleDateString();
    }
  }
</script>

<div class="dashboard">
  <header class="page-header">
    <h1>Dashboard</h1>
    <p class="text-muted">Overview of your system's activity</p>
  </header>

  {#if loading}
    <div class="text-center mt-4">
      <div class="spinner"></div>
      <p class="text-muted mt-2">Loading...</p>
    </div>
  {:else if error}
    <div class="card mt-3">
      <p style="color: #ff6b6b;">Error: {error}</p>
    </div>
  {:else if status}
    <!-- Stats Grid -->
    <div class="grid grid-2 mt-3">
      <div class="card stat-card">
        <h3>Current Fronter</h3>
        <p class="stat-value">{status.current_fronter || 'Unknown'}</p>
        {#if status.last_switch}
          <p class="text-muted">Last switch: {formatTimestamp(status.last_switch)}</p>
        {/if}
      </div>

      <div class="card stat-card">
        <h3>System Stats</h3>
        <div class="stats-grid">
          <div>
            <p class="stat-label">Members</p>
            <p class="stat-number">{status.total_alters}</p>
          </div>
          <div>
            <p class="stat-label">Total Switches</p>
            <p class="stat-number">{status.total_switches}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Recent Switches -->
    <div class="card mt-3">
      <h2>Recent Switches</h2>
      {#if recentSwitches.length === 0}
        <p class="text-muted">No switches recorded yet</p>
      {:else}
        <div class="switches-list">
          {#each recentSwitches as sw (sw.id)}
            <div class="switch-item">
              <div class="switch-header">
                <span class="switch-time">{formatTimestamp(sw.timestamp)}</span>
                {#if sw.co_conscious}
                  <span class="badge">Co-conscious</span>
                {/if}
              </div>
              <p class="switch-alters">
                {sw.alter_ids.join(', ')}
              </p>
              {#if sw.notes}
                <p class="text-muted">{sw.notes}</p>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Quick Actions -->
    <div class="card mt-3">
      <h2>Quick Actions</h2>
      <div class="actions-grid">
        <button class="btn" onclick={() => window.location.href = '/members'}>
          Log Switch
        </button>
        <button class="btn btn-secondary" onclick={() => window.location.href = '/timeline'}>
          View Timeline
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .dashboard {
    max-width: 1200px;
    margin: 0 auto;
  }

  .page-header h1 {
    margin: 0;
    font-size: 2.5rem;
  }

  .stat-card .stat-value {
    font-size: 2rem;
    font-weight: bold;
    color: var(--accent);
    margin: 0.5rem 0;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
    margin-top: 1rem;
  }

  .stat-label {
    color: var(--text-secondary);
    font-size: 0.85rem;
    margin: 0;
  }

  .stat-number {
    font-size: 1.75rem;
    font-weight: bold;
    color: var(--text-primary);
    margin: 0.25rem 0 0 0;
  }

  .switches-list {
    margin-top: 1rem;
  }

  .switch-item {
    padding: 1rem;
    border-radius: 8px;
    background: var(--bg-primary);
    margin-bottom: 0.75rem;
    border: 1px solid var(--border);
  }

  .switch-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .switch-time {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .badge {
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    background: var(--accent);
    color: white;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .switch-alters {
    font-weight: 600;
    margin: 0.5rem 0;
    color: var(--text-primary);
  }

  .actions-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-top: 1rem;
  }
</style>
