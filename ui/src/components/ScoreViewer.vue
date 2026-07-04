<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from 'vue';
import ABCJS from 'abcjs';
import { jsPDF } from 'jspdf';
import { exportAbc, exportMusicXml } from '@/services/tauri';
import { useI18n } from '@/composables/useI18n';
import { useCompositionStore } from '@/stores/composition';
import { usePlaybackStore } from '@/stores/playback';
import { useSelectionStore } from '@/stores/selection';

type TabId = 'abc' | 'musicxml' | 'pdf';

const { t } = useI18n();
const compStore = useCompositionStore();
const playback = usePlaybackStore();
const selection = useSelectionStore();

const activeTab = ref<TabId>('abc');
const loading = ref(false);
const error = ref<string | null>(null);
const abcText = ref('');
const musicXml = ref('');
const svgMarkup = ref('');

const abcContainer = ref<HTMLDivElement | null>(null);
const svgContainer = ref<HTMLDivElement | null>(null);
const pdfContainer = ref<HTMLDivElement | null>(null);
const scorePaneRef = ref<HTMLDivElement | null>(null);

// eslint-disable-next-line @typescript-eslint/no-explicit-any
let verovioToolkit: any = null;

async function ensureVerovio() {
  if (verovioToolkit) return verovioToolkit;
  const createVerovioModule = (await import('verovio/wasm')).default;
  const { VerovioToolkit } = await import('verovio/esm');
  const module = await createVerovioModule();
  verovioToolkit = new VerovioToolkit(module);
  return verovioToolkit;
}

async function loadExports() {
  if (!compStore.summary) return;
  loading.value = true;
  error.value = null;
  try {
    const [abc, xml] = await Promise.all([exportAbc(), exportMusicXml()]);
    abcText.value = abc;
    musicXml.value = xml;
    await renderActiveTab();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

function renderAbc() {
  if (!abcContainer.value || !abcText.value) return;
  abcContainer.value.innerHTML = '';
  ABCJS.renderAbc(abcContainer.value, abcText.value, {
    responsive: 'resize',
    add_classes: true,
    foregroundColor: '#1a1a1a',
    wrap: {
      minSpacing: 1.8,
      maxSpacing: 2.7,
      preferredMeasuresPerLine: 4,
    },
  });
}

async function renderMusicXml() {
  if (!musicXml.value) return;
  const toolkit = await ensureVerovio();
  toolkit.loadData(musicXml.value);
  svgMarkup.value = toolkit.renderToSVG(1, {});
  if (svgContainer.value) {
    svgContainer.value.innerHTML = svgMarkup.value;
  }
}

async function renderPdfPreview() {
  await renderMusicXml();
  if (pdfContainer.value) {
    pdfContainer.value.innerHTML = svgMarkup.value;
  }
}

async function renderActiveTab() {
  await nextTick();
  if (activeTab.value === 'abc') {
    renderAbc();
  } else if (activeTab.value === 'musicxml') {
    await renderMusicXml();
  } else {
    await renderPdfPreview();
  }
  syncScrollToPlayhead();
}

function activeScrollContainer(): HTMLElement | null {
  if (activeTab.value === 'abc') return abcContainer.value;
  if (activeTab.value === 'musicxml') return svgContainer.value;
  return pdfContainer.value;
}

function syncScrollToPlayhead() {
  const el = activeScrollContainer();
  if (!el || el.scrollWidth <= el.clientWidth) return;
  const ratio = playback.playheadRatio;
  const maxScroll = el.scrollWidth - el.clientWidth;
  el.scrollLeft = ratio * maxScroll;
}

async function downloadPdf() {
  if (!svgMarkup.value) {
    await renderMusicXml();
  }
  if (!svgMarkup.value) return;

  const parser = new DOMParser();
  const doc = parser.parseFromString(svgMarkup.value, 'image/svg+xml');
  const svgEl = doc.documentElement;
  const width = Number(svgEl.getAttribute('width')?.replace(/[^\d.]/g, '') || 800);
  const height = Number(svgEl.getAttribute('height')?.replace(/[^\d.]/g, '') || 600);

  const blob = new Blob([svgMarkup.value], { type: 'image/svg+xml;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const img = new Image();

  await new Promise<void>((resolve, reject) => {
    img.onload = () => resolve();
    img.onerror = () => reject(new Error('Failed to rasterize score SVG'));
    img.src = url;
  });

  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext('2d');
  if (!ctx) {
    URL.revokeObjectURL(url);
    return;
  }
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, width, height);
  ctx.drawImage(img, 0, 0, width, height);
  URL.revokeObjectURL(url);

  const pdf = new jsPDF({
    orientation: width > height ? 'landscape' : 'portrait',
    unit: 'pt',
    format: [width, height],
  });
  pdf.addImage(canvas.toDataURL('image/png'), 'PNG', 0, 0, width, height);
  pdf.save(`${compStore.summary?.title ?? 'aurora'}.pdf`);
}

watch(activeTab, () => {
  renderActiveTab().catch((e) => {
    error.value = e instanceof Error ? e.message : String(e);
  });
});

watch(
  () => compStore.summary,
  (summary) => {
    if (summary) {
      loadExports().catch(() => {
        /* handled in loadExports */
      });
    }
  },
);

watch(
  () => compStore.pianoRollNotes,
  () => {
    if (compStore.summary) {
      loadExports().catch(() => {
        /* handled in loadExports */
      });
    }
  },
  { deep: true },
);

watch(
  () => [playback.playheadRatio, playback.globalBeat, selection.scrollX],
  () => syncScrollToPlayhead(),
);

onMounted(() => {
  if (compStore.summary) {
    loadExports().catch(() => {
      /* handled in loadExports */
    });
  }
});
</script>

<template>
  <section ref="scorePaneRef" class="score-viewer panel">
    <div class="header-row">
      <h2>{{ t('score.title') }}</h2>
      <button
        class="refresh-btn"
        :disabled="!compStore.summary || loading"
        @click="loadExports"
      >
        {{ loading ? t('score.loading') : t('score.refresh') }}
      </button>
    </div>

    <div class="tabs">
      <button
        :class="{ active: activeTab === 'abc' }"
        :disabled="!compStore.summary"
        @click="activeTab = 'abc'"
      >
        {{ t('score.tabAbc') }}
      </button>
      <button
        :class="{ active: activeTab === 'musicxml' }"
        :disabled="!compStore.summary"
        @click="activeTab = 'musicxml'"
      >
        {{ t('score.tabMusicXml') }}
      </button>
      <button
        :class="{ active: activeTab === 'pdf' }"
        :disabled="!compStore.summary"
        @click="activeTab = 'pdf'"
      >
        {{ t('score.tabPdf') }}
      </button>
    </div>

    <p v-if="error" class="error">{{ error }}</p>
    <p v-else-if="!compStore.summary" class="empty">{{ t('score.empty') }}</p>

    <div v-show="activeTab === 'abc'" ref="abcContainer" class="score-pane abc-pane panel-scroll" />
    <div
      v-show="activeTab === 'musicxml'"
      ref="svgContainer"
      class="score-pane svg-pane panel-scroll"
    />
    <div v-show="activeTab === 'pdf'" class="pdf-tab">
      <div ref="pdfContainer" class="score-pane svg-pane panel-scroll" />
      <button class="download-pdf" :disabled="!compStore.summary" @click="downloadPdf">
        {{ t('score.downloadPdf') }}
      </button>
    </div>

    <div v-if="compStore.summary" class="playhead-indicator">
      <div class="playhead-marker" :style="{ left: `${playback.playheadRatio * 100}%` }" />
    </div>
  </section>
</template>

<style scoped>
.score-viewer {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  background: var(--bg-panel);
  border: 1px solid var(--border-muted);
  border-radius: 8px;
  padding: 0.75rem 1rem;
}

.header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.5rem;
  flex-shrink: 0;
}

.score-viewer h2 {
  margin: 0;
  font-size: 0.875rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
}

.refresh-btn {
  padding: 0.25rem 0.6rem;
  font-size: 0.75rem;
  border: 1px solid var(--border-muted);
  border-radius: 4px;
  background: var(--bg-panel-elevated);
  color: inherit;
  cursor: pointer;
}

.tabs {
  display: flex;
  gap: 0.35rem;
  margin-bottom: 0.5rem;
  flex-shrink: 0;
}

.tabs button {
  flex: 1;
  padding: 0.35rem 0.5rem;
  font-size: 0.75rem;
  border: 1px solid var(--border-muted);
  border-radius: 4px;
  background: var(--bg-panel-elevated);
  color: var(--text-muted);
  cursor: pointer;
}

.tabs button.active {
  background: var(--accent-soft);
  border-color: var(--accent);
  color: var(--accent);
}

.tabs button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.score-pane {
  flex: 1;
  min-height: 0;
  overflow: auto;
  background: var(--score-paper);
  color: var(--score-ink);
  border-radius: 4px;
  padding: 0.5rem;
}

.pdf-tab {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  gap: 0.5rem;
}

.download-pdf {
  flex-shrink: 0;
  padding: 0.4rem 0.75rem;
  border: 1px solid var(--border-muted);
  border-radius: 6px;
  background: var(--bg-panel-elevated);
  color: inherit;
  cursor: pointer;
}

.download-pdf:hover:not(:disabled) {
  background: var(--border-muted);
}

.download-pdf:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.playhead-indicator {
  position: relative;
  height: 4px;
  margin-top: 0.35rem;
  background: var(--scrollbar-track);
  border-radius: 2px;
  flex-shrink: 0;
}

.playhead-marker {
  position: absolute;
  top: 0;
  width: 2px;
  height: 100%;
  background: var(--playhead);
  transform: translateX(-1px);
  transition: left 0.05s linear;
}

.empty,
.error {
  font-size: 0.8125rem;
  margin: 0 0 0.5rem;
  flex-shrink: 0;
}

.error {
  color: var(--error);
}

.empty {
  color: var(--text-muted);
}
</style>
