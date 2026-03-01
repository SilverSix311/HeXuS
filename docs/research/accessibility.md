# Accessibility Considerations for HeXuS

**Last Updated:** 2026-03-01  
**Status:** Research Complete

## Executive Summary

DID/plural systems often experience comorbidities including ADHD, autism, PTSD, anxiety, and depression. HeXuS must accommodate users facing **multiple, intersecting challenges** simultaneously. This document provides concrete UI/UX recommendations based on research into neurodivergent-friendly design, crisis management apps, and plural system tracking tools.

**Core Principle:** Flexibility first. What helps one alter or on one day may overwhelm another alter or on a different day. Every feature should be **customizable per alter**.

---

## 1. Cognitive Accessibility

### 1.1 Executive Function Challenges (ADHD)

**Problem:** Task initiation difficulties, forgetfulness, time blindness, overwhelm from multi-step processes.

**Concrete UI/UX Solutions:**

#### Quick Logging (Zero-Friction Capture)
- **One-tap logging from anywhere:**
  - Persistent floating action button (FAB) accessible from all screens
  - Lock screen widget for instant logging without unlocking
  - Home screen widget showing "Quick Switch" button
  
- **Voice input for everything:**
  - Optional speech-to-text for all text fields
  - "Say who's fronting" voice command
  - Voice notes attached to switch logs
  
- **Gesture-based shortcuts:**
  - Swipe patterns for common actions (e.g., swipe right = log current time, swipe left = add note)
  - Shake phone to open quick-add dialog
  - Double-tap notification to log switch

#### Task Breakdown and Defaults
- **Smart defaults to reduce decisions:**
  - Auto-fill timestamp to current time
  - Pre-populate "last known fronter" when logging a switch
  - Suggest recent co-fronters based on patterns
  
- **Progressive disclosure:**
  - Basic log: "Who + when" (2 fields)
  - Expandable "More details" section for mood, trigger, notes
  - Advanced fields hidden by default, revealed on demand

#### Time Management Supports
- **Visual time indicators:**
  - Progress bars showing "time since last switch" in hours/days
  - Color-coded timeline view (today = bright, yesterday = muted, last week = pale)
  - "You've been fronting for 3 hours" gentle reminders

- **Pomodoro-style focus mode:**
  - Optional "Focus hours" when notifications are silenced
  - Customizable quiet periods per alter (e.g., "No notifications 10pm-8am")

### 1.2 Memory Gaps (Amnesia Between Switches)

**Problem:** Alters may have zero memory of what happened while another was fronting. App must function as "external memory."

**Concrete UI/UX Solutions:**

#### Context Reconstruction
- **"What did I miss?" landing page:**
  - When app opens, immediately show: Who was fronting before you, what they logged, any notes left for you
  - Timeline with visual markers for gaps ("Lost time: 4 hours")
  - Highlight unread internal messages

- **Fronting history visualization:**
  - Graph view showing switches over time (like Simply Plural)
  - Color-coded by alter for pattern recognition
  - Tap any segment to see notes/context from that period

#### Internal Communication
- **Message board for alters:**
  - Leave notes for "next fronter" or specific alters
  - Mark messages urgent/normal/casual (color-coded)
  - Notification when you have unread messages
  
- **Voice messages between alters:**
  - Record audio notes ("Hey, I started this task, here's where I left off")
  - Reduces reading load compared to text

#### Decision Support
- **Shared context displays:**
  - "Current tasks in progress" widget
  - "Appointments today" reminder banner
  - "Last meal eaten: X hours ago" health tracking

- **Per-alter voting system:**
  - For system decisions (e.g., "Should we do X?")
  - Visual tally of votes, consensus indicators
  - Helps avoid one alter making unilateral choices

### 1.3 Overwhelm and Overstimulation

**Problem:** Too much visual/auditory stimuli, complex interfaces, or information density triggers shutdowns.

**Concrete UI/UX Solutions:**

#### Visual Minimalism
- **One task per screen philosophy:**
  - Avoid dashboard overload — each view has ONE purpose
  - Main screen shows only: Current fronter, time fronting, quick-add button
  - Everything else behind clearly labeled tabs/menus

- **Whitespace and breathing room:**
  - Generous padding (minimum 16px between interactive elements)
  - Limit to 3-5 items visible without scrolling
  - No infinite scrolling — paginate with clear "Load more" buttons

#### Sensory Control
- **Reduce motion options:**
  - Toggle to disable all animations (system-wide)
  - Static UI mode (no transitions, instant state changes)
  - Respect OS-level "reduce motion" settings

- **Simplified navigation:**
  - Bottom navigation bar (max 5 items: Home, Log, Chat, Alters, Settings)
  - Breadcrumb trails for nested screens
  - Escape hatch: Always visible "Back to Home" button

#### Cognitive Load Management
- **Auto-save everywhere:**
  - No "Are you sure?" dialogs unless destructive (delete data)
  - Draft logs saved automatically every 30 seconds
  - Return to exact state when re-opening app

- **Smart notifications (see Section 2.2):**
  - Batch similar notifications (e.g., "3 new messages" not 3 separate pings)
  - Configurable quiet hours
  - Opt-in for all notifications (default = silent)

---

## 2. UI/UX Considerations

### 2.1 Simple vs. Feature-Rich Tradeoffs

**Guiding Philosophy:** Core features must be dead simple. Advanced features must be **progressively revealed** and **per-alter configurable**.

**Concrete Implementation:**

#### Tiered Complexity Levels
- **Level 1: Essential Mode (default for new alters)**
  - Log switch (who + when)
  - View today's switches
  - Read internal messages
  - Emergency grounding button
  
- **Level 2: Standard Mode**
  - Add mood, triggers, notes to logs
  - View history graphs (past week)
  - Create new alter profiles
  - Export data
  
- **Level 3: Power User Mode**
  - Custom tags/categories
  - Analytics (fronting patterns, trigger analysis)
  - API access for integrations
  - Advanced privacy controls

**Per-alter setting:** "Interface complexity: Essential / Standard / Power User"

#### Feature Discovery Without Overwhelm
- **Onboarding for each alter:**
  - First time an alter uses app: "Hi [name], let's set up your preferences" (5 screens max)
  - Option to copy settings from another alter or start fresh
  
- **Contextual tips (dismissible):**
  - First time viewing timeline: "Tap any segment to see details" (show once)
  - First time adding note: "You can use voice input" tooltip
  - Never show same tip twice per alter

### 2.2 Notification Fatigue

**Problem:** Excessive notifications worsen anxiety, distract ADHD users, and can trigger distressed alters.

**Concrete UI/UX Solutions:**

#### Granular Notification Controls (Per-Alter!)
- **Notification categories with individual toggles:**
  - Switch events ("X is now fronting")
  - Internal messages ("You have 2 unread notes")
  - Time-based reminders ("You've been fronting for 6 hours")
  - Health prompts ("Water reminder")
  - Emergency alerts (crisis protocol activated)

- **Per-alter notification profiles:**
  - Alter A: All notifications enabled, sound + vibration
  - Alter B: Visual only, no sound (sensory sensitive)
  - Alter C: Emergency only (easily overwhelmed)
  - Alter D: All disabled (needs silence to function)

#### Intelligent Notification Batching
- **Delay and bundle:**
  - Wait 2 minutes after first message, then send "3 new messages" instead of 3 pings
  - Daily digest option: One notification at chosen time with summary
  
- **Adaptive frequency:**
  - If user dismisses 3 notifications in a row without opening app: Reduce frequency automatically
  - Ask: "You're dismissing notifications a lot. Want to adjust settings?"

#### Quiet Hours & Focus Modes
- **Scheduled silence:**
  - Set quiet hours per day (e.g., 10pm-8am, customizable per alter)
  - "Focus mode" toggle: Mute all non-emergency notifications for X hours
  
- **Context-aware silence:**
  - Detect phone's "Do Not Disturb" mode and respect it
  - Option to sync with calendar events (mute during meetings)

#### Notification Content Design
- **Calm, supportive language:**
  - ✅ "Hey, you've been fronting for a while. Want to log a check-in?"
  - ❌ "FRONTING TIME EXCEEDED! LOG NOW!"
  
- **Non-intrusive visuals:**
  - No bright red alert colors (use soft blue/green)
  - Gentle vibration patterns (short single buzz, not alarming triple-buzz)
  - Optional silent notification (badge icon only)

### 2.3 Quick Logging (Minimal Friction)

**Already covered in Section 1.1 — see "Quick Logging (Zero-Friction Capture)"**

Additional considerations:

#### Pre-filled Templates
- **Common switch scenarios:**
  - "Waking up" template (auto-fills morning timestamp, suggests fronter from yesterday evening)
  - "Emergency switch" template (flags as urgent, prompts for trigger/context)
  - "Routine switch" template (minimal fields, assumes no issue)

- **User-created templates:**
  - Alter can save their own log patterns
  - Example: "Work mode" template for alter who handles job tasks

#### Offline-First Architecture
- **Never block on network:**
  - All logging happens locally first
  - Sync in background when connection available
  - Visual indicator for pending syncs ("3 logs waiting to sync")

### 2.4 Customization Per Alter

**Problem:** Each alter is a distinct person with unique needs, preferences, and sensitivities.

**Concrete UI/UX Solutions:**

#### Per-Alter Settings (Comprehensive)
Every alter gets their own profile with:

**Visual Preferences:**
- Theme: Light / Dark / High Contrast / Custom colors
- Font size: Small / Medium / Large / Extra Large
- Font family: System default / Dyslexia-friendly / Custom
- Accent color (used for their logs/messages)

**Interface Preferences:**
- Complexity level: Essential / Standard / Power User
- Reduce motion: On / Off
- Sound effects: On / Off
- Haptic feedback: On / Off / Custom intensity

**Notification Preferences:**
- (See Section 2.2 — per-alter notification profiles)

**Functional Preferences:**
- Default log privacy: Private / Shared with system / Shareable externally
- Require confirmation before: Deleting logs / Changing fronter / Messaging others
- Auto-lock timeout: Immediate / 5 min / 15 min / Never

#### Visual Identity Per Alter
- **Avatar/icon for each alter:**
  - Used in timeline, message threads, logs
  - Quick visual recognition ("Oh, Blue was fronting then")
  
- **Color-coding throughout app:**
  - Alter A's logs = purple accent, Alter B's = green, etc.
  - Timeline segments colored by fronter
  - Messages in chat use alter's chosen color

#### Profile Sharing & Copying
- **"Clone settings from another alter":**
  - New alter can inherit settings from similar alter
  - Reduces setup friction
  
- **"Use system defaults":**
  - Shared baseline settings for all alters
  - Individual alters can override specific settings

---

## 3. Crisis Considerations

### 3.1 Distressed Alter Fronts

**Problem:** An alter in crisis may be disoriented, overwhelmed, or unable to navigate complex interfaces.

**Concrete UI/UX Solutions:**

#### Crisis Mode Activation
- **Automatic detection (opt-in):**
  - Rapid app opens/closes (checking repeatedly)
  - Repeated use of grounding features
  - Message keywords: "help," "scared," "can't breathe"
  - Prompt: "You seem distressed. Switch to Crisis Mode?"

- **Manual trigger:**
  - Persistent "SOS" button on all screens (red circle in corner)
  - Shake phone 3 times to activate (configurable)
  - Voice command: "Crisis mode"

#### Crisis Mode Interface
When activated, app transforms to ultra-simple, high-contrast, calming design:

**Layout:**
- Full-screen, single-focus view
- Large, widely-spaced buttons (easy to tap when shaking/dissociating)
- Pastel background (soft blue/green, never red)
- No complex navigation — everything on one screen

**Core Functions (4 buttons max):**
1. **Grounding Exercise** (launches breathing tool or 5-4-3-2-1 technique)
2. **Emergency Contacts** (one-tap call/text to pre-set people)
3. **Safe Messages** (pre-written messages: "I'm struggling," "Need space," "Not okay")
4. **Exit Crisis Mode** (return to normal interface)

**Additional Crisis Features:**
- **Disable destructive actions:** Can't delete logs, change settings, or access complex features in crisis mode
- **Auto-exit after calming:** Optional timer (e.g., "Crisis mode for 30 minutes, then ask if I want to continue")
- **Crisis log:** Automatically creates timestamped crisis event log (for therapy/pattern tracking)

### 3.2 Grounding Features

**Concrete UI/UX Solutions:**

#### Built-In Grounding Tools
- **5-4-3-2-1 Sensory Exercise:**
  - Guided prompts: "Name 5 things you can see…"
  - Text input or voice input for responses
  - Gentle timer (30 seconds per sense)
  - Calming background colors/animations (optional)

- **Breathing Exercises:**
  - Visual breathing guide (expanding/contracting circle)
  - 4-7-8 technique (breathe in 4, hold 7, out 8)
  - Box breathing for PTSD (4-4-4-4)
  - Haptic feedback option (phone vibrates with breath rhythm)

- **Safe Space Visualization:**
  - Pre-loaded calming images (nature, abstract patterns)
  - User can upload personal safe space photos
  - Full-screen immersive view, gentle music option

#### Quick Access to Grounding
- **Grounding button on main screen:**
  - Always visible (not buried in menus)
  - Opens picker: "What would help right now?" (breathing, sensory, distraction)
  
- **Widget for phone home screen:**
  - "Panic Button" widget → instant grounding exercise
  - No need to unlock phone or navigate app

### 3.3 Emergency Contacts

**Concrete UI/UX Solutions:**

#### Pre-Configured Safety Network
- **Emergency contact list (per-alter or shared):**
  - Up to 5 trusted contacts (like MY3 app)
  - For each: Name, relationship, phone, preferred contact method (call/text/messaging app)
  - Optional notes: "Call mom for medical stuff, text friend for emotional support"

- **One-tap communication:**
  - Crisis Mode: Tap "Alert [Name]" → sends pre-written message + optional GPS location
  - Example message: "Hey, I'm not okay right now. Can you check in? - [System name]"
  - Option to send to one person or broadcast to all emergency contacts

#### Crisis Messaging Templates
- **Pre-written messages (editable):**
  - "I'm struggling and need support"
  - "Having a panic attack, need grounding"
  - "Switched unexpectedly, disoriented"
  - "Need someone to talk to"
  
- **Include context automatically:**
  - Optional: Attach current fronter name
  - Optional: GPS location ("I'm at [location]")
  - Optional: Recent log entries (for therapist/support person who has context)

#### Crisis Resources
- **Built-in crisis hotlines:**
  - National: 988 (US suicide/crisis line), Crisis Text Line (text HOME to 741741)
  - International: List of hotlines by country (auto-detect locale)
  - One-tap dial or text from Crisis Mode

- **Therapist Quick Contact:**
  - Dedicated "My Therapist" contact slot
  - Different from emergency contacts (for scheduled vs. urgent needs)
  - Leave voice message or send secure message if therapist uses patient portal

---

## 4. Sensory Needs

### 4.1 Color and Contrast Options

**Concrete UI/UX Solutions:**

#### Theme Options (Per-Alter)
- **Light Mode:**
  - White/cream background, dark gray text
  - Pastel accent colors
  - WCAG AAA contrast (7:1 minimum)

- **Dark Mode:**
  - True black (#000000) or dark gray (#121212) background
  - White or light gray text
  - Muted accent colors (avoid neon/bright colors)
  - High contrast ratio (7:1 minimum)

- **High Contrast Mode:**
  - Maximum contrast colors (black on white or white on black)
  - Thick borders around interactive elements
  - No subtle shading/gradients
  - Ideal for low vision or sensory processing differences

- **Custom Color Mode:**
  - User picks: Background, text, accent, and border colors
  - Live preview while customizing
  - Presets: "Calming blues," "Warm earth tones," "Monochrome"

#### Accessibility Testing
- **Contrast checker built-in:**
  - Warns if custom colors don't meet WCAG standards
  - Suggests adjustments ("Make text darker for better readability")

- **Simulator mode:**
  - Preview interface as seen by colorblind users (deuteranopia, protanopia, tritanopia)
  - Ensures color-coding has non-color redundancy (icons, labels, patterns)

### 4.2 Sound and Vibration Preferences

**Concrete UI/UX Solutions:**

#### Granular Audio Controls (Per-Alter)
- **Sound effects:**
  - Toggle for UI sounds (button clicks, switch logging, etc.)
  - Volume slider (independent of system volume)
  - Sound profile picker: Silent / Minimal / Full
  
- **Notification sounds:**
  - Choose sound per notification type (message, switch, reminder)
  - Upload custom sounds
  - Use system default or app-provided options (gentle chimes, nature sounds)

- **Voice feedback:**
  - Optional text-to-speech for confirmations ("Switch logged")
  - Useful for visually impaired users or those who need auditory reinforcement

#### Haptic Feedback Options
- **Vibration patterns:**
  - Toggle for haptic feedback (on/off)
  - Intensity slider: Gentle / Medium / Strong
  - Pattern customization: Single buzz / Double buzz / Pattern

- **Per-action haptics:**
  - Switch logged: Short buzz
  - Message received: Gentle double-tap
  - Crisis mode activated: Distinct pattern (triple buzz)

#### Sensory Overload Protection
- **Auto-mute after threshold:**
  - If user dismisses 5 notifications in 10 minutes: "Want to mute sounds temporarily?"
  - Adaptive learning: Reduce sound frequency if user consistently disables it

### 4.3 Dark Mode Necessity

**Why It's Non-Negotiable:**
- **ADHD:** Reduces visual distractions, helps focus
- **Autism:** Minimizes sensory overload from bright screens
- **PTSD/Anxiety:** Calming, less jarring
- **Migraines:** Prevents light-triggered headaches (common comorbidity)
- **Nighttime use:** Many systems track at night (before bed, after waking) — bright screens disrupt sleep

**Concrete Implementation:**

#### Automatic Dark Mode
- **System-level sync:**
  - Respect OS dark mode setting by default
  - Option to override (always light, always dark, auto)

- **Time-based switching:**
  - Auto-enable dark mode at sunset (use device location)
  - Custom schedule: "Dark mode 8pm-8am"

#### True Black vs. Dark Gray
- **OLED optimization:**
  - True black (#000000) for OLED screens (saves battery, deeper contrast)
  - Dark gray (#121212) option for LCD screens (reduces halation)
  - User can choose preference per-alter

#### Dimming Options
- **In-app brightness control:**
  - Slider to dim below system brightness (for ultra-sensitive users)
  - "Night mode" extra dim setting
  - Warn: "Very dim — may be hard to read"

---

## 5. Implementation Priorities

### Phase 1: Minimum Viable Accessibility (MVP)
**Must-haves for initial release:**
1. Quick logging (one-tap switch logging, voice input)
2. Dark mode (system-synced, true black option)
3. Per-alter notification settings (toggle categories on/off)
4. Crisis mode (SOS button → grounding + emergency contacts)
5. Basic customization (theme, font size, accent color per alter)
6. Timeline view with fronting history (color-coded by alter)

### Phase 2: Enhanced Cognitive Support
1. Auto-save everywhere
2. Progressive disclosure (complexity levels)
3. Notification batching
4. Offline-first architecture
5. "What did I miss?" landing page
6. Internal message board

### Phase 3: Advanced Accessibility
1. Full per-alter customization (all settings)
2. Custom color themes
3. Reduce motion mode
4. Screen reader optimization
5. Grounding exercise library
6. Analytics for pattern tracking (optional, power user mode)

---

## 6. Testing and Validation

### User Testing with Plural Systems
- **Recruit diverse testers:**
  - Systems with ADHD, autism, PTSD, anxiety (multiple comorbidities)
  - Varying system sizes (2 alters vs. 20+)
  - Different tech literacy levels

- **Accessibility audit:**
  - Screen reader compatibility (NVDA, JAWS, VoiceOver)
  - Keyboard navigation (all functions accessible without mouse/touch)
  - Color contrast automated testing (Lighthouse, aXe)

### Feedback Loops
- **In-app feedback button:**
  - "Report accessibility issue" → direct to dev team
  - Attach screenshots, logs (with user consent)
  
- **Community advisory board:**
  - 5-10 plural system members who test pre-release features
  - Monthly feedback sessions

---

## 7. Resources and References

### Existing Apps Analyzed
- **Simply Plural:** Front tracking, internal chat, member profiles (gold standard for plural systems)
- **30/30:** Gesture-based task management (ADHD-friendly)
- **Tiimo:** Visual planner for ADHD/autism (award-winning accessibility)
- **Virtual Hope Box:** Crisis grounding tools (PTSD/anxiety)
- **Suicide Safety Plan:** Emergency protocols, one-tap access

### Design Standards
- **WCAG 2.1 AAA:** Color contrast, keyboard navigation, screen reader support
- **Material Design Accessibility:** Google's guidelines for Android
- **Apple Human Interface Guidelines:** iOS accessibility best practices

### Research Sources
- W3C: "How People with Disabilities Use the Web"
- CHADD: "Apps for Executive Function Challenges"
- UX Design for Neurodiversity (DevQube, UX Magazine)

---

## 8. Open Questions for Brandon

1. **Crisis mode scope:** Should it include "safe mode" that locks out certain alters from posting externally or making changes?
2. **Privacy model:** Who owns the data? System as whole, or each alter individually? (Affects deletion, export, sharing permissions)
3. **Integration with therapy:** Build-in export for therapists (PDF reports, anonymized data)? Or keep app separate from clinical tools?
4. **Child alters:** Special protections needed? (Content filtering, simplified interface, restricted communication?)
5. **System size limits:** How to handle very large systems (50+ alters) without overwhelming UI? Favorites/pinning? Search functionality?

---

**End of Document**

*This is a living document. Update as user research and testing reveal new needs.*
