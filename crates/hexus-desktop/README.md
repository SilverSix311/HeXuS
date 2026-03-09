# HeXuS Desktop

Privacy-first biometric monitoring system for plural systems (DID/OSDD).

## Tech Stack

- **Framework:** Tauri 2.0
- **Frontend:** SvelteKit 2.x + Svelte 5
- **Build Tool:** Vite 6
- **Language:** TypeScript

## Features

- 📊 **Dashboard** - System status and recent switches
- 👥 **Members** - Alter profile management
- 📈 **Timeline** - Switch history with biometric data visualization
- ⚙️ **Settings** - Privacy controls and device configuration
- 🌙 **Dark Mode** - System-wide theming with CSS variables
- 🔒 **Privacy-First** - All data stored locally, no cloud sync

## Development

### Prerequisites

- Node.js 18+ (v22 recommended)
- Rust and Cargo (for Tauri)
- Tauri CLI

### Setup

```bash
# Install dependencies (set NODE_ENV=development to install devDependencies)
NODE_ENV=development npm install

# Sync SvelteKit types
npx svelte-kit sync

# Run dev server (frontend only)
npm run dev

# Run Tauri app (with hot reload)
npm run tauri dev
```

### Dev Server

Frontend dev server runs on `http://localhost:5173` with hot module replacement.

## Project Structure

```
hexus-desktop/
├── src/                      # Svelte frontend
│   ├── routes/
│   │   ├── +layout.svelte   # Main layout with sidebar
│   │   ├── +page.svelte     # Dashboard
│   │   ├── members/         # Members page
│   │   ├── timeline/        # Timeline page
│   │   └── settings/        # Settings page
│   ├── app.css              # Global styles
│   └── app.html             # HTML template
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── main.rs          # Tauri entry point
│   │   └── lib.rs           # Core logic
│   └── tauri.conf.json      # Tauri configuration
├── static/                  # Static assets
└── package.json             # NPM dependencies
```

## Design System

### Color Variables

CSS variables defined in `src/routes/+layout.svelte`:

- `--bg-primary` - Main background
- `--bg-secondary` - Card background
- `--bg-tertiary` - Hover states
- `--text-primary` - Main text
- `--text-secondary` - Muted text
- `--border` - Border colors
- `--accent` - Primary accent (#4ecdc4)
- `--accent-hover` - Accent hover state
- `--shadow` - Box shadows

Dark mode is automatically applied via `.dark` class on `<html>`.

## Backend Integration

The frontend currently uses mock Tauri API calls that will be replaced with real Rust backend functions:

- `get_status()` - System status
- `get_switches()` - Switch history
- `get_alters()` - Alter profiles

These are defined in `src-tauri/src/main.rs` and exposed via `#[tauri::command]`.

## Privacy Guarantees

- ✅ All data stored locally in SQLite
- ✅ No network requests
- ✅ No telemetry
- ✅ Optional encryption at rest
- ✅ No third-party dependencies for core functionality

## License

MIT
