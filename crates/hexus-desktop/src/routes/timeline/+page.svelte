<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface Switch {
    id: string;
    timestamp: string;
    alter_ids: string[];
    co_conscious: boolean;
    notes: string | null;
    trigger: string | null;
  }

  let switches = $state<Switch[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let selectedRange = $state('7d');

  onMount(async () => {
    await loadSwitches();
  });

  async function loadSwitches() {
    try {
      loading = true;
      switches = await invoke<Switch[]>('get_switches', { limit: 50 });
      loading = false;
    } catch (e) {
      error = String(e);
      loading = false;
    }
  }

  function formatDate(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function formatTime(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function groupByDay(switches: Switch[]): Map<string, Switch[]> {
    const grouped = new Map<string, Switch[]>();
    
    switches.forEach(sw => {
      const date = new Date(sw.timestamp);
      const day = date.toLocaleDateString('en-US', {
        weekday: 'long',
        year: 'numeric',
        month: 'long',
        day: 'numeric'
      });
      
      if (!grouped.has(day)) {
        grouped.set(day, []);
      }
      grouped.get(day)!.push(sw);
    });
    
    return grouped;
  }

  $: groupedSwitches = groupByDay(switches);
</script>

<div class="timeline-page">
  <header class="page-header">
    <div>
      <h1>Timeline</h1>
      <p class="text-muted">Switch history and biometric data</p>
    </div>
    
    <div class="filters">
      <select bind:value={selectedRange} class="range-select">
        <option value="1d">Last 24 hours</option>
        <option value="7d">Last 7 days</option>
        <option value="30d">Last 30 days</option>
        <option value="all">All time</option>
      </select>
    </div>
  </header>

  {#if loading}
    <div class="text-center mt-4">
      <div class="spinner"></div>
      <p class="text-muted mt-2">Loading timeline...</p>
    </div>
  {:else if error}
    <div class="card mt-3">
      <p style="color: #ff6b6b;">Error: {error}</p>
    </div>
  {:else if switches.length === 0}
    <div class="card mt-3 text-center">
      <h3>No Timeline Data</h3>
      <p class="text-muted">Switch events will appear here as they're logged</p>
    </div>
  {:else}
    <div class="timeline-container mt-3">
      {#each [...groupedSwitches.entries()] as [day, daySwitches] (day)}
        <div class="day-group">
          <h3 class="day-header">{day}</h3>
          
          <div class="timeline-rail">
            {#each daySwitches as sw (sw.id)}
              <div class="timeline-item">
                <div class="timeline-marker"></div>
                <div class="timeline-content card">
                  <div class="timeline-header">
                    <span class="timeline-time">{formatTime(sw.timestamp)}</span>
                    {#if sw.co_conscious}
                      <span class="badge">Co-conscious</span>
                    {/if}
                  </div>
                  
                  <p class="fronters">
                    <strong>{sw.alter_ids.join(', ')}</strong>
                  </p>
                  
                  {#if sw.notes}
                    <p class="notes">{sw.notes}</p>
                  {/if}
                  
                  {#if sw.trigger}
                    <p class="trigger">
                      <span class="trigger-label">Trigger:</span> {sw.trigger}
                    </p>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>

    <!-- Placeholder for biometric chart -->
    <div class="card mt-3">
      <h2>Biometric Data</h2>
      <div class="chart-placeholder">
        <p class="text-muted">📈 Biometric charts will be displayed here</p>
        <p class="text-muted" style="font-size: 0.85rem;">HRV, heart rate, sleep data, etc.</p>
      </div>
    </div>
  {/if}
</div>

<style>
  .timeline-page {
    max-width: 900px;
    margin: 0 auto;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .page-header h1 {
    margin: 0;
    font-size: 2.5rem;
  }

  .filters {
    display: flex;
    gap: 1rem;
  }

  .range-select {
    padding: 0.5rem 1rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 0.95rem;
    cursor: pointer;
  }

  .day-group {
    margin-bottom: 2rem;
  }

  .day-header {
    margin: 0 0 1rem 0;
    color: var(--text-secondary);
    font-size: 1.1rem;
    font-weight: 600;
  }

  .timeline-rail {
    position: relative;
    padding-left: 2rem;
  }

  .timeline-rail::before {
    content: '';
    position: absolute;
    left: 8px;
    top: 0;
    bottom: 0;
    width: 2px;
    background: var(--border);
  }

  .timeline-item {
    position: relative;
    margin-bottom: 1.5rem;
  }

  .timeline-marker {
    position: absolute;
    left: -1.5rem;
    top: 1rem;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--accent);
    border: 3px solid var(--bg-primary);
    z-index: 1;
  }

  .timeline-content {
    margin-left: 1rem;
  }

  .timeline-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .timeline-time {
    font-size: 0.85rem;
    color: var(--text-secondary);
    font-weight: 600;
  }

  .badge {
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    background: var(--accent);
    color: white;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .fronters {
    margin: 0.5rem 0;
    font-size: 1.1rem;
    color: var(--text-primary);
  }

  .notes {
    margin: 0.75rem 0;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .trigger {
    margin: 0.5rem 0 0 0;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .trigger-label {
    font-weight: 600;
    color: var(--text-primary);
  }

  .chart-placeholder {
    text-align: center;
    padding: 4rem 2rem;
    background: var(--bg-primary);
    border-radius: 8px;
    border: 2px dashed var(--border);
  }
</style>
