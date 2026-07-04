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

/// Placeholder PDF bytes: valid PDF header wrapping SVG preview content.
///
/// Production engraving should use Verovio WASM on the frontend or MuseScore CLI.
pub fn export_pdf_bytes(comp: &Composition, ir: &MusicIr) -> Result<Vec<u8>, ExportError> {
    let preview = export_svg_preview(comp, ir)?;
    Ok(build_placeholder_pdf(&preview.svg, &preview.musicxml))
}

/// Minimal PDF 1.4 document embedding SVG as a comment stream (placeholder for Verovio).
fn build_placeholder_pdf(svg: &str, musicxml: &str) -> Vec<u8> {
    let title = "Aurora Composer Score Preview";
    let stream_body = format!(
        "BT /F1 12 Tf 50 750 Td ({title}) Tj ET\n\
         % SVG preview ({svg_len} bytes)\n\
         % MusicXML ({xml_len} bytes) — render with Verovio for engraving\n",
        title = escape_pdf_text(title),
        svg_len = svg.len(),
        xml_len = musicxml.len(),
    );

    let mut objects = Vec::new();
    objects.push(format!("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n"));
    objects.push(format!(
        "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n"
    ));
    objects.push(format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
         /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n"
    ));
    objects.push(format!(
        "4 0 obj\n<< /Length {} >>\nstream\n{stream_body}endstream\nendobj\n",
        stream_body.len()
    ));
    objects.push(
        "5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n".to_string(),
    );

    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n");
    let mut offsets = vec![0usize];
    for obj in &objects {
        offsets.push(pdf.len());
        pdf.extend_from_slice(obj.as_bytes());
    }
    let xref_start = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_start}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    pdf
}

fn escape_pdf_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
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

    #[test]
    fn pdf_bytes_has_valid_header() {
        let comp = sample_composition();
        let ir = project_ast_to_ir(&comp, DEFAULT_PPQ).unwrap();
        let bytes = export_pdf_bytes(&comp, &ir).unwrap();
        assert!(bytes.starts_with(b"%PDF-1.4"));
        assert!(bytes.ends_with(b"%%EOF\n") || bytes.ends_with(b"%%EOF"));
    }
}
