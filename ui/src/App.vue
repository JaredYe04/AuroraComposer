<script setup lang="ts">
import { onMounted } from 'vue';
import TimelineView from '@/components/TimelineView.vue';
import PianoRoll from '@/components/PianoRoll.vue';
import EventInspector from '@/components/EventInspector.vue';
import PlayerPanel from '@/components/PlayerPanel.vue';
import { useCompositionStore } from '@/stores/composition';
import { useParameterStore } from '@/stores/parameters';
import { useSelectionStore } from '@/stores/selection';
import { keyName } from '@/types/aurora';

const paramStore = useParameterStore();
const compStore = useCompositionStore();
const selection = useSelectionStore();

const keyOptions = Array.from({ length: 12 }, (_, i) => ({ pc: i, label: keyName(i) }));
const styles = ['classical', 'jazz', 'pop', 'ambient'];

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
      <h1>Aurora Composer</h1>
      <span class="badge">Phase 3</span>
      <PlayerPanel class="header-player" />
    </header>

    <main class="workspace">
      <!-- Left: Parameters -->
      <aside class="column params-column">
        <section class="panel">
          <h2>Parameters</h2>

          <fieldset>
            <legend>Style &amp; Form</legend>
            <label>
              Key
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
              Style
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
              Bars: {{ paramStore.snapshot.bars }}
              <input
                type="range"
                min="4"
                max="64"
                step="4"
                :value="paramStore.snapshot.bars"
                @input="
                  paramStore.setParameters({
                    bars: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
            <label>
              Beam width: {{ paramStore.snapshot.beam_width }}
              <input
                type="range"
                min="1"
                max="32"
                :value="paramStore.snapshot.beam_width"
                @input="
                  paramStore.setParameters({
                    beam_width: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
          </fieldset>

          <fieldset>
            <legend>Emotion</legend>
            <label>
              Valence: {{ paramStore.snapshot.emotion_valence.toFixed(2) }}
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                :value="paramStore.snapshot.emotion_valence"
                @input="
                  paramStore.setParameters({
                    emotion_valence: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
            <label>
              Arousal: {{ paramStore.snapshot.emotion_arousal.toFixed(2) }}
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                :value="paramStore.snapshot.emotion_arousal"
                @input="
                  paramStore.setParameters({
                    emotion_arousal: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
          </fieldset>

          <fieldset>
            <legend>Harmony</legend>
            <label>
              Complexity: {{ paramStore.snapshot.harmony_complexity.toFixed(2) }}
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                :value="paramStore.snapshot.harmony_complexity"
                @input="
                  paramStore.setParameters({
                    harmony_complexity: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
          </fieldset>

          <fieldset>
            <legend>Counterpoint</legend>
            <label>
              Strictness: {{ paramStore.snapshot.counterpoint_strictness.toFixed(2) }}
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                :value="paramStore.snapshot.counterpoint_strictness"
                @input="
                  paramStore.setParameters({
                    counterpoint_strictness: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
          </fieldset>

          <fieldset>
            <legend>Drums</legend>
            <label>
              Density: {{ paramStore.snapshot.drum_density.toFixed(2) }}
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                :value="paramStore.snapshot.drum_density"
                @input="
                  paramStore.setParameters({
                    drum_density: Number(($event.target as HTMLInputElement).value),
                  })
                "
              />
            </label>
          </fieldset>

          <button class="primary" :disabled="compStore.generating" @click="compStore.generate()">
            {{ compStore.generating ? 'Generating…' : 'Generate' }}
          </button>
        </section>
      </aside>

      <!-- Center: Timeline + Piano Roll -->
      <section class="column center-column">
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
          <span>{{ compStore.summary.bars }} bars</span>
          <span>{{ compStore.summary.note_count }} notes</span>
          <span>{{ compStore.summary.tempo_bpm }} BPM</span>
          <span v-if="selection.selectedMeasure">Measure {{ selection.selectedMeasure }}</span>
        </div>

        <TimelineView :model="compStore.timeline" />
        <PianoRoll :notes="compStore.pianoRollNotes" />

        <p v-if="compStore.error" class="error">{{ compStore.error }}</p>
      </section>

      <!-- Right: Inspector + Export -->
      <aside class="column inspector-column">
        <EventInspector />

        <section class="panel export-panel">
          <h2>Export</h2>
          <div class="actions">
            <button :disabled="!compStore.summary || compStore.playing" @click="compStore.play()">
              {{ compStore.playing ? 'Playing…' : 'Play' }}
            </button>
            <button :disabled="!compStore.playing" @click="compStore.stop()">Stop</button>
            <button :disabled="!compStore.summary" @click="compStore.downloadMidi()">
              Download MIDI
            </button>
            <button :disabled="!compStore.summary" @click="compStore.downloadMusicXml()">
              Download MusicXML
            </button>
            <button :disabled="!compStore.summary" @click="compStore.downloadAbc()">
              Download ABC
            </button>
            <button :disabled="!compStore.summary" @click="compStore.loadSvgPreview()">
              Score Preview
            </button>
          </div>
          <div
            v-if="compStore.lastSvgPreview"
            class="score-preview"
            v-html="compStore.lastSvgPreview"
          />
        </section>
      </aside>
    </main>
  </div>
</template>

<style scoped>
.app {
  min-height: 100vh;
  background: #0f1419;
  color: #e6edf3;
  font-family: 'Segoe UI', system-ui, sans-serif;
  display: flex;
  flex-direction: column;
}

.header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem 1.5rem;
  border-bottom: 1px solid #30363d;
  flex-shrink: 0;
}

.header h1 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
}

.badge {
  font-size: 0.75rem;
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  background: #8957e5;
  color: #fff;
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
  display: grid;
  grid-template-columns: 240px 1fr 280px;
  gap: 0.75rem;
  padding: 0.75rem;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.column {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  min-height: 0;
  overflow: auto;
}

.params-column {
  overflow-y: auto;
}

.center-column {
  overflow-y: auto;
}

.inspector-column {
  overflow-y: auto;
}

.panel {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 1rem 1.25rem;
}

.panel h2 {
  margin: 0 0 1rem;
  font-size: 0.875rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #8b949e;
}

fieldset {
  border: 1px solid #30363d;
  border-radius: 6px;
  padding: 0.75rem;
  margin: 0 0 0.75rem;
}

legend {
  font-size: 0.75rem;
  color: #8b949e;
  padding: 0 0.35rem;
}

label {
  display: block;
  margin-bottom: 0.75rem;
  font-size: 0.8125rem;
}

label:last-child {
  margin-bottom: 0;
}

select,
input[type='range'] {
  display: block;
  width: 100%;
  margin-top: 0.35rem;
}

select {
  padding: 0.4rem;
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 4px;
  color: inherit;
}

button {
  padding: 0.5rem 1rem;
  border: 1px solid #30363d;
  border-radius: 6px;
  background: #21262d;
  color: inherit;
  cursor: pointer;
}

button:hover:not(:disabled) {
  background: #30363d;
}

button.primary {
  width: 100%;
  margin-top: 0.25rem;
  background: #238636;
  border-color: #238636;
  font-weight: 600;
}

button.primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.progress {
  padding: 0.5rem 0.75rem;
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 6px;
}

.progress-bar {
  height: 4px;
  background: #21262d;
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #58a6ff;
  transition: width 0.2s;
}

.progress-text {
  margin: 0.35rem 0 0;
  font-size: 0.75rem;
  color: #8b949e;
}

.summary-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  padding: 0.5rem 0.75rem;
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 6px;
  font-size: 0.8125rem;
  color: #8b949e;
}

.summary-bar strong {
  color: #e6edf3;
}

.export-panel .actions {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.export-panel button {
  width: 100%;
}

.score-preview {
  margin-top: 0.75rem;
  padding: 0.5rem;
  background: #fff;
  border-radius: 4px;
  overflow-x: auto;
}

.score-preview :deep(svg) {
  max-width: 100%;
  height: auto;
}

.error {
  color: #f85149;
  font-size: 0.875rem;
}
</style>
