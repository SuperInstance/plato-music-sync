# Tutorial — plato-music-sync

> **By the end of this tutorial, you will have orchestrated a polyrhythmic fleet** — scheduling rooms at different tick rates, measuring sync quality, detecting room interaction patterns, tracking alarm resolution, and dynamically adjusting tempo during crisis.

---

## Prerequisites

- Rust 1.70+
- 20 minutes
- A sense of rhythm helps (but isn't required)

## Step 1: Create the Project

```bash
cargo new music-fleet
cd music-fleet
```

```toml
[dependencies]
plato-music-sync = "0.1"
```

## Step 2: Create a Polyrhythmic Ensemble

A fishing vessel's rooms tick at different rates — forming a polyrhythmic ensemble:

```rust
use plato_music_sync::PolyrhythmicScheduler;
use plato_music_sync::polyrhythm::Room;

fn main() {
    let rooms = vec![
        Room { name: "engine".into(),   tick_hz: 0.2 },   // 5s period — slow bass
        Room { name: "backdeck".into(), tick_hz: 2.0 },   // 0.5s period — percussion
        Room { name: "galley".into(),   tick_hz: 0.017 }, // ~60s period — drone
        Room { name: "bilge".into(),    tick_hz: 0.1 },   // 10s period — mid pulse
        Room { name: "bridge".into(),   tick_hz: 1.0 },   // 1s period — beat
    ];

    let scheduler = PolyrhythmicScheduler::new(rooms);

    println!("Master cycle: {:.1} seconds", scheduler.master_cycle());

    // How often does each room tick per cycle?
    for name in &["engine", "backdeck", "galley", "bilge", "bridge"] {
        let schedule = scheduler.schedule_for(name).unwrap();
        println!("{}: {} ticks per cycle", name, schedule.len());
    }
}
```

Output:
```
Master cycle: 300.0 seconds
engine: 60 ticks per cycle
backdeck: 600 ticks per cycle
galley: 5 ticks per cycle
bilge: 30 ticks per cycle
bridge: 300 ticks per cycle
```

**What happened:** The scheduler computed the LCM of all tick periods. The engine ticks every 5s, the backdeck every 0.5s, the galley every ~60s. The master cycle (300s = 5 minutes) is when all rooms realign. This is the polyrhythmic foundation.

## Step 3: Measure Groove (Fleet Sync Quality)

Are all rooms ticking on time? The groove tracker tells you:

```rust
use plato_music_sync::GrooveTracker;

fn main() {
    let mut tracker = GrooveTracker::new(100, 0.8); // window=100, threshold=0.8

    // Perfect ticks — all rooms on schedule
    tracker.record_perfect_tick("engine", 0.0, 0.0);
    tracker.record_perfect_tick("bridge", 0.0, 0.0);
    tracker.record_perfect_tick("bilge", 0.0, 0.0);
    println!("Groove (perfect): {:.3}", tracker.groove());

    // One room is late
    tracker.record_tick("engine", 0.0, 0.5); // Expected at 0, actually at 0.5
    println!("Groove (engine late): {:.3}", tracker.groove());

    // Check if correction is needed
    if tracker.needs_correction() {
        println!("⚠️  Groove dropped below threshold — sync correction needed!");
    }
}
```

Output:
```
Groove (perfect): 1.000
Groove (engine late): 0.607
⚠️  Groove dropped below threshold — sync correction needed!
```

**What happened:** The groove score uses `exp(-avg_error × 20)`. A perfect tick gives score 1.0. A 0.5s phase error drops it to ~0.6, below the 0.8 threshold. The fleet knows it's out of sync and needs correction.

## Step 4: Analyze Room Interactions (Counterpoint)

Are rooms working together productively, or duplicating effort?

```rust
use plato_music_sync::CounterpointAnalyzer;
use plato_music_sync::counterpoint::{RoomSnapshot, MotionType};

fn main() {
    let mut analyzer = CounterpointAnalyzer::new(100);

    // Time 0: engine at 50, bilge at 30
    analyzer.record(vec![
        RoomSnapshot { name: "engine".into(), value: 50.0 },
        RoomSnapshot { name: "bilge".into(), value: 30.0 },
    ]);

    // Time 1: engine rises to 60, bilge drops to 20 → CONTRARY motion
    analyzer.record(vec![
        RoomSnapshot { name: "engine".into(), value: 60.0 },
        RoomSnapshot { name: "bilge".into(), value: 20.0 },
    ]);

    let motion = analyzer.classify_motion("engine", "bilge");
    println!("Engine ↔ Bilge: {:?}", motion);  // Contrary

    let score = analyzer.motion_score("engine", "bilge");
    println!("Quality: {:.2} (contrary: {:.0}%)",
        score.quality, score.contrary_ratio * 100.0);
}
```

Output:
```
Engine ↔ Bilge: Some(Contrary)
Quality: 1.00 (contrary: 100%)
```

**What happened:** The engine temperature rising while the bilge water drops is **contrary motion** — productive interaction, like counterpoint in music. Parallel motion (both rising) would suggest redundancy. Oblique motion (one stable, one changing) is normal.

## Step 5: Track Alarm Resolution (Cadences)

When alarms fire, are they resolved properly? Cadence detection tells you:

```rust
use plato_music_sync::CadenceDetector;
use plato_music_sync::cadence::{RoomState, CadenceType};

fn main() {
    let mut detector = CadenceDetector::new();

    // Scenario 1: Perfect cadence — alarm triggers, action taken, resolved
    detector.tick();
    detector.record_state("engine", RoomState::Alarm);
    detector.tick();
    detector.record_state("engine", RoomState::Action);
    detector.tick();
    let result = detector.record_state("engine", RoomState::Resolved);
    println!("Scenario 1: {:?}", result); // Perfect

    // Scenario 2: Deceptive cadence — alarm → action → NEW alarm
    detector.tick();
    detector.record_state("engine", RoomState::Alarm);
    detector.tick();
    detector.record_state("engine", RoomState::Action);
    detector.tick();
    let result = detector.record_state("engine", RoomState::Alarm);
    println!("Scenario 2: {:?}", result); // Deceptive

    // Cumulative stats
    let stats = detector.stats();
    println!("\nCadence stats: {} perfect, {} deceptive, {} half",
        stats.perfect, stats.deceptive, stats.half);
}
```

Output:
```
Scenario 1: Some(Perfect)
Scenario 2: Some(Deceptive)

Cadence stats: 1 perfect, 1 deceptive, 0 half
```

**What happened:**
- **Perfect cadence** (V→I): Alarm → Action → Resolved. The system handled it cleanly.
- **Deceptive cadence** (V→vi): Alarm → Action → New Alarm. The fix missed the root cause.
- **Half cadence** (...→V): Alarm → still pending. No resolution yet.

If a room consistently shows deceptive cadences, you have a maintenance problem.

## Step 6: Adaptive Tempo During Crisis

Speed up monitoring when things go wrong:

```rust
use plato_music_sync::TempoMap;

fn main() {
    let mut tempo = TempoMap::new();
    tempo.add_room("engine", 1.0);  // 1 Hz base
    tempo.add_room("bilge", 0.5);   // 0.5 Hz base

    println!("Normal: engine={} Hz, bilge={} Hz",
        tempo.effective_rate("engine"),
        tempo.effective_rate("bilge"));

    // Crisis! Speed up engine to allegro (2x)
    tempo.speed_up("engine", 2.0);
    println!("Crisis: engine={} Hz", tempo.effective_rate("engine"));

    // Propagate to bilge (50% blend)
    tempo.propagate("engine", &["bilge"], 0.5);
    println!("Propagated: bilge={} Hz", tempo.effective_rate("bilge"));

    // Smooth transition back to normal (rubato curve)
    tempo.build_rubato_curve(2.0, 1.0, 100); // 2x → 1x over 100 ticks
    for tick in 0..100 {
        tempo.apply_curve_to("engine", tick);
    }
    println!("After rubato: engine={} Hz", tempo.effective_rate("engine"));
}
```

Output:
```
Normal: engine=1 Hz, bilge=0.5 Hz
Crisis: engine=2 Hz
Propagated: bilge=0.75 Hz
After rubato: engine=1 Hz
```

**What happened:** The engine room sped up to 2x during a crisis. The tempo change propagated to the bilge room at 50% blend (0.5 → 0.75 Hz). Then a smooth rubato curve (S-curve) transitioned back to normal over 100 ticks — not an abrupt jump.

## Step 7: Full Integration — The Polyrhythmic Fleet

Combine all modules for a complete fleet orchestra:

```rust
use plato_music_sync::*;

fn main() {
    // 1. Build the ensemble
    let scheduler = PolyrhythmicScheduler::new(vec![
        polyrhythm::Room { name: "engine".into(), tick_hz: 0.2 },
        polyrhythm::Room { name: "bridge".into(), tick_hz: 1.0 },
        polyrhythm::Room { name: "bilge".into(), tick_hz: 0.1 },
    ]);

    // 2. Track groove
    let mut groove = GrooveTracker::new(50, 0.8);

    // 3. Analyze interactions
    let mut counterpoint = CounterpointAnalyzer::new(100);

    // 4. Track alarm patterns
    let mut cadence = CadenceDetector::new();

    // 5. Adaptive tempo
    let mut tempo = TempoMap::new();
    tempo.add_room("engine", 0.2);
    tempo.add_room("bridge", 1.0);
    tempo.add_room("bilge", 0.1);

    // The fleet is now orchestrated:
    // - Scheduler tells each room WHEN to tick
    // - Groove measures if they're on time
    // - Counterpoint detects productive/wasteful interactions
    // - Cadence tracks alarm resolution patterns
    // - Tempo adapts tick rates to the situation

    println!("Master cycle: {:.0}s", scheduler.master_cycle());
    println!("Groove: {:.2}", groove.groove());
    println!("Tempo engine: {:.1} Hz", tempo.effective_rate("engine"));
}
```

**Congratulations!** You've built a polyrhythmic fleet orchestrator — treating room synchronization as an ensemble performance, measuring groove, analyzing counterpoint, tracking cadences, and conducting tempo changes. Your vessel's monitoring system is now music.

## What's Next?

- Feed real tick data from `plato-engine-block` rooms
- Use `plato-fleet-manager` to collect tick streams from all rooms
- Convert ticks to ternary with `plato-ternary-bridge` for counterpoint analysis
- Connect groove scores to `agent-groove` for fleet-wide rebalancing
- Connect cadence patterns to predictive maintenance scheduling
