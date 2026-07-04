<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { listWasmPlugins, registerWasmPlugin } from '@/services/tauri';
import type { PluginInfo } from '@/types/aurora';

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
    message.value = 'Plugin registered.';
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
  <section class="panel plugin-panel">
    <div class="header-row">
      <h2>Plugins</h2>
      <button class="small-btn" :disabled="loading" @click="refresh">
        {{ loading ? '…' : 'Refresh' }}
      </button>
    </div>

    <p class="hint">
      Built-in style plugins (classical, jazz, pop) and AI stub are always active. Register
      external WASM plugins below.
    </p>

    <button class="register-btn" @click="onRegister">Register WASM Plugin…</button>

    <ul v-if="plugins.length" class="plugin-list">
      <li v-for="plugin in plugins" :key="plugin.id" class="plugin-card">
        <span class="plugin-name">{{ plugin.name }}</span>
        <span class="plugin-meta">{{ plugin.id }} · {{ plugin.execution_tier }}</span>
        <span class="plugin-state">{{ plugin.state }}</span>
      </li>
    </ul>
    <p v-else class="empty">No external WASM plugins discovered.</p>

    <p v-if="message" class="message">{{ message }}</p>
  </section>
</template>

<style scoped>
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
  color: #8b949e;
}

.small-btn {
  padding: 0.2rem 0.5rem;
  font-size: 0.7rem;
  border: 1px solid #30363d;
  border-radius: 4px;
  background: #21262d;
  color: inherit;
  cursor: pointer;
}

.hint {
  font-size: 0.75rem;
  color: #6e7681;
  margin: 0 0 0.75rem;
  line-height: 1.4;
}

.register-btn {
  width: 100%;
  margin-bottom: 0.75rem;
  padding: 0.4rem 0.75rem;
  font-size: 0.8125rem;
  border: 1px solid #30363d;
  border-radius: 6px;
  background: #21262d;
  color: inherit;
  cursor: pointer;
}

.register-btn:hover {
  background: #30363d;
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
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 4px;
  font-size: 0.75rem;
}

.plugin-name {
  font-weight: 600;
  color: #e6edf3;
}

.plugin-meta {
  color: #8b949e;
  word-break: break-all;
}

.plugin-state {
  color: #58a6ff;
  text-transform: capitalize;
}

.empty {
  font-size: 0.75rem;
  color: #6e7681;
  margin: 0;
}

.message {
  margin: 0.5rem 0 0;
  font-size: 0.75rem;
  color: #8b949e;
}
</style>
