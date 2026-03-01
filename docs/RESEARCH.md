# HeXuS Research Notes

## DID and Physiological Differences

### Documented Phenomena

Research and clinical observations have documented that different alters within a DID system can exhibit:

- **Different allergic responses** — One alter allergic to a substance, another not
- **Different vision** — Requiring different prescriptions per alter
- **Different handedness** — One alter left-handed, another right-handed
- **Different blood glucose patterns** — Diabetic responses varying by alter
- **Different pain tolerances** — Some alters feel pain more/less acutely
- **Different heart rate patterns** — Baseline HR and HRV varying by alter

### Mechanism Hypothesis

The brain controls autonomic functions. Different alters represent different neural activation patterns, which can cascade into different:
- Hormone levels
- Immune responses  
- Autonomic nervous system states
- Motor control preferences

This isn't "psychosomatic" in a dismissive sense — it's the brain literally running different configurations.

## Biometric Markers to Track

### High Signal (Established Correlation)
- **Heart Rate Variability (HRV)** — Strongly correlated with stress, emotional state
- **Galvanic Skin Response (GSR)** — Immediate stress/arousal indicator
- **Heart Rate** — Baseline differs per emotional state

### Medium Signal (Promising)
- **Blood Glucose** — If different alters process stress differently
- **Sleep Patterns** — Which alter "wakes up" may vary
- **Temperature** — Peripheral temperature correlates with stress

### Exploratory (Needs Research)
- **Typing patterns** — Keystroke dynamics, speed, error rate
- **Voice characteristics** — Pitch, cadence, vocabulary (if audio captured)
- **Movement patterns** — Gait, posture, gesture frequency

## The Labeling Problem

The core challenge: we need *labeled* data to train any classifier.

**Solution approaches:**
1. **Manual journaling** — User logs "Klyde is fronting" with timestamp
2. **Retrospective labeling** — Review data, tag periods after the fact
3. **Trigger correlation** — Known triggers → expected fronter
4. **Confidence gradients** — "Probably Klyde" vs "Definitely Klyde"

**Bootstrapping:**
- Start with clear-cut, known fronting periods
- Build initial baselines
- Use anomaly detection to surface "uncertain" periods for labeling
- Iterate toward better classification

## Related Projects / Prior Art

- **PluralKit** — Discord bot for plural systems, logging/switching
- **Simply Plural** — Mobile app for tracking fronting
- **Biometric research in DID** — Academic papers (to compile)

## Open Questions

1. How quickly do physiological markers change during a switch?
2. Are there consistent "transition signatures" during switches?
3. Do co-fronting states have distinct biometric patterns?
4. Can we detect switches *before* conscious awareness?
