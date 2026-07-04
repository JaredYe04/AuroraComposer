pub mod abc;
pub mod ir;
pub mod midi;
pub mod musicxml;
pub mod pdf;

#[cfg(test)]
mod test_fixtures;

pub use abc::{export_abc, abc_voice_count_warning, AbcExportConfig, AbcExportMode};
pub use ir::{project_ast_to_ir, MusicIr, DEFAULT_PPQ};
pub use midi::{export_midi, export_midi_to_file, MidiExportConfig, SmfFormat};
pub use musicxml::{export_musicxml, MusicXmlExportConfig, MusicXmlProfile};
pub use pdf::{export_svg_preview, SvgPreviewResult};

use aurora_ast::Composition;
use aurora_core::ExportError;

/// Export pipeline entry points.
pub struct ExportPipeline;

impl ExportPipeline {
    pub fn ast_to_ir(comp: &Composition) -> Result<MusicIr, ExportError> {
        project_ast_to_ir(comp, DEFAULT_PPQ)
    }

    pub fn to_midi(comp: &Composition, config: &MidiExportConfig) -> Result<Vec<u8>, ExportError> {
        let ir = project_ast_to_ir(comp, config.tpqn)?;
        export_midi(&ir, config)
    }

    pub fn to_musicxml(
        comp: &Composition,
        config: &MusicXmlExportConfig,
    ) -> Result<String, ExportError> {
        let ir = project_ast_to_ir(comp, config.divisions_per_quarter as u16)?;
        export_musicxml(comp, &ir, config)
    }

    pub fn to_abc(comp: &Composition, config: &AbcExportConfig) -> Result<String, ExportError> {
        let ir = project_ast_to_ir(comp, DEFAULT_PPQ)?;
        export_abc(comp, &ir, config)
    }

    pub fn to_svg_preview(comp: &Composition) -> Result<SvgPreviewResult, ExportError> {
        let ir = project_ast_to_ir(comp, DEFAULT_PPQ)?;
        export_svg_preview(comp, &ir)
    }
}
