//! Atomic patch algebra for AST mutations.
//!
//! See `docs/02-music-model/ast.md` §9.4.

use aurora_core::{AuroraError, NodeId};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::events::Event;
use crate::nodes::{
    Composition, HarmonySlot, Measure, MeasureVoice, Movement, Phrase, Section, VoiceId,
};
use crate::provenance::{PatchId, ProvenanceAgent};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatchRecord {
    pub patch: Patch,
    pub inverse: Patch,
    pub timestamp: String,
    pub agent: ProvenanceAgent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Patch {
    pub id: PatchId,
    pub ops: Vec<PatchOp>,
    pub inverse: Option<PatchId>,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOp {
    InsertNode {
        parent: NodeId,
        index: usize,
        node: AstNodePayload,
    },
    DeleteNode {
        node_id: NodeId,
    },
    ReplaceNode {
        node_id: NodeId,
        node: AstNodePayload,
    },
    MoveNode {
        node_id: NodeId,
        new_parent: NodeId,
        new_index: usize,
    },
    UpdateField {
        node_id: NodeId,
        path: FieldPath,
        value: Value,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldPath(pub Vec<String>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "node")]
pub enum AstNodePayload {
    Event(Event),
    HarmonySlot(HarmonySlot),
    MeasureVoice(MeasureVoice),
    Measure(Measure),
    Phrase(Phrase),
    Section(Section),
    Movement(Movement),
}

/// Apply all patch operations atomically; returns updated composition on success.
pub fn apply_patch(comp: &Composition, patch: &Patch) -> Result<Composition, AuroraError> {
    let mut snapshot = comp.clone();
    for op in &patch.ops {
        apply_op(&mut snapshot, op)?;
    }
    Ok(snapshot)
}

fn apply_op(comp: &mut Composition, op: &PatchOp) -> Result<(), AuroraError> {
    match op {
        PatchOp::InsertNode {
            parent,
            index,
            node,
        } => insert_node(comp, *parent, *index, node),
        PatchOp::DeleteNode { node_id } => delete_node(comp, *node_id),
        PatchOp::ReplaceNode { node_id, node } => replace_node(comp, *node_id, node),
        PatchOp::MoveNode { .. } | PatchOp::UpdateField { .. } => Err(AuroraError::PatchFailed(
            "MoveNode and UpdateField are not yet implemented".into(),
        )),
    }
}

fn insert_node(
    comp: &mut Composition,
    parent: NodeId,
    index: usize,
    node: &AstNodePayload,
) -> Result<(), AuroraError> {
    if let Some(measure) = find_measure_mut(comp, parent) {
        return match node {
            AstNodePayload::Event(event) => {
                if let Some(mv) = measure.voices.get_mut(index) {
                    mv.events.push(event.clone());
                    Ok(())
                } else {
                    Err(AuroraError::PatchFailed(format!(
                        "voice index {index} out of range for measure {}",
                        measure.id.index
                    )))
                }
            }
            AstNodePayload::HarmonySlot(slot) => {
                measure.harmony_slots.insert(index, slot.clone());
                Ok(())
            }
            AstNodePayload::MeasureVoice(mv) => {
                measure.voices.insert(index, mv.clone());
                Ok(())
            }
            _ => Err(AuroraError::PatchFailed(
                "unsupported node type for measure parent".into(),
            )),
        };
    }

    if let Some(section) = find_section_mut(comp, parent) {
        if let AstNodePayload::Phrase(phrase) = node {
            section.phrases.insert(index, phrase.clone());
            return Ok(());
        }
    }

    if let Some(movement) = find_movement_mut(comp, parent) {
        if let AstNodePayload::Section(section) = node {
            movement.sections.insert(index, section.clone());
            return Ok(());
        }
    }

    if comp.id == parent {
        if let AstNodePayload::Movement(movement) = node {
            comp.movements.insert(index, movement.clone());
            return Ok(());
        }
    }

    Err(AuroraError::PatchFailed(format!(
        "parent node {} not found or incompatible payload",
        parent.index
    )))
}

fn delete_node(comp: &mut Composition, node_id: NodeId) -> Result<(), AuroraError> {
    for movement in &mut comp.movements {
        for section in &mut movement.sections {
            if section.id == node_id {
                return Err(AuroraError::PatchFailed(
                    "deleting sections via patch not supported in v0.1".into(),
                ));
            }
            for phrase in &mut section.phrases {
                if phrase.id == node_id {
                    return Err(AuroraError::PatchFailed(
                        "deleting phrases via patch not supported in v0.1".into(),
                    ));
                }
                for measure in &mut phrase.measures {
                    if measure.id == node_id {
                        return Err(AuroraError::PatchFailed(
                            "deleting measures via patch not supported in v0.1".into(),
                        ));
                    }
                    measure.harmony_slots.retain(|s| s.id != node_id);
                    for mv in &mut measure.voices {
                        mv.events.retain(|e| e.id() != node_id);
                    }
                }
            }
        }
    }
    Ok(())
}

fn replace_node(
    comp: &mut Composition,
    node_id: NodeId,
    node: &AstNodePayload,
) -> Result<(), AuroraError> {
    for movement in &mut comp.movements {
        for section in &mut movement.sections {
            for phrase in &mut section.phrases {
                for measure in &mut phrase.measures {
                    if measure.id == node_id {
                        if let AstNodePayload::Measure(replacement) = node {
                            *measure = replacement.clone();
                            return Ok(());
                        }
                    }
                    for (i, slot) in measure.harmony_slots.iter().enumerate() {
                        if slot.id == node_id {
                            if let AstNodePayload::HarmonySlot(replacement) = node {
                                measure.harmony_slots[i] = replacement.clone();
                                return Ok(());
                            }
                        }
                    }
                    for mv in &mut measure.voices {
                        for (i, event) in mv.events.iter().enumerate() {
                            if event.id() == node_id {
                                if let AstNodePayload::Event(replacement) = node {
                                    mv.events[i] = replacement.clone();
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Err(AuroraError::PatchFailed(format!(
        "node {} not found for replace",
        node_id.index
    )))
}

fn find_measure_mut<'a>(comp: &'a mut Composition, id: NodeId) -> Option<&'a mut Measure> {
    for movement in &mut comp.movements {
        for section in &mut movement.sections {
            for phrase in &mut section.phrases {
                for measure in &mut phrase.measures {
                    if measure.id == id {
                        return Some(measure);
                    }
                }
            }
        }
    }
    None
}

fn find_section_mut<'a>(comp: &'a mut Composition, id: NodeId) -> Option<&'a mut Section> {
    for movement in &mut comp.movements {
        for section in &mut movement.sections {
            if section.id == id {
                return Some(section);
            }
        }
    }
    None
}

fn find_movement_mut<'a>(comp: &'a mut Composition, id: NodeId) -> Option<&'a mut Movement> {
    for movement in &mut comp.movements {
        if movement.id == id {
            return Some(movement);
        }
    }
    None
}

/// Insert an event into a measure voice by indices.
pub fn patch_insert_event(
    comp: &Composition,
    measure_id: NodeId,
    voice_id: VoiceId,
    event: Event,
) -> Result<Composition, AuroraError> {
    let patch = Patch {
        id: PatchId(1),
        ops: vec![PatchOp::InsertNode {
            parent: measure_id,
            index: voice_index(comp, measure_id, voice_id)?,
            node: AstNodePayload::Event(event),
        }],
        inverse: None,
        description: "insert event".into(),
    };
    apply_patch(comp, &patch)
}

fn voice_index(comp: &Composition, measure_id: NodeId, voice_id: VoiceId) -> Result<usize, AuroraError> {
    for movement in &comp.movements {
        for section in &movement.sections {
            for phrase in &section.phrases {
                for measure in &phrase.measures {
                    if measure.id == measure_id {
                        for (i, mv) in measure.voices.iter().enumerate() {
                            if mv.voice_id == voice_id {
                                return Ok(i);
                            }
                        }
                        return Err(AuroraError::PatchFailed(format!(
                            "voice {:?} not in measure",
                            voice_id.0
                        )));
                    }
                }
            }
        }
    }
    Err(AuroraError::PatchFailed(format!(
        "measure {} not found",
        measure_id.index
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::CompositionBuilder;
    use crate::events::{Event, NoteEvent, TieSpec};
    use crate::provenance::{PipelineStageId, Provenance};
    use crate::types::{BeatOffset, NoteType, Pitch, WrittenDuration};

    fn sample_note(id: u64) -> Event {
        Event::Note(NoteEvent {
            base: crate::events::TimedEventBase {
                id: NodeId::new(id, 0),
                offset: BeatOffset::zero(),
                duration: WrittenDuration {
                    note_type: NoteType::Quarter,
                    dots: 0,
                    tuplet: None,
                },
                provenance: Provenance::generated(PipelineStageId::Melody, "2026-01-01"),
                visible: true,
            },
            pitch: Pitch::from_midi(62),
            velocity: 80,
            tie: TieSpec::None,
            articulations: vec![],
            ornaments: vec![],
            lyric: None,
            pitch_role: None,
            stem_direction: None,
            beam_group: None,
            is_drum: false,
            drum_map: None,
        })
    }

    #[test]
    fn apply_patch_inserts_event_into_measure_voice() {
        let comp = CompositionBuilder::new().one_measure().build();
        let measure_id = comp.movements[0].sections[0].phrases[0].measures[0].id;
        let note = sample_note(100);
        let updated = patch_insert_event(&comp, measure_id, VoiceId(0), note).unwrap();
        let events = &updated.movements[0].sections[0].phrases[0].measures[0].voices[0].events;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id().index, 100);
    }

    #[test]
    fn apply_patch_is_atomic_on_failure() {
        let comp = CompositionBuilder::new().one_measure().build();
        let patch = Patch {
            id: PatchId(2),
            ops: vec![
                PatchOp::InsertNode {
                    parent: NodeId::new(9999, 0),
                    index: 0,
                    node: AstNodePayload::Event(sample_note(1)),
                },
            ],
            inverse: None,
            description: "bad".into(),
        };
        assert!(apply_patch(&comp, &patch).is_err());
    }
}
