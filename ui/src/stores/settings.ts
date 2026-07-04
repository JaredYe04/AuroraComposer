import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export type ThemeMode = 'dark' | 'light';
export type Locale = 'en' | 'zh';
export type MelodyEngine = 'gm' | 'sine' | 'square' | 'triangle' | 'sawtooth';

const STORAGE_KEY = 'aurora-settings';

interface StoredSettings {
  theme: ThemeMode;
  locale: Locale;
  leftPanelWidth: number;
  rightPanelWidth: number;
  rightPanelRatio?: number;
  melodyEngine?: MelodyEngine;
}

function defaultRightPanelWidth(): number {
  if (typeof window === 'undefined') return 600;
  return Math.min(1200, Math.round(window.innerWidth * 0.5));
}

function loadStored(): Partial<StoredSettings> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? (JSON.parse(raw) as Partial<StoredSettings>) : {};
  } catch {
    return {};
  }
}

function applyTheme(theme: ThemeMode) {
  document.documentElement.dataset.theme = theme;
}

export const useSettingsStore = defineStore('settings', () => {
  const stored = loadStored();

  const theme = ref<ThemeMode>(stored.theme ?? 'dark');
  const locale = ref<Locale>(stored.locale ?? 'zh');
  const leftPanelWidth = ref(stored.leftPanelWidth ?? 260);
  const rightPanelWidth = ref(stored.rightPanelWidth ?? defaultRightPanelWidth());
  const rightPanelRatio = ref<number | undefined>(stored.rightPanelRatio);
  const melodyEngine = ref<MelodyEngine>(stored.melodyEngine ?? 'gm');

  function persist() {
    const data: StoredSettings = {
      theme: theme.value,
      locale: locale.value,
      leftPanelWidth: leftPanelWidth.value,
      rightPanelWidth: rightPanelWidth.value,
      rightPanelRatio: rightPanelRatio.value,
      melodyEngine: melodyEngine.value,
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
  }

  function setTheme(next: ThemeMode) {
    theme.value = next;
    applyTheme(next);
    persist();
  }

  function toggleTheme() {
    setTheme(theme.value === 'dark' ? 'light' : 'dark');
  }

  function setLocale(next: Locale) {
    locale.value = next;
    persist();
  }

  function setLeftPanelWidth(w: number) {
    leftPanelWidth.value = Math.max(180, Math.min(480, w));
    persist();
  }

  function setRightPanelWidth(w: number) {
    rightPanelWidth.value = Math.max(220, Math.min(1200, w));
    if (typeof window !== 'undefined') {
      rightPanelRatio.value = w / window.innerWidth;
    }
    persist();
  }

  function setMelodyEngine(next: MelodyEngine) {
    melodyEngine.value = next;
    persist();
  }

  function init() {
    applyTheme(theme.value);
  }

  watch([leftPanelWidth, rightPanelWidth], persist);

  return {
    theme,
    locale,
    leftPanelWidth,
    rightPanelWidth,
    rightPanelRatio,
    melodyEngine,
    setTheme,
    toggleTheme,
    setLocale,
    setLeftPanelWidth,
    setRightPanelWidth,
    setMelodyEngine,
    init,
  };
});
