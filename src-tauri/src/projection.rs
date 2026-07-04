//! Timeline and provenance projections for the Phase 3 UI.

use aurora_ast::{
    events::MarkerKind, CadenceType, Composition, Event, PipelineStageId, Provenance,
    ProvenanceSource, RuleRef, SectionRole,
};
use aurora_core::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct TimelineModel {
    pub total_measures: u32,
    pub sections: Vec<TimelineSection>,
    pub phrases: Vec<TimelinePhrase>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TimelineSection {
    pub id: NodeId,
    pub role: SectionRole,
    pub label: Option<String>,
    pub start_measure: u32,
    pub end_measure: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct TimelinePhrase {
    pub id: NodeId,
    pub section_id: NodeId,
    pub start_measure: u32,
    pub end_measure: u32,
    pub cadence: Option<CadenceType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventLocator {
    pub node_id: Option<NodeId>,
    pub measure_global: Option<u32>,
    pub voice_index: Option<u16>,
    pub event_index: Option<u16>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProvenanceChain {
    pub event_id: NodeId,
    pub event_summary: EventSummary,
    pub entries: Vec<ProvenanceChainEntry>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EventSummary {
    pub kind: String,
    pub pitch_display: Option<String>,
    pub duration_display: Option<String>,
    pub voice_name: String,
    pub measure_global: u32,
    pub beat_numer: u32,
    pub beat_denom: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProvenanceChainEntry {
    pub provenance: Provenance,
    pub rules: Vec<RuleDefinition>,
    pub display_summary: String,
    pub depth: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct RuleDefinition {
    pub id: String,
    pub display_id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub weight: f64,
    pub contribution_score: Option<f64>,
}

pub struct LocatedEvent {
    pub event: Event,
    pub voice_name: String,
    pub measure_global: u32,
}

pub fn build_timeline(comp: &Composition) -> TimelineModel {
    let mut sections = Vec::new();
    let mut phrases = Vec::new();
    let mut total_measures = 0u32;

    for movement in &comp.movements {
        for section in &movement.sections {
            let mut section_start = u32::MAX;
            let mut section_end = 0u32;

            for phrase in &section.phrases {
                if phrase.measures.is_empty() {
                    continue;
                }
                let phrase_start = phrase.measures[0].number.global;
                let phrase_end = phrase.measures[phrase.measures.len() - 1].number.global;
                section_start = section_start.min(phrase_start);
                section_end = section_end.max(phrase_end);
                total_measures = total_measures.max(phrase_end);

                phrases.push(TimelinePhrase {
                    id: phrase.id,
                    section_id: section.id,
                    start_measure: phrase_start,
                    end_measure: phrase_end,
                    cadence: phrase.metadata.cadence,
                });
            }

            if section_start != u32::MAX {
                sections.push(TimelineSection {
                    id: section.id,
                    role: section.metadata.role,
                    label: section.metadata.label.clone(),
                    start_measure: section_start,
                    end_measure: section_end,
                });
            }
        }
    }

    TimelineModel {
        total_measures,
        sections,
        phrases,
    }
}

pub fn find_event(comp: &Composition, node_id: NodeId) -> Option<LocatedEvent> {
    for movement in &comp.movements {
        for section in &movement.sections {
            for phrase in &section.phrases {
                for measure in &phrase.measures {
                    for mv in &measure.voices {
                        let voice_name = comp
                            .voice_registry
                            .voices
                            .iter()
                            .find(|v| v.id == mv.voice_id)
                            .map(|v| v.name.clone())
                            .unwrap_or_else(|| format!("Voice {}", mv.voice_id.0));

                        for event in &mv.events {
                            if event.id() == node_id {
                                return Some(LocatedEvent {
                                    event: event.clone(),
                                    voice_name,
                                    measure_global: measure.number.global,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn find_event_by_locator(comp: &Composition, locator: &EventLocator) -> Option<LocatedEvent> {
    if let Some(node_id) = locator.node_id {
        return find_event(comp, node_id);
    }

    let measure_global = locator.measure_global?;
    let voice_index = locator.voice_index.unwrap_or(0) as usize;
    let event_index = locator.event_index.unwrap_or(0) as usize;

    for movement in &comp.movements {
        for section in &movement.sections {
            for phrase in &section.phrases {
                for measure in &phrase.measures {
                    if measure.number.global != measure_global {
                        continue;
                    }
                    let mv = measure.voices.get(voice_index)?;
                    let event = mv.events.get(event_index)?;
                    let voice_name = comp
                        .voice_registry
                        .voices
                        .iter()
                        .find(|v| v.id == mv.voice_id)
                        .map(|v| v.name.clone())
                        .unwrap_or_else(|| format!("Voice {}", mv.voice_id.0));
                    return Some(LocatedEvent {
                        event: event.clone(),
                        voice_name,
                        measure_global,
                    });
                }
            }
        }
    }
    None
}

pub fn resolve_event_provenance(comp: &Composition, locator: &EventLocator) -> Option<Provenance> {
    find_event_by_locator(comp, locator)
        .map(|loc| loc.event.provenance().clone())
}

pub fn resolve_provenance_chain(
    comp: &Composition,
    locator: &EventLocator,
) -> Option<ProvenanceChain> {
    let located = find_event_by_locator(comp, locator)?;
    build_provenance_chain(comp, located.event.id())
}

pub fn build_provenance_chain(comp: &Composition, node_id: NodeId) -> Option<ProvenanceChain> {
    let located = find_event(comp, node_id)?;
    let mut entries = Vec::new();
    let mut current_id = Some(node_id);
    let mut depth = 0u32;

    while let Some(id) = current_id {
        let loc = find_event(comp, id)?;
        let prov = loc.event.provenance().clone();
        let rules: Vec<RuleDefinition> = if prov.rule_refs.is_empty() {
            prov.rule_ids
                .iter()
                .enumerate()
                .map(|(i, id)| {
                    let category = id.split('.').next().unwrap_or("rule").to_string();
                    RuleDefinition {
                        id: id.clone(),
                        display_id: format!("{} Rule #{}", capitalize(&category), i + 1),
                        name: id.replace('.', " ").replace('_', " "),
                        category,
                        description: String::new(),
                        weight: 1.0,
                        contribution_score: None,
                    }
                })
                .collect()
        } else {
            prov.rule_refs
                .iter()
                .map(rule_ref_to_definition)
                .collect()
        };
        let display_summary = format_provenance_summary(&prov, &rules);

        entries.push(ProvenanceChainEntry {
            provenance: prov.clone(),
            rules,
            display_summary,
            depth,
        });

        current_id = prov.parent.map(|p| p.node_id);
        depth += 1;
        if depth > 32 {
            break;
        }
    }

    Some(ProvenanceChain {
        event_id: node_id,
        event_summary: event_summary(&located),
        entries,
    })
}

fn event_summary(loc: &LocatedEvent) -> EventSummary {
    let (kind, pitch_display) = match &loc.event {
        Event::Note(n) => ("Note".into(), Some(midi_to_name(n.pitch.midi))),
        Event::Chord(c) => {
            let pitches = c
                .pitches
                .iter()
                .map(|p| midi_to_name(p.pitch.midi))
                .collect::<Vec<_>>()
                .join("+");
            ("Chord".into(), Some(pitches))
        }
        Event::Rest(_) => ("Rest".into(), None),
        Event::Marker(m) => {
            let label = match &m.marker {
                MarkerKind::RehearsalMark { label } => label.clone(),
                MarkerKind::SectionBoundary { label, .. } => {
                    label.clone().unwrap_or_else(|| "Section".into())
                }
                _ => "Marker".into(),
            };
            ("Marker".into(), Some(label))
        }
        Event::Automation(_) => ("Automation".into(), None),
    };

    let duration = loc.event.duration();
    let duration_display = Some(format!("{:?}", duration.note_type));

    EventSummary {
        kind,
        pitch_display,
        duration_display,
        voice_name: loc.voice_name.clone(),
        measure_global: loc.measure_global,
        beat_numer: loc.event.offset().numer,
        beat_denom: loc.event.offset().denom,
    }
}

fn midi_to_name(midi: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (i16::from(midi) / 12) - 1;
    let pc = usize::from(midi % 12);
    format!("{}{}", NAMES[pc], octave)
}

fn rule_ref_to_definition(r: &RuleRef) -> RuleDefinition {
    let category = r.id.split('.').next().unwrap_or("rule").to_string();
    let name = r
        .id
        .split('.')
        .nth(1)
        .unwrap_or(&r.id)
        .replace('_', " ");
    let display_num = r.id.len() as u32 % 100 + 1;
    RuleDefinition {
        id: r.id.clone(),
        display_id: format!(
            "{} Rule #{}",
            capitalize(&category),
            display_num
        ),
        name: capitalize(&name),
        category,
        description: String::new(),
        weight: r.weight.unwrap_or(1.0),
        contribution_score: r.score,
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn format_provenance_summary(prov: &Provenance, rules: &[RuleDefinition]) -> String {
    let rule_label = rules
        .first()
        .map(|r| r.display_id.clone())
        .or_else(|| {
            prov.stage.map(|s| format!("{} Engine", stage_label(s)))
        })
        .unwrap_or_else(|| match prov.source {
            ProvenanceSource::ManualEdit => "Manual Edit".into(),
            ProvenanceSource::Imported => "Import".into(),
            ProvenanceSource::Repaired => "Repair Engine".into(),
            ProvenanceSource::Plugin => "Plugin".into(),
            ProvenanceSource::Transformed => "Transform".into(),
            ProvenanceSource::Generated => "Unknown Rule".into(),
        });

    let score = prov
        .eval_score
        .map(|s| format!("{:+.0}", s))
        .unwrap_or_else(|| "—".into());
    let reason = prov
        .explanation
        .clone()
        .unwrap_or_else(|| "No explanation".into());

    format!("Generated by: {rule_label}, Score: {score}, Reason: {reason}")
}

fn stage_label(stage: PipelineStageId) -> &'static str {
    match stage {
        PipelineStageId::StyleResolver => "Style",
        PipelineStageId::EmotionResolver => "Emotion",
        PipelineStageId::StructurePlanning => "Structure",
        PipelineStageId::ThemePlanning => "Theme",
        PipelineStageId::HarmonySkeleton => "Harmony",
        PipelineStageId::RhythmSkeleton => "Rhythm",
        PipelineStageId::Melody => "Melody",
        PipelineStageId::Counterpoint => "Counterpoint",
        PipelineStageId::Bass => "Bass",
        PipelineStageId::Drums => "Drums",
        PipelineStageId::Decoration => "Decoration",
        PipelineStageId::Repair => "Repair",
        PipelineStageId::Manual => "Manual",
        PipelineStageId::Custom(_) => "Custom",
    }
}
