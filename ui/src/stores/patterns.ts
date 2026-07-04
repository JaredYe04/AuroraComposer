import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { CompositionSummary } from '@/types/aurora';

const PALETTE = [
  '#58a6ff',
  '#3fb950',
  '#d29922',
  '#f85149',
  '#a371f7',
  '#39c5cf',
  '#ff7b72',
  '#79c0ff',
];

function randomColor(): string {
  return PALETTE[Math.floor(Math.random() * PALETTE.length)] ?? '#58a6ff';
}

let patternCounter = 0;

export interface Pattern {
  id: string;
  name: string;
  color: string;
  bars: number;
  beatsPerMeasure: number;
  tempoBpm: number;
}

export interface CreatePatternOptions {
  name?: string;
  bars?: number;
  beatsPerMeasure?: number;
  tempoBpm?: number;
}

export const usePatternsStore = defineStore('patterns', () => {
  const patterns = ref<Pattern[]>([]);
  const activePatternId = ref<string | null>(null);

  function createPattern(opts: CreatePatternOptions = {}): Pattern {
    patternCounter += 1;
    const pattern: Pattern = {
      id: `pat-${Date.now()}-${patternCounter}`,
      name: opts.name ?? `Pattern ${patternCounter}`,
      color: randomColor(),
      bars: opts.bars ?? 8,
      beatsPerMeasure: opts.beatsPerMeasure ?? 4,
      tempoBpm: opts.tempoBpm ?? 120,
    };
    patterns.value = [...patterns.value, pattern];
    if (!activePatternId.value) {
      activePatternId.value = pattern.id;
    }
    return pattern;
  }

  function registerFromComposition(summary: CompositionSummary, beatsPerMeasure: number) {
    return createPattern({
      name: `Pattern ${patterns.value.length + 1}`,
      bars: summary.bars,
      beatsPerMeasure,
      tempoBpm: summary.tempo_bpm,
    });
  }

  function ensureDefault(opts: CreatePatternOptions = {}): Pattern {
    if (patterns.value.length > 0) {
      if (!activePatternId.value) {
        activePatternId.value = patterns.value[0]?.id ?? null;
      }
      return patterns.value[0]!;
    }
    return createPattern({
      name: opts.name ?? 'Pattern 1',
      bars: opts.bars ?? 8,
      beatsPerMeasure: opts.beatsPerMeasure ?? 4,
      tempoBpm: opts.tempoBpm ?? 120,
    });
  }

  function resetToDefault(opts: CreatePatternOptions = {}) {
    patterns.value = [];
    activePatternId.value = null;
    patternCounter = 0;
    return ensureDefault(opts);
  }

  function duplicatePattern(id: string): Pattern | null {
    const src = patterns.value.find((p) => p.id === id);
    if (!src) return null;
    patternCounter += 1;
    const copy: Pattern = {
      ...src,
      id: `pat-${Date.now()}-${patternCounter}`,
      name: `${src.name} copy`,
      color: randomColor(),
    };
    patterns.value = [...patterns.value, copy];
    activePatternId.value = copy.id;
    return copy;
  }

  function deletePattern(id: string) {
    if (patterns.value.length <= 1) return;
    patterns.value = patterns.value.filter((p) => p.id !== id);
    if (activePatternId.value === id) {
      activePatternId.value = patterns.value[0]?.id ?? null;
    }
  }

  function renamePattern(id: string, name: string) {
    const p = patterns.value.find((x) => x.id === id);
    if (p && name.trim()) p.name = name.trim();
  }

  function randomizeColor(id: string) {
    const p = patterns.value.find((x) => x.id === id);
    if (p) p.color = randomColor();
  }

  function setActivePattern(id: string | null) {
    activePatternId.value = id;
  }

  function clearAll() {
    patterns.value = [];
    activePatternId.value = null;
    patternCounter = 0;
  }

  return {
    patterns,
    activePatternId,
    createPattern,
    registerFromComposition,
    ensureDefault,
    resetToDefault,
    duplicatePattern,
    deletePattern,
    renamePattern,
    randomizeColor,
    setActivePattern,
    clearAll,
  };
});
