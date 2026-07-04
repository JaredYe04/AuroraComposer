use std::collections::HashMap;

use aurora_ast::{
    events::{Event, NoteEvent, RestEvent},
    nodes::{Composition, KeySignature, Mode, TimeSignature, VoiceDef, VoiceRole},
    types::{BeatOffset, NoteType, WrittenDuration},
    VoiceId,
};
use aurora_core::{ExportError, NodeId};
use chrono::Utc;
use serde::{Deserialize, Serialize};

pub const DEFAULT_PPQ: u16 = 480;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MusicIr {
    pub header: IrHeader,
    pub timeline: ResolvedTimeline,
    pub events: Vec<IrEvent>,
    pub channel_index: Vec<ChannelView>,
    pub voice_map: HashMap<VoiceId, ChannelId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrHeader {
    pub ppq: u16,
    pub total_ticks: u64,
    pub total_seconds: f64,
    pub title: String,
    pub projection_version: u16,
    pub projected_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolvedTimeline {
    pub tempo_events: Vec<IrTempoEvent>,
    pub meter_events: Vec<IrMeterEvent>,
    pub key_events: Vec<IrKeyEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrTempoEvent {
    pub tick: u64,
    pub bpm: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrMeterEvent {
    pub tick: u64,
    pub beats: u8,
    pub beat_type: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrKeyEvent {
    pub tick: u64,
    pub fifths: i8,
    pub mode: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChannelId(pub u8);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelView {
    pub channel: ChannelId,
    pub voice_id: VoiceId,
    pub name: String,
    pub program: u8,
    pub event_indices: Vec<usize>,
    pub is_drum: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IrEvent {
    Note(IrNote),
    Rest(IrRest),
    TempoChange(IrTempoChange),
    TimeSignatureChange(IrTimeSignatureChange),
    KeyChange(IrKeyChange),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrEventBase {
    pub tick: u64,
    pub channel: ChannelId,
    pub voice_id: VoiceId,
    pub ast_node_id: NodeId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrNote {
    pub base: IrEventBase,
    pub midi: u8,
    pub velocity: u8,
    pub duration_ticks: u32,
    pub end_tick: u64,
    pub is_drum: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrRest {
    pub base: IrEventBase,
    pub duration_ticks: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrTempoChange {
    pub tick: u64,
    pub bpm: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrTimeSignatureChange {
    pub tick: u64,
    pub beats: u8,
    pub beat_type: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrKeyChange {
    pub tick: u64,
    pub fifths: i8,
    pub mode: String,
}

pub fn project_ast_to_ir(comp: &Composition, ppq: u16) -> Result<MusicIr, ExportError> {
    if comp.movements.is_empty() {
        return Err(ExportError::Projection("composition has no movements".into()));
    }

    let default_meter = comp.global.meter_map.default;
    let measure_ticks = (measure_quarters(default_meter) * f64::from(ppq)).round() as u64;

    let mut events = Vec::new();
    let mut voice_map = HashMap::new();
    let voice_lookup: HashMap<VoiceId, &VoiceDef> = comp
        .voice_registry
        .voices
        .iter()
        .map(|v| (v.id, v))
        .collect();

    for voice in &comp.voice_registry.voices {
        let channel = drum_channel(voice);
        voice_map.insert(voice.id, channel);
    }

    let mut global_measure = 0u32;
    for movement in &comp.movements {
        for section in &movement.sections {
            for phrase in &section.phrases {
                for measure in &phrase.measures {
                    let measure_start_tick = u64::from(global_measure) * measure_ticks;
                    let meter = measure.attributes.meter.unwrap_or(default_meter);

                    if global_measure == 0 || measure.attributes.meter.is_some() {
                        events.push(IrEvent::TimeSignatureChange(IrTimeSignatureChange {
                            tick: measure_start_tick,
                            beats: meter.beats,
                            beat_type: meter.beat_type,
                        }));
                    }

                    if let Some(key) = measure.attributes.key {
                        events.push(IrEvent::KeyChange(IrKeyChange {
                            tick: measure_start_tick,
                            fifths: key_fifths(key),
                            mode: mode_label(key.mode).into(),
                        }));
                    }

                    for mv in &measure.voices {
                        voice_lookup.get(&mv.voice_id).ok_or_else(|| {
                            ExportError::Projection(format!("unknown voice {:?}", mv.voice_id))
                        })?;
                        let channel = voice_map[&mv.voice_id];

                        for event in &mv.events {
                            match event {
                                Event::Note(note) => {
                                    events.push(IrEvent::Note(note_to_ir(
                                        note,
                                        measure_start_tick,
                                        channel,
                                        mv.voice_id,
                                        ppq,
                                    )));
                                }
                                Event::Rest(rest) => {
                                    events.push(IrEvent::Rest(rest_to_ir(
                                        rest,
                                        measure_start_tick,
                                        channel,
                                        mv.voice_id,
                                        ppq,
                                        meter,
                                    )));
                                }
                                Event::Chord(chord) => {
                                    for tone in &chord.pitches {
                                        let synthetic = NoteEvent {
                                            base: chord.base.clone(),
                                            pitch: tone.pitch,
                                            velocity: chord.velocity,
                                            tie: aurora_ast::TieSpec::None,
                                            articulations: chord.articulations.clone(),
                                            ornaments: vec![],
                                            lyric: None,
                                            pitch_role: None,
                                            stem_direction: None,
                                            beam_group: None,
                                            is_drum: false,
                                            drum_map: None,
                                        };
                                        events.push(IrEvent::Note(note_to_ir(
                                            &synthetic,
                                            measure_start_tick,
                                            channel,
                                            mv.voice_id,
                                            ppq,
                                        )));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    global_measure += 1;
                }
            }
        }
    }

    let tempo_bpm = comp.global.tempo_map.default_bpm;
    events.push(IrEvent::TempoChange(IrTempoChange { tick: 0, bpm: tempo_bpm }));
    events.sort_by_key(|e| (event_tick(e), event_channel(e).unwrap_or(ChannelId(0))));

    let total_ticks = u64::from(global_measure) * measure_ticks;
    let total_seconds = tick_to_seconds(total_ticks, tempo_bpm, ppq);
    let channel_index = build_channel_index(&events, comp);

    let default_key = comp.global.key_map.default;
    let mut key_events = vec![IrKeyEvent {
        tick: 0,
        fifths: key_fifths(default_key),
        mode: mode_label(default_key.mode).into(),
    }];
    for event in &events {
        if let IrEvent::KeyChange(k) = event {
            key_events.push(IrKeyEvent {
                tick: k.tick,
                fifths: k.fifths,
                mode: k.mode.clone(),
            });
        }
    }
    key_events.sort_by_key(|k| k.tick);
    key_events.dedup_by_key(|k| k.tick);

    let mut meter_events = vec![IrMeterEvent {
        tick: 0,
        beats: default_meter.beats,
        beat_type: default_meter.beat_type,
    }];
    for event in &events {
        if let IrEvent::TimeSignatureChange(t) = event {
            meter_events.push(IrMeterEvent {
                tick: t.tick,
                beats: t.beats,
                beat_type: t.beat_type,
            });
        }
    }
    meter_events.sort_by_key(|m| m.tick);
    meter_events.dedup_by_key(|m| m.tick);

    let timeline = ResolvedTimeline {
        tempo_events: vec![IrTempoEvent { tick: 0, bpm: tempo_bpm }],
        meter_events,
        key_events,
    };

    Ok(MusicIr {
        header: IrHeader {
            ppq,
            total_ticks,
            total_seconds,
            title: comp.metadata.title.clone(),
            projection_version: 1,
            projected_at: Utc::now().to_rfc3339(),
        },
        timeline,
        events,
        channel_index,
        voice_map,
    })
}

fn note_to_ir(
    note: &NoteEvent,
    measure_start: u64,
    channel: ChannelId,
    voice_id: VoiceId,
    ppq: u16,
) -> IrNote {
    let offset_ticks = (beat_offset_quarters(note.base.offset) * f64::from(ppq)).round() as u64;
    let duration_ticks = written_duration_ticks(&note.base.duration, ppq);
    let start = measure_start + offset_ticks;
    IrNote {
        base: IrEventBase {
            tick: start,
            channel,
            voice_id,
            ast_node_id: note.base.id,
        },
        midi: note.pitch.midi,
        velocity: note.velocity,
        duration_ticks,
        end_tick: start + u64::from(duration_ticks),
        is_drum: note.is_drum,
    }
}

fn rest_to_ir(
    rest: &RestEvent,
    measure_start: u64,
    channel: ChannelId,
    voice_id: VoiceId,
    ppq: u16,
    meter: TimeSignature,
) -> IrRest {
    let offset_ticks = (beat_offset_quarters(rest.base.offset) * f64::from(ppq)).round() as u64;
    let duration_ticks = match rest.rest_type {
        aurora_ast::events::RestType::Measure => {
            (measure_quarters(meter) * f64::from(ppq)).round() as u32
        }
        aurora_ast::events::RestType::Normal => written_duration_ticks(&rest.base.duration, ppq),
        aurora_ast::events::RestType::MultiMeasure(n) => {
            (measure_quarters(meter) * f64::from(n) * f64::from(ppq)).round() as u32
        }
    };
    IrRest {
        base: IrEventBase {
            tick: measure_start + offset_ticks,
            channel,
            voice_id,
            ast_node_id: rest.base.id,
        },
        duration_ticks,
    }
}

fn build_channel_index(events: &[IrEvent], comp: &Composition) -> Vec<ChannelView> {
    comp.voice_registry
        .voices
        .iter()
        .map(|voice| {
            let channel = drum_channel(voice);
            let event_indices: Vec<usize> = events
                .iter()
                .enumerate()
                .filter(|(_, e)| event_channel(e) == Some(channel))
                .map(|(i, _)| i)
                .collect();
            ChannelView {
                channel,
                voice_id: voice.id,
                name: voice.name.clone(),
                program: voice.instrument.gm_program,
                event_indices,
                is_drum: is_drum_voice(voice),
            }
        })
        .collect()
}

fn drum_channel(voice: &VoiceDef) -> ChannelId {
    if is_drum_voice(voice) {
        ChannelId(9)
    } else {
        ChannelId(voice.midi_channel.saturating_sub(1).min(15))
    }
}

fn is_drum_voice(voice: &VoiceDef) -> bool {
    matches!(voice.role, VoiceRole::Drums | VoiceRole::Percussion)
}

fn beat_offset_quarters(offset: BeatOffset) -> f64 {
    if offset.denom == 0 {
        return 0.0;
    }
    offset.numer as f64 / offset.denom as f64
}

fn written_duration_ticks(duration: &WrittenDuration, ppq: u16) -> u32 {
    (written_duration_quarters(duration) * f64::from(ppq)).round() as u32
}

fn written_duration_quarters(duration: &WrittenDuration) -> f64 {
    let base = match duration.note_type {
        NoteType::Maxima => 32.0,
        NoteType::Longa => 16.0,
        NoteType::Breve => 8.0,
        NoteType::Whole => 4.0,
        NoteType::Half => 2.0,
        NoteType::Quarter => 1.0,
        NoteType::Eighth => 0.5,
        NoteType::Sixteenth => 0.25,
        NoteType::ThirtySecond => 0.125,
        NoteType::SixtyFourth => 0.0625,
        NoteType::OneHundredTwentyEighth => 0.03125,
    };
    let mut dur = base;
    let mut dot_value = base / 2.0;
    for _ in 0..duration.dots {
        dur += dot_value;
        dot_value /= 2.0;
    }
    if let Some(tuplet) = duration.tuplet {
        dur *= f64::from(tuplet.normal) / f64::from(tuplet.actual);
    }
    dur
}

fn measure_quarters(meter: TimeSignature) -> f64 {
    f64::from(meter.beats) * (4.0 / f64::from(meter.beat_type))
}

fn key_fifths(key: KeySignature) -> i8 {
    let tonic_pc = match key.mode {
        Mode::NaturalMinor | Mode::HarmonicMinor | Mode::MelodicMinor => (key.tonic.pc + 3) % 12,
        _ => key.tonic.pc,
    };
    major_key_fifths(tonic_pc)
}

fn major_key_fifths(pc: u8) -> i8 {
    match pc {
        0 => 0,
        1 => -5,
        2 => 2,
        3 => -3,
        4 => 4,
        5 => -1,
        6 => -6,
        7 => 1,
        8 => -4,
        9 => 3,
        10 => -2,
        _ => -1,
    }
}

fn mode_label(mode: Mode) -> &'static str {
    match mode {
        Mode::Major => "major",
        Mode::NaturalMinor | Mode::HarmonicMinor | Mode::MelodicMinor => "minor",
        _ => "major",
    }
}

fn event_tick(event: &IrEvent) -> u64 {
    match event {
        IrEvent::Note(n) => n.base.tick,
        IrEvent::Rest(r) => r.base.tick,
        IrEvent::TempoChange(t) => t.tick,
        IrEvent::TimeSignatureChange(t) => t.tick,
        IrEvent::KeyChange(k) => k.tick,
    }
}

fn event_channel(event: &IrEvent) -> Option<ChannelId> {
    match event {
        IrEvent::Note(n) => Some(n.base.channel),
        IrEvent::Rest(r) => Some(r.base.channel),
        _ => None,
    }
}

fn tick_to_seconds(tick: u64, bpm: f64, ppq: u16) -> f64 {
    if bpm <= 0.0 {
        return 0.0;
    }
    tick as f64 / (f64::from(ppq) * bpm / 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::sample_composition;

    #[test]
    fn project_ast_to_ir_produces_notes() {
        let comp = sample_composition();
        let ir = project_ast_to_ir(&comp, DEFAULT_PPQ).unwrap();
        assert_eq!(ir.header.ppq, 480);
        assert!(ir.events.iter().any(|e| matches!(e, IrEvent::Note(_))));
        assert!(!ir.channel_index.is_empty());
    }

    #[test]
    fn note_tick_alignment() {
        let comp = sample_composition();
        let ir = project_ast_to_ir(&comp, DEFAULT_PPQ).unwrap();
        let note = ir
            .events
            .iter()
            .find_map(|e| match e {
                IrEvent::Note(n) => Some(n),
                _ => None,
            })
            .unwrap();
        assert_eq!(note.duration_ticks, 480);
    }
}
