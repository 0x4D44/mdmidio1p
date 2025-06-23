use midly::{
    Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent,
    TrackEventKind,
};
use midly::num::{u4, u7, u28, u15};
use std::io::Result as IoResult;

// ---------------------------------------------------------------------
// 1) Debug helper: safe_sub_u28
// ---------------------------------------------------------------------
fn safe_sub_u28(a: u28, b: u28, context: &str) -> u28 {
    if b > a {
        eprintln!("DEBUG: Underflow about to happen in {context}!");
        eprintln!("DEBUG:   a={:?}, b={:?}", a, b);
        panic!("attempting to subtract with overflow in {context}!");
    }
    a - b
}

// ---------------------------------------------------------------------
// 2) Helpers: create 'static TrackEvents
// ---------------------------------------------------------------------

/// A simple Note On event
fn note_on_event(delta: u28, channel: u4, note: u7, velocity: u7) -> TrackEvent<'static> {
    TrackEvent {
        delta,
        kind: TrackEventKind::Midi {
            channel,
            message: MidiMessage::NoteOn { key: note, vel: velocity },
        },
    }
}

/// A simple Note Off event
fn note_off_event(delta: u28, channel: u4, note: u7, velocity: u7) -> TrackEvent<'static> {
    TrackEvent {
        delta,
        kind: TrackEventKind::Midi {
            channel,
            message: MidiMessage::NoteOff { key: note, vel: velocity },
        },
    }
}

// ---------------------------------------------------------------------
// 3) Example chord data structure
// ---------------------------------------------------------------------

#[derive(Clone)]
struct Chord {
    root: u8,
    intervals: Vec<u8>,
}

// We'll define a minimal chord progression:
fn get_demo_chords() -> Vec<Chord> {
    vec![
        Chord { root: 60, intervals: vec![0,4,7] },  // C major
        Chord { root: 67, intervals: vec![0,4,7] },  // G major
        Chord { root: 65, intervals: vec![0,4,7] },  // F major
    ]
}

// ---------------------------------------------------------------------
// 4) The chord generation with debug logs
// ---------------------------------------------------------------------

/// We generate "strumming" events for each chord.  
/// For debugging, we have `safe_sub_u28` calls and `eprintln!` logs.
fn generate_chord_track_events(
    chords: &[Chord],
    start_tick: u28,
    ticks_per_measure: u28,
    channel: u4,
    base_velocity: u8,
) -> Vec<TrackEvent<'static>> {
    let mut events = Vec::new();

    let mut abs_time = start_tick;
    let mut last_abs_time = start_tick;

    // Hard-coded strum offsets
    let pattern = [0, 120, 240, 360];

    eprintln!("DEBUG: generate_chord_track_events() called.");
    eprintln!("DEBUG:  start_tick={start_tick:?}, ticks_per_measure={ticks_per_measure:?}, base_vel={base_velocity}");
    eprintln!("DEBUG:  channel={channel}");
    eprintln!("DEBUG:  chords.len()={}", chords.len());

    for (ch_idx, chord) in chords.iter().enumerate() {
        eprintln!(
            "DEBUG: chord index {ch_idx}, root={}, intervals={:?}, abs_time={abs_time:?}, last_abs_time={last_abs_time:?}",
            chord.root, chord.intervals
        );
        for &offset in &pattern {
            let note_on_abs = abs_time + u28::from(offset);
            let note_off_abs = note_on_abs + u28::from(40);

            eprintln!(
                "DEBUG:   offset={offset}, note_on_abs={note_on_abs:?}, note_off_abs={note_off_abs:?}, last_abs_time={last_abs_time:?}"
            );

            // For simplicity, let's just use chord.intervals[0], ignoring chord.intervals[1..].
            let midi_note = chord.root + chord.intervals[0];

            // Note On
            let delta_on = safe_sub_u28(note_on_abs, last_abs_time, "chord note_on delta");
            events.push(note_on_event(
                delta_on,
                channel,
                u7::from(midi_note),
                u7::from(base_velocity),
            ));
            last_abs_time = note_on_abs;

            // Note Off
            let delta_off = safe_sub_u28(note_off_abs, last_abs_time, "chord note_off delta");
            events.push(note_off_event(
                delta_off,
                channel,
                u7::from(midi_note),
                u7::from(64),
            ));
            last_abs_time = note_off_abs;
        }

        // Move forward one measure for the next chord
        abs_time += ticks_per_measure;
    }

    events
}

// ---------------------------------------------------------------------
// 5) Minimal main() that calls generate_chord_track_events
// ---------------------------------------------------------------------

fn main() -> IoResult<()> {
    // We'll do a minimal example:
    let header = Header {
        format: Format::Parallel,
        timing: Timing::Metrical(u15::from(480)), // 480 ticks/quarter
    };

    // We'll only have one track for demonstration
    let mut track = Track::new();

    // A simple chord progression
    let chords = get_demo_chords();

    // Let's generate chord events
    // If you want to test "bad" logic, try messing with these values:
    let chord_events = generate_chord_track_events(
        &chords,
        u28::from(0),
        u28::from(1920), // 4/4 measure with 480 TQ
        u4::from(0),
        64,
    );
    track.extend(chord_events);

    // End of track
    track.push(TrackEvent {
        delta: u28::from(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    // Build an SMF
    let smf = Smf {
        header,
        tracks: vec![track],
    };

    // Save it
    smf.save("output.mid")?;
    println!("Output.mid created");
    Ok(())
}

// ---------------------------------------------------------------------
// 6) Tests
// ---------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_chord_track_events_not_empty() {
        // We'll call the generator with typical values.
        let chords = get_demo_chords();
        let events = generate_chord_track_events(
            &chords,
            u28::from(0),
            u28::from(1920),
            u4::from(0),
            64,
        );
        assert!(!events.is_empty());
    }

    #[test]
    fn test_midi_file_creation() {
        let _ = main();
        assert!(fs::metadata("output.mid").is_ok());
    }
}
