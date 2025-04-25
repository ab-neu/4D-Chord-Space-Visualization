use midly::MidiMessage;
use midly::Smf;
use midly::TrackEventKind;
use std::fs;
use std::path::Path;

pub fn parse(path: &Path) -> Result<Vec<[i32; 4]>, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let smf = Smf::parse(&data)?;

    let tpq = match smf.header.timing {
        midly::Timing::Metrical(t) => t.as_int() as u32,
        _ => return Err("Unsupported timing".into()),
    };
    let ticks_per_16th = tpq / 4;

    // Each track becomes one voice line
    let mut voice_timelines = vec![vec![]; 4];
    for (track_idx, track) in smf.tracks.iter().take(4).enumerate() {
        let mut abs_tick = 0u32;
        let mut notes_by_tick = std::collections::BTreeMap::new();

        for event in track {
            abs_tick += event.delta.as_int();

            if let TrackEventKind::Midi { message, .. } = event.kind {
                if let MidiMessage::NoteOn { key, vel } = message {
                    if vel > 0 {
                        notes_by_tick.insert(abs_tick, key.as_int() as i32);
                    }
                }
            }
        }

        // Now build the timeline per 16th slot, sustaining notes
        let mut tick = 0;
        let max_tick = *notes_by_tick.keys().last().unwrap_or(&0);
        let mut last_note = 0;

        while tick <= max_tick {
            if let Some(&note) = notes_by_tick.get(&tick) {
                last_note = note;
            }

            voice_timelines[track_idx].push(last_note);
            tick += ticks_per_16th;
        }
    }

    // Align all voices into a single Vec<[i32; 4]>
    let len = voice_timelines.iter().map(Vec::len).max().unwrap_or(0);

    // 🔧 Backfill initial silent voices
    for timeline in &mut voice_timelines {
        if let Some(first_nonzero) = timeline.iter().find(|&&note| note != 0).copied() {
            for note in timeline.iter_mut() {
                if *note == 0 {
                    *note = first_nonzero;
                } else {
                    break;
                }
            }
        }
    }
    let mut combined = Vec::with_capacity(len);
    for i in 0..len {
        let frame = [
            *voice_timelines.get(0).and_then(|v| v.get(i)).unwrap_or(&0),
            *voice_timelines.get(1).and_then(|v| v.get(i)).unwrap_or(&0),
            *voice_timelines.get(2).and_then(|v| v.get(i)).unwrap_or(&0),
            *voice_timelines.get(3).and_then(|v| v.get(i)).unwrap_or(&0),
        ];
        combined.push(frame);
    }

    Ok(combined)
}
