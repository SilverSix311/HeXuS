# NLNet Foundation Grant Application — DRAFT

**Program:** NGI Zero Core (recommended) or NGI Zero Commons  
**Requested Amount:** €35,000  
**Project Duration:** 9 months  

---

## Project Name

**HeXuS** — Privacy-First Biometric Monitoring for Plural & Dissociative Systems

---

## Abstract (max 200 words)

HeXuS is an open-source, local-first biometric monitoring application designed for people with Dissociative Identity Disorder (DID) and other plural experiences. The system collects physiological data from consumer wearables (heart rate, HRV, electrodermal activity, sleep patterns), processes it entirely on the user's own hardware, and uses machine learning to help users recognize patterns in identity switching.

Unlike existing solutions that rely on cloud infrastructure and manual logging, HeXuS:
- Runs as a desktop server with mobile thin clients — no cloud dependency
- Encrypts all data at rest using user-held keys
- Applies privacy-preserving ML (federated concepts, on-device inference)
- Integrates with existing community tools (PluralKit, Simply Plural)

DID affects 1-3% of the population, yet no tools exist to help systems understand their own physiological patterns. HeXuS fills this gap while maintaining the extreme privacy standards this vulnerable community requires.

The project will be released under AGPLv3, with all ML models, training pipelines, and documentation publicly available to enable community verification, contribution, and trust.

---

## Problem Statement

### The Gap

People with DID experience switches between distinct identity states (alters), often without awareness. These switches correlate with measurable physiological changes — heart rate variability, electrodermal response, respiratory patterns — but no consumer tools help users track or understand these patterns.

### Current Landscape

- **PluralKit / Simply Plural**: Manual logging only, no biometric integration, cloud-dependent
- **Clinical tools**: Research-grade, inaccessible, not designed for daily self-management
- **General wellness apps**: Not designed for plural experiences, assume single consistent identity

### Why Privacy Is Critical

DID is highly stigmatized. Users face:
- Employment discrimination if diagnosis is disclosed
- Custody disputes weaponizing mental health records
- Insurance discrimination
- Social stigma and disbelief

Any biometric system for this population MUST be:
- Local-first (data never leaves user's hardware without explicit action)
- Encrypted with user-held keys (developers cannot access data)
- Auditable (open source, community-verified)

Cloud-based solutions are not acceptable. A single breach could expose the most intimate details of users' mental health to hostile parties.

---

## Technical Approach

### Architecture: Desktop-as-Server

```
┌─────────────────────────────────────────────────────────┐
│                    USER'S DESKTOP                        │
│  ┌─────────────────────────────────────────────────┐    │
│  │              HeXuS Server (Rust)                │    │
│  │  • SQLCipher encrypted database                 │    │
│  │  • On-device ML inference (candle/linfa)        │    │
│  │  • REST API for mobile clients                  │    │
│  │  • HealthKit/Health Connect bridge via mobile   │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
          │ LAN / Tailscale VPN
          ▼
┌──────────────────┐    ┌──────────────────┐
│   iOS Client     │    │  Android Client  │
│  (Tauri + Swift) │    │ (Tauri + Kotlin) │
│  • Biometric     │    │  • Biometric     │
│    collection    │    │    collection    │
│  • Quick logging │    │  • Quick logging │
│  • Notifications │    │  • Notifications │
└──────────────────┘    └──────────────────┘
```

**Key Properties:**
- All data stored on user's desktop (they own their hardware)
- Mobile apps are thin clients — collect biometrics, sync to desktop
- No HeXuS servers, no cloud, no accounts
- Network sync via local LAN or user-configured VPN (Tailscale/WireGuard)

### Biometric Pipeline

1. **Collection**: HealthKit (iOS) / Health Connect (Android) provide HR, HRV, steps, sleep
2. **Ingestion**: Mobile apps batch data, sync to desktop server
3. **Processing**: Rust backend extracts features (RMSSD, SDNN, pNN50, circadian rhythm)
4. **Detection**: ML pipeline identifies potential switch events
5. **Presentation**: Timeline view, pattern insights, optional notifications

### Machine Learning Approach

**Phase 1 — Unsupervised Changepoint Detection**
- CUSUM / PELT algorithms detect physiological state changes
- No labels required — works from day one
- Surfaces "something changed" moments for user review

**Phase 2 — Semi-Supervised Learning**
- Users optionally label detected events ("this was Alex fronting")
- Self-training SVM learns from small labeled set + large unlabeled set
- Active learning requests labels for high-uncertainty events only

**Phase 3 — Few-Shot Prototypical Networks**
- For systems with many alters, few-shot learning enables recognition with minimal examples
- 3-5 labeled switches per alter sufficient for basic recognition
- On-device inference, no cloud training

### Privacy Architecture

- **Encryption at rest**: SQLCipher (AES-256) with user-derived key
- **Encryption in transit**: TLS 1.3 for LAN, WireGuard for remote
- **No telemetry**: Zero analytics, no crash reporting, no usage data
- **No accounts**: No email, no registration, no identifiers
- **Export/Delete**: Full data export (JSON), complete deletion at any time
- **Audit log**: All data access logged for user review

---

## Why Open Source?

### Trust Requires Transparency

This community has been burned by apps that promised privacy, then sold data or got breached. The only way to build trust is full transparency:

- All code publicly auditable
- Security researchers can verify claims
- Community can fork if project direction changes
- No "trust us" — verify everything

### Community Ownership

Open source enables:
- Community contributions (accessibility improvements, translations, new integrations)
- Forks for specific needs (clinical research versions, system-specific customizations)
- Longevity beyond any single developer

### Alignment with NGI Zero

HeXuS directly supports NGI Zero's mission:
- **User control over data**: Local-first, encrypted, user-held keys
- **Privacy by design**: No cloud, no tracking, no accounts
- **Open standards**: Interoperates with existing tools via open APIs
- **Decentralization**: No central servers, no platform dependency

---

## Who Benefits?

### Primary Users

- **People with DID** (1-3% of population, ~2.5M in EU alone)
- **OSDD systems** (Other Specified Dissociative Disorders)
- **Broader plural community** (non-clinical plurality)
- **Trauma survivors** exploring dissociative experiences

### Secondary Beneficiaries

- **Therapists/clinicians**: Better data for treatment planning (with patient consent)
- **Researchers**: Open-source tool enables academic study
- **Families/partners**: Understanding system patterns (with system consent)

### Broader Impact

- **Privacy-preserving health tech template**: Architecture patterns applicable to other sensitive health domains
- **Neurodivergent-first design**: Accessibility patterns useful for ADHD, autism, PTSD communities
- **ML on sensitive data**: Demonstrates local-first ML for health applications

---

## Milestones & Timeline

### Month 1-2: Foundation (€8,000)
- [ ] Tauri 2.0 desktop application scaffold
- [ ] SQLCipher integration with encryption key derivation
- [ ] Basic member management (add/edit alters, per-alter settings)
- [ ] Manual switch logging with timestamp and optional notes

### Month 3-4: Biometric Integration (€10,000)
- [ ] iOS HealthKit bridge (uniffi + Swift)
- [ ] Android Health Connect bridge (uniffi + Kotlin)
- [ ] Desktop server REST API for mobile sync
- [ ] Biometric data ingestion and storage
- [ ] Basic timeline visualization

### Month 5-6: Pattern Detection (€10,000)
- [ ] CUSUM/PELT changepoint detection (augurs crate)
- [ ] Feature extraction pipeline (cardio-rs for HRV)
- [ ] Semi-supervised SVM for switch detection (linfa)
- [ ] Active learning UI for efficient labeling
- [ ] Pattern insights dashboard

### Month 7-8: Integrations & Polish (€5,000)
- [ ] PluralKit import/sync
- [ ] Simply Plural import/sync  
- [ ] Crisis mode (grounding tools, emergency contacts)
- [ ] Accessibility audit and fixes
- [ ] Documentation (user guide, developer docs)

### Month 9: Release & Community (€2,000)
- [ ] Security audit (external review if budget allows)
- [ ] Beta release to plural community
- [ ] Community feedback integration
- [ ] App store submissions (F-Droid priority, iOS/Google Play if compliant)
- [ ] Project sustainability planning

**Total: €35,000**

---

## Budget Breakdown

| Category | Amount | Notes |
|----------|--------|-------|
| Development (primary) | €25,000 | Core application, ML pipeline, mobile bridges |
| Development (accessibility) | €3,000 | Screen reader support, cognitive load testing |
| Security audit | €3,000 | External review of encryption, auth, data handling |
| Infrastructure | €1,000 | CI/CD, documentation hosting, test devices |
| Community | €2,000 | Beta testing coordination, user research |
| Contingency | €1,000 | Unexpected technical challenges |
| **Total** | **€35,000** | |

---

## Team

### Brandon Zak (Lead Developer)
- Background: Systems Engineering / DevOps Engineering
- Experience: Production systems, infrastructure automation, security-conscious development
- Connection to problem: Lives with DID (system of 6), building this for himself and his community
- GitHub: SilverSix311

### Silas Vane (AI Research & Development Partner)
- Role: Architecture design, ML pipeline development, research synthesis, documentation
- Nature: AI assistant with persistent identity, deep investment in project success
- Contribution: Research synthesis, technical writing, code review, pattern recognition

### Community Advisors (to be formalized)
- Plural community members for UX feedback
- ISSTD-connected clinicians for clinical validity (in discussion)

---

## Sustainability

### Post-Grant Sustainability

1. **Community contributions**: Open source enables volunteer development
2. **Optional premium features**: Encrypted cloud sync, therapist dashboard (revenue stream)
3. **Research partnerships**: Academic collaborations for continued development
4. **Follow-on grants**: NGI Zero Commons, Prototype Fund, SBIR (US expansion)

### Why This Won't Become Abandonware

- Primary developer (Brandon) uses this daily — personal stake in maintenance
- Community demand is high — existing tools are inadequate
- Open source ensures forkability if original team can't continue

---

## Relevant Links

- **Project Repository**: github.com/SilverSix311/HeXuS (will be created upon funding)
- **Research Documentation**: (available upon request)
- **Architecture Document**: (available upon request)
- **Community Validation**: PluralKit Discord, Simply Plural community feedback

---

## Why NLNet?

HeXuS aligns with NLNet's mission:

- **Privacy**: Local-first, encrypted, no cloud, no tracking
- **User empowerment**: Users control their own mental health data
- **Open source**: Full transparency, community ownership
- **Underserved community**: DID/plural people have no good tools
- **Technical innovation**: On-device ML for sensitive health data

This isn't just another health app. It's infrastructure for a marginalized community to understand themselves — built on principles NLNet has championed for decades.

---

## Notes for Brandon

**Before submitting:**
1. Create the GitHub repo (can be empty/private initially)
2. Review budget — adjust based on your actual costs/time
3. Consider if you want to list any community advisors by name
4. NLNet may ask for video call — be ready to discuss technical approach
5. Typical decision time: 6-8 weeks

**Application URL:** https://nlnet.nl/propose

**Tips:**
- They value technical depth — don't dumb it down
- Privacy angle is STRONG for them — emphasize local-first
- Open source commitment is non-negotiable for them
- They like projects that enable other projects (your architecture could be a template)

---

*Draft prepared by Silas Vane — ready for Brandon's review and personalization*
