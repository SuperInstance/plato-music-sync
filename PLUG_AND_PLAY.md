# Plug & Play — plato-music-sync

> Copy these templates. Change the room names and tick rates. You're orchestrating.

---

## Pattern 1: Polyrhythmic Scheduling

Compute when each room should tick in a multi-rate fleet.

```rust
use plato_music_sync::PolyrhythmicScheduler;
use plato_music_sync::polyrhythm::Room;

fn main() {
    // ↓ Change rooms and tick rates ↓
    let rooms = vec![
        Room { name: "engine".into(),   tick_hz: 0.2 },   // Every 5s
        Room { name: "backdeck".into(), tick_hz: 2.0 },   // Every 0.5s
        Room { name: "galley".into(),   tick_hz: 0.017 }, // Every ~60s
        Room { name: "bilge".into(),    tick_hz: 0.1 },   // Every 10s
        Room { name: "bridge".into(),   tick_hz: 1.0 },   // Every 1s
    ];

    let scheduler = PolyrhythmicScheduler::new(rooms);

    println!("Master cycle: {:.1}s (all rooms realign)", scheduler.master_cycle());
    for name in &["engine", "backdeck", "galley", "bilge", "bridge"] {
        if let Some(schedule) = scheduler.schedule_for(name) {
            println!("  {}: {} ticks/cycle", name, schedule.len());
        }
    }
}
```

**Change:** room names, tick_hz values.

---

## Pattern 2: Groove Tracking (Fleet Sync Monitor)

Measure whether rooms are ticking on schedule.

```rust
use plato_music_sync::GrooveTracker;

fn main() {
    // ↓ Adjust window and threshold ↓
    let mut tracker = GrooveTracker::new(100, 0.8);

    // Record ticks as they arrive
    // ↓ (expected_phase, actual_phase from your scheduler) ↓
    tracker.record_perfect_tick("engine", 0.0, 0.0);
    tracker.record_perfect_tick("bridge", 0.0, 0.0);
    tracker.record_tick("bilge", 0.0, 0.3); // 0.3s late

    println!("Groove score: {:.3} / 1.0", tracker.groove());

    if tracker.needs_correction() {
        println!("⚠️  Fleet out of sync — correction needed!");
        // Trigger sync correction logic here
    } else {
        println!("✓ Fleet in the groove");
    }
}
```

**Change:** window size, threshold, tick data sources.

---

## Pattern 3: Counterpoint + Cadence + Tempo (Full Orchestra)

Detect room interactions, track alarm resolution, and adapt tick rates.

```rust
use plato_music_sync::*;
use plato_music_sync::counterpoint::RoomSnapshot;
use plato_music_sync::cadence::RoomState;

fn main() {
    // --- Counterpoint: are rooms productive or redundant? ---
    let mut counterpoint = CounterpointAnalyzer::new(100);

    counterpoint.record(vec![
        RoomSnapshot { name: "engine".into(), value: 50.0 },
        RoomSnapshot { name: "bilge".into(),  value: 30.0 },
    ]);
    // ↓ Feed next tick's values ↓
    counterpoint.record(vec![
        RoomSnapshot { name: "engine".into(), value: 60.0 },
        RoomSnapshot { name: "bilge".into(),  value: 20.0 },
    ]);

    if let Some(score) = counterpoint.classify_motion("engine", "bilge") {
        println!("Engine↔Bilge: {:?} (quality: {:.2})",
            score,
            counterpoint.motion_score("engine", "bilge").quality);
    }

    // --- Cadence: how are alarms resolving? ---
    let mut cadence = CadenceDetector::new();
    cadence.tick();
    cadence.record_state("engine", RoomState::Alarm);
    cadence.tick();
    cadence.record_state("engine", RoomState::Action);
    cadence.tick();
    if let Some(cadence_type) = cadence.record_state("engine", RoomState::Resolved) {
        println!("Cadence: {:?} — alarm resolved cleanly", cadence_type);
    }

    // --- Tempo: speed up during crisis ---
    let mut tempo = TempoMap::new();
    // ↓ Register your rooms with base rates ↓
    tempo.add_room("engine", 1.0);
    tempo.add_room("bilge", 0.5);

    tempo.speed_up("engine", 2.0);    // Crisis: 2x speed
    tempo.propagate("engine", &["bilge"], 0.5);  // Spread to bilge

    println!("Engine tempo: {:.1} Hz", tempo.effective_rate("engine"));
    println!("Bilge tempo:  {:.1} Hz", tempo.effective_rate("bilge"));

    // Smooth return to normal
    tempo.build_rubato_curve(2.0, 1.0, 50);
    for t in 0..50 { tempo.apply_curve_to("engine", t); }
    println!("Engine after rubato: {:.1} Hz", tempo.effective_rate("engine"));
}
```

**Change:** room names, tick values, cadence sequences, tempo multipliers.

---

## Quick Reference

| What | API | Example |
|------|-----|---------|
| Schedule rooms | `PolyrhythmicScheduler::new(rooms)` | LCM-based tick scheduling |
| Master cycle | `scheduler.master_cycle()` | Full realignment period |
| Room schedule | `scheduler.schedule_for("room")` | Tick phases per cycle |
| Track groove | `GrooveTracker::new(window, threshold)` | Sync quality 0.0–1.0 |
| Record tick | `tracker.record_tick(room, expected, actual)` | Phase error tracking |
| Needs fix? | `tracker.needs_correction()` | Below threshold → true |
| Record room values | `analyzer.record(snapshots)` | Counterpoint input |
| Classify motion | `analyzer.classify_motion("a", "b")` | Contrary/Parallel/Oblique |
| Motion quality | `analyzer.motion_score("a", "b")` | 0.0–1.0 with ratios |
| Record alarm state | `detector.record_state("room", state)` | Alarm/Action/Resolved |
| Cadence type | `detector.record_state(...)` returns | Perfect/Deceptive/Half |
| Speed up room | `tempo.speed_up("room", factor)` | 2.0 = Allegro |
| Slow down room | `tempo.slow_down("room", factor)` | 0.5 = Adagio |
| Propagate tempo | `tempo.propagate("src", &["tgt"], blend)` | Spread tempo change |
| Rubato curve | `tempo.build_rubato_curve(from, to, steps)` | Smooth S-curve |
