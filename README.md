# plato-music-sync

**Music cognition patterns for synchronizing Plato rooms.**

This crate uses music cognition theory to model and coordinate distributed room systems. The key insight: rooms ticking at different frequencies form a **polyrhythmic ensemble**. The engine room ticks at 0.2 Hz (slow bass), the backdeck at 2 Hz (fast percussion), the galley at 0.017 Hz (ambient drone). Music cognition provides the tools to keep them in sync.

## The Music-Cognition Isomorphism

Room synchronization and musical ensemble performance share deep structural parallels:

| Music Concept | Room Sync Analog |
|---|---|
| Polyrhythm | Multiple rooms at different tick rates |
| Groove | Fleet alignment / sync quality |
| Counterpoint | Productive vs wasteful room interactions |
| Cadence | Alarm → action → resolution patterns |
| Tempo | Adaptive tick rate adjustment |
| Rubato | Smooth tempo curves during transitions |

Every room is a voice in the ensemble. Every tick cycle is a rhythm. The fleet's health is the music it makes together.

## Modules

### `polyrhythm` — Polyrhythmic Scheduling

Coordinate rooms with different tick rates using LCM-based scheduling. Each room has a "time signature" derived from its tick frequency. The LCM of all tick rates (as rational numbers) gives the **master cycle** — the shortest period after which all rooms realign.

```rust
use plato_music_sync::PolyrhythmicScheduler;
use plato_music_sync::polyrhythm::Room;

let rooms = vec![
    Room { name: "engine".into(), tick_hz: 0.2 },   // Slow bass
    Room { name: "backdeck".into(), tick_hz: 2.0 },  // Fast percussion
    Room { name: "galley".into(), tick_hz: 0.017 },  // Ambient drone
];

let scheduler = PolyrhythmicScheduler::new(rooms);
println!("Master cycle: {} seconds", scheduler.master_cycle());
println!("Engine ticks per cycle: {}", scheduler.schedule_for("engine").unwrap().len());
```

The schedule maps each phase of the master cycle to which rooms should tick. Phase errors are detected when rooms tick late or early relative to their expected position in the cycle.

### `groove` — Groove Tracking

Measure how "in the groove" the fleet is. The **groove score** ranges from 0.0 (total chaos) to 1.0 (perfect sync). Each room's tick should land on the expected phase within the master cycle.

```rust
use plato_music_sync::GrooveTracker;

let mut tracker = GrooveTracker::new(100, 0.8); // window=100, threshold=0.8

// Perfect tick — no phase error
tracker.record_perfect_tick("engine", 0.5, 1.0);

// Check groove
if tracker.needs_correction() {
    println!("Groove dropped to {} — sync correction needed!", tracker.groove());
}
```

The groove score uses exponential decay over phase errors: `groove = exp(-avg_error * 20)`. When groove drops below the configured threshold, it signals that sync correction is needed. The tracker uses a sliding window of recent events so groove can recover as rooms catch up.

**Connection to agent-groove:** The groove tracker's scoring model connects to agent-groove's groove scheduling model, which uses the same metric to decide when to rebalance work across the fleet.

### `counterpoint` — Counterpoint Analysis

Detect productive vs wasteful room interactions using species counterpoint rules from music theory. Three types of motion are classified:

- **Contrary motion** — one room's value goes up while another goes down (productive interaction, like counterpoint in music)
- **Parallel motion** — both rooms move in the same direction (potentially wasteful/redundant, like parallel fifths)
- **Oblique motion** — one room stable, other changing (normal operation, like a pedal point)

```rust
use plato_music_sync::CounterpointAnalyzer;
use plato_music_sync::counterpoint::{RoomSnapshot, MotionType};

let mut analyzer = CounterpointAnalyzer::new(100);

analyzer.record(vec![
    RoomSnapshot { name: "engine".into(), value: 50.0 },
    RoomSnapshot { name: "bilge".into(), value: 30.0 },
]);
analyzer.record(vec![
    RoomSnapshot { name: "engine".into(), value: 60.0 },  // Up
    RoomSnapshot { name: "bilge".into(), value: 20.0 },   // Down → contrary!
]);

assert_eq!(analyzer.classify_motion("engine", "bilge"), Some(MotionType::Contrary));

let score = analyzer.motion_score("engine", "bilge");
println!("Quality: {:.2} (contrary: {:.0}%)", score.quality, score.contrary_ratio * 100.0);
```

The `MotionScore` quantifies coordination quality: contrary motion is weighted as productive (1.0), oblique as neutral (0.5), and parallel as potentially redundant (0.2). This mirrors how species counterpoint evaluates interval relationships.

**Connection to agent-counterpoint:** The counterpoint analyzer connects to agent-counterpoint's species counterpoint rules, which enforce constraints on acceptable interval progressions between room pairs.

### `cadence` — Cadence Detection

Detect resolution patterns in room state, inspired by musical cadences:

- **Perfect cadence** (V→I): alarm triggers → action taken → alarm resolves. The system resolves cleanly.
- **Deceptive cadence** (V→vi): alarm triggers → action taken → new alarm appears. The fix missed the root cause.
- **Half cadence** (...→V): alarm triggers → still pending. No resolution yet.

```rust
use plato_music_sync::CadenceDetector;
use plato_music_sync::cadence::{RoomState, CadenceType};

let mut detector = CadenceDetector::new();

// Perfect cadence: alarm → action → resolved
detector.tick();
detector.record_state("engine", RoomState::Alarm);
detector.tick();
detector.record_state("engine", RoomState::Action);
detector.tick();
let result = detector.record_state("engine", RoomState::Resolved);
assert_eq!(result, Some(CadenceType::Perfect));

let stats = detector.stats();
println!("Perfect: {}, Deceptive: {}, Half: {}", stats.perfect, stats.deceptive, stats.half);
```

Tracking cadence patterns over time enables **predictive maintenance**: if a room consistently produces deceptive cadences, the root cause is being missed. A pattern of half cadences suggests alarms that are being ignored.

### `tempo` — Tempo Map

Adaptive tick rate adjustment. Rooms speed up during crisis (allegro/presto) and slow down during stable periods (adagio/grave). Tempo changes propagate through the fleet.

```rust
use plato_music_sync::TempoMap;

let mut tempo = TempoMap::new();
tempo.add_room("engine", 1.0);
tempo.add_room("bilge", 0.5);

// Crisis: speed up engine
tempo.speed_up("engine", 2.0); // Now at 2x → Allegro
assert_eq!(tempo.effective_rate("engine"), 2.0);

// Propagate to other rooms
tempo.propagate("engine", &["bilge"], 0.5); // Blend 50%

// Smooth rubato curve
tempo.build_rubato_curve(1.0, 3.0, 100); // Accelerando over 100 ticks
for tick in 0..100 {
    tempo.apply_curve_to("engine", tick);
}
```

**Tempo markings** follow standard musical terminology:

| Marking | Multiplier | Use Case |
|---|---|---|
| Grave | 0.25x | Deep stability |
| Adagio | 0.5x | Stable period |
| Andante | 1.0x | Normal operation |
| Moderato | 1.5x | Slightly elevated |
| Allegro | 2.0x | Active response |
| Presto | 4.0x | Full crisis |

The **rubato curve** uses smoothstep interpolation (S-curve) for natural accelerando/ritardando transitions, connecting to agent-rubato's tempo curve model.

**Connection to agent-rubato:** The tempo map's curve model connects to agent-rubato, which provides sophisticated tempo curves for expressively-shaped transitions between operating modes.

## Architecture

```
┌─────────────────────────────────────────────────┐
│              plato-music-sync                    │
│                                                  │
│  ┌──────────────┐    ┌──────────────────────┐   │
│  │ Polyrhythmic │    │   Groove Tracker     │   │
│  │  Scheduler   │───▶│ (fleet alignment)    │   │
│  │ (LCM cycles) │    └──────────────────────┘   │
│  └──────────────┘              │                 │
│         │                      │                 │
│         ▼                      ▼                 │
│  ┌──────────────┐    ┌──────────────────────┐   │
│  │ Tempo Map    │    │ Counterpoint Analyzer│   │
│  │ (adaptive    │    │ (motion quality)     │   │
│  │  tick rates) │    └──────────────────────┘   │
│  └──────────────┘              │                 │
│         │                      │                 │
│         └──────┬───────────────┘                 │
│                ▼                                  │
│  ┌──────────────────────┐                        │
│  │  Cadence Detector    │                        │
│  │  (resolution patterns│                        │
│  │   → predictive maint)│                        │
│  └──────────────────────┘                        │
└─────────────────────────────────────────────────┘
```

## Agent Connections

This crate integrates with the SuperInstance agent ecosystem:

- **agent-groove** — Groove scheduling model for fleet-wide rebalancing decisions
- **agent-counterpoint** — Species counterpoint rules for room interaction constraints
- **agent-rubato** — Tempo curve model for expressive operating mode transitions
- **agent-ensemble** — Full ensemble coordination across all rooms

## Fishing Boat Example

A typical fishing boat has 5 rooms forming a polyrhythmic ensemble:

| Room | Tick Rate | Musical Role |
|---|---|---|
| Engine | 0.2 Hz (5s period) | Slow bass — foundation |
| Backdeck | 2.0 Hz (0.5s period) | Fast percussion — activity |
| Galley | 0.017 Hz (~60s period) | Ambient drone — long cycles |
| Bilge | 0.1 Hz (10s period) | Mid-range pulse |
| Bridge | 1.0 Hz (1s period) | Steady beat — coordination |

The master cycle is the LCM of all periods. In this ensemble, the groove score tells you if the boat is running smoothly. The counterpoint analyzer tells you if engine and bilge are working productively together. The cadence detector tracks whether alarms are being properly resolved. And the tempo map adjusts tick rates when things heat up.

## Testing

```bash
cargo test
```

26 tests covering all modules plus integration scenarios:
- 5 polyrhythm tests (LCM, scheduling, phase errors, fishing boat)
- 4 groove tests (perfect sync, late room, recovery, threshold)
- 5 counterpoint tests (contrary/parallel/oblique motion, scores)
- 4 cadence tests (perfect/deceptive/half, pattern tracking)
- 5 tempo tests (speed up/down, propagate, rubato, crisis)
- 4 integration tests (full scenarios)

## License

MIT
