# ML Approaches for HeXuS Alter Detection System

**Research Document**  
*Date: 2026-03-01*  
*Focus: Pattern detection and classification for biometric-based alter identification in DID systems*

---

## Executive Summary

This document outlines practical machine learning approaches for detecting and classifying alter switches in a DID (Dissociative Identity Disorder) biometric monitoring system. The core challenge is working with **limited labeled data** while detecting **state changes** in continuous biometric streams (heart rate, EDA, keystroke dynamics, voice patterns).

**Key Recommendations:**
1. **Start with changepoint detection** (CUSUM/online methods) for unsupervised state change alerts
2. **Bootstrap with active learning** to efficiently build labeled dataset
3. **Use semi-supervised learning** to leverage unlabeled biometric data
4. **Implement on-device in Rust** using Candle or Burn for privacy and performance

---

## 1. Time Series Anomaly & Changepoint Detection

### Overview
Detect when biometric streams shift between states (alter switches) without requiring labeled data initially.

### Recommended Techniques

#### 1.1 CUSUM (Cumulative Sum) - **PRIORITY**
**What it does:** Detects mean/variance shifts in streaming data by tracking cumulative deviations from baseline.

**Pros:**
- ✅ Low computational cost (O(1) per sample)
- ✅ Works online/real-time
- ✅ Minimal latency (< 1 second)
- ✅ No training data required
- ✅ Works well for physiological signals (HR, EDA)

**Cons:**
- ❌ Assumes known pre-change parameters (requires calibration period)
- ❌ Sensitive to noise (needs filtering)
- ❌ Only detects mean/variance changes (not complex patterns)

**Data Requirements:**
- 5-10 minute baseline per alter (when known)
- No labeled alter-switch events needed initially
- Works with single-channel streams (HR, EDA separately)

**Implementation Complexity:** ⭐⭐☆☆☆ (Low)
- Simple rolling statistics
- 50-100 lines of Rust code
- Can use `linfa` or custom implementation

**HeXuS Use Case:**
```
1. Establish baseline for each biometric during known stable periods
2. Run CUSUM in real-time on incoming streams
3. Alert when cumulative deviation exceeds threshold
4. Log timestamps of detected changes
5. User labels some detections → builds training dataset
```

---

#### 1.2 Sliding Window Approaches
**What it does:** Computes cost functions (variance, KL-divergence) over fixed windows to flag changepoints.

**Pros:**
- ✅ Handles multiple change types (mean, variance, distribution)
- ✅ Tunable sensitivity via window size
- ✅ Can combine multiple biometric signals

**Cons:**
- ❌ Higher computational cost than CUSUM
- ❌ Window size trades off latency vs accuracy
- ❌ May miss rapid switches

**Data Requirements:**
- Window size: 30-60 seconds for biometrics
- No labels needed

**Implementation Complexity:** ⭐⭐⭐☆☆ (Medium)

---

#### 1.3 PELT (Pruned Exact Linear Time) - **Offline Analysis**
**What it does:** Finds optimal changepoints in complete time series using dynamic programming.

**Pros:**
- ✅ Exact solution (optimal segmentation)
- ✅ O(N) complexity under uniform conditions
- ✅ Great for post-hoc analysis

**Cons:**
- ❌ Requires full data (not real-time)
- ❌ Only useful for retrospective labeling

**HeXuS Use Case:**
- Run overnight on previous day's data
- Auto-generate candidate alter-switch timestamps
- User reviews/labels next day

**Implementation:** Use `ruptures` Python library, call via FFI or export/import data

---

### Real-Time vs Batch Tradeoffs

| Approach | Latency | Accuracy | CPU Cost | Use Case |
|----------|---------|----------|----------|----------|
| **CUSUM** | <1s | Good | Very Low | Live monitoring alerts |
| **Sliding Window** | 30-60s | Better | Medium | Live with more context |
| **PELT** | N/A (batch) | Best | High | Nightly auto-labeling |

**Recommendation:** Start with CUSUM for real-time + PELT nightly for retrospective labeling.

---

## 2. Classification Approaches (Limited Labeled Data)

### The Challenge
- Initially: 0-50 labeled alter-switch examples
- Need to classify "who's fronting" from biometric patterns
- New alters may emerge over time (open-set problem)

### Recommended Techniques

#### 2.1 Few-Shot Learning with Prototypical Networks - **RECOMMENDED**

**What it does:** Learns to classify from just 1-5 examples per class by computing "prototypes" (embeddings) for each alter in latent space.

**How it works:**
1. Train a feature extractor (CNN/RNN) on support set (few examples per alter)
2. Compute prototype = mean embedding of support examples for each alter
3. Classify query samples by nearest prototype (Euclidean distance)

**Pros:**
- ✅ Works with 1-5 examples per alter
- ✅ Naturally handles new alters (just add new prototype)
- ✅ State-of-the-art for biometrics (signatures, iris, face)
- ✅ Can combine multiple biometric modalities

**Cons:**
- ❌ Requires pre-training on similar data (emotion recognition datasets)
- ❌ Performance degrades if alters have very similar biometrics

**Data Requirements:**
- **Initial:** 50-100 labeled alter-switch events (10 per alter for 5-10 alters)
- **Per new alter:** 1-5 labeled examples
- **Pre-training:** Use public emotion/stress datasets (DREAMER, DEAP)

**Implementation Complexity:** ⭐⭐⭐⭐☆ (High)
- Requires neural network training
- Use Burn or Candle in Rust
- Can import pre-trained ONNX models

**HeXuS Architecture:**
```
Biometric streams → Feature extraction (CNN/LSTM) → Embedding space
                                                     ↓
Support set (known examples) → Prototypes ← Compute means
                                  ↓
Query sample embedding → Compare to prototypes → Predict alter
```

---

#### 2.2 Semi-Supervised Learning (Self-Training) - **PRIORITY**

**What it does:** Trains on small labeled set, then iteratively labels high-confidence unlabeled data to expand training set.

**How it works:**
1. Train initial classifier (SVM, Random Forest) on labeled data
2. Predict on unlabeled data
3. Add high-confidence predictions (>90% probability) to training set
4. Retrain and repeat

**Pros:**
- ✅ Achieves 81% accuracy with just 2.5-10% labeled data
- ✅ Works with traditional ML (no deep learning needed)
- ✅ Reduces overfitting vs pure supervised
- ✅ Proven on behavioral biometrics

**Cons:**
- ❌ Can amplify errors if initial model is poor
- ❌ Requires confidence calibration

**Data Requirements:**
- **Initial:** 50-100 labeled examples total
- **Unlabeled:** All collected biometric data (thousands of samples)

**Implementation Complexity:** ⭐⭐⭐☆☆ (Medium)
- Use `linfa` for SVM/Random Forest in Rust
- Simple iterative loop

**HeXuS Workflow:**
```
Day 1: User labels 50 changepoints manually
Day 2: Train SVM → Auto-label 500 unlabeled samples (high confidence)
Day 3: Retrain on 550 samples → Auto-label 2000 more
Week 2: Model trained on 5000+ samples (mostly auto-labeled)
```

---

#### 2.3 Active Learning - **Label Efficiency Booster**

**What it does:** Intelligently selects which unlabeled samples to ask user to label (maximizes information gain per label).

**Query Strategies:**
- **Uncertainty sampling:** Label samples where model is least confident
- **Query-by-committee:** Label samples where ensemble of models disagrees
- **Diversity sampling:** Label samples far from existing labeled data

**Pros:**
- ✅ **Achieves 95% performance with only 16% of labels needed**
- ✅ Reduces user labeling burden by 5-10x
- ✅ Combines perfectly with semi-supervised learning

**Cons:**
- ❌ Requires user in the loop
- ❌ Can be biased toward edge cases

**Data Requirements:**
- Start with 10-20 seed labels
- Actively query 5-10 per day
- Reach good performance with 50-100 total labels

**Implementation Complexity:** ⭐⭐⭐☆☆ (Medium)

**HeXuS Integration:**
```
1. Model encounters ambiguous biometric pattern
2. Flags for user review: "Was this alter A or B?"
3. User labels via phone notification
4. Model immediately retrains with new label
5. Repeat until confident
```

---

#### 2.4 One-Class Classification per Alter

**What it does:** Instead of multi-class (which alter?), train separate binary classifier per alter (is this alter X or not?).

**Pros:**
- ✅ Handles new alters gracefully (just add new classifier)
- ✅ Works with very limited data per alter
- ✅ Can detect "unknown" alters (no classifier triggers)

**Cons:**
- ❌ N classifiers for N alters
- ❌ Calibration needed to avoid conflicts

**Data Requirements:**
- 20-30 examples per alter
- Can be bootstrapped with few-shot meta-learning

**Implementation Complexity:** ⭐⭐⭐☆☆ (Medium)

---

### Bootstrapping Strategy (Recommended Workflow)

**Week 1: Cold Start**
1. Use CUSUM to detect 50 candidate changepoints
2. User manually labels 50 examples (10 min/day)
3. Train initial SVM with semi-supervised self-training
4. Achieve ~60-70% accuracy

**Week 2-3: Active Expansion**
5. Deploy active learning queries (5 labels/day)
6. Semi-supervised expands to 500+ auto-labeled samples
7. Accuracy improves to ~80%

**Month 2+: Few-Shot Refinement**
8. Train few-shot prototypical network on collected data
9. New alters require only 1-5 examples
10. Accuracy reaches 85-90%

---

## 3. Relevant ML Research & Techniques

### 3.1 Emotion Recognition from Biometrics

**Key Findings:**
- Multi-modal fusion (HR + EDA + breathing) significantly outperforms single signals
- Deep learning (CNN-GRU with attention) achieves **95%+ accuracy** on valence/arousal
- Traditional ML (SVM with hand-crafted features) achieves **72-84%** with less data

**Relevant Datasets (for pre-training):**
- **DREAMER:** EEG + ECG for emotion, 23 subjects
- **DEAP:** EEG/physiological for valence/arousal/dominance
- **AMIGOS:** Multimodal emotion dataset

**Features to Extract:**
- **Heart Rate:** Mean, STD, HRV (time/frequency domain)
- **EDA:** Mean, slope, peaks, rise time
- **Combined:** Cross-correlation, coherence

**HeXuS Application:**
Different alters may have distinct emotional baselines → Use emotion recognition features as input to alter classifier.

---

### 3.2 Keystroke Dynamics

**Key Findings:**
- **78-84% user identification accuracy** from free-text typing
- Features: Dwell time (key hold), flight time (key-to-key interval), digraphs
- Hybrid approaches (keystroke + mouse) achieve 84%

**HeXuS Application:**
- Different alters may have distinct typing patterns
- Passive, continuous authentication
- Can run in background without active biometric collection

**Features:**
```rust
struct KeystrokeFeatures {
    mean_dwell_time: f32,      // Average key hold duration
    std_dwell_time: f32,
    mean_flight_time: f32,     // Average time between keys
    digraph_times: Vec<f32>,   // Specific key pair timings
    typing_speed: f32,         // WPM
    error_rate: f32,           // Backspace frequency
}
```

**Implementation:**
- Use `rdev` crate for keyboard event hooking
- Window size: 50-200 keystrokes
- Real-time classification every 30s of typing

---

### 3.3 Voice-Based Identification

**Key Findings:**
- Neural networks achieve **88%+ speaker ID** with short clips (5-10s)
- MFCCs (Mel-Frequency Cepstral Coefficients) are primary features
- Modern: ECAPA-TDNN, x-vectors for embeddings

**Pros:**
- ✅ Works with natural speech (no special phrases)
- ✅ Can run on audio from calls, voice notes

**Cons:**
- ❌ Requires audio recording (privacy concern)
- ❌ Complex feature extraction

**HeXuS Application:**
- Optional modality for systems with voice access
- Useful for phone-based HeXuS clients
- Can fuse with keystroke/biometric for multi-modal

---

### 3.4 Self-Supervised Pretraining

**Time-Frequency Consistency (TF-C):**
- Pre-train encoder on unlabeled biometric data
- Uses contrastive loss between time-domain and frequency-domain views
- Achieves domain-invariant representations
- Fine-tune on small labeled dataset

**Benefits for HeXuS:**
- ✅ Leverage all collected unlabeled biometric data
- ✅ Learn robust features before classification
- ✅ Reduces labeled data requirements

**Implementation:**
1. Collect weeks of unlabeled biometric streams
2. Pre-train encoder with TF-C (Burn/Candle)
3. Freeze encoder, train classifier head on 50-100 labeled examples
4. Achieves better accuracy than training from scratch

---

### 3.5 Hidden Markov Models (HMMs)

**What it does:** Models alters as hidden states, biometrics as observations.

**Pros:**
- ✅ Naturally models state transitions (alter switches)
- ✅ Incorporates temporal dynamics
- ✅ Handles noisy observations
- ✅ Viterbi algorithm finds most likely state sequence

**Cons:**
- ❌ Assumes Markov property (state only depends on previous state)
- ❌ Requires known number of states (alters)
- ❌ Parameter estimation needs labeled sequences

**HeXuS Use Case:**
```
Hidden states: [Alter A, Alter B, Alter C, Co-fronting AB, ...]
Observations: [HR, EDA, keystroke features, ...]
Transitions: P(switch from A to B), P(stay in A)
Emissions: P(observe HR=80 | Alter A)
```

**When to use:**
- After collecting labeled sequences
- When temporal patterns matter (e.g., Alter A often precedes Alter B)
- For smoothing noisy predictions

**Implementation Complexity:** ⭐⭐⭐⭐☆ (High)
- Custom Rust implementation or Python `hmmlearn` via FFI

---

## 4. Rust ML Ecosystem for On-Device Inference

### Overview
Rust enables privacy-preserving, on-device inference without cloud dependencies.

### 4.1 Candle (Hugging Face) - **RECOMMENDED FOR INFERENCE**

**What it is:** Minimalist deep learning framework from Hugging Face, optimized for inference.

**Strengths:**
- ✅ **Designed for production inference** (serverless, edge)
- ✅ CPU, GPU (CUDA), Metal, WASM support
- ✅ Loads PyTorch, ONNX, SafeTensors models
- ✅ Lightweight binaries (no Python runtime)
- ✅ Direct Hugging Face model integration
- ✅ Flash-attention, quantization support

**Weaknesses:**
- ❌ Less mature for training (use PyTorch then export)
- ❌ Smaller ecosystem than PyTorch

**HeXuS Use Case:**
1. Train few-shot model in Python (PyTorch)
2. Export to ONNX or SafeTensors
3. Load in HeXuS Rust client with Candle
4. Run real-time inference on-device (laptop, phone via FFI)

**Example Code:**
```rust
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::...;

let device = Device::Cpu; // Or Cuda(0) for GPU
let weights = VarBuilder::from_safetensors(...);
let model = AlterClassifier::load(&weights, &device)?;

// Run inference on biometric features
let features = Tensor::new(&biometric_features, &device)?;
let logits = model.forward(&features)?;
let alter_id = logits.argmax(1)?;
```

**Implementation Complexity:** ⭐⭐⭐☆☆ (Medium)

---

### 4.2 Burn - **RECOMMENDED FOR TRAINING**

**What it is:** Full deep learning framework in pure Rust with training + inference.

**Strengths:**
- ✅ **End-to-end training in Rust** (no Python needed)
- ✅ Backend-agnostic (Ndarray, Torch, WGPU, CUDA)
- ✅ ONNX import/export
- ✅ Automatic differentiation, optimizers, loss functions
- ✅ Type-safe tensor shapes (compile-time checks)
- ✅ No-std support (embedded devices)

**Weaknesses:**
- ❌ Smaller pre-trained model ecosystem
- ❌ Less documentation than PyTorch

**HeXuS Use Case:**
- Pure-Rust training pipeline (if avoiding Python)
- On-device incremental learning (model updates on user's device)
- Embedded deployment (low-power wearables)

**Example Code:**
```rust
use burn::prelude::*;
use burn::nn::{Linear, LinearConfig};
use burn::optim::{Adam, AdamConfig};

#[derive(Module, Debug)]
struct AlterClassifier<B: Backend> {
    fc1: Linear<B>,
    fc2: Linear<B>,
}

impl<B: Backend> AlterClassifier<B> {
    fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        let x = self.fc1.forward(x).relu();
        self.fc2.forward(x)
    }
}

// Training loop
let mut model = AlterClassifier::new();
let mut optim = AdamConfig::new().init();
let loss = CrossEntropyLoss::new();

for (features, labels) in dataloader {
    let output = model.forward(features);
    let loss_val = loss.forward(output, labels);
    let grads = loss_val.backward();
    model = optim.step(lr, model, grads);
}
```

**Implementation Complexity:** ⭐⭐⭐⭐☆ (High for full pipeline)

---

### 4.3 Linfa - **TRADITIONAL ML**

**What it is:** Scikit-learn equivalent for Rust (traditional ML algorithms).

**Strengths:**
- ✅ SVM, Random Forest, K-Means, PCA, etc.
- ✅ No neural network complexity
- ✅ Fast training on small datasets
- ✅ Interpretable models

**Weaknesses:**
- ❌ No deep learning
- ❌ Limited pre-built feature extractors

**HeXuS Use Case:**
- Initial baseline models (SVM, Random Forest)
- Semi-supervised self-training
- Feature selection (PCA, mutual information)

**Example Code:**
```rust
use linfa::prelude::*;
use linfa_svm::Svm;

// Train SVM classifier
let dataset = Dataset::new(features, labels);
let model = Svm::<f64, Pr>::params()
    .gaussian_kernel(0.5)
    .fit(&dataset)?;

// Predict
let prediction = model.predict(&new_features);
```

**Implementation Complexity:** ⭐⭐☆☆☆ (Low)

---

### 4.4 Comparison & Recommendations

| Framework | Best For | Training | Inference | Complexity |
|-----------|----------|----------|-----------|------------|
| **Candle** | Production inference, pre-trained models | ❌ | ✅✅✅ | Medium |
| **Burn** | Full Rust pipeline, on-device training | ✅✅✅ | ✅✅ | High |
| **Linfa** | Quick baselines, traditional ML | ✅✅ | ✅✅ | Low |

**HeXuS Recommendation:**
1. **Phase 1 (Baseline):** Use `linfa` for SVM/Random Forest + semi-supervised learning
2. **Phase 2 (Neural):** Train few-shot model in PyTorch, export ONNX, load with `candle`
3. **Phase 3 (Advanced):** Migrate to `burn` for on-device incremental learning

---

### Training Pipeline Recommendations

**Option A: Hybrid (Recommended)**
```
1. Collect data in Rust (HeXuS client)
2. Export to CSV/Parquet
3. Train in Python (PyTorch, scikit-learn) - easier ecosystem
4. Export to ONNX
5. Load in Rust with Candle for inference
6. Periodically retrain offline
```

**Option B: Pure Rust**
```
1. Collect data in Rust
2. Train with Burn or Linfa
3. Inference with same framework
4. On-device incremental updates
```

**Tradeoffs:**
- Hybrid: Easier development, better tooling, requires Python
- Pure Rust: Privacy, no dependencies, harder debugging

**For HeXuS:** Start with Option A (hybrid), migrate to Option B once stable.

---

## 5. Recommended Architecture for HeXuS

### Phase 1: Unsupervised Monitoring (Week 1)
```
Biometric streams → CUSUM changepoint detection → Alert user
                                                    ↓
                                             User labels some
                                                    ↓
                                          Build initial dataset
```

**Implementation:**
- Real-time CUSUM in Rust (`linfa` or custom)
- Notification system for labeling
- SQLite database for labels

---

### Phase 2: Semi-Supervised Bootstrap (Week 2-4)
```
Labeled data → Train SVM (linfa) → Predict on unlabeled
                                         ↓
                            Add high-confidence predictions
                                         ↓
                                 Retrain iteratively
                                         ↓
                            Active learning queries
```

**Implementation:**
- SVM with `linfa`
- Self-training loop
- Active learning: query 5-10 samples/day

---

### Phase 3: Few-Shot Neural Model (Month 2+)
```
Collected data → Pre-train encoder (TF-C, self-supervised)
                                ↓
                    Prototypical network training (PyTorch)
                                ↓
                         Export to ONNX
                                ↓
                    Load with Candle in HeXuS
                                ↓
            Real-time inference + active learning updates
```

**Implementation:**
- Train in Python, deploy in Rust
- Periodic retraining (weekly)
- On-device caching of prototypes

---

### Data Flow Architecture
```
┌─────────────────────────────────────────────────────┐
│ HeXuS Client (Rust)                                 │
│ ┌─────────────┐  ┌──────────────┐  ┌─────────────┐ │
│ │ Biometric   │→ │ Feature      │→ │ Changepoint │ │
│ │ Collection  │  │ Extraction   │  │ Detection   │ │
│ └─────────────┘  └──────────────┘  └─────────────┘ │
│                                            ↓         │
│                  ┌──────────────────────────────┐   │
│                  │ Classification               │   │
│                  │ - Phase 1: Rule-based        │   │
│                  │ - Phase 2: SVM (linfa)       │   │
│                  │ - Phase 3: Neural (candle)   │   │
│                  └──────────────────────────────┘   │
│                                            ↓         │
│                  ┌──────────────────────────────┐   │
│                  │ Active Learning              │   │
│                  │ - Query ambiguous samples    │   │
│                  │ - User labels via phone      │   │
│                  └──────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
                            ↓
         ┌──────────────────────────────────┐
         │ Offline Training (Optional)      │
         │ - Python/PyTorch                 │
         │ - Export ONNX                    │
         │ - Update client                  │
         └──────────────────────────────────┘
```

---

## 6. Data Requirements Summary

### Minimal Viable Dataset (Week 1)
- **Unlabeled:** 1 week continuous biometric data
- **Labeled:** 50 changepoint examples (5-10 per alter)
- **Features:** HR mean/std, EDA mean/slope, keystroke timing

### Good Performance (Month 1)
- **Unlabeled:** 1 month continuous data
- **Labeled:** 200-300 examples (semi-supervised auto-labeled)
- **Features:** + HRV, breathing rate, voice (optional)

### Production Quality (Month 3+)
- **Unlabeled:** 3+ months data (for self-supervised pretraining)
- **Labeled:** 500+ examples per alter (active learning)
- **Features:** Multi-modal fusion (all available signals)

### Per-Alter Requirements
| Phase | Examples Needed | Accuracy Expected |
|-------|-----------------|-------------------|
| Cold start | 10-20 | 60-70% |
| Bootstrap | 50-100 | 75-85% |
| Few-shot | 100-200 | 85-90% |
| Production | 200+ | 90-95% |

---

## 7. Implementation Complexity Estimates

### Phase 1: CUSUM + Manual Labeling
- **Time:** 1-2 weeks
- **Complexity:** ⭐⭐☆☆☆
- **Components:**
  - CUSUM implementation: 100 lines Rust
  - Notification system: 50 lines
  - Label storage: SQLite integration

### Phase 2: Semi-Supervised SVM
- **Time:** 2-3 weeks
- **Complexity:** ⭐⭐⭐☆☆
- **Components:**
  - Feature extraction: 200 lines
  - SVM training (`linfa`): 100 lines
  - Self-training loop: 150 lines
  - Active learning: 100 lines

### Phase 3: Few-Shot Neural Network
- **Time:** 4-6 weeks
- **Complexity:** ⭐⭐⭐⭐☆
- **Components:**
  - PyTorch prototypical network: 500 lines Python
  - ONNX export: 50 lines
  - Candle inference: 300 lines Rust
  - Model update pipeline: 200 lines

### Total Development Estimate
- **MVP (Phase 1+2):** 1-2 months
- **Full System (Phase 3):** 3-4 months
- **Team Size:** 1-2 developers

---

## 8. Key Risks & Mitigations

### Risk 1: Insufficient Labeled Data
**Mitigation:** 
- Start with unsupervised CUSUM (no labels needed)
- Active learning to minimize labeling burden
- Semi-supervised to leverage unlabeled data

### Risk 2: Alters with Similar Biometrics
**Mitigation:**
- Multi-modal fusion (combine HR + keystroke + voice)
- Temporal patterns (HMM to capture switch sequences)
- One-class classifiers per alter (detect "unknown")

### Risk 3: Rust ML Ecosystem Immaturity
**Mitigation:**
- Hybrid Python training / Rust inference
- Use stable libraries (linfa for baselines)
- ONNX as interchange format

### Risk 4: Privacy / On-Device Constraints
**Mitigation:**
- All processing on-device (no cloud)
- Quantized models for efficiency
- Candle's low-overhead inference

### Risk 5: Concept Drift (Alters Change Over Time)
**Mitigation:**
- Online learning with sliding window
- Periodic retraining (weekly/monthly)
- Active learning for continuous adaptation

---

## 9. Next Steps

### Immediate (Week 1)
1. ✅ Implement CUSUM changepoint detection
2. ✅ Create labeling interface (phone notifications)
3. ✅ Collect baseline biometric data

### Short-Term (Month 1)
4. ✅ Extract features (HR, EDA, keystroke)
5. ✅ Train SVM baseline with `linfa`
6. ✅ Implement semi-supervised self-training
7. ✅ Deploy active learning queries

### Medium-Term (Month 2-3)
8. ✅ Collect 500+ labeled examples per alter
9. ✅ Pre-train encoder with self-supervised TF-C
10. ✅ Train few-shot prototypical network (PyTorch)
11. ✅ Export ONNX, integrate Candle inference

### Long-Term (Month 4+)
12. ✅ Migrate to Burn for on-device training
13. ✅ Implement multi-modal fusion
14. ✅ Add HMM temporal modeling
15. ✅ Continuous model updates

---

## 10. References & Resources

### Papers
- **Few-Shot Biometrics:** "Few-Shot Learning for Biometric Verification" (arXiv:2211.06761)
- **Time-Frequency Consistency:** TF-C for Self-Supervised Time Series (Harvard/Zitnik Lab)
- **Keystroke Dynamics:** "Keystroke Dynamics User Authentication" (arXiv:2307.05529)
- **Emotion Recognition:** "Multimodal Emotion Recognition Using Deep Learning" (PMC6069143)
- **Active Learning:** "Confidence-based Acquisition for Active Learning" (MIT Press)

### Datasets (for pre-training)
- **DREAMER:** EEG/ECG emotion dataset (23 subjects)
- **DEAP:** Valence/arousal physiological dataset
- **AMIGOS:** Multimodal emotion dataset

### Rust Libraries
- **Candle:** github.com/huggingface/candle
- **Burn:** github.com/tracel-ai/burn
- **Linfa:** github.com/rust-ml/linfa
- **rdev:** Keyboard/mouse event hooking

### Tools
- **Python:** PyTorch, scikit-learn, ruptures (changepoint detection)
- **ONNX:** Model interchange format
- **LM Studio / Ollama:** Local LLM for feature engineering ideas

---

## Conclusion

**Recommended Path for HeXuS:**

1. **Start simple:** CUSUM changepoint detection + manual labeling (Week 1)
2. **Bootstrap efficiently:** Semi-supervised SVM with active learning (Month 1)
3. **Scale with neural networks:** Few-shot prototypical network (Month 2-3)
4. **Deploy in Rust:** Candle inference for privacy + performance (Month 3+)

**Key Insight:** You don't need thousands of labeled examples. With modern techniques (few-shot learning, semi-supervised, active learning), you can achieve 85-90% accuracy with just 50-200 labeled examples per alter, collected iteratively.

**Privacy-First:** All techniques can run on-device in Rust. No cloud, no data leakage.

**Practical:** Start with traditional ML (linfa), graduate to neural networks when you have enough data. The Rust ecosystem is ready.

---

*End of Research Document*
