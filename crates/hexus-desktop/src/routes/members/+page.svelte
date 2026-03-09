<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface Alter {
    id: string;
    name: string;
    pronouns: string | null;
    color: string | null;
    description: string | null;
    avatar_url: string | null;
    created_at: string;
  }

  let alters = $state<Alter[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      alters = await invoke<Alter[]>('get_alters');
      loading = false;
    } catch (e) {
      error = String(e);
      loading = false;
    }
  });

  function getInitials(name: string): string {
    return name
      .split(' ')
      .map(n => n[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
  }
</script>

<div class="members-page">
  <header class="page-header">
    <div>
      <h1>System Members</h1>
      <p class="text-muted">Manage alter profiles and view statistics</p>
    </div>
    <button class="btn">Add Member</button>
  </header>

  {#if loading}
    <div class="text-center mt-4">
      <div class="spinner"></div>
      <p class="text-muted mt-2">Loading members...</p>
    </div>
  {:else if error}
    <div class="card mt-3">
      <p style="color: #ff6b6b;">Error: {error}</p>
    </div>
  {:else if alters.length === 0}
    <div class="card mt-3 text-center">
      <h3>No Members Yet</h3>
      <p class="text-muted">Add your first system member to get started</p>
      <button class="btn mt-3">Add Member</button>
    </div>
  {:else}
    <div class="grid grid-2 mt-3">
      {#each alters as alter (alter.id)}
        <div class="card member-card">
          <div class="member-header">
            <div 
              class="avatar" 
              style="background-color: {alter.color || '#4ecdc4'}"
            >
              {#if alter.avatar_url}
                <img src={alter.avatar_url} alt={alter.name} />
              {:else}
                <span>{getInitials(alter.name)}</span>
              {/if}
            </div>
            <div class="member-info">
              <h3>{alter.name}</h3>
              {#if alter.pronouns}
                <p class="pronouns">{alter.pronouns}</p>
              {/if}
            </div>
          </div>

          {#if alter.description}
            <p class="description">{alter.description}</p>
          {/if}

          <div class="member-actions">
            <button class="btn btn-secondary">View Profile</button>
            <button class="btn btn-secondary">Edit</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .members-page {
    max-width: 1200px;
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

  .member-card {
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .member-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px var(--shadow);
  }

  .member-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .avatar {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    font-weight: bold;
    color: white;
    overflow: hidden;
    flex-shrink: 0;
  }

  .avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .member-info h3 {
    margin: 0;
    font-size: 1.5rem;
  }

  .pronouns {
    margin: 0.25rem 0 0 0;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .description {
    color: var(--text-secondary);
    line-height: 1.6;
    margin: 1rem 0;
  }

  .member-actions {
    display: flex;
    gap: 0.75rem;
    margin-top: 1rem;
  }

  .member-actions button {
    flex: 1;
  }
</style>
