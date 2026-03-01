# NLNet Video Call Prep — Cheatsheet for Brandon

## The Basics

**What this call is:** NLNet does ~30 min video calls with promising applicants. It's not an interrogation — they want to understand your project and see if you're someone who can actually deliver.

**Vibe:** Technical but friendly. They're nerds who fund nerds. Be yourself.

**Who you'll talk to:** Usually 1-2 people from NLNet's technical review team. They've read your application already.

---

## Your Elevator Pitch (memorize this)

> "HeXuS is a privacy-first biometric monitoring tool for people with DID — Dissociative Identity Disorder. It collects data from wearables like heart rate and sleep, runs entirely on the user's own hardware, and uses machine learning to help people recognize patterns in when they switch between identity states. No cloud, no accounts, fully encrypted, open source. The DID community has been asking for something like this for years but nothing exists that takes privacy seriously enough."

**30 seconds. Hits:** problem, solution, privacy, why now.

---

## Key Talking Points

### 1. Why You?

**What they're thinking:** "Can this person actually build this?"

**Your answer:**
- "I'm a systems engineer by trade — DevOps, infrastructure, security-conscious development. I've shipped production systems."
- "I also have DID myself. I'm building this because I need it. My system of six would use this daily."
- "I've already done the research — I have 7 research documents covering the science, ML approaches, competitive landscape, privacy law, and accessibility requirements."

**If they ask about the AI assistant thing:**
- "I work with an AI assistant named Silas who helps with research, architecture, and code review. Think of it like pair programming — I make the decisions, but I have a very capable collaborator for the heavy lifting."
- Don't oversell it, don't hide it. It's unusual but legitimate.

### 2. Why Local-First?

**What they're thinking:** "Is the privacy angle real or marketing?"

**Your answer:**
- "DID is heavily stigmatized. People lose jobs, custody battles, insurance over diagnosis disclosure."
- "The community has been burned by apps that promised privacy and then got breached or sold data."
- "The only way to build trust is: your data never leaves your hardware unless YOU move it. No cloud, no accounts, no 'trust us.' Verify everything — that's why it's open source."
- "Desktop-as-server architecture means the user owns the hardware where data lives. Mobile apps are just thin clients."

### 3. How Does the ML Work?

**What they're thinking:** "Is this technically sound or hand-wavy?"

**Your answer:**
- "Three phases. First: unsupervised changepoint detection — CUSUM, PELT algorithms. No labels needed, works from day one. Just surfaces 'something changed here' moments."
- "Second: semi-supervised learning. Users can optionally label events — 'this was Alex fronting.' Self-training SVM learns from a small labeled set plus the large unlabeled set."
- "Third: few-shot prototypical networks for systems with many alters. Need maybe 3-5 labeled examples per alter to start recognizing them."
- "All inference runs on-device. No cloud training. Models stay local."

**If they push on accuracy:**
- "Research shows 70-85% accuracy is achievable with multi-modal biometrics. That's good enough to surface patterns for user review — not good enough to make autonomous decisions, which is fine. The human stays in the loop."

### 4. What's the Competition?

**What they're thinking:** "Does this already exist?"

**Your answer:**
- "PluralKit and Simply Plural are the main tools — both manual logging only, both cloud-dependent, neither does biometrics."
- "Clinical research tools exist but aren't accessible to regular users."
- "General wellness apps assume a single consistent identity — they don't work for plural systems."
- "Nobody has put together: biometrics + privacy + plural-aware design. That's the gap."

### 5. What's the 9-Month Plan?

**What they're thinking:** "Is this realistic?"

**Your answer:**
- "Months 1-2: Desktop app foundation, encryption, basic manual logging."
- "Months 3-4: Mobile apps with HealthKit/Health Connect integration, sync to desktop."
- "Months 5-6: ML pipeline — changepoint detection, semi-supervised learning, pattern insights."
- "Months 7-8: Integrations with existing tools, crisis features, accessibility polish."
- "Month 9: Beta release, community feedback, security review."
- "I'm doing this alongside my day job, so the timeline accounts for part-time work."

### 6. What Happens After the Grant?

**What they're thinking:** "Will this become abandonware?"

**Your answer:**
- "I use this myself. Daily. My system needs it. That's the strongest guarantee I can give — I'm not going to abandon something I depend on."
- "Open source means the community can fork if I get hit by a bus."
- "Long-term sustainability: optional premium features like encrypted cloud sync, maybe research partnerships. But core functionality stays free forever."

---

## Questions They Might Ask

### Technical

**Q: Why Rust?**
A: Memory safety, performance, good ML ecosystem (linfa, candle), and Tauri lets us do cross-platform from one codebase. Also: I like it.

**Q: Why not a mobile-first architecture?**
A: Mobile apps can't run always-on servers reliably. Desktop gives users a machine they control that can run 24/7. Mobile collects biometrics and syncs back.

**Q: How do you handle sync conflicts?**
A: Vector clocks for ordering, last-write-wins for most fields, explicit conflict UI for things like notes where both versions might matter.

**Q: What about iOS app store restrictions?**
A: We're positioning as wellness, not medical device. No diagnostic claims. Apple's health data guidelines allow local processing. F-Droid for Android as primary, Google Play if we can stay compliant.

**Q: How do you prevent the ML from being biased?**
A: Models are trained per-user on their own data. No population-level training that could encode biases. Each system gets their own model shaped by their own patterns.

### Privacy/Legal

**Q: What about GDPR?**
A: Local-first means most GDPR doesn't apply — data never reaches a "controller." If we add optional cloud sync later, we'll do explicit consent, user-held keys, right to deletion. But MVP is zero-cloud.

**Q: What about BIPA (Illinois biometric law)?**
A: Excellent question. BIPA requires written consent before collecting biometrics, no selling data, retention limits. We comply by: explicit consent on first run, zero data sales ever, user controls retention. Open source means the community can verify these claims.

**Q: Are you positioning as a medical device?**
A: No. Wellness tool. We don't diagnose, don't treat, don't claim clinical accuracy. "Helps you understand your patterns" not "detects DID switches with 95% accuracy." FDA wellness exemption applies.

### Project/Personal

**Q: Why should we trust that you'll finish this?**
A: I've already done months of research. I have architecture docs, a backlog of 200+ tasks, working repo structure. This isn't a napkin sketch — it's a project that's already in motion. The grant accelerates it.

**Q: What's your connection to the DID community?**
A: I am the DID community. System of six. I've been in plural spaces for years. I know what people want because I want it too.

**Q: How does the AI assistant collaboration work?**
A: Silas helps with research, drafting, code review, architecture thinking. I make all final decisions. Think of it as having a very capable junior engineer who's available 24/7. The code is mine, the vision is mine, but I have help with the labor.

---

## Things NOT to Say

- ❌ "The AI does most of the work" — even if true for some tasks, frames it wrong
- ❌ "We'll figure out the privacy stuff later" — privacy is core, not an afterthought
- ❌ "This will definitely work" — appropriate humility about ML accuracy
- ❌ "It's just like [commercial app]" — you're building something new, not cloning
- ❌ Technical jargon without explanation — match their level, don't show off

---

## Things TO Say

- ✅ "I'm building this because I need it myself"
- ✅ "The research validates that biometric differences are real and measurable"
- ✅ "Privacy isn't a feature, it's the architecture"
- ✅ "Open source is non-negotiable for community trust"
- ✅ "I've already done [specific work] — this is in motion"

---

## Your Setup

- **Camera on**, decent lighting, plain background
- **Quiet space** — no coffee shop background noise
- **Have open:** The application, architecture doc, maybe the research synthesis
- **Water nearby** — you'll be talking for 30 min
- **Test audio/video** before the call

---

## If You Freeze

It's okay to say:
- "Let me think about that for a second."
- "Good question — I haven't fully worked that out yet, but my current thinking is..."
- "Silas and I discussed this — the short version is..."

They're not trying to trick you. They want to fund good projects.

---

## After the Call

- Send a thank-you email within 24 hours
- If they asked for anything (docs, clarification), send it fast
- Typical decision: 2-4 weeks after call
- If rejected: ask for feedback, apply again next cycle with improvements

---

## Quick Reference Card

Print this or keep on second monitor:

```
ELEVATOR PITCH:
Privacy-first biometric monitoring for DID.
Wearables → local ML → pattern recognition.
No cloud, no accounts, encrypted, open source.

WHY ME:
- Systems engineer, shipped production
- Have DID myself, building for my community
- Already did the research (7 docs, 200+ task backlog)

WHY LOCAL-FIRST:
- DID is stigmatized, breaches are catastrophic
- Community burned by "trust us" apps
- Only way to build trust: verify everything

ML APPROACH:
- Phase 1: Changepoint detection (unsupervised)
- Phase 2: Semi-supervised SVM (few labels)
- Phase 3: Few-shot neural nets (many alters)
- All on-device, no cloud

9-MONTH PLAN:
1-2: Foundation + encryption
3-4: Mobile + biometrics
5-6: ML pipeline
7-8: Integrations + polish
9: Beta + community

AFTER GRANT:
- I use it daily, won't abandon
- Open source = forkable
- Premium features for sustainability
```

---

*You've got this. You know this project better than anyone. The call is just explaining what you've already built in your head.*

🌑
