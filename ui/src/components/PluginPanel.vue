<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { listWasmPlugins, registerWasmPlugin } from '@/services/tauri';
import type { PluginInfo } from '@/types/aurora';
import { useI18n } from '@/composables/useI18n';

defineProps<{
  embedded?: boolean;
}>();

const { t } = useI18n();
const plugins = ref<PluginInfo[]>([]);
const loading = ref(false);
const message = ref<string | null>(null);

async function refresh() {
  loading.value = true;
  message.value = null;
  try {
    plugins.value = await listWasmPlugins();
  } catch (e) {
    message.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

async function onRegister() {
  message.value = null;
  try {
    const path = await open({
      filters: [
        { name: 'WASM Plugin', extensions: ['wasm'] },
        { name: 'Plugin Manifest', extensions: ['json'] },
      ],
      multiple: false,
    });
    if (!path) return;
    await registerWasmPlugin(path);
    message.value = t('plugins.registered');
    await refresh();
  } catch (e) {
    message.value = e instanceof Error ? e.message : String(e);
  }
}

onMounted(() => {
  refresh().catch(() => {
    /* browser dev without Tauri */
  });
});
</script>

<template>
  <section class="panel plugin-panel" :class="{ embedded }">
    <div class="header-row">
      <h2>{{ t('plugins.title') }}</h2>
      <button class="small-btn" :disabled="loading" @click="refresh">
        {{ loading ? '…' : t('plugins.refresh') }}
      </button>
    </div>

    <p class="hint">{{ t('plugins.hint') }}</p>

    <button class="register-btn" @click="onRegister">{{ t('plugins.register') }}</button>

    <ul v-if="plugins.length" class="plugin-list">
      <li v-for="plugin in plugins" :key="plugin.id" class="plugin-card">
        <span class="plugin-name">{{ plugin.name }}</span>
        <span class="plugin-meta">{{ plugin.id }} · {{ plugin.execution_tier }}</span>
        <span class="plugin-state">{{ plugin.state }}</span>
      </li>
    </ul>
    <p v-else class="empty">{{ t('plugins.empty') }}</p>

    <p v-if="message" class="message">{{ message }}</p>
  </section>
</template>

<style scoped>
.plugin-panel {
  background: var(--bg-panel);
  border: 1px solid var(--border-muted);
  border-radius: 8px;
  padding: 0.75rem 1rem;
}

.plugin-panel.embedded {
  border: none;
  border-radius: 0;
  box-shadow: none;
}

.plugin-panel .header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.5rem;
}

.plugin-panel h2 {
  margin: 0;
  font-size: 0.875rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
}

.small-btn {
  padding: 0.2rem 0.5rem;
  font-size: 0.7rem;
  border: 1px solid var(--border-muted);
  border-radius: 4px;
  background: var(--bg-panel-elevated);
  color: inherit;
  cursor: pointer;
}

.small-btn:hover:not(:disabled) {
  background: var(--border-subtle);
}

.hint {
  font-size: 0.75rem;
  color: var(--text-faint);
  margin: 0 0 0.75rem;
  line-height: 1.4;
}

.register-btn {
  width: 100%;
  margin-bottom: 0.75rem;
  padding: 0.4rem 0.75rem;
  font-size: 0.8125rem;
  border: 1px solid var(--border-muted);
  border-radius: 6px;
  background: var(--bg-panel-elevated);
  color: inherit;
  cursor: pointer;
}

.register-btn:hover {
  background: var(--border-subtle);
}

.plugin-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.plugin-card {
  display: grid;
  gap: 0.15rem;
  padding: 0.5rem;
  margin-bottom: 0.35rem;
  background: var(--bg-input);
  border: 1px solid var(--border-muted);
  border-radius: 4px;
  font-size: 0.75rem;
}

.plugin-name {
  font-weight: 600;
  color: var(--text-primary);
}

.plugin-meta {
  color: var(--text-muted);
  word-break: break-all;
}

.plugin-state {
  color: var(--accent);
  text-transform: capitalize;
}

.empty {
  font-size: 0.75rem;
  color: var(--text-faint);
  margin: 0;
}

.message {
  margin: 0.5rem 0 0;
  font-size: 0.75rem;
  color: var(--text-muted);
}
</style>
