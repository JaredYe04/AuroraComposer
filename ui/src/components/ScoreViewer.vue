<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from 'vue';
import ABCJS from 'abcjs';
import ScoreViewport from '@/components/ScoreViewport.vue';
import IconButton from '@/components/IconButton.vue';
import { exportAbc, exportMusicXml, exportPdfBytes } from '@/services/tauri';
import { promptAndSaveBytes } from '@/services/download';
import { useI18n } from '@/composables/useI18n';
import { useCompositionStore } from '@/stores/composition';
import { usePlaybackStore } from '@/stores/playback';
import { useSettingsStore } from '@/stores/settings';

type TabId = 'abc' | 'musicxml' | 'pdf';

const { t } = useI18n();
const compStore = useCompositionStore();
const playback = usePlaybackStore();
const settings = useSettingsStore();

const activeTab = ref<TabId>('musicxml');
const loading = ref(false);
const error = ref<string | null>(null);
const abcText = ref('');
const musicXml = ref('');
const svgMarkup = ref('');
const pageCount = ref(1);
const currentPage = ref(1);

const abcContainer = ref<HTMLDivElement | null>(null);
const svgContainer = ref<HTMLDivElement | null>(null);
const pdfContainer = ref<HTMLDivElement | null>(null);
const abcViewportRef = ref<InstanceType<typeof ScoreViewport> | null>(null);
const svgViewportRef = ref<InstanceType<typeof ScoreViewport> | null>(null);
const pdfViewportRef = ref<InstanceType<typeof ScoreViewport> | null>(null);

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

function scoreInkColor(): string {
  return settings.theme === 'light' ? '#1a1a1a' : '#e6edf3';
}

function renderAbc() {
  if (!abcContainer.value || !abcText.value) return;
  abcContainer.value.innerHTML = '';
  ABCJS.renderAbc(abcContainer.value, abcText.value, {
    responsive: 'resize',
    add_classes: true,
    foregroundColor: scoreInkColor(),
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
  toolkit.setOptions(
    JSON.stringify({
      adjustPageHeight: true,
      mmOutput: true,
      spacingSystem: 12,
      spacingStaff: 6,
      breaks: 'auto',
      scale: 40,
    }),
  );
  toolkit.loadData(musicXml.value);
  pageCount.value = Math.max(1, toolkit.getPageCount());
  if (currentPage.value > pageCount.value) {
    currentPage.value = 1;
  }
  svgMarkup.value = toolkit.renderToSVG(currentPage.value, {});
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
    requestAnimationFrame(() => abcViewportRef.value?.fitToContainer());
  } else if (activeTab.value === 'musicxml') {
    await renderMusicXml();
    requestAnimationFrame(() => svgViewportRef.value?.fitToContainer());
  } else {
    await renderPdfPreview();
    requestAnimationFrame(() => pdfViewportRef.value?.fitToContainer());
  }
}

async function onPageChange(page: number) {
  currentPage.value = page;
  if (activeTab.value === 'musicxml' || activeTab.value === 'pdf') {
    await renderMusicXml();
    if (activeTab.value === 'pdf' && pdfContainer.value) {
      pdfContainer.value.innerHTML = svgMarkup.value;
    }
    requestAnimationFrame(() => {
      svgViewportRef.value?.fitToContainer();
      pdfViewportRef.value?.fitToContainer();
    });
  }
}

async function downloadPdf() {
  try {
    error.value = null;
    const bytes = await exportPdfBytes();
    const name = `${compStore.summary?.title ?? 'aurora'}.pdf`;
    await promptAndSaveBytes(name, new Uint8Array(bytes), [
      { name: 'PDF', extensions: ['pdf'] },
    ]);
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
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
  () => compStore.revision,
  () => {
    if (compStore.summary) {
      loadExports().catch(() => {
        /* handled in loadExports */
      });
    }
  },
);

watch(
  () => settings.theme,
  () => {
    if (activeTab.value === 'abc') {
      renderAbc();
      requestAnimationFrame(() => abcViewportRef.value?.fitToContainer());
    }
  },
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
  <section class="score-viewer panel">
    <div class="header-row">
      <h2>{{ t('score.title') }}</h2>
      <IconButton
        icon="refresh"
        :title="loading ? t('score.loading') : t('score.refresh')"
        :disabled="!compStore.summary || loading"
        @click="loadExports"
      />
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

    <ScoreViewport
      v-show="activeTab === 'abc' && compStore.summary"
      ref="abcViewportRef"
      class="score-viewport-wrap"
    >
      <div ref="abcContainer" class="score-content" />
    </ScoreViewport>

    <ScoreViewport
      v-show="activeTab === 'musicxml' && compStore.summary"
      ref="svgViewportRef"
      class="score-viewport-wrap"
      :show-paging="true"
      :page-count="pageCount"
      :current-page="currentPage"
      @update:current-page="onPageChange"
    >
      <div ref="svgContainer" class="score-content" />
    </ScoreViewport>

    <div v-show="activeTab === 'pdf' && compStore.summary" class="pdf-tab">
      <ScoreViewport
        ref="pdfViewportRef"
        class="score-viewport-wrap"
        :show-paging="true"
        :page-count="pageCount"
        :current-page="currentPage"
        @update:current-page="onPageChange"
      >
        <div ref="pdfContainer" class="score-content" />
      </ScoreViewport>
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

.score-viewport-wrap {
  flex: 1;
  min-height: 0;
}

.score-content {
  display: inline-block;
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
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
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
