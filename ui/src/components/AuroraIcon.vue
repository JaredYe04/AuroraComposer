<script setup lang="ts">
import { Icon } from '@iconify/vue';
import { computed } from 'vue';
import { icons, type IconName } from '@/assets/icons';

const props = withDefaults(
  defineProps<{
    name: IconName;
    size?: number | string;
    title?: string;
  }>(),
  { size: 20 },
);

const iconId = computed(() => icons[props.name]);
</script>

<template>
  <Icon
    :icon="iconId"
    :width="size"
    :height="size"
    class="aurora-icon"
    :aria-label="title"
    role="img"
  />
</template>

<style scoped>
.aurora-icon {
  display: inline-block;
  vertical-align: middle;
  color: currentColor;
  flex-shrink: 0;
}
</style>
