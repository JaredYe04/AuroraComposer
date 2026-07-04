<script setup lang="ts">
import { onMounted } from 'vue';
import TimelineView from '@/components/TimelineView.vue';
import PianoRoll from '@/components/PianoRoll.vue';
import EventInspector from '@/components/EventInspector.vue';
import PlayerPanel from '@/components/PlayerPanel.vue';
import ScoreViewer from '@/components/ScoreViewer.vue';
import ProjectMenu from '@/components/ProjectMenu.vue';
import PluginPanel from '@/components/PluginPanel.vue';
import ResizeHandle from '@/components/ResizeHandle.vue';
import { useI18n } from '@/composables/useI18n';
import { useCompositionStore } from '@/stores/composition';
import { useParameterStore } from '@/stores/parameters';
import { usePlaybackStore } from '@/stores/playback';
import { useSelectionStore } from '@/stores/selection';
import { useSettingsStore } from '@/stores/settings';
import { keyName } from '@/types/aurora';
import { formatFixed } from '@/utils/format';

const { t } = useI18n();
const paramStore = useParameterStore();
const compStore = useCompositionStore();
const selection = useSelectionStore();
const settings = useSettingsStore();
const playback = usePlaybackStore();

const keyOptions = Array.from({ length: 12 }, (_, i) => ({ pc: i, label: keyName(i) }));
const styles = ['classical', 'jazz', 'pop', 'ambient'];

function onResizeLeft(delta: number) {
  settings.setLeftPanelWidth(settings.leftPanelWidth + delta);
}

function onResizeRight(delta: number) {
  settings.setRightPanelWidth(settings.rightPanelWidth - delta);
}

onMounted(() => {
  paramStore.load().catch(() => {
    /* browser dev without Tauri */
  });
  compStore.loadComposition().catch(() => {
    /* no composition yet */
  });
});
</script>

<template>
  <div class="app">
    <header class="header">
      <h1>{{ t('app.title') }}</h1>
      <span class="badge">{{ t('app.badge') }}</span>
      <ProjectMenu />

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
        class="side-panel panel-scroll"
        :style="{ width: `${settings.leftPanelWidth}px` }"
      >
        <section class="panel">
          <h2>{{ t('params.title') }}</h2>

          <fieldset>
            <legend>{{ t('params.styleForm') }}</legend>
            <label>
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
            <label>
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
            <label>
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
            <label>
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
            <legend>{{ t('params.emotion') }}</legend>
            <label>
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
            <label>
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
            <label>
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
          </fieldset>

          <fieldset>
            <legend>{{ t('params.counterpoint') }}</legend>
            <label>
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
            <label>
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
          </fieldset>

          <button class="primary" :disabled="compStore.generating" @click="compStore.generate()">
            {{ compStore.generating ? t('params.generating') : t('params.generate') }}
          </button>
        </section>

        <PluginPanel />
      </aside>

      <ResizeHandle @resize="onResizeLeft" />

      <section class="center-column">
        <div v-if="compStore.progress" class="progress">
          <div class="progress-bar">
            <div
              class="progress-fill"
              :style="{ width: `${Math.round(compStore.progress.percent * 100)}%` }"
            />
          </div>
          <p class="progress-text">
            {{ compStore.progress.stage_name }} — {{ compStore.progress.message }}
            ({{ Math.round(compStore.progress.percent * 100) }}%)
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

        <TimelineView
          :model="compStore.timeline"
          :playhead-measure="playback.playheadMeasure"
        />
        <PianoRoll :notes="compStore.pianoRollNotes" />

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
          <div class="actions">
            <button :disabled="!compStore.summary || playback.isPlaying" @click="compStore.play()">
              {{ playback.isPlaying ? t('playback.stop') : t('playback.play') }}
            </button>
            <button :disabled="!playback.isPlaying" @click="compStore.stop()">
              {{ t('playback.stop') }}
            </button>
            <button :disabled="!compStore.summary" @click="compStore.downloadMidi()">
              {{ t('export.downloadMidi') }}
            </button>
            <button :disabled="!compStore.summary" @click="compStore.downloadMusicXml()">
              {{ t('export.downloadMusicXml') }}
            </button>
            <button :disabled="!compStore.summary" @click="compStore.downloadAbc()">
              {{ t('export.downloadAbc') }}
            </button>
          </div>
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

.center-column {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  overflow: hidden;
}

.panel {
  background: var(--bg-panel);
  border: 1px solid var(--border-muted);
  border-radius: 8px;
  padding: 0.75rem 1rem;
  flex-shrink: 0;
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

.progress {
  padding: 0.4rem 0.65rem;
  background: var(--bg-panel);
  border: 1px solid var(--border-muted);
  border-radius: 6px;
  flex-shrink: 0;
}

.progress-bar {
  height: 4px;
  background: var(--border-subtle);
  border-radius: 2px;
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

.export-panel .actions {
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
