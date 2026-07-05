<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import TimelineView from '@/components/TimelineView.vue';
import PianoRoll from '@/components/PianoRoll.vue';
import PatternBar from '@/components/PatternBar.vue';
import PatternPlaylist from '@/components/PatternPlaylist.vue';
import EventInspector from '@/components/EventInspector.vue';
import PlayerPanel from '@/components/PlayerPanel.vue';
import ScoreViewer from '@/components/ScoreViewer.vue';
import ProjectMenu from '@/components/ProjectMenu.vue';
import PluginMenu from '@/components/PluginMenu.vue';
import VoiceSwitcher from '@/components/VoiceSwitcher.vue';
import IconButton from '@/components/IconButton.vue';
import AuroraIcon from '@/components/AuroraIcon.vue';
import ScrollPanel from '@/components/ScrollPanel.vue';
import ResizeHandle from '@/components/ResizeHandle.vue';
import type { IconName } from '@/assets/icons';
import { useI18n } from '@/composables/useI18n';
import { GENERATION_PRESETS } from '@/presets/generationPresets';
import { useCompositionStore } from '@/stores/composition';
import { useParameterStore } from '@/stores/parameters';
import { usePlaybackStore } from '@/stores/playback';
import { usePianoToolStore, type PianoToolMode } from '@/stores/pianoTool';
import { useSelectionStore } from '@/stores/selection';
import { useSettingsStore } from '@/stores/settings';
import { SNAP_PRESETS, snapPresetLabel, useSnapGridStore } from '@/stores/snapGrid';
import { keyName, nodeIdKey, overallJobProgress } from '@/types/aurora';
import { extractPianoRollNotes, isDrumVoice } from '@/utils/pianoRoll';
import { formatFixed } from '@/utils/format';
import { normalizeSeed } from '@/utils/seed';

const { t, locale } = useI18n();
const paramStore = useParameterStore();
const compStore = useCompositionStore();
const selection = useSelectionStore();
const settings = useSettingsStore();
const playback = usePlaybackStore();
const pianoTool = usePianoToolStore();
const snapGrid = useSnapGridStore();

const editorView = ref<'piano' | 'playlist'>('piano');

const keyOptions = Array.from({ length: 12 }, (_, i) => ({ pc: i, label: keyName(i) }));
const modeOptions = [
  { value: 'major', label: 'Major / Ionian' },
  { value: 'minor', label: 'Natural Minor' },
  { value: 'dorian', label: 'Dorian' },
  { value: 'phrygian', label: 'Phrygian' },
  { value: 'lydian', label: 'Lydian' },
  { value: 'mixolydian', label: 'Mixolydian' },
];
const progressionModes = [
  { value: 'loop', labelKey: 'params.progressionLoop' as const },
  { value: 'flow', labelKey: 'params.progressionFlow' as const },
];
const accompanimentInstruments = [
  { value: 'auto', labelKey: 'params.accompanimentAuto' as const },
  { value: 'piano', labelKey: 'params.accompanimentPiano' as const },
  { value: 'strings', labelKey: 'params.accompanimentStrings' as const },
];
const styles = ['classical', 'jazz', 'pop', 'ambient'];
const presetOptions = GENERATION_PRESETS;

const toolModes: PianoToolMode[] = ['pointer', 'box', 'brush', 'eraser', 'split'];

const toolIcons: Record<PianoToolMode, IconName> = {
  pointer: 'pointer',
  box: 'boxSelect',
  brush: 'brush',
  eraser: 'eraser',
  split: 'split',
};

function toolLabel(mode: PianoToolMode) {
  const key = `tools.${mode}` as `tools.${PianoToolMode}`;
  return t(key);
}

async function onSeedInput(e: Event) {
  const raw = Number((e.target as HTMLInputElement).value);
  await paramStore.applySeed(normalizeSeed(raw));
}

async function togglePlayback() {
  if (playback.isPlaying) {
    await compStore.stop();
  } else if (compStore.summary) {
    await compStore.play();
  }
}

const filteredNotes = computed(() => {
  if (!compStore.composition) return [];
  return extractPianoRollNotes(
    compStore.composition,
    selection.activeVoiceId ?? undefined,
  );
});

const activeVoiceIsDrum = computed(() => {
  const voice = selection.voices.find((v) => v.id === selection.activeVoiceId);
  return voice ? isDrumVoice(voice) : false;
});

const generationOverallPercent = computed(() =>
  Math.round(overallJobProgress(compStore.progress, compStore.generating) * 100),
);

const generationStageLabel = computed(() => {
  const p = compStore.progress;
  if (!p) return compStore.generating ? '…' : '';
  const total = p.total_stages ?? 14;
  return `${p.stage_index}/${total}`;
});

function onResizeLeft(delta: number) {
  settings.setLeftPanelWidth(settings.leftPanelWidth + delta);
}

function onResizeRight(delta: number) {
  settings.setRightPanelWidth(settings.rightPanelWidth - delta);
}

function onVoiceChange() {
  if (compStore.composition) {
    compStore.pianoRollNotes = extractPianoRollNotes(
      compStore.composition,
      selection.activeVoiceId ?? undefined,
    );
  }
}

function selectedNotes() {
  return filteredNotes.value.filter((n) =>
    selection.selectedEventIds.has(nodeIdKey(n.nodeId)),
  );
}

async function handleCopy() {
  const notes = selectedNotes();
  if (notes.length === 0) return;
  pianoTool.copyNotes(notes);
}

async function handlePaste() {
  if (pianoTool.clipboard.length === 0 || !compStore.composition) return;
  const voiceId = selection.activeVoiceId ?? pianoTool.clipboard[0]?.voiceId ?? 0;
  const voice = selection.voices.find((v) => v.id === voiceId);
  const isDrum = voice ? isDrumVoice(voice) : false;
  const { measure } = { measure: playback.playheadMeasure };

  for (const note of pianoTool.clipboard) {
    await compStore.insertNote({
      measureGlobal: measure,
      voiceId,
      beatNumer: note.startBeat.numer,
      beatDenom: note.startBeat.denom,
      midi: note.pitchMidi,
      isDrum,
    });
  }
}

async function handleDelete() {
  const notes = selectedNotes();
  for (const note of notes) {
    await compStore.deleteNote(note.nodeId);
  }
  selection.clearSelection();
}

async function handleDuplicate() {
  const notes = selectedNotes();
  if (notes.length === 0) return;
  const voiceId = selection.activeVoiceId ?? notes[0]?.voiceId ?? 0;
  for (const note of notes) {
    await compStore.insertNote({
      measureGlobal: note.startMeasure + 1,
      voiceId,
      beatNumer: note.startBeat.numer,
      beatDenom: note.startBeat.denom,
      midi: note.pitchMidi,
      isDrum: activeVoiceIsDrum.value,
    });
  }
}

function handleKeyDown(e: KeyboardEvent) {
  const target = e.target as HTMLElement | null;
  if (
    target &&
    (target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.tagName === 'SELECT' ||
      target.isContentEditable)
  ) {
    return;
  }

  if (e.code === 'Space') {
    e.preventDefault();
    if (playback.isPlaying || compStore.playing) {
      void compStore.stop();
    } else if (compStore.summary) {
      void compStore.play();
    }
    return;
  }

  if (e.ctrlKey || e.metaKey) {
    if (e.key === 'c' || e.key === 'C') {
      e.preventDefault();
      void handleCopy();
      return;
    }
    if (e.key === 'v' || e.key === 'V') {
      e.preventDefault();
      void handlePaste();
      return;
    }
    if (e.key === 'b' || e.key === 'B') {
      e.preventDefault();
      void handleDuplicate();
      return;
    }
  }

  if (e.key === 'Delete' || e.key === 'Backspace') {
    if (selection.selectedEventIds.size > 0) {
      e.preventDefault();
      void handleDelete();
    }
  }
}

onMounted(() => {
  paramStore.load().catch(() => {
    /* browser dev without Tauri */
  });
  compStore.initWorkspace().catch(() => {
    /* browser dev without Tauri */
  });
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<template>
  <div class="app">
    <header class="header">
      <h1>{{ t('app.title') }}</h1>
      <span class="badge">{{ t('app.badge') }}</span>
      <ProjectMenu />
      <PluginMenu />

      <div class="header-settings">
        <label class="setting-inline">
          {{ t('settings.theme') }}
          <select :value="settings.theme" @change="settings.setTheme(($event.target as HTMLSelectElement).value as 'dark' | 'light')">
            <option value="dark">{{ t('settings.themeDark') }}</option>
            <option value="light">{{ t('settings.themeLight') }}</option>
          </select>
        </label>
        <label class="setting-inline">
          {{ t('settings.language') }}
          <select :value="settings.locale" @change="settings.setLocale(($event.target as HTMLSelectElement).value as 'en' | 'zh')">
            <option value="zh">{{ t('settings.langZh') }}</option>
            <option value="en">{{ t('settings.langEn') }}</option>
          </select>
        </label>
      </div>

      <PlayerPanel class="header-player" />
    </header>

    <main class="workspace">
      <aside
        class="side-panel params-side"
        :style="{ width: `${settings.leftPanelWidth}px` }"
      >
        <section class="panel params-panel">
          <h2>{{ t('params.title') }}</h2>

          <ScrollPanel class="params-scroll">
            <fieldset>
            <legend>{{ t('params.styleForm') }}</legend>
            <label :title="t('params.keyHint')">
              {{ t('params.key') }}
              <select
                :value="paramStore.snapshot.key"
                @change="
                  paramStore.setParameters({
                    key: Number(($event.target as HTMLSelectElement).value),
                  })
                "
              >
                <option v-for="k in keyOptions" :key="k.pc" :value="k.pc">{{ k.label }}</option>
              </select>
            </label>
            <label :title="t('params.modeHint')">
              {{ t('params.mode') }}
              <select
                :value="paramStore.snapshot.mode ?? 'major'"
                @change="
                  paramStore.setParameters({
                    mode: ($event.target as HTMLSelectElement).value,
                  })
                "
              >
                <option v-for="m in modeOptions" :key="m.value" :value="m.value">{{ m.label }}</option>
              </select>
            </label>
            <label :title="t('params.styleHint')">
              {{ t('params.style') }}
              <select
                :value="paramStore.snapshot.style"
                @change="
                  paramStore.setParameters({
                    style: ($event.target as HTMLSelectElement).value,
                  })
                "
              >
                <option v-for="s in styles" :key="s" :value="s">{{ s }}</option>
              </select>
            </label>
            <label title="Select a style preset and auto-fill parameters">
              Preset / 预设
              <select
                :value="paramStore.selectedPresetId"
                @change="paramStore.applyPreset(($event.target as HTMLSelectElement).value)"
              >
                <option value="">Custom / 自定义</option>
                <option v-for="p in presetOptions" :key="p.id" :value="p.id">{{ p.label }}</option>
              </select>
            </label>
            <label :title="t('params.barsHint')">
              {{ t('params.bars') }}: {{ paramStore.snapshot.bars ?? 8 }}
              <input
                type="range"
                min="4"
                max="64"
                step="4"
                :value="paramStore.snapshot.bars ?? 8"
                @input="
                  paramStore.setParameters({
                    bars: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
            <label title="Global tempo in BPM">
              BPM: {{ Math.round(paramStore.snapshot.tempo_bpm ?? 120) }}
              <input
                type="range"
                min="40"
                max="220"
                step="1"
                :value="paramStore.snapshot.tempo_bpm ?? 120"
                @input="
                  paramStore.setParameters({
                    tempo_bpm: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
            <label :title="t('params.beamWidthHint')">
              {{ t('params.beamWidth') }}: {{ paramStore.snapshot.beam_width ?? 8 }}
              <input
                type="range"
                min="1"
                max="32"
                :value="paramStore.snapshot.beam_width ?? 8"
                @input="
                  paramStore.setParameters({
                    beam_width: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
          </fieldset>

          <fieldset>
            <legend>{{ t('params.random') }}</legend>
            <div class="seed-row" :title="t('params.seedHint')">
              <label class="seed-label" for="param-seed">{{ t('params.seed') }}</label>
              <input
                id="param-seed"
                class="seed-input"
                type="number"
                min="0"
                step="1"
                :value="paramStore.snapshot.seed"
                @change="onSeedInput"
              />
              <IconButton
                class="seed-dice"
                icon="dice"
                :title="t('params.rollSeed')"
                @click="paramStore.rollRandomSeed()"
              />
            </div>
          </fieldset>

          <fieldset>
            <legend>{{ t('params.emotion') }}</legend>
            <label :title="t('params.valenceHint')">
              {{ t('params.valence') }}: {{ formatFixed(paramStore.snapshot.emotion_valence, 2, 0.5) }}
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                :value="paramStore.snapshot.emotion_valence ?? 0.5"
                @input="
                  paramStore.setParameters({
                    emotion_valence: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
            <label :title="t('params.arousalHint')">
              {{ t('params.arousal') }}: {{ formatFixed(paramStore.snapshot.emotion_arousal, 2, 0.5) }}
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                :value="paramStore.snapshot.emotion_arousal ?? 0.5"
                @input="
                  paramStore.setParameters({
                    emotion_arousal: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
          </fieldset>

          <fieldset>
            <legend>{{ t('params.harmony') }}</legend>
            <label :title="t('params.complexityHint')">
              {{ t('params.complexity') }}: {{ formatFixed(paramStore.snapshot.harmony_complexity, 2, 0.5) }}
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                :value="paramStore.snapshot.harmony_complexity ?? 0.5"
                @input="
                  paramStore.setParameters({
                    harmony_complexity: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
            <label :title="t('params.progressionHint')">
              {{ t('params.progression') }}
              <select
                :value="paramStore.snapshot.progression_mode ?? 'loop'"
                @change="
                  paramStore.setParameters({
                    progression_mode: ($event.target as HTMLSelectElement).value,
                  })
                "
              >
                <option v-for="pm in progressionModes" :key="pm.value" :value="pm.value">
                  {{ t(pm.labelKey) }}
                </option>
              </select>
            </label>
            <label :title="t('params.tonalConservatismHint')">
              {{ t('params.tonalConservatism') }}: {{ formatFixed(paramStore.snapshot.tonal_conservatism, 2, 0.65) }}
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                :value="paramStore.snapshot.tonal_conservatism ?? 0.65"
                @input="
                  paramStore.setParameters({
                    tonal_conservatism: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
            <label :title="t('params.accompanimentHint')">
              {{ t('params.accompaniment') }}
              <select
                :value="paramStore.snapshot.accompaniment_instrument ?? 'auto'"
                @change="
                  paramStore.setParameters({
                    accompaniment_instrument: ($event.target as HTMLSelectElement).value,
                  })
                "
              >
                <option
                  v-for="inst in accompanimentInstruments"
                  :key="inst.value"
                  :value="inst.value"
                >
                  {{ t(inst.labelKey) }}
                </option>
              </select>
            </label>
          </fieldset>

          <fieldset>
            <legend>{{ t('params.counterpoint') }}</legend>
            <label :title="t('params.strictnessHint')">
              {{ t('params.strictness') }}: {{ formatFixed(paramStore.snapshot.counterpoint_strictness, 2, 0.5) }}
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                :value="paramStore.snapshot.counterpoint_strictness ?? 0.5"
                @input="
                  paramStore.setParameters({
                    counterpoint_strictness: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
          </fieldset>

          <fieldset>
            <legend>{{ t('params.drums') }}</legend>
            <label :title="t('params.densityHint')">
              {{ t('params.density') }}: {{ formatFixed(paramStore.snapshot.drum_density, 2, 0.5) }}
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                :value="paramStore.snapshot.drum_density ?? 0.5"
                @input="
                  paramStore.setParameters({
                    drum_density: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
            <label :title="t('params.accentEmphasisHint')">
              {{ t('params.accentEmphasis') }}: {{ formatFixed(paramStore.snapshot.drum_accent_emphasis, 2, 0.75) }}
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                :value="paramStore.snapshot.drum_accent_emphasis ?? 0.75"
                @input="
                  paramStore.setParameters({
                    drum_accent_emphasis: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
            <label :title="t('params.hihatDensityHint')">
              {{ t('params.hihatDensity') }}: {{ formatFixed(paramStore.snapshot.drum_hihat_density, 2, 0.6) }}
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                :value="paramStore.snapshot.drum_hihat_density ?? 0.6"
                @input="
                  paramStore.setParameters({
                    drum_hihat_density: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
          </fieldset>
          </ScrollPanel>

          <button
            class="primary generate-btn"
            :disabled="compStore.generating"
            @click="compStore.generate()"
          >
            {{ compStore.generating ? t('params.generating') : t('params.generate') }}
          </button>
        </section>
      </aside>

      <ResizeHandle @resize="onResizeLeft" />

      <section class="center-column">
        <div v-if="compStore.generating || compStore.progress" class="progress">
          <div class="progress-header">
            <span class="progress-stage">{{ generationStageLabel }}</span>
            <span class="progress-percent">{{ generationOverallPercent }}%</span>
          </div>
          <div class="progress-bar">
            <div
              class="progress-fill"
              :style="{ width: `${generationOverallPercent}%` }"
            />
          </div>
          <p v-if="compStore.progress" class="progress-text">
            {{ compStore.progress.stage_name }} — {{ compStore.progress.message }}
          </p>
        </div>

        <div v-if="compStore.summary" class="summary-bar">
          <strong>{{ compStore.summary.title }}</strong>
          <span>{{ compStore.summary.bars }} {{ t('summary.bars') }}</span>
          <span>{{ compStore.summary.note_count }} {{ t('summary.notes') }}</span>
          <span>{{ compStore.summary.tempo_bpm }} BPM</span>
          <span v-if="selection.selectedMeasure">
            {{ t('summary.measure') }} {{ selection.selectedMeasure }}
          </span>
        </div>

        <div class="tool-bar">
          <div class="view-switch">
            <IconButton
              icon="pianoRoll"
              :title="t('editor.pianoRoll')"
              :active="editorView === 'piano'"
              @click="editorView = 'piano'"
            />
            <IconButton
              icon="playlist"
              :title="t('editor.playlist')"
              :active="editorView === 'playlist'"
              @click="editorView = 'playlist'"
            />
          </div>
          <span class="tool-sep" />
          <IconButton
            v-for="mode in toolModes"
            :key="mode"
            :icon="toolIcons[mode]"
            :title="toolLabel(mode)"
            :active="pianoTool.mode === mode"
            @click="pianoTool.setMode(mode)"
          />
          <span class="tool-sep" />
          <label class="snap-select">
            <AuroraIcon name="snap" :size="16" />
            <select
              :value="snapGrid.preset"
              @change="
                snapGrid.setPreset(
                  ($event.target as HTMLSelectElement).value as typeof snapGrid.preset,
                )
              "
            >
              <option v-for="p in SNAP_PRESETS" :key="p" :value="p">
                {{ snapPresetLabel(p, locale) }}
              </option>
            </select>
          </label>
        </div>

        <TimelineView
          :model="compStore.timeline"
          :playhead-measure="playback.playheadMeasure"
        />
        <PatternBar />
        <template v-if="editorView === 'piano'">
          <VoiceSwitcher @change="onVoiceChange" />
          <PianoRoll
            :notes="filteredNotes"
            :beats-per-measure="playback.beatsPerMeasure"
            :is-drum="activeVoiceIsDrum"
          />
        </template>
        <PatternPlaylist v-else />

        <p v-if="compStore.error" class="error">{{ compStore.error }}</p>
      </section>

      <ResizeHandle @resize="onResizeRight" />

      <aside
        class="side-panel panel-scroll"
        :style="{ width: `${settings.rightPanelWidth}px` }"
      >
        <EventInspector />
        <ScoreViewer />

        <section class="panel export-panel">
          <h2>{{ t('export.title') }}</h2>
          <div class="transport-row">
            <IconButton
              variant="transport"
              :icon="playback.isPlaying ? 'stop' : 'play'"
              :title="playback.isPlaying ? t('playback.stop') : t('playback.play')"
              :disabled="!compStore.summary"
              @click="togglePlayback"
            />
          </div>
          <div class="export-actions">
            <button :disabled="!compStore.summary" @click="compStore.downloadMidi()">
              <AuroraIcon name="download" :size="16" />
              {{ t('export.downloadMidi') }}
            </button>
            <button :disabled="!compStore.summary" @click="compStore.downloadMusicXml()">
              {{ t('export.downloadMusicXml') }}
            </button>
            <button :disabled="!compStore.summary" @click="compStore.downloadAbc()">
              {{ t('export.downloadAbc') }}
            </button>
          </div>
          <p v-if="compStore.exportError" class="error">{{ compStore.exportError }}</p>
        </section>
      </aside>
    </main>
  </div>
</template>

<style scoped>
.app {
  height: 100%;
  background: var(--bg-app);
  color: var(--text-primary);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 1rem;
  border-bottom: 1px solid var(--border-muted);
  flex-shrink: 0;
}

.header h1 {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
}

.badge {
  font-size: 0.7rem;
  padding: 0.15rem 0.45rem;
  border-radius: 4px;
  background: var(--badge);
  color: #fff;
}

.header-settings {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-left: 0.5rem;
}

.setting-inline {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.75rem;
  color: var(--text-muted);
}

.setting-inline select {
  padding: 0.2rem 0.35rem;
  font-size: 0.75rem;
  background: var(--bg-input);
  border: 1px solid var(--border-muted);
  border-radius: 4px;
  color: inherit;
}

.header-player {
  margin-left: auto;
  border: none;
  background: transparent;
  padding: 0;
}

.header-player :deep(h3) {
  display: none;
}

.workspace {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  padding: 0.5rem;
  gap: 0;
}

.side-panel {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-height: 0;
  overflow: hidden;
}

.params-side {
  flex: 0 0 auto;
  height: 100%;
}

.params-panel {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding-bottom: 0.75rem;
}

.params-panel h2 {
  flex-shrink: 0;
}

.params-scroll {
  flex: 1;
  min-height: 0;
}

.seed-row {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  margin-bottom: 0.6rem;
}

.seed-label {
  flex-shrink: 0;
  margin: 0;
  font-size: 0.8125rem;
  white-space: nowrap;
}

.seed-input {
  flex: 1 1 0;
  min-width: 0;
  width: auto;
  margin: 0;
  padding: 0.35rem;
  font-variant-numeric: tabular-nums;
  background: var(--bg-input);
  border: 1px solid var(--border-muted);
  border-radius: 4px;
  color: inherit;
}

.seed-dice {
  flex-shrink: 0;
}

.generate-btn {
  flex-shrink: 0;
  width: 100%;
  margin-top: 0.5rem;
}

.panel {
  background: var(--bg-panel);
  border: 1px solid var(--border-muted);
  border-radius: 8px;
  padding: 0.75rem 1rem;
}

.panel:not(.params-panel) {
  flex-shrink: 0;
}

.center-column {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  overflow: hidden;
}

.panel h2 {
  margin: 0 0 0.75rem;
  font-size: 0.875rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
}

fieldset {
  border: 1px solid var(--border-muted);
  border-radius: 6px;
  padding: 0.6rem;
  margin: 0 0 0.6rem;
}

legend {
  font-size: 0.75rem;
  color: var(--text-muted);
  padding: 0 0.35rem;
}

label {
  display: block;
  margin-bottom: 0.6rem;
  font-size: 0.8125rem;
}

label:last-child {
  margin-bottom: 0;
}

select,
input[type='range'] {
  display: block;
  width: 100%;
  margin-top: 0.3rem;
}

select {
  padding: 0.35rem;
  background: var(--bg-input);
  border: 1px solid var(--border-muted);
  border-radius: 4px;
  color: inherit;
}

button {
  padding: 0.45rem 0.85rem;
  border: 1px solid var(--border-muted);
  border-radius: 6px;
  background: var(--bg-panel-elevated);
  color: inherit;
  cursor: pointer;
}

button:hover:not(:disabled) {
  background: var(--border-muted);
}

button.primary {
  width: 100%;
  margin-top: 0.25rem;
  background: var(--success);
  border-color: var(--success);
  font-weight: 600;
  color: #fff;
}

button.primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.tool-bar {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  flex-shrink: 0;
  flex-wrap: wrap;
}

.view-switch {
  display: flex;
  gap: 0.2rem;
}

.tool-sep {
  width: 1px;
  height: 1.25rem;
  background: var(--border-muted);
  margin: 0 0.15rem;
}

.snap-select {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.7rem;
  color: var(--text-muted);
  margin-left: auto;
}

.snap-select select {
  max-width: 6.5rem;
  padding: 0.15rem 0.25rem;
  font-size: 0.7rem;
  border: 1px solid var(--border-muted);
  border-radius: 4px;
  background: var(--bg-panel-elevated);
  color: inherit;
}

.tool-btn {
  padding: 0.25rem 0.55rem;
  font-size: 0.7rem;
}

.tool-btn.active {
  background: var(--accent-soft);
  border-color: var(--accent);
  color: var(--accent);
}

.progress {
  padding: 0.4rem 0.65rem;
  background: var(--bg-panel);
  border: 1px solid var(--border-muted);
  border-radius: 6px;
  flex-shrink: 0;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.25rem;
  font-size: 0.75rem;
  color: var(--text-muted);
}

.progress-stage {
  font-variant-numeric: tabular-nums;
}

.progress-percent {
  font-variant-numeric: tabular-nums;
  color: var(--text-primary);
}

.progress-bar {
  height: 6px;
  background: var(--border-subtle);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  transition: width 0.2s;
}

.progress-text {
  margin: 0.3rem 0 0;
  font-size: 0.75rem;
  color: var(--text-muted);
}

.summary-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem;
  padding: 0.4rem 0.65rem;
  background: var(--bg-panel);
  border: 1px solid var(--border-muted);
  border-radius: 6px;
  font-size: 0.8125rem;
  color: var(--text-muted);
  flex-shrink: 0;
}

.summary-bar strong {
  color: var(--text-primary);
}

.export-panel .transport-row {
  display: flex;
  justify-content: center;
  margin-bottom: 0.5rem;
}

.export-panel .export-actions {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.export-panel button {
  width: 100%;
}

.error {
  color: var(--error);
  font-size: 0.875rem;
  flex-shrink: 0;
}
</style>
