<script setup lang="ts">
import { ref } from 'vue';
import { open, save } from '@tauri-apps/plugin-dialog';
import { newProject, saveProject } from '@/services/tauri';
import { useI18n } from '@/composables/useI18n';
import { useCompositionStore } from '@/stores/composition';
import { useParameterStore } from '@/stores/parameters';

const { t } = useI18n();

const compStore = useCompositionStore();
const paramStore = useParameterStore();
const busy = ref(false);
const message = ref<string | null>(null);

async function onNew() {
  busy.value = true;
  message.value = null;
  try {
    const summary = await newProject();
    await paramStore.load();
    await compStore.resetWorkspace(summary);
  } catch (e) {
    message.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

async function onSave() {
  if (!compStore.summary) {
    message.value = 'Nothing to save — generate a composition first.';
    return;
  }
  busy.value = true;
  message.value = null;
  try {
    const path = await save({
      filters: [{ name: 'Aurora Project', extensions: ['aurora'] }],
      defaultPath: `${compStore.summary.title}.aurora`,
    });
    if (path) {
      await saveProject(path);
      message.value = 'Project saved.';
    }
  } catch (e) {
    message.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

async function onLoad() {
  busy.value = true;
  message.value = null;
  try {
    const path = await open({
      filters: [{ name: 'Aurora Project', extensions: ['aurora'] }],
      multiple: false,
    });
    if (path) {
      await compStore.loadFromProject(path);
      await paramStore.load();
      message.value = 'Project loaded.';
    }
  } catch (e) {
    message.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="project-menu">
    <button :disabled="busy" @click="onNew">{{ t('project.new') }}</button>
    <button :disabled="busy" @click="onLoad">{{ t('project.load') }}</button>
    <button :disabled="busy || !compStore.summary" @click="onSave">{{ t('project.save') }}</button>
    <span v-if="message" class="message">{{ message }}</span>
  </div>
</template>

<style scoped>
.project-menu {
  display: flex;
  align-items: center;
  gap: 0.35rem;
}

.project-menu button {
  padding: 0.3rem 0.65rem;
  font-size: 0.75rem;
  border: 1px solid var(--border-muted);
  border-radius: 4px;
  background: var(--bg-panel-elevated);
  color: inherit;
  cursor: pointer;
}

.project-menu button:hover:not(:disabled) {
  background: var(--border-subtle);
}

.project-menu button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.message {
  font-size: 0.7rem;
  color: var(--text-muted);
  margin-left: 0.25rem;
}
</style>
