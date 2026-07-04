import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export type ThemeMode = 'dark' | 'light';
export type Locale = 'en' | 'zh';

const STORAGE_KEY = 'aurora-settings';

interface StoredSettings {
  theme: ThemeMode;
  locale: Locale;
  leftPanelWidth: number;
  rightPanelWidth: number;
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
  const rightPanelWidth = ref(stored.rightPanelWidth ?? 300);

  function persist() {
    const data: StoredSettings = {
      theme: theme.value,
      locale: locale.value,
      leftPanelWidth: leftPanelWidth.value,
      rightPanelWidth: rightPanelWidth.value,
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
    rightPanelWidth.value = Math.max(220, Math.min(520, w));
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
    setTheme,
    toggleTheme,
    setLocale,
    setLeftPanelWidth,
    setRightPanelWidth,
    init,
  };
});
