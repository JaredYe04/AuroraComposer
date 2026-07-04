use std::path::Path;

use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

use crate::ir::{IrEvent, MusicIr};
use aurora_core::ExportError;

#[derive(Clone, Debug)]
pub enum SmfFormat {
    Type0,
    Type1,
}

#[derive(Clone, Debug)]
pub struct MidiExportConfig {
    pub format: SmfFormat,
    pub tpqn: u16,
    pub include_key_signature: bool,
    pub include_track_names: bool,
    pub drum_channel: u8,
}

impl Default for MidiExportConfig {
    fn default() -> Self {
        Self {
            format: SmfFormat::Type1,
            tpqn: 480,
            include_key_signature: true,
            include_track_names: true,
            drum_channel: 10,
        }
    }
}

pub fn export_midi(ir: &MusicIr, config: &MidiExportConfig) -> Result<Vec<u8>, ExportError> {
    let smf = build_smf(ir, config)?;
    let mut buf = Vec::new();
    smf.write(&mut buf)
        .map_err(|e| ExportError::Midi(e.to_string()))?;
    Ok(buf)
}

pub fn export_midi_to_file(
    ir: &MusicIr,
    path: &Path,
    config: &MidiExportConfig,
) -> Result<(), ExportError> {
    let bytes = export_midi(ir, config)?;
    std::fs::write(path, bytes).map_err(|e| ExportError::Midi(e.to_string()))
}

fn build_smf(ir: &MusicIr, config: &MidiExportConfig) -> Result<Smf<'static>, ExportError> {
    let format = match config.format {
        SmfFormat::Type0 => Format::SingleTrack,
        SmfFormat::Type1 => Format::Parallel,
    };

    let mut tracks: Vec<Vec<TrackEvent<'static>>> = Vec::new();

    let mut conductor = Vec::new();
    conductor.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::TrackName(b"Aurora Composer")),
    });

    if let Some(tempo) = ir.timeline.tempo_events.first() {
        let us_per_quarter = (60_000_000.0 / tempo.bpm).round() as u32;
        conductor.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(us_per_quarter.into())),
        });
    }

    if let Some(meter) = ir.timeline.meter_events.first() {
        conductor.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::TimeSignature(
                meter.beats,
                2u8.pow(u32::from(meter.beat_type).ilog2() as u32) as u8,
                24,
                8,
            )),
        });
    }

    if config.include_key_signature {
        if let Some(key) = ir.timeline.key_events.first() {
            conductor.push(TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::KeySignature(
                    key.fifths,
                    key.mode == "major",
                )),
            });
        }
    }

    conductor.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });
    tracks.push(conductor);

    for view in &ir.channel_index {
        let mut track = Vec::new();
        let channel = view.channel.0;

        if config.include_track_names {
            track.push(TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::TrackName(leak_bytes(&view.name))),
            });
        }

        if !view.is_drum {
            track.push(TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Midi {
                    channel: channel.into(),
                    message: MidiMessage::ProgramChange {
                        program: view.program.into(),
                    },
                },
            });
        }

        let mut current_tick = 0u64;
        for &idx in &view.event_indices {
            if let IrEvent::Note(note) = &ir.events[idx] {
                let delta_on = note.base.tick.saturating_sub(current_tick);
                track.push(TrackEvent {
                    delta: u32::try_from(delta_on)
                        .map_err(|_| ExportError::Midi("tick overflow".into()))?
                        .into(),
                    kind: TrackEventKind::Midi {
                        channel: channel.into(),
                        message: MidiMessage::NoteOn {
                            key: note.midi.into(),
                            vel: note.velocity.into(),
                        },
                    },
                });
                track.push(TrackEvent {
                    delta: note.duration_ticks.into(),
                    kind: TrackEventKind::Midi {
                        channel: channel.into(),
                        message: MidiMessage::NoteOff {
                            key: note.midi.into(),
                            vel: 0.into(),
                        },
                    },
                });
                current_tick = note.end_tick;
            }
        }

        track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });
        tracks.push(track);
    }

    Ok(Smf {
        header: Header {
            format,
            timing: Timing::Metrical(config.tpqn.into()),
        },
        tracks,
    })
}

fn leak_bytes(s: &str) -> &'static [u8] {
    Box::leak(s.as_bytes().to_vec().into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{project_ast_to_ir, DEFAULT_PPQ};
    use crate::test_fixtures::sample_composition;

    #[test]
    fn export_midi_produces_valid_smf_header() {
        let comp = sample_composition();
        let ir = project_ast_to_ir(&comp, DEFAULT_PPQ).unwrap();
        let bytes = export_midi(&ir, &MidiExportConfig::default()).unwrap();

        assert!(bytes.len() > 14);
        assert_eq!(&bytes[0..4], b"MThd");
        assert_eq!(bytes[8], 0);
        assert_eq!(bytes[9], 1);

        let smf = Smf::parse(&bytes).unwrap();
        assert_eq!(smf.header.format, Format::Parallel);
        assert!(smf.tracks.len() >= 2);
    }

    #[test]
    fn export_midi_contains_note_on() {
        let comp = sample_composition();
        let ir = project_ast_to_ir(&comp, DEFAULT_PPQ).unwrap();
        let bytes = export_midi(&ir, &MidiExportConfig::default()).unwrap();
        assert!(bytes.contains(&0x3C));
        assert!(bytes.contains(&80));
    }
}
