# Developer Guide — plato-music-sync

> Architecture deep-dive, module walkthrough, extension points, and contributing guide for polyrhythmic room synchronization.

---

## Architecture Overview

`plato-music-sync` applies music cognition theory to distributed room synchronization. The core insight: rooms ticking at different frequencies form a **polyrhythmic ensemble**, and music provides the mathematical framework to keep them coordinated.

Each room is a "voice" in the ensemble. Tick rates are "rhythms." Fleet alignment is "groove." Room interactions are "counterpoint." Alarm resolution patterns are "cadences." Adaptive tick rates are "tempo."

### Module Map

```
┌──────────────────┐     ┌──────────────────┐
│  Polyrhythmic    │────▶│  Groove Tracker   │
│  Scheduler       │     │  (sync quality)   │
│  (LCM cycles)    │     └──────────────────┘
└──────────────────┘              │
         │                        │
         ▼                        ▼
┌──────────────────┐     ┌──────────────────┐
│  Tempo Map       │     │  Counterpoint    │
│  (adaptive rates)│     │  Analyzer        │
│                  │     │  (motion quality)│
└──────────────────┘     └──────────────────┘
         │                        │
         └────────┬───────────────┘
                  ▼
         ┌──────────────────┐
         │  Cadence         │
         │  Detector        │
         │  (resolution     │
         │   patterns)      │
         └──────────────────┘
```

---

## Module-by-Module Walkthrough

### `polyrhythm` — PolyrhythmicScheduler

Coordinates rooms with different tick rates using LCM-based scheduling. Each room has a tick frequency (Hz). The scheduler computes the **master cycle** — the shortest period after which all rooms realign — using the LCM of tick periods expressed as rational numbers.

```rust
pub struct PolyrhythmicScheduler {
    rooms: Vec<Room>,
    master_cycle: f64,         // Duration in seconds
    schedules: HashMap<String, Vec<f64>>,  // room → tick phases
}
```

**Key methods:**
- `new(rooms)` — Compute master cycle and schedules
- `master_cycle()` — Duration of one complete polyrhythmic cycle
- `schedule_for(room)` — Tick phases within the master cycle for a given room
- `detect_phase_errors(actual_phases)` — Find rooms ticking early/late

**The math:** If rooms tick at 0.2 Hz (5s), 1.0 Hz (1s), and 2.0 Hz (0.5s), the master cycle is LCM(5, 1, 0.5) = 5 seconds. During each 5-second cycle:
- 0.2 Hz room ticks once (at t=0)
- 1.0 Hz room ticks 5 times (at t=0,1,2,3,4)
- 2.0 Hz room ticks 10 times (at t=0,0.5,1,...,4.5)

**Extension point:** Add custom scheduling strategies (e.g., priority-based, jitter-resistant) by implementing alternative schedule computation methods.

### `groove` — GrooveTracker

Measures fleet alignment quality as a single floating-point score (0.0–1.0):

```
groove = exp(-avg_phase_error × 20)
```

The exponential decay penalizes phase errors heavily — even small misalignments drop the groove significantly. The tracker uses a sliding window of recent tick events so the groove can recover as rooms catch up.

```rust
pub struct GrooveTracker {
    window_size: usize,
    threshold: f64,    // Below this → needs correction
    events: VecDeque<GrooveEvent>,
}
```

**Key methods:**
- `new(window_size, threshold)` — Configure tracking
- `record_tick(room, expected_phase, actual_phase)` — Record a tick event
- `record_perfect_tick(room, expected, actual)` — Shortcut for on-time ticks
- `groove()` — Current groove score (0.0–1.0)
- `needs_correction()` — Whether groove dropped below threshold

**Connection to agent-groove:** The groove score model connects to the agent-groove scheduling model, which uses the same metric to decide when to rebalance work across the fleet.

**Extension point:** Customize the decay function (e.g., linear, polynomial) or add per-room groove scores for finer-grained analysis.

### `counterpoint` — CounterpointAnalyzer

Detects productive vs. wasteful room interactions using species counterpoint rules from music theory. Three types of motion between room pairs:

| Motion | Definition | Musical Analog | Quality |
|--------|-----------|----------------|---------|
| Contrary | One room up, other down | Species counterpoint | Productive (1.0) |
| Oblique | One stable, other changing | Pedal point | Neutral (0.5) |
| Parallel | Both move same direction | Parallel fifths | Redundant (0.2) |

```rust
pub struct CounterpointAnalyzer {
    window: usize,
    history: Vec<Vec<RoomSnapshot>>,
}
```

**Key methods:**
- `new(window)` — Configure analysis window
- `record(snapshots)` — Record a tick's room values
- `classify_motion(room_a, room_b)` — Classify motion between two rooms
- `motion_score(room_a, room_b)` — Quality score with ratio breakdown

The `MotionScore` struct:
```rust
pub struct MotionScore {
    pub quality: f64,          // 0.0–1.0
    pub contrary_ratio: f64,   // Fraction of contrary motion
    pub parallel_ratio: f64,   // Fraction of parallel motion
    pub oblique_ratio: f64,    // Fraction of oblique motion
}
```

**Extension point:** Add custom motion types (e.g., "divergent" for rooms moving rapidly apart) or weighted scoring that accounts for sensor importance.

### `cadence` — CadenceDetector

Detects resolution patterns in room state, inspired by musical cadences:

| Cadence | Pattern | Musical Analog | Meaning |
|---------|---------|----------------|---------|
| Perfect | Alarm → Action → Resolved | V→I | Clean resolution ✓ |
| Deceptive | Alarm → Action → New Alarm | V→vi | Root cause missed ✗ |
| Half | Alarm → (pending) | ...→V | Still unresolved ⏳ |

```rust
pub struct CadenceDetector {
    room_states: HashMap<String, Vec<RoomState>>,
    stats: CadenceStats,
}
```

**Key methods:**
- `new()` — Create detector
- `tick()` — Advance the detector by one tick
- `record_state(room, state)` — Record a room's current state (Alarm/Action/Resolved)
- `stats()` — Cumulative cadence statistics

**Predictive maintenance:** If a room consistently produces deceptive cadences, the root cause is being missed. A pattern of half cadences suggests alarms are being ignored. Tracking these patterns over time enables proactive intervention.

**Extension point:** Add new cadence types (e.g., "plagal" for gradual resolution, "phrygian" for unexpected resolution to a different state).

### `tempo` — TempoMap

Adaptive tick rate adjustment using musical tempo terminology:

| Marking | Multiplier | Use Case |
|---------|-----------|----------|
| Grave | 0.25x | Deep stability |
| Adagio | 0.5x | Stable period |
| Andante | 1.0x | Normal operation |
| Moderato | 1.5x | Slightly elevated |
| Allegro | 2.0x | Active response |
| Presto | 4.0x | Full crisis |

```rust
pub struct TempoMap {
    rooms: HashMap<String, RoomTempo>,
    rubato_curve: Option<Vec<f64>>,
}
```

**Key methods:**
- `new()` — Create empty tempo map
- `add_room(id, base_rate)` — Register a room with its base tick rate
- `speed_up(room, factor)` / `slow_down(room, factor)` — Adjust tempo
- `effective_rate(room)` — Current tick rate after adjustments
- `propagate(source, targets, blend)` — Propagate tempo change to other rooms
- `build_rubato_curve(from, to, steps)` — Create smoothstep S-curve transition
- `apply_curve_to(room, step)` — Apply curve value at a given step

**Rubato curves** use smoothstep interpolation for natural accelerando/ritardando transitions:

```
smoothstep(t) = 3t² - 2t³
```

**Connection to agent-rubato:** The tempo map's curve model connects to agent-rubato, which provides sophisticated tempo curves for expressively-shaped transitions.

**Extension point:** Add custom curve shapes (e.g., exponential, logarithmic) or automatic tempo adjustment based on alarm frequency.

---

## Testing Strategy

26 tests covering all modules plus integration scenarios:

- **Polyrhythm (5 tests):** LCM computation, scheduling, phase errors, fishing boat ensemble
- **Groove (4 tests):** Perfect sync, late room, recovery, threshold detection
- **Counterpoint (5 tests):** Contrary/parallel/oblique classification, quality scores
- **Cadence (4 tests):** Perfect/deceptive/half cadence detection, pattern tracking
- **Tempo (5 tests):** Speed up/down, propagation, rubato curves, crisis tempo
- **Integration (4 tests):** Full scenarios combining multiple modules

Run with:

```bash
cargo test
```

---

## Contributing Guide

### Adding a New Cadence Type

1. Add a variant to `CadenceType` in `cadence.rs`
2. Add detection logic in `record_state()`
3. Add the new type to `CadenceStats`
4. Write tests: sequences that trigger each new cadence type

### Adding a Custom Tempo Curve

1. Add a new method to `TempoMap` (e.g., `build_exponential_curve()`)
2. Store the curve values in the `rubato_curve` field
3. `apply_curve_to()` works with any curve — just change how it's built
4. Test the curve shape at start, middle, and end

### Adding a Motion Classification

1. Add a variant to `MotionType` in `counterpoint.rs`
2. Add detection logic in `classify_motion()`
3. Assign a quality weight (0.0–1.0)
4. Update `MotionScore` to include the new type's ratio

### Code Style

- All public types derive `Debug, Clone, PartialEq`
- Use f64 for all timing/rate calculations
- Musical terminology in API names (groove, cadence, rubato, counterpoint)
- Keep the music analogy consistent — it's the crate's identity

---

## Design Decisions

| Decision | Rationale |
|----------|-----------|
| LCM-based scheduling | Exact realignment; deterministic; no drift |
| Exponential decay for groove | Penalizes even small errors; responsive to degradation |
| Three motion types | Matches species counterpoint; covers all pairwise relationships |
| Musical cadence terminology | Gives operators intuitive language for alarm patterns |
| Tempo markings | Standard musical terms map naturally to operating modes |
| Smoothstep for rubato | S-curve produces natural acceleration/deceleration |
