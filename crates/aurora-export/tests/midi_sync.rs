//! Verify exported MIDI aligns drum hits with melodic downbeats.

use aurora_core::{ParameterBundle, UiParameterSnapshot};
use aurora_engine::generate_composition;
use aurora_export::ir::{project_ast_to_ir, DEFAULT_PPQ};
use aurora_export::midi::{export_midi, MidiExportConfig};
use midly::{Format, Smf, Timing, TrackEventKind};

fn ui_default_params() -> ParameterBundle {
    let mut p = ParameterBundle::from(UiParameterSnapshot::default());
    p.style.genre = "pop".into();
    p.form.section_lengths = vec![8];
    p.search.seed = Some(42);
    p.drums.density = 0.7;
    p
}

#[derive(Debug)]
struct MidiNoteHit {
    track: usize,
    channel: u8,
    tick: u64,
    midi: u8,
}

fn collect_smf_notes(bytes: &[u8]) -> Vec<MidiNoteHit> {
    let smf = Smf::parse(bytes).expect("valid smf");
    let mut out = Vec::new();
    for (track_idx, track) in smf.tracks.iter().enumerate() {
        let mut abs = 0u64;
        let mut channel = 0u8;
        for ev in track {
            abs += u64::from(ev.delta.as_int());
            if let TrackEventKind::Midi { channel: ch, message } = ev.kind {
                channel = ch.as_int();
                if let midly::MidiMessage::NoteOn { key, vel } = message {
                    if vel.as_int() > 0 {
                        out.push(MidiNoteHit {
                            track: track_idx,
                            channel,
                            tick: abs,
                            midi: key.as_int(),
                        });
                    }
                }
            }
        }
    }
    out.sort_by_key(|n| (n.tick, n.channel, n.midi));
    out
}

fn ir_kick_ticks(ir: &aurora_export::ir::MusicIr) -> Vec<u64> {
    ir.events
        .iter()
        .filter_map(|e| match e {
            aurora_export::ir::IrEvent::Note(n) if n.is_drum && n.midi == 36 => Some(n.base.tick),
            _ => None,
        })
        .collect()
}

fn ir_melody_downbeat_ticks(ir: &aurora_export::ir::MusicIr) -> Vec<u64> {
    let melody_channel = ir
        .channel_index
        .iter()
        .find(|v| !v.is_drum && v.name == "Melody")
        .map(|v| v.channel)
        .expect("melody channel");
    let ppq = ir.header.ppq as u64;
    ir.events
        .iter()
        .filter_map(|e| match e {
            aurora_export::ir::IrEvent::Note(n)
                if n.base.channel == melody_channel && n.base.tick % ppq == 0 =>
            {
                Some(n.base.tick)
            }
            _ => None,
        })
        .collect()
}

fn ast_drum_kick_offsets(comp: &aurora_ast::Composition, drums_voice: aurora_ast::VoiceId) -> Vec<(u32, aurora_ast::BeatOffset)> {
    let mut out = Vec::new();
    for movement in &comp.movements {
        for section in &movement.sections {
            for phrase in &section.phrases {
                for measure in &phrase.measures {
                    if let Some(voice) = measure.voices.iter().find(|v| v.voice_id == drums_voice) {
                        for event in &voice.events {
                            if let aurora_ast::Event::Note(n) = event {
                                if n.is_drum && n.pitch.midi == 36 {
                                    out.push((measure.number.global, n.base.offset));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

#[test]
fn exported_midi_drums_align_with_ir_and_ast() {
    let comp = generate_composition(ui_default_params()).expect("generate");
    let drums_voice = comp
        .voice_registry
        .voices
        .iter()
        .find(|v| v.role == aurora_ast::VoiceRole::Drums)
        .expect("drums")
        .id;

    let ir = project_ast_to_ir(&comp, DEFAULT_PPQ).unwrap();
    let bytes = export_midi(&ir, &MidiExportConfig::default()).unwrap();
    let smf = Smf::parse(&bytes).unwrap();
    assert_eq!(smf.header.format, Format::Parallel);

    let ppq = match smf.header.timing {
        Timing::Metrical(t) => t.as_int(),
        _ => panic!("expected metrical timing"),
    };
    assert_eq!(ppq, DEFAULT_PPQ);

    let hits = collect_smf_notes(&bytes);
    let midi_kicks: Vec<u64> = hits
        .iter()
        .filter(|h| h.channel == 9 && h.midi == 36)
        .map(|h| h.tick)
        .collect();
    let ir_kicks = ir_kick_ticks(&ir);

    eprintln!("AST kick offsets (measure, offset): {:?}", ast_drum_kick_offsets(&comp, drums_voice));
    eprintln!("IR kick ticks: {:?}", &ir_kicks[..ir_kicks.len().min(8)]);
    eprintln!("MIDI kick ticks: {:?}", &midi_kicks[..midi_kicks.len().min(8)]);

    assert!(!ir_kicks.is_empty(), "expected kicks in IR");
    assert!(!midi_kicks.is_empty(), "expected kicks in exported MIDI");
    assert_eq!(
        ir_kicks, midi_kicks,
        "MIDI kick ticks must match IR projection"
    );

    // Every IR kick should land on a quarter-note grid (beat 1 / 3 in 4/4).
    for tick in &ir_kicks {
        assert_eq!(
            tick % (ppq as u64),
            0,
            "kick tick {tick} not on quarter grid"
        );
    }

    // First melody downbeat and first kick share tick 0.
    let melody_downbeats = ir_melody_downbeat_ticks(&ir);
    assert_eq!(melody_downbeats.first(), Some(&0), "melody should start at tick 0");
    assert_eq!(ir_kicks.first(), Some(&0), "kick should start at tick 0");

    // Hi-hat closed (42) should sit on 16th grid in both IR and MIDI.
    let ir_hats: Vec<u64> = ir
        .events
        .iter()
        .filter_map(|e| match e {
            aurora_export::ir::IrEvent::Note(n) if n.is_drum && n.midi == 42 => Some(n.base.tick),
            _ => None,
        })
        .take(16)
        .collect();
    let midi_hats: Vec<u64> = hits
        .iter()
        .filter(|h| h.channel == 9 && h.midi == 42)
        .map(|h| h.tick)
        .take(16)
        .collect();
    eprintln!("IR hi-hat ticks: {:?}", ir_hats);
    eprintln!("MIDI hi-hat ticks: {:?}", midi_hats);
    assert_eq!(ir_hats, midi_hats, "hi-hat MIDI must match IR");

    let sixteenth = ppq as u64 / 4;
    for tick in &ir_hats {
        assert_eq!(
            tick % sixteenth,
            0,
            "hi-hat tick {tick} not on 16th grid (sixteenth={sixteenth})"
        );
    }
}

#[test]
#[ignore = "manual: writes target/test-export.mid for ui/scripts/verify-midi-timing.mjs"]
fn write_midi_fixture() {
    use std::path::Path;
    let comp = generate_composition(ui_default_params()).expect("generate");
    let ir = project_ast_to_ir(&comp, DEFAULT_PPQ).unwrap();
    let bytes = export_midi(&ir, &MidiExportConfig::default()).unwrap();
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/test-export.mid");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, &bytes).unwrap();
    eprintln!("wrote {}", path.display());
}
