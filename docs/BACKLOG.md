# HeXuS Development Backlog
**Generated:** 2026-03-01  
**Source:** Comprehensive analysis of 7 research documents

---

## Table of Contents
1. [Core Features (MVP)](#1-core-features-mvp)
2. [Enhanced Features (v2)](#2-enhanced-features-v2)
3. [Nice-to-Have (Future)](#3-nice-to-have-future)
4. [Research/Validation Needed](#4-researchvalidation-needed)
5. [Legal/Compliance Requirements](#5-legalcompliance-requirements)
6. [Technical Infrastructure](#6-technical-infrastructure)
7. [Implementation Roadmap](#7-implementation-roadmap)

---

## 1. Core Features (MVP)

### 1.1 Biometric Data Collection
**Priority: HIGH | Complexity: MEDIUM-COMPLEX**

#### Heart Rate & HRV Monitoring
- [ ] **HealthKit integration (iOS)** - Swift bridge → uniffi → Rust core
  - Priority: HIGH | Complexity: MEDIUM
  - Extract HR mean, STD, HRV (SDNN, RMSSD)
  - Background sync with configurable intervals
  - Reference: cardio-rs crate for HRV calculation

- [ ] **Health Connect integration (Android)** - Kotlin bridge → uniffi → Rust core
  - Priority: HIGH | Complexity: MEDIUM
  - Match HealthKit feature parity
  - Handle permission flow gracefully

- [ ] **Galvanic Skin Response (GSR) tracking**
  - Priority: MEDIUM | Complexity: MEDIUM
  - Support wearables that provide EDA data
  - Extract mean, slope, peaks, rise time

- [ ] **Skin temperature monitoring**
  - Priority: MEDIUM | Complexity: SIMPLE
  - Simple, non-invasive measurement
  - Available from many wearables

- [ ] **Movement/activity pattern tracking**
  - Priority: MEDIUM | Complexity: SIMPLE
  - Accelerometer data for alter-specific signatures
  - Capture posture/gait differences

#### Optional Wearable APIs
- [ ] **Oura Ring API integration** - For existing Oura users
  - Priority: LOW | Complexity: MEDIUM
  - OAuth2 + REST (requires active subscription)
  - Nocturnal HRV, sleep stages, readiness scores
  
- [ ] **Whoop API integration** - For existing Whoop users
  - Priority: LOW | Complexity: MEDIUM
  - Recovery scores, strain, HRV (RMSSD)
  - Webhook support for real-time updates

- [ ] **CGM integration (Dexcom API)** - For diabetic systems
  - Priority: LOW | Complexity: COMPLEX
  - Requires HIPAA authorization process
  - Correlate glucose with switches (validation research)

### 1.2 Switch Tracking
**Priority: HIGH | Complexity: SIMPLE-MEDIUM**

- [ ] **Quick switch logging** - One-tap logging from anywhere
  - Priority: HIGH | Complexity: SIMPLE
  - Floating action button (FAB) on all screens
  - Lock screen widget for instant logging
  - Voice input: "Who's fronting?"

- [ ] **Automatic timestamp capture** - Default to current time
  - Priority: HIGH | Complexity: SIMPLE
  - User can edit if backdating

- [ ] **Co-fronting/co-conscious status** - Support multiple fronters
  - Priority: HIGH | Complexity: SIMPLE
  - Select multiple alters for co-fronting
  - Visual indicators for co-consciousness vs. full switch

- [ ] **Switch history timeline** - Visual graph of fronts over time
  - Priority: HIGH | Complexity: MEDIUM
  - Color-coded by alter
  - Tap segment to see details (notes, mood, triggers)
  - Similar to Simply Plural's graph view

- [ ] **Time gap detection** - Flag periods with no logged fronter
  - Priority: MEDIUM | Complexity: SIMPLE
  - Visual marker: "Lost time: 4 hours"
  - Prompt to fill in if known

### 1.3 Member Management
**Priority: HIGH | Complexity: SIMPLE-MEDIUM**

- [ ] **Alter profile creation** - Name, avatar, pronouns, description
  - Priority: HIGH | Complexity: SIMPLE
  - Custom fields (age, role, color, emoji)
  - Upload photos or choose from library

- [ ] **Per-alter settings** - Individual preferences
  - Priority: HIGH | Complexity: MEDIUM
  - Theme (light/dark/high contrast)
  - Font size, reduce motion, notification settings
  - Privacy controls (what's shareable)

- [ ] **Member groups** - Organize alters into categories
  - Priority: MEDIUM | Complexity: SIMPLE
  - Examples: "Protectors," "Littles," "Frequent fronters"
  - Filter/sort by group

- [ ] **Profile templates** - Quick setup for new alters
  - Priority: LOW | Complexity: SIMPLE
  - Clone settings from existing alter
  - Pre-filled templates for common archetypes

### 1.4 Internal Communication
**Priority: HIGH | Complexity: MEDIUM**

- [ ] **System-wide message board** - Leave notes for other alters
  - Priority: HIGH | Complexity: MEDIUM
  - Threaded conversations
  - Mark urgent/normal/casual (color-coded)
  - Notification for unread messages

- [ ] **"What did I miss?" landing page** - Context for new fronter
  - Priority: HIGH | Complexity: MEDIUM
  - Show: Previous fronter, their notes, unread messages
  - Timeline of recent switches with context

- [ ] **Voice notes between alters** - Audio messages
  - Priority: MEDIUM | Complexity: MEDIUM
  - Record/playback (especially for littles or non-verbal alters)
  - Attach to switches or send as standalone messages

### 1.5 Crisis Management
**Priority: HIGH | Complexity: MEDIUM**

- [ ] **Crisis mode activation** - SOS button on all screens
  - Priority: HIGH | Complexity: SIMPLE
  - Manual trigger: Persistent red button
  - Voice command: "Crisis mode"
  - Optional auto-detect (rapid app opens, keywords)

- [ ] **Crisis mode UI** - Ultra-simple, calming interface
  - Priority: HIGH | Complexity: MEDIUM
  - Full-screen, large buttons, pastel background
  - 4 functions: Grounding, Emergency Contacts, Safe Messages, Exit

- [ ] **Grounding tools built-in**
  - Priority: HIGH | Complexity: SIMPLE-MEDIUM
  - 5-4-3-2-1 sensory exercise (guided prompts)
  - Breathing exercises (visual guide, 4-7-8, box breathing)
  - Haptic feedback option (vibrate with breath rhythm)

- [ ] **Emergency contact list** - Pre-configured safety network
  - Priority: HIGH | Complexity: SIMPLE
  - Up to 5 contacts with one-tap call/text
  - Pre-written crisis messages ("I'm not okay, can you check in?")
  - Optional GPS location sharing

- [ ] **Crisis resource integration** - Built-in hotlines
  - Priority: HIGH | Complexity: SIMPLE
  - 988 (US), Crisis Text Line (HOME to 741741)
  - International hotlines (auto-detect locale)
  - One-tap dial or text from Crisis Mode

### 1.6 Privacy & Security
**Priority: HIGH | Complexity: MEDIUM-COMPLEX**

- [ ] **Local-first data storage** - SQLite on device
  - Priority: HIGH | Complexity: MEDIUM
  - All data stored locally by default
  - No cloud dependency for core functionality

- [ ] **Encryption at rest** - SQLCipher for database
  - Priority: HIGH | Complexity: MEDIUM
  - AES-256 encryption
  - User-set passphrase or biometric unlock

- [ ] **Optional cloud sync** - E2E encrypted
  - Priority: MEDIUM | Complexity: COMPLEX
  - Zero-knowledge architecture (can't read user data)
  - Cross-device sync for users who want it

- [ ] **Biometric consent flow (BIPA compliance)** - Before any collection
  - Priority: HIGH | Complexity: MEDIUM
  - Written informed consent with signature
  - Disclose: Purpose, duration, retention schedule
  - Allow withdrawal of consent

### 1.7 Cross-Platform Foundation
**Priority: HIGH | Complexity: COMPLEX**

- [ ] **Tauri 2.0 mobile apps** - iOS and Android
  - Priority: HIGH | Complexity: COMPLEX
  - Single codebase for both platforms
  - Native performance with web frontend

- [ ] **Desktop apps** - Linux, macOS, Windows
  - Priority: MEDIUM | Complexity: MEDIUM
  - Same Tauri codebase
  - Seamless sync with mobile

- [ ] **Offline-first architecture** - Works without internet
  - Priority: HIGH | Complexity: MEDIUM
  - All features functional offline
  - Background sync when connection available

### 1.8 Accessibility (MVP Tier)
**Priority: HIGH | Complexity: SIMPLE-MEDIUM**

- [ ] **Dark mode (true black + dark gray options)** - System-synced
  - Priority: HIGH | Complexity: SIMPLE
  - Respect OS dark mode setting
  - Per-alter override

- [ ] **Font size control** - Small to Extra Large
  - Priority: HIGH | Complexity: SIMPLE
  - Per-alter setting
  - Live preview

- [ ] **High contrast mode** - WCAG AAA (7:1 ratio)
  - Priority: MEDIUM | Complexity: SIMPLE
  - Maximum contrast colors
  - Thick borders on interactive elements

- [ ] **Reduce motion toggle** - Disable animations
  - Priority: MEDIUM | Complexity: SIMPLE
  - Static UI mode
  - Respect OS-level setting

- [ ] **Quick logging shortcuts** - Zero-friction capture
  - Priority: HIGH | Complexity: SIMPLE
  - Lock screen widget
  - Home screen widget
  - Voice input

---

## 2. Enhanced Features (v2)

### 2.1 Pattern Detection & Analytics
**Priority: HIGH | Complexity: COMPLEX**

- [ ] **CUSUM changepoint detection** - Real-time switch detection
  - Priority: HIGH | Complexity: MEDIUM
  - Detect mean/variance shifts in biometric streams
  - Alert user to potential switches for confirmation
  - Reference: Online statistical methods

- [ ] **PELT offline analysis** - Nightly auto-labeling
  - Priority: MEDIUM | Complexity: MEDIUM
  - Run on previous day's data
  - Generate candidate switch timestamps
  - User reviews next day

- [ ] **Semi-supervised learning** - Auto-label high-confidence samples
  - Priority: HIGH | Complexity: COMPLEX
  - Train SVM on small labeled set (50-100 examples)
  - Iteratively expand training data
  - Reference: Linfa for SVM

- [ ] **Active learning queries** - Smart labeling prompts
  - Priority: MEDIUM | Complexity: MEDIUM
  - Ask user to label ambiguous cases
  - Maximize information gain per label
  - 5-10 queries/day → 95% performance with 16% labels

- [ ] **Time series forecasting** - Switch prediction
  - Priority: MEDIUM | Complexity: COMPLEX
  - Use augurs or oxidiviner
  - "You might switch soon based on HRV patterns"
  - Confidence intervals

- [ ] **Trigger correlation analysis** - Pattern reports
  - Priority: HIGH | Complexity: MEDIUM
  - "You switch more on Tuesdays when sleep < 6 hours"
  - Overlay biometric data with logged triggers
  - Visual heatmaps (time of day, day of week)

- [ ] **Emotion recognition features** - From biometrics
  - Priority: MEDIUM | Complexity: COMPLEX
  - Pre-train on DREAMER/DEAP datasets
  - HR + EDA → valence/arousal
  - Different alters have distinct emotional baselines

### 2.2 Advanced Communication
**Priority: MEDIUM | Complexity: MEDIUM-COMPLEX**

- [ ] **Threaded conversations** - Organize message threads
  - Priority: MEDIUM | Complexity: MEDIUM
  - Reply to specific messages
  - Branch conversations by topic

- [ ] **Reactions and emoji support** - Quick responses
  - Priority: MEDIUM | Complexity: SIMPLE
  - Like/emoji react to messages
  - Reduce cognitive load vs. typing replies

- [ ] **Rich media support** - Images, audio, video
  - Priority: MEDIUM | Complexity: MEDIUM
  - Attach files to messages/logs
  - Voice notes, photos, sketches

- [ ] **Markdown formatting** - Rich text in messages
  - Priority: LOW | Complexity: SIMPLE
  - Bold, italic, lists, code blocks
  - Preview mode

- [ ] **Per-member voting system** - Collaborative decisions
  - Priority: MEDIUM | Complexity: SIMPLE
  - "Should we do X?" poll
  - Visual tally, consensus indicators
  - Helps avoid unilateral choices

### 2.3 Integrations
**Priority: HIGH | Complexity: MEDIUM-COMPLEX**

- [ ] **PluralKit API sync** - Import/export switches
  - Priority: HIGH | Complexity: MEDIUM
  - REST API for switch logs
  - Bi-directional sync (HeXuS ↔ PluralKit)
  - Map alters between systems

- [ ] **Simply Plural API sync** - Import/export fronts
  - Priority: HIGH | Complexity: MEDIUM
  - REST + WebSocket for real-time
  - Import existing front history
  - Merge member profiles

- [ ] **Discord bot integration** - Message proxying in Discord
  - Priority: MEDIUM | Complexity: COMPLEX
  - Native support for Discord presence
  - Alternative to PluralKit (reliability issues)
  - Server-specific avatars/tags

### 2.4 Multi-Modal Biometrics
**Priority: MEDIUM | Complexity: COMPLEX**

- [ ] **Keystroke dynamics** - Typing pattern analysis
  - Priority: MEDIUM | Complexity: MEDIUM
  - Features: Dwell time, flight time, digraphs
  - 78-84% user identification accuracy
  - Passive, continuous authentication

- [ ] **Voice-based identification** - Speaker recognition
  - Priority: LOW | Complexity: COMPLEX
  - MFCCs, ECAPA-TDNN embeddings
  - 88%+ accuracy with short clips
  - Privacy concern: Requires audio recording

- [ ] **Multi-modal fusion** - Combine all signals
  - Priority: HIGH | Complexity: COMPLEX
  - HR + EDA + keystroke + voice → higher accuracy
  - Significantly outperforms single signals
  - Reference: Research shows 95%+ with multi-modal

### 2.5 Enhanced Analytics UI
**Priority: MEDIUM | Complexity: MEDIUM**

- [ ] **Visual timeline with overlays** - Biometrics + switches
  - Priority: HIGH | Complexity: MEDIUM
  - Graph showing HR/HRV over time
  - Switch markers overlaid
  - "HRV dropped 30% before last switch"

- [ ] **Fronting statistics** - Who fronts when, for how long
  - Priority: MEDIUM | Complexity: SIMPLE
  - Per-alter fronting time (daily/weekly/monthly)
  - Time of day heatmap
  - Co-fronting frequency matrix

- [ ] **Trigger identification reports** - Environmental/physiological
  - Priority: MEDIUM | Complexity: MEDIUM
  - Correlate switches with: Stress, fatigue, hunger, triggers
  - Machine learning clustering of similar episodes
  - Export to therapist

- [ ] **Long-term trend visualization** - System evolution
  - Priority: LOW | Complexity: MEDIUM
  - Yearly/monthly front summaries
  - Member emergence tracking
  - Anniversary reminders

### 2.6 Support Network Tools
**Priority: MEDIUM | Complexity: MEDIUM**

- [ ] **Therapist view mode** - Read-only access with permissions
  - Priority: MEDIUM | Complexity: MEDIUM
  - Granular controls: What they see, when
  - Export summaries for sessions
  - HIPAA-aligned if needed

- [ ] **Partner/friend dashboard** - See front status without full access
  - Priority: MEDIUM | Complexity: SIMPLE
  - Share current fronter + optional notes
  - Permission levels: "Can see switches" vs. "Can see everything"

- [ ] **Shareable reports** - PDF/email summaries
  - Priority: MEDIUM | Complexity: SIMPLE
  - Generate for therapy sessions
  - Anonymize sensitive details
  - Biometric correlations included

### 2.7 System Discovery & Onboarding
**Priority: MEDIUM | Complexity: MEDIUM**

- [ ] **Guided onboarding flow** - For new systems
  - Priority: MEDIUM | Complexity: SIMPLE
  - Welcome tour (5 screens max)
  - Set up first alter
  - Explain core features

- [ ] **Member discovery prompts** - Help map system
  - Priority: MEDIUM | Complexity: SIMPLE
  - "Who handles work tasks?"
  - "Who fronts when stressed?"
  - Template profiles for common types

- [ ] **Visual system mapping** - Tree/graph view of relationships
  - Priority: LOW | Complexity: COMPLEX
  - Show alter connections (co-fronts with, protects, etc.)
  - Interactive graph (zoom, filter)

---

## 3. Nice-to-Have (Future)

### 3.1 Advanced Accessibility
**Priority: MEDIUM | Complexity: MEDIUM-COMPLEX**

- [ ] **Screen reader optimization** - NVDA, JAWS, VoiceOver
  - Priority: MEDIUM | Complexity: MEDIUM
  - ARIA labels for all elements
  - Semantic HTML in webviews
  - Test with blind/low-vision users

- [ ] **Keyboard navigation** - All functions accessible
  - Priority: MEDIUM | Complexity: SIMPLE
  - Tab order, shortcuts (Ctrl+S to save)
  - Focus indicators

- [ ] **Dyslexia-friendly fonts** - OpenDyslexic option
  - Priority: LOW | Complexity: SIMPLE
  - Font family selector per-alter
  - Increased letter spacing

- [ ] **Haptic feedback customization** - Per-action patterns
  - Priority: LOW | Complexity: SIMPLE
  - Intensity slider, pattern picker
  - Useful for sensory grounding

### 3.2 Rich Media Journaling
**Priority: LOW | Complexity: MEDIUM**

- [ ] **Photo galleries per alter** - Mood boards, appearance changes
  - Priority: LOW | Complexity: SIMPLE
  - Swipe gallery
  - Tag photos by mood, context

- [ ] **Audio recordings** - Co-conscious conversations
  - Priority: LOW | Complexity: MEDIUM
  - Record system discussions
  - Transcription (optional)

- [ ] **Video journals** - Express without typing
  - Priority: LOW | Complexity: MEDIUM
  - Embed in logs or messages
  - Privacy: Local storage only

- [ ] **Art/sketch integration** - Drawing canvas
  - Priority: LOW | Complexity: MEDIUM
  - Digital journaling for visual alters
  - Attach sketches to messages

### 3.3 Plugin Ecosystem
**Priority: LOW | Complexity: COMPLEX**

- [ ] **Plugin API** - Community extensions
  - Priority: LOW | Complexity: COMPLEX
  - Rust FFI or WASM plugins
  - Sandboxed for security

- [ ] **Custom field types** - Tags, relationships, attributes
  - Priority: LOW | Complexity: MEDIUM
  - User-defined fields for alters
  - Schema editor

- [ ] **Theme engine** - Beyond light/dark
  - Priority: LOW | Complexity: MEDIUM
  - Community-created themes
  - CSS customization

### 3.4 Advanced ML Features
**Priority: LOW | Complexity: COMPLEX**

- [ ] **Few-shot prototypical network** - 1-5 examples per alter
  - Priority: MEDIUM | Complexity: COMPLEX
  - Train in PyTorch, export ONNX
  - Load with Candle for inference
  - New alters require minimal examples

- [ ] **Hidden Markov Models** - State transitions
  - Priority: LOW | Complexity: COMPLEX
  - Model alters as hidden states
  - Predict switch sequences
  - "Alter A often precedes Alter B"

- [ ] **Self-supervised pretraining** - Leverage unlabeled data
  - Priority: LOW | Complexity: COMPLEX
  - Time-Frequency Consistency (TF-C)
  - Pre-train encoder on weeks of data
  - Fine-tune on small labeled set

- [ ] **On-device incremental learning** - Model updates locally
  - Priority: LOW | Complexity: COMPLEX
  - Use Burn for training
  - No cloud dependency
  - Privacy-preserving

### 3.5 Extended Wearable Support
**Priority: LOW | Complexity: MEDIUM**

- [ ] **Fitbit Web API** - Legacy support
  - Priority: LOW | Complexity: MEDIUM
  - Being phased out by Google
  - Only if significant user demand

- [ ] **Garmin Connect API** - For Garmin users
  - Priority: LOW | Complexity: MEDIUM
  - Requires business entity approval
  - Vitals, sleep, training data

- [ ] **FreeStyle Libre (via third-party)** - CGM for non-Dexcom users
  - Priority: LOW | Complexity: COMPLEX
  - Terra API or Thryve
  - Unofficial, reliability concerns

### 3.6 Specialized Tools
**Priority: LOW | Complexity: MEDIUM**

- [ ] **Safe word/phrase prompts** - For co-consciousness
  - Priority: LOW | Complexity: SIMPLE
  - Interrupt conflicts
  - Pre-set system agreements

- [ ] **Internal meeting scheduler** - Virtual headspace
  - Priority: LOW | Complexity: MEDIUM
  - Leave notes, vote, discuss
  - Async collaboration tool

- [ ] **Memory logs for amnesia** - Photo/GPS context
  - Priority: LOW | Complexity: MEDIUM
  - "Where was I? What did I do?"
  - Auto-capture location, photos (with consent)

---

## 4. Research/Validation Needed

### 4.1 Scientific Validation
**Priority: HIGH | Complexity: N/A (Research)**

- [ ] **Validate switch detection with wearables** - Can sensors detect switches?
  - Study: Correlate sensor data with logged switches (n=20-50 systems)
  - Determine sensitivity/specificity
  - Identify optimal parameter combinations (HR + EDA + movement?)

- [ ] **Test patterns across diverse DID/OSDD population**
  - Do patterns generalize across systems?
  - Validate for: DID, OSDD-1a, OSDD-1b, polyfragmented systems
  - Account for different system structures (2 vs. 50+ alters)

- [ ] **Longitudinal stability testing** - Do patterns remain stable?
  - Track systems for months to years
  - Account for: Integration, splits, system changes, life events
  - Ensure algorithms adapt

- [ ] **Blood glucose in diabetic DID systems** - Fill research gap
  - Partner with diabetic systems using CGMs
  - Correlate CGM data with switch logs
  - Validate/refute clinical anecdotes

### 4.2 User Research
**Priority: HIGH | Complexity: N/A (Research)**

- [ ] **User interviews** - 10-15 systems (DID, OSDD-1a, OSDD-1b)
  - Validate research findings
  - Identify unmet needs
  - Prioritize feature requests

- [ ] **Usability testing** - Prototype with neurodivergent testers
  - Test with: ADHD, autism, PTSD, anxiety comorbidities
  - Shorter sessions, pre-briefings
  - Iterate on friction points

- [ ] **Beta testing group** - Recruit r/DID, r/OSDD, r/plural
  - Open call for testers
  - Diverse system sizes, tech literacy levels
  - Monthly feedback sessions

### 4.3 Ethical & Clinical Input
**Priority: HIGH | Complexity: N/A (Consultation)**

- [ ] **Therapist consultation** - Trauma specialists
  - Ethical guardrails review
  - Clinical utility assessment
  - Therapist dashboard feature design

- [ ] **Ethics advisory board** - Independent experts
  - Review privacy architecture
  - Algorithmic bias assessment
  - Ongoing consent protocols

- [ ] **Security audit** - Third-party penetration testing
  - Before public launch
  - Annual thereafter
  - Bug bounty program (post-launch)

### 4.4 Technical Validation
**Priority: MEDIUM | Complexity: N/A (Testing)**

- [ ] **Bias testing** - Algorithmic fairness across demographics
  - Test accuracy by: Race, gender, age, disability
  - Disaggregated metrics
  - Independent audit

- [ ] **Battery life testing** - Continuous monitoring impact
  - Optimize sync intervals
  - Batch processing vs. real-time
  - User-configurable frequency

- [ ] **Offline performance testing** - Edge cases
  - Extended offline periods (days/weeks)
  - Sync conflict resolution
  - Data integrity under failures

---

## 5. Legal/Compliance Requirements

### 5.1 BIPA Compliance (Illinois Biometric Privacy Act)
**Priority: CRITICAL | Complexity: MEDIUM**

- [ ] **Written informed consent before collection** - Electronic signature
  - Must obtain BEFORE any biometric data collection
  - Disclose: Purpose, duration, retention schedule
  - Checkbox with intent to sign

- [ ] **Public policy notice** - Published before collection
  - Available to users
  - Plain language

- [ ] **Retention schedule + auto-delete** - 3 years max or when purpose fulfilled
  - Implement auto-deletion
  - User can delete anytime

- [ ] **No profit from biometric data** - Cannot sell/lease/trade
  - No third-party sharing without consent
  - Document all data flows

- [ ] **Secure handling policy** - Written documentation
  - Same standard as other confidential data
  - Encryption, access controls, audit logs

**Penalties:** $1,000 negligent / $5,000 intentional per violation + private right of action

### 5.2 GDPR Compliance (If Serving EU Users)
**Priority: HIGH | Complexity: COMPLEX**

- [ ] **Data Protection Impact Assessment (DPIA)** - Mandatory for special category data
  - Assess re-identification risks
  - Mitigation strategies
  - Necessity and proportionality

- [ ] **Explicit consent for mental health data** - Article 9 exception
  - Clear affirmative action
  - Granular (separate consent per purpose)
  - Withdrawable with easy mechanism
  - Documented

- [ ] **User rights implementation**
  - Access: Export all data held
  - Rectification: Correct inaccurate data
  - Erasure: Complete deletion on request
  - Restriction: Limit processing
  - Data portability: Machine-readable format (JSON, CSV)
  - Object: Opt-out of marketing

- [ ] **Breach notification within 72 hours** - To supervisory authority
  - Incident response plan
  - User notification process

**Penalties:** Up to €20M or 4% global revenue

### 5.3 State Privacy Laws (U.S.)
**Priority: HIGH | Complexity: MEDIUM**

- [ ] **California CCPA/CPRA compliance**
  - Biometric data = "sensitive personal information"
  - Opt-out for sharing/selling
  - Notice, data minimization
  - Private right of action for breaches

- [ ] **Montana mental health platform rules** - Effective Oct 2025
  - HIPAA-like standards for non-HIPAA apps
  - $5,000+ penalties
  - Model for future state laws

- [ ] **Texas CUBI + TRAIGA** - Biometric consent
  - Informed consent for commercial use
  - Destroy within 1 year after purpose expires
  - No consent implied from public images

- [ ] **Washington biometric law** - Notice and consent
  - Mechanism to block commercial use
  - Attorney General enforcement

### 5.4 FDA Medical Device Classification
**Priority: MEDIUM | Complexity: COMPLEX (if applicable)**

- [ ] **Document intended use** - Maintain records
  - Position as "general wellness" (not medical device)
  - Avoid diagnostic/therapeutic claims
  - Clear disclaimers: "Not medical advice"

- [ ] **Medical disclaimers in UI** - Required by app stores + ethical
  - "This app is not a substitute for professional medical advice"
  - Remind users to consult doctor
  - Crisis resources prominently displayed

- [ ] **IF evolving to therapeutic: FDA clearance pathway**
  - Budget $100K-$1M+, 6-18 months
  - Clinical validation required
  - Human factors testing
  - Post-market surveillance

### 5.5 App Store Policies
**Priority: HIGH | Complexity: SIMPLE**

- [ ] **No diagnostic claims without FDA clearance** - Apple/Google requirement
  - Strict scrutiny for health apps
  - No sensor-only diagnostics (BP, glucose from phone)

- [ ] **Accuracy disclaimers** - For biometric measurements
  - Disclose methodology
  - Limitations of measurements

- [ ] **Privacy policy in plain language** - 8th-grade reading level
  - Published before data collection
  - Clear data practices

- [ ] **Age-appropriate design** - COPPA compliance for under 13
  - Parental controls if applicable
  - Proper age ratings

### 5.6 Privacy & Security Standards
**Priority: CRITICAL | Complexity: MEDIUM-COMPLEX**

- [ ] **Encryption at rest and in transit** - AES-256 or equivalent
  - SQLCipher for database
  - HTTPS/TLS 1.3 for network

- [ ] **Role-based access control** - Multi-factor authentication
  - Biometric unlock option
  - Per-alter data access controls

- [ ] **Incident response plan** - Data breach procedures
  - User notification <24 hours
  - Forensics, remediation
  - Public disclosure if required

- [ ] **Annual security audits** - Independent third-party
  - Penetration testing
  - Vulnerability scanning
  - Compliance verification

---

## 6. Technical Infrastructure

### 6.1 Core Framework
**Priority: HIGH | Complexity: COMPLEX**

- [ ] **Tauri 2.0 setup** - Cross-platform foundation
  - Initialize project structure
  - Configure for iOS, Android, desktop (Linux/Mac/Win)
  - Set up build pipelines

- [ ] **Rust backend architecture**
  - Module structure: biometric collection, feature extraction, pattern detection, database, sync
  - Error handling (anyhow, thiserror)
  - Logging (tracing, log)

- [ ] **Frontend framework** - Svelte or React
  - Component library setup
  - TypeScript for type safety
  - State management (Svelte stores or Redux)

### 6.2 Mobile Native Bridges
**Priority: HIGH | Complexity: MEDIUM-COMPLEX**

- [ ] **uniffi setup** - FFI for iOS/Android
  - Rust → Swift (HealthKit)
  - Rust → Kotlin (Health Connect)
  - Type-safe bindings generation

- [ ] **HealthKit bridge (iOS)** - Swift → uniffi → Rust
  - Request permissions (HR, HRV, sleep, activity)
  - Background fetch
  - Data format conversion (OMH/FHIR)

- [ ] **Health Connect bridge (Android)** - Kotlin → uniffi → Rust
  - Match HealthKit feature parity
  - Handle fragmentation (Android 14+)
  - Fallback to Google Fit for older devices

### 6.3 Data Storage
**Priority: HIGH | Complexity: MEDIUM**

- [ ] **SQLite local database** - Primary storage
  - Schema design: alters, switches, biometrics, messages, settings
  - Migration system (sqlx migrations)
  - Indexes for performance

- [ ] **TimescaleDB consideration** - For self-hosted backend
  - PostgreSQL + time series extension
  - Hypertables for biometric data
  - Continuous aggregates for analytics

- [ ] **Optional: InfluxDB** - Purpose-built TSDB
  - High-performance time series
  - Downsampling, retention policies
  - Trade-off: Less SQL-like than TimescaleDB

### 6.4 Biometric Processing
**Priority: HIGH | Complexity: MEDIUM-COMPLEX**

- [ ] **cardio-rs integration** - HRV calculation
  - Time-domain (SDNN, RMSSD)
  - Frequency-domain (LF, HF, LF/HF ratio)
  - Input: RR intervals

- [ ] **Feature extraction pipeline**
  - HR: Mean, STD, min, max
  - EDA: Mean, slope, peaks, rise time
  - Movement: Accelerometer stats
  - Keystroke: Dwell time, flight time, digraphs

- [ ] **Data normalization** - Per-user baselines
  - Z-score normalization
  - Rolling window statistics
  - Handle individual variability

### 6.5 Pattern Detection
**Priority: HIGH | Complexity: COMPLEX**

- [ ] **CUSUM implementation** - Real-time changepoint detection
  - Rolling mean/variance
  - Threshold tuning per user
  - 50-100 lines Rust

- [ ] **PELT integration** - Offline analysis
  - Python ruptures library via FFI
  - Or port to Rust
  - Batch processing overnight

- [ ] **SVM baseline (linfa)** - Traditional ML
  - Train on labeled switches
  - Semi-supervised self-training
  - Active learning queries

- [ ] **Neural network (Candle)** - Advanced ML
  - Load ONNX models from PyTorch
  - Few-shot prototypical network
  - On-device inference

### 6.6 Sync & API
**Priority: MEDIUM | Complexity: MEDIUM-COMPLEX**

- [ ] **REST API for backend** - Optional cloud sync
  - Endpoints: Auth, sync, export, therapist access
  - Rate limiting, auth tokens
  - HTTPS only

- [ ] **E2E encryption for sync** - Zero-knowledge architecture
  - Encrypt data before upload
  - Server cannot decrypt
  - Key derivation from user passphrase

- [ ] **PluralKit API client** - REST integration
  - OAuth2 flow
  - CRUD operations on switches
  - Error handling (rate limits, downtime)

- [ ] **Simply Plural API client** - REST + WebSocket
  - Real-time updates via WebSocket
  - Offline sync queue

### 6.7 UI/UX Components
**Priority: HIGH | Complexity: MEDIUM**

- [ ] **Chart library** - Timeline visualizations
  - Chart.js or Recharts
  - Line charts (HR/HRV over time)
  - Heatmaps (fronting patterns)

- [ ] **Notification system** - Push notifications
  - Platform-specific: iOS (APNs), Android (FCM)
  - Tauri notification plugin
  - Granular controls per-alter

- [ ] **Theme system** - Light/Dark/High Contrast
  - CSS variables for colors
  - Per-alter persistence
  - Live preview

### 6.8 Testing Infrastructure
**Priority: MEDIUM | Complexity: MEDIUM**

- [ ] **Unit tests** - Rust (cargo test)
  - Test critical logic: HRV calc, changepoint detection, encryption
  - Code coverage >80%

- [ ] **Integration tests** - End-to-end flows
  - Log switch → store → retrieve
  - Sync → encrypt → upload → download → decrypt

- [ ] **UI tests** - Frontend (Playwright, Cypress)
  - Critical paths: Login, log switch, view timeline
  - Accessibility checks (aXe)

- [ ] **Performance tests** - Load testing
  - 1000+ switches, 50+ alters
  - Sync with 1MB+ biometric data
  - Battery impact benchmarks

### 6.9 CI/CD Pipeline
**Priority: MEDIUM | Complexity: MEDIUM**

- [ ] **GitHub Actions** - Automated builds
  - Rust tests, linting (clippy)
  - Frontend tests (Vitest, Jest)
  - Multi-platform builds (iOS, Android, desktop)

- [ ] **Release automation** - App store submissions
  - Code signing (iOS certificates, Android keystore)
  - Fastlane for deployment
  - Changelog generation

---

## 7. Implementation Roadmap

### Phase 1: Foundation (Months 1-2)
**Goal:** Functional MVP with manual switch tracking + basic biometrics

#### Week 1-2: Project Setup
- [ ] Initialize Tauri 2.0 project
- [ ] Set up Rust backend structure
- [ ] Configure SQLite database
- [ ] Implement basic alter profile CRUD

#### Week 3-4: Biometric Collection
- [ ] HealthKit bridge (iOS) - HR, HRV
- [ ] Health Connect bridge (Android) - HR, HRV
- [ ] Store biometric samples locally
- [ ] Basic cardio-rs HRV calculation

#### Week 5-6: Switch Tracking
- [ ] Quick logging UI (one-tap)
- [ ] Timeline view (color-coded by alter)
- [ ] Time gap detection
- [ ] Internal message board

#### Week 7-8: Privacy & Polish
- [ ] BIPA consent flow
- [ ] Encryption at rest (SQLCipher)
- [ ] Dark mode
- [ ] Accessibility audit (basic)

**Deliverable:** MVP app for internal testing

---

### Phase 2: Pattern Detection (Months 3-4)
**Goal:** Semi-automated switch detection, basic analytics

#### Week 9-10: Changepoint Detection
- [ ] CUSUM implementation
- [ ] Notification for potential switches
- [ ] User labeling interface

#### Week 11-12: ML Baseline
- [ ] Feature extraction (HR, HRV, EDA)
- [ ] SVM training with linfa
- [ ] Semi-supervised self-training loop

#### Week 13-14: Analytics UI
- [ ] Visual timeline with biometric overlays
- [ ] Fronting statistics dashboard
- [ ] Trigger correlation reports

#### Week 15-16: Testing & Iteration
- [ ] Beta testing with 5-10 systems
- [ ] Bug fixes
- [ ] UX improvements based on feedback

**Deliverable:** Private beta with pattern detection

---

### Phase 3: Integrations & Launch Prep (Months 5-6)
**Goal:** Cross-platform sync, external integrations, public launch

#### Week 17-18: PluralKit/Simply Plural
- [ ] PluralKit API integration
- [ ] Simply Plural API integration
- [ ] Bi-directional sync

#### Week 19-20: Cloud Sync (Optional)
- [ ] E2E encrypted backend
- [ ] Cross-device sync
- [ ] Conflict resolution

#### Week 21-22: Compliance & Security
- [ ] Legal review (BIPA, GDPR)
- [ ] Security audit (penetration test)
- [ ] Privacy policy finalization

#### Week 23-24: App Store Submission
- [ ] iOS App Store submission
- [ ] Google Play submission
- [ ] Desktop distribution (GitHub, website)

**Deliverable:** Public v1.0 release

---

### Phase 4: Enhanced Features (Months 7-9)
**Goal:** Advanced ML, multi-modal biometrics, support network tools

#### Week 25-28: Neural Networks
- [ ] Few-shot prototypical network (PyTorch)
- [ ] ONNX export
- [ ] Candle integration for inference

#### Week 29-32: Multi-Modal
- [ ] Keystroke dynamics
- [ ] Voice analysis (optional)
- [ ] Multi-modal fusion

#### Week 33-36: Support Tools
- [ ] Therapist view mode
- [ ] Partner dashboard
- [ ] Shareable reports (PDF)

**Deliverable:** v2.0 release

---

### Phase 5: Long-Term (Months 10+)
**Goal:** Community growth, research collaboration, advanced features

- [ ] Plugin ecosystem
- [ ] Rich media journaling
- [ ] System discovery tools
- [ ] Research partnerships (validate switch detection)
- [ ] International expansion (localization, GDPR full compliance)

---

## Summary Statistics

**Total Items:** 200+

### By Category
- **Core Features (MVP):** 53 items
- **Enhanced Features (v2):** 45 items
- **Nice-to-Have (Future):** 31 items
- **Research/Validation:** 13 items
- **Legal/Compliance:** 28 items
- **Technical Infrastructure:** 37 items

### By Priority
- **CRITICAL:** 8 items (BIPA, GDPR, encryption, security)
- **HIGH:** 89 items (MVP features, core biometrics, switch tracking)
- **MEDIUM:** 76 items (enhanced features, integrations, analytics)
- **LOW:** 34 items (nice-to-have, future expansions)

### By Complexity
- **SIMPLE:** 48 items (UI tweaks, basic features)
- **MEDIUM:** 79 items (integrations, APIs, ML baselines)
- **COMPLEX:** 42 items (neural networks, multi-modal, advanced ML)

### Timeline Estimate
- **MVP (Phase 1):** 2 months
- **Pattern Detection (Phase 2):** 2 months
- **Launch Prep (Phase 3):** 2 months
- **Enhanced Features (Phase 4):** 3 months
- **Total to v2.0:** 9 months (with 1-2 developers)

---

## Next Actions

1. **Review with Brandon** - Prioritize based on goals
2. **Create GitHub project board** - Track progress
3. **Set up development environment** - Tauri, Rust, uniffi
4. **Begin Phase 1: Week 1-2** - Project setup
5. **Recruit beta testers** - Post in r/DID, r/OSDD, r/plural

---

**Document Status:** COMPLETE  
**Generated By:** Silas Vane (Sub-agent)  
**Date:** 2026-03-01  
**Next Review:** Start of Phase 1

*This backlog is living documentation. Update as priorities shift and new insights emerge.*
