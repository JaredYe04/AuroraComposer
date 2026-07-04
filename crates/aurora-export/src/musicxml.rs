use aurora_ast::Composition;
use aurora_core::ExportError;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event as XmlEvent};
use quick_xml::Writer;
use std::io::Cursor;

use crate::ir::{IrEvent, MusicIr};

const AURORA_NS: &str = "http://aurora-composer.dev/ns/v1";

#[derive(Clone, Debug)]
pub enum MusicXmlProfile {
    Interchange,
    Publish,
    Preview,
    Full,
}

#[derive(Clone, Debug)]
pub struct MusicXmlExportConfig {
    pub profile: MusicXmlProfile,
    pub divisions_per_quarter: u32,
    pub include_provenance: bool,
    pub pretty_print: bool,
}

impl Default for MusicXmlExportConfig {
    fn default() -> Self {
        Self {
            profile: MusicXmlProfile::Interchange,
            divisions_per_quarter: 480,
            include_provenance: true,
            pretty_print: false,
        }
    }
}

pub fn export_musicxml(
    comp: &Composition,
    ir: &MusicIr,
    config: &MusicXmlExportConfig,
) -> Result<String, ExportError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    writer
        .write_event(XmlEvent::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(xml_err)?;

    let mut root = BytesStart::new("score-partwise");
    root.push_attribute(("version", "4.0"));
    root.push_attribute(("xmlns:aurora", AURORA_NS));
    writer.write_event(XmlEvent::Start(root)).map_err(xml_err)?;

    write_identification(&mut writer, comp)?;
    write_defaults(&mut writer, config)?;
    write_part_list(&mut writer, ir)?;

    for view in &ir.channel_index {
        write_part(&mut writer, comp, ir, view, config)?;
    }

    writer
        .write_event(XmlEvent::End(BytesEnd::new("score-partwise")))
        .map_err(xml_err)?;

    let bytes = writer.into_inner().into_inner();
    String::from_utf8(bytes).map_err(|e| ExportError::MusicXml(e.to_string()))
}

fn write_identification(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    comp: &Composition,
) -> Result<(), ExportError> {
    writer
        .write_event(XmlEvent::Start(BytesStart::new("identification")))
        .map_err(xml_err)?;

    writer
        .write_event(XmlEvent::Start(BytesStart::new("encoding")))
        .map_err(xml_err)?;
    write_text_el(writer, "software", "Aurora Composer 0.1")?;
    write_text_el(writer, "encoding-date", &comp.metadata.created_at)?;
    writer
        .write_event(XmlEvent::End(BytesEnd::new("encoding")))
        .map_err(xml_err)?;

    writer
        .write_event(XmlEvent::Start(BytesStart::new("miscellaneous")))
        .map_err(xml_err)?;
    write_misc_field(writer, "aurora-namespace", AURORA_NS)?;
    writer
        .write_event(XmlEvent::End(BytesEnd::new("miscellaneous")))
        .map_err(xml_err)?;

    if !comp.metadata.title.is_empty() {
        writer
            .write_event(XmlEvent::Start(BytesStart::new("work")))
            .map_err(xml_err)?;
        write_text_el(writer, "work-title", &comp.metadata.title)?;
        writer
            .write_event(XmlEvent::End(BytesEnd::new("work")))
            .map_err(xml_err)?;
    }

    writer
        .write_event(XmlEvent::End(BytesEnd::new("identification")))
        .map_err(xml_err)?;
    Ok(())
}

fn write_defaults(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    config: &MusicXmlExportConfig,
) -> Result<(), ExportError> {
    writer
        .write_event(XmlEvent::Start(BytesStart::new("defaults")))
        .map_err(xml_err)?;
    writer
        .write_event(XmlEvent::Start(BytesStart::new("scaling")))
        .map_err(xml_err)?;
    write_text_el(writer, "millimeters", "7")?;
    write_text_el(writer, "tenths", "40")?;
    writer
        .write_event(XmlEvent::End(BytesEnd::new("scaling")))
        .map_err(xml_err)?;

    // Tempo in sound element for playback
    writer
        .write_event(XmlEvent::Start(BytesStart::new("sound")))
        .map_err(xml_err)?;
    let _ = config;
    writer
        .write_event(XmlEvent::End(BytesEnd::new("sound")))
        .map_err(xml_err)?;

    writer
        .write_event(XmlEvent::End(BytesEnd::new("defaults")))
        .map_err(xml_err)?;
    Ok(())
}

fn write_part_list(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    ir: &MusicIr,
) -> Result<(), ExportError> {
    writer
        .write_event(XmlEvent::Start(BytesStart::new("part-list")))
        .map_err(xml_err)?;

    for (i, view) in ir.channel_index.iter().enumerate() {
        let part_id = format!("P{}", i + 1);
        let inst_id = format!("{part_id}-I1");

        writer
            .write_event(XmlEvent::Start(BytesStart::new("score-part").with_attributes([
                ("id", part_id.as_str()),
            ])))
            .map_err(xml_err)?;

        write_text_el(writer, "part-name", &view.name)?;

        writer
            .write_event(XmlEvent::Start(BytesStart::new("score-instrument").with_attributes([
                ("id", inst_id.as_str()),
            ])))
            .map_err(xml_err)?;
        write_text_el(writer, "instrument-name", &view.name)?;
        writer
            .write_event(XmlEvent::End(BytesEnd::new("score-instrument")))
            .map_err(xml_err)?;

        writer
            .write_event(XmlEvent::Start(BytesStart::new("midi-instrument").with_attributes([
                ("id", inst_id.as_str()),
            ])))
            .map_err(xml_err)?;
        write_text_el(
            writer,
            "midi-channel",
            &view.channel.0.saturating_add(1).to_string(),
        )?;
        write_text_el(writer, "midi-program", &(view.program + 1).to_string())?;
        writer
            .write_event(XmlEvent::End(BytesEnd::new("midi-instrument")))
            .map_err(xml_err)?;

        writer
            .write_event(XmlEvent::End(BytesEnd::new("score-part")))
            .map_err(xml_err)?;
    }

    writer
        .write_event(XmlEvent::End(BytesEnd::new("part-list")))
        .map_err(xml_err)?;
    Ok(())
}

fn write_part(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    comp: &Composition,
    ir: &MusicIr,
    view: &crate::ir::ChannelView,
    config: &MusicXmlExportConfig,
) -> Result<(), ExportError> {
    let part_idx = ir
        .channel_index
        .iter()
        .position(|v| v.voice_id == view.voice_id)
        .unwrap_or(0);
    let part_id = format!("P{}", part_idx + 1);

    writer
        .write_event(XmlEvent::Start(BytesStart::new("part").with_attributes([
            ("id", part_id.as_str()),
        ])))
        .map_err(xml_err)?;

    let ppq = ir.header.ppq;
    let measure_ticks =
        (measure_quarters(comp.global.meter_map.default) * f64::from(ppq)).round() as u64;
    let measure_count = if measure_ticks > 0 {
        (ir.header.total_ticks / measure_ticks).max(1)
    } else {
        1
    };

    for m in 0..measure_count {
        let measure_num = (m + 1).to_string();
        writer
            .write_event(XmlEvent::Start(BytesStart::new("measure").with_attributes([
                ("number", measure_num.as_str()),
            ])))
            .map_err(xml_err)?;

        if m == 0 {
            write_measure_attributes(writer, comp, config, ir)?;
        }

        let measure_start = m * measure_ticks;
        let measure_end = measure_start + measure_ticks;

        for &idx in &view.event_indices {
            if let IrEvent::Note(note) = &ir.events[idx] {
                if note.base.tick >= measure_start && note.base.tick < measure_end {
                    write_note(writer, note, config, comp)?;
                }
            }
        }

        writer
            .write_event(XmlEvent::End(BytesEnd::new("measure")))
            .map_err(xml_err)?;
    }

    writer
        .write_event(XmlEvent::End(BytesEnd::new("part")))
        .map_err(xml_err)?;
    Ok(())
}

fn write_measure_attributes(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    comp: &Composition,
    config: &MusicXmlExportConfig,
    ir: &MusicIr,
) -> Result<(), ExportError> {
    writer
        .write_event(XmlEvent::Start(BytesStart::new("attributes")))
        .map_err(xml_err)?;
    write_text_el(
        writer,
        "divisions",
        &config.divisions_per_quarter.to_string(),
    )?;

    writer
        .write_event(XmlEvent::Start(BytesStart::new("key")))
        .map_err(xml_err)?;
    let mode = match comp.global.key_map.default.mode {
        aurora_ast::nodes::Mode::Major => "major",
        aurora_ast::nodes::Mode::NaturalMinor
        | aurora_ast::nodes::Mode::HarmonicMinor
        | aurora_ast::nodes::Mode::MelodicMinor => "minor",
        _ => "major",
    };
    write_text_el(
        writer,
        "fifths",
        &ir.timeline.key_events.first().map(|k| k.fifths.to_string()).unwrap_or_else(|| "0".into()),
    )?;
    write_text_el(writer, "mode", mode)?;
    writer
        .write_event(XmlEvent::End(BytesEnd::new("key")))
        .map_err(xml_err)?;

    writer
        .write_event(XmlEvent::Start(BytesStart::new("time")))
        .map_err(xml_err)?;
    write_text_el(
        writer,
        "beats",
        &comp.global.meter_map.default.beats.to_string(),
    )?;
    write_text_el(
        writer,
        "beat-type",
        &comp.global.meter_map.default.beat_type.to_string(),
    )?;
    writer
        .write_event(XmlEvent::End(BytesEnd::new("time")))
        .map_err(xml_err)?;

    writer
        .write_event(XmlEvent::Start(BytesStart::new("clef")))
        .map_err(xml_err)?;
    write_text_el(writer, "sign", "G")?;
    write_text_el(writer, "line", "2")?;
    writer
        .write_event(XmlEvent::End(BytesEnd::new("clef")))
        .map_err(xml_err)?;

    writer
        .write_event(XmlEvent::End(BytesEnd::new("attributes")))
        .map_err(xml_err)?;
    Ok(())
}

fn write_note(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    note: &crate::ir::IrNote,
    config: &MusicXmlExportConfig,
    comp: &Composition,
) -> Result<(), ExportError> {
    let pitch = note_to_spelling(note.midi);
    let duration = note.duration_ticks;

    let mut note_el = BytesStart::new("note");
    let node_id = format!("{}-{}", note.base.ast_node_id.index, note.base.ast_node_id.generation);
    if config.include_provenance {
        note_el.push_attribute(("aurora:id", node_id.as_str()));
        note_el.push_attribute((
            "aurora:provenance",
            r#"{"v":1,"origin":"generation","stage":"melody"}"#,
        ));
    }
    writer.write_event(XmlEvent::Start(note_el)).map_err(xml_err)?;

    writer
        .write_event(XmlEvent::Start(BytesStart::new("pitch")))
        .map_err(xml_err)?;
    let step = pitch.0;
    write_text_el(writer, "step", step)?;
    if pitch.1 != 0 {
        write_text_el(writer, "alter", &pitch.1.to_string())?;
    }
    write_text_el(writer, "octave", &pitch.2.to_string())?;
    writer
        .write_event(XmlEvent::End(BytesEnd::new("pitch")))
        .map_err(xml_err)?;

    write_text_el(writer, "duration", &duration.to_string())?;
    write_text_el(writer, "type", duration_to_type(duration, config.divisions_per_quarter))?;

    if comp.global.tempo_map.default_bpm > 0.0 {
        writer
            .write_event(XmlEvent::Start(BytesStart::new("sound").with_attributes([(
                "tempo",
                comp.global.tempo_map.default_bpm.to_string().as_str(),
            )])))
            .map_err(xml_err)?;
        writer
            .write_event(XmlEvent::End(BytesEnd::new("sound")))
            .map_err(xml_err)?;
    }

    writer
        .write_event(XmlEvent::End(BytesEnd::new("note")))
        .map_err(xml_err)?;
    Ok(())
}

fn duration_to_type(duration: u32, divisions: u32) -> &'static str {
    let quarter = divisions;
    if duration >= quarter * 4 {
        "whole"
    } else if duration >= quarter * 2 {
        "half"
    } else if duration >= quarter {
        "quarter"
    } else if duration >= quarter / 2 {
        "eighth"
    } else {
        "16th"
    }
}

fn write_text_el(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    name: &str,
    text: &str,
) -> Result<(), ExportError> {
    writer
        .write_event(XmlEvent::Start(BytesStart::new(name)))
        .map_err(xml_err)?;
    writer
        .write_event(XmlEvent::Text(BytesText::new(text)))
        .map_err(xml_err)?;
    writer
        .write_event(XmlEvent::End(BytesEnd::new(name)))
        .map_err(xml_err)?;
    Ok(())
}

fn write_misc_field(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    name: &str,
    value: &str,
) -> Result<(), ExportError> {
    writer
        .write_event(XmlEvent::Start(BytesStart::new("miscellaneous-field").with_attributes([
            ("name", name),
        ])))
        .map_err(xml_err)?;
    writer
        .write_event(XmlEvent::Text(BytesText::new(value)))
        .map_err(xml_err)?;
    writer
        .write_event(XmlEvent::End(BytesEnd::new("miscellaneous-field")))
        .map_err(xml_err)?;
    Ok(())
}

fn xml_err(e: std::io::Error) -> ExportError {
    ExportError::MusicXml(e.to_string())
}

fn measure_quarters(meter: aurora_ast::TimeSignature) -> f64 {
    f64::from(meter.beats) * (4.0 / f64::from(meter.beat_type))
}

/// Returns (step, alter, octave) for MusicXML pitch element.
fn note_to_spelling(midi: u8) -> (&'static str, i8, i8) {
    let octave = (i16::from(midi) / 12) - 1;
    let semitone = midi % 12;
    let (step, alter) = match semitone {
        0 => ("C", 0),
        1 => ("C", 1),
        2 => ("D", 0),
        3 => ("D", 1),
        4 => ("E", 0),
        5 => ("F", 0),
        6 => ("F", 1),
        7 => ("G", 0),
        8 => ("G", 1),
        9 => ("A", 0),
        10 => ("A", 1),
        _ => ("B", 0),
    };
    (step, alter, octave as i8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{project_ast_to_ir, DEFAULT_PPQ};
    use crate::test_fixtures::sample_composition;

    #[test]
    fn export_musicxml_contains_score_partwise() {
        let comp = sample_composition();
        let ir = project_ast_to_ir(&comp, DEFAULT_PPQ).unwrap();
        let xml = export_musicxml(&comp, &ir, &MusicXmlExportConfig::default()).unwrap();
        assert!(xml.contains("score-partwise"));
        assert!(xml.contains("version=\"4.0\""));
        assert!(xml.contains("<part-name>Voice 0</part-name>"));
        assert!(xml.contains("<step>C</step>"));
    }
}
