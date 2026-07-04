import { computed } from 'vue';
import { messages, type MessageKey } from '@/i18n/messages';
import { useSettingsStore } from '@/stores/settings';

export function useI18n() {
  const settings = useSettingsStore();

  const t = (key: MessageKey): string => {
    return messages[settings.locale][key] ?? messages.en[key] ?? key;
  };

  const locale = computed(() => settings.locale);

  return { t, locale };
}
