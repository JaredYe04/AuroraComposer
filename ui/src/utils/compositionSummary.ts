import type { Composition, CompositionSummary, Event } from '@/types/aurora';

function isNoteLike(event: Event): boolean {
  return event.kind === 'Note' || event.kind === 'Chord';
}

export function summaryFromComposition(comp: Composition): CompositionSummary {
  let bars = 0;
  let noteCount = 0;
  for (const movement of comp.movements) {
    for (const section of movement.sections) {
      for (const phrase of section.phrases) {
        bars += phrase.measures.length;
        for (const measure of phrase.measures) {
          for (const voice of measure.voices) {
            noteCount += voice.events.filter(isNoteLike).length;
          }
        }
      }
    }
  }
  return {
    title: comp.metadata.title,
    bars: Math.max(1, bars),
    voice_count: comp.voice_registry.voices.length,
    note_count: noteCount,
    tempo_bpm: comp.global.tempo_map.default_bpm,
    key: comp.global.key_map.default.tonic.pc,
  };
}
