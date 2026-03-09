<script lang="ts">
  import { onMount } from 'svelte';
  import '../app.css';

  let darkMode = $state(true);

  onMount(() => {
    // Check for saved dark mode preference
    const saved = localStorage.getItem('darkMode');
    darkMode = saved !== null ? saved === 'true' : true;
    applyTheme();
  });

  function applyTheme() {
    if (darkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
    localStorage.setItem('darkMode', darkMode.toString());
  }

  function toggleDarkMode() {
    darkMode = !darkMode;
    applyTheme();
  }
</script>

<div class="app-container">
  <nav class="sidebar">
    <div class="logo">
      <h1>HeXuS</h1>
      <p class="tagline">Biometric Monitor</p>
    </div>

    <ul class="nav-links">
      <li>
        <a href="/" class:active={window.location.pathname === '/'}>
          <span class="icon">📊</span>
          Dashboard
        </a>
      </li>
      <li>
        <a href="/members" class:active={window.location.pathname === '/members'}>
          <span class="icon">👥</span>
          Members
        </a>
      </li>
      <li>
        <a href="/timeline" class:active={window.location.pathname === '/timeline'}>
          <span class="icon">📈</span>
          Timeline
        </a>
      </li>
      <li>
        <a href="/settings" class:active={window.location.pathname === '/settings'}>
          <span class="icon">⚙️</span>
          Settings
        </a>
      </li>
    </ul>

    <button class="theme-toggle" onclick={toggleDarkMode}>
      {darkMode ? '🌙' : '☀️'}
    </button>
  </nav>

  <main class="content">
    <slot />
  </main>
</div>

<style>
  :global(:root) {
    /* Light mode colors */
    --bg-primary: #ffffff;
    --bg-secondary: #f5f5f5;
    --bg-tertiary: #e0e0e0;
    --text-primary: #1a1a1a;
    --text-secondary: #666666;
    --border: #d0d0d0;
    --accent: #4ecdc4;
    --accent-hover: #3db8b0;
    --shadow: rgba(0, 0, 0, 0.1);
  }

  :global(.dark) {
    /* Dark mode colors */
    --bg-primary: #1a1a1a;
    --bg-secondary: #2d2d2d;
    --bg-tertiary: #3d3d3d;
    --text-primary: #f0f0f0;
    --text-secondary: #a0a0a0;
    --border: #404040;
    --accent: #4ecdc4;
    --accent-hover: #5fd9d1;
    --shadow: rgba(0, 0, 0, 0.3);
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
      Ubuntu, Cantarell, sans-serif;
    background: var(--bg-primary);
    color: var(--text-primary);
    transition: background-color 0.2s, color 0.2s;
  }

  .app-container {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  .sidebar {
    width: 250px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 1.5rem;
    box-sizing: border-box;
  }

  .logo {
    margin-bottom: 2rem;
  }

  .logo h1 {
    margin: 0;
    font-size: 2rem;
    background: linear-gradient(135deg, var(--accent), #ff6b6b);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .tagline {
    margin: 0.25rem 0 0 0;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .nav-links {
    list-style: none;
    padding: 0;
    margin: 0;
    flex: 1;
  }

  .nav-links li {
    margin-bottom: 0.5rem;
  }

  .nav-links a {
    display: flex;
    align-items: center;
    padding: 0.75rem 1rem;
    border-radius: 8px;
    text-decoration: none;
    color: var(--text-primary);
    transition: background 0.2s;
  }

  .nav-links a:hover {
    background: var(--bg-tertiary);
  }

  .nav-links a.active {
    background: var(--accent);
    color: white;
  }

  .icon {
    margin-right: 0.75rem;
    font-size: 1.25rem;
  }

  .theme-toggle {
    padding: 0.75rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-primary);
    cursor: pointer;
    font-size: 1.25rem;
    transition: background 0.2s, transform 0.1s;
  }

  .theme-toggle:hover {
    background: var(--bg-tertiary);
  }

  .theme-toggle:active {
    transform: scale(0.95);
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 2rem;
    box-sizing: border-box;
  }
</style>
