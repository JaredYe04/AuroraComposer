use aurora_ast::Composition;
use aurora_core::ExportError;

use crate::ir::{IrEvent, MusicIr};
use crate::musicxml::{export_musicxml, MusicXmlExportConfig, MusicXmlProfile};

/// Result of the v0.1 score preview pipeline.
#[derive(Clone, Debug)]
pub struct SvgPreviewResult {
    pub svg: String,
    pub musicxml: String,
    pub pdf_note: String,
}

/// Generate MusicXML (preview profile) and a simplified SVG staff preview from IR.
///
/// Full PDF engraving requires Verovio WASM (frontend) or MuseScore CLI (external).
pub fn export_svg_preview(
    comp: &Composition,
    ir: &MusicIr,
) -> Result<SvgPreviewResult, ExportError> {
    let musicxml = export_musicxml(
        comp,
        ir,
        &MusicXmlExportConfig {
            profile: MusicXmlProfile::Preview,
            include_provenance: false,
            ..Default::default()
        },
    )?;

    let svg = render_svg_from_ir(comp, ir);

    Ok(SvgPreviewResult {
        svg,
        musicxml,
        pdf_note: "PDF export requires Verovio WASM (in-app preview) or MuseScore CLI: \
                   MuseScore4 -o output.pdf score.musicxml"
            .into(),
    })
}

fn render_svg_from_ir(comp: &Composition, ir: &MusicIr) -> String {
    let width = 820.0;
    let staff_height = 80.0;
    let margin = 40.0;
    let ppq = f64::from(ir.header.ppq);
    let meter = comp.global.meter_map.default;
    let measure_ticks = measure_quarters(meter) * ppq;
    let measure_count = if measure_ticks > 0.0 {
        (f64::from(ir.header.total_ticks as u32) / measure_ticks).ceil().max(1.0) as u32
    } else {
        1
    };

    let view = ir
        .channel_index
        .iter()
        .find(|v| !v.is_drum)
        .or_else(|| ir.channel_index.first());

    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">\n\
  <rect width=\"100%\" height=\"100%\" fill=\"white\"/>\n\
  <text x=\"{margin}\" y=\"24\" font-family=\"Georgia, serif\" font-size=\"16\" fill=\"rgb(17,17,17)\">{title}</text>\n\
  <text x=\"{margin}\" y=\"42\" font-size=\"10\" fill=\"rgb(102,102,102)\">Aurora Composer — simplified preview (use Verovio for engraving)</text>\n",
        width = width,
        height = margin * 2.0 + staff_height + 30.0,
        margin = margin,
        title = xml_escape(&comp.metadata.title),
    );

    let staff_top = margin + 50.0;
    let staff_width = width - margin * 2.0;
    let measure_width = staff_width / f64::from(measure_count.max(1));

    for i in 0..5 {
        let y = staff_top + f64::from(i) * staff_height / 4.0;
        svg.push_str(&format!(
            "  <line x1=\"{margin}\" y1=\"{y:.1}\" x2=\"{x2:.1}\" y2=\"{y:.1}\" stroke=\"rgb(51,51,51)\" stroke-width=\"1\"/>\n",
            margin = margin,
            y = y,
            x2 = width - margin,
        ));
    }

    if let Some(view) = view {
        for &idx in &view.event_indices {
            if let IrEvent::Note(note) = &ir.events[idx] {
                let measure_idx = (f64::from(note.base.tick as u32) / measure_ticks).floor() as u32;
                let tick_in_measure = f64::from((note.base.tick % measure_ticks as u64) as u32);
                let x = margin
                    + f64::from(measure_idx) * measure_width
                    + (tick_in_measure / measure_ticks) * measure_width;
                let y = pitch_to_staff_y(note.midi, staff_top, staff_height);
                svg.push_str(&format!(
                    "  <ellipse cx=\"{x:.1}\" cy=\"{y:.1}\" rx=\"5\" ry=\"4\" fill=\"rgb(17,17,17)\"/>\n",
                ));
            }
        }
    }

    for m in 0..=measure_count {
        let x = margin + f64::from(m) * measure_width;
        svg.push_str(&format!(
            "  <line x1=\"{x:.1}\" y1=\"{top:.1}\" x2=\"{x:.1}\" y2=\"{bot:.1}\" stroke=\"rgb(153,153,153)\" stroke-width=\"1\"/>\n",
            top = staff_top,
            bot = staff_top + staff_height,
        ));
    }

    svg.push_str("</svg>");
    svg
}

fn pitch_to_staff_y(midi: u8, staff_top: f64, staff_height: f64) -> f64 {
    // Map MIDI around middle C (60) onto 5-line staff; step ≈ half line spacing.
    let middle_line = staff_top + staff_height / 2.0;
    let semitones_from_c4 = i16::from(midi) - 60;
    middle_line - f64::from(semitones_from_c4) * (staff_height / 16.0)
}

fn measure_quarters(meter: aurora_ast::TimeSignature) -> f64 {
    f64::from(meter.beats) * (4.0 / f64::from(meter.beat_type))
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{project_ast_to_ir, DEFAULT_PPQ};
    use crate::test_fixtures::sample_composition;

    #[test]
    fn svg_preview_contains_svg_root() {
        let comp = sample_composition();
        let ir = project_ast_to_ir(&comp, DEFAULT_PPQ).unwrap();
        let result = export_svg_preview(&comp, &ir).unwrap();
        assert!(result.svg.starts_with("<svg"));
        assert!(result.svg.contains("<ellipse"));
        assert!(result.musicxml.contains("score-partwise"));
        assert!(result.pdf_note.contains("Verovio"));
    }
}
