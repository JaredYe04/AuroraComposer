import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import type { NodeId, VoiceDef } from '@/types/aurora';
import { nodeIdKey, nodeIdsEqual } from '@/types/aurora';
import { useCompositionStore } from './composition';

export const useSelectionStore = defineStore('selection', () => {
  const selectedEventIds = ref<Set<string>>(new Set());
  const selectedMeasure = ref<number | null>(null);
  const selectedSectionId = ref<string | null>(null);
  const hoveredEventId = ref<string | null>(null);
  const activeVoiceId = ref<number | null>(null);
  const zoomX = ref(48);
  const scrollX = ref(0);

  const voices = computed((): VoiceDef[] => {
    const comp = useCompositionStore().composition;
    return comp?.voice_registry.voices ?? [];
  });

  const primaryEventId = computed((): NodeId | null => {
    const first = selectedEventIds.value.values().next().value;
    if (!first) return null;
    const [index, generation] = first.split(':').map(Number);
    return { index, generation };
  });

  function selectEvent(nodeId: NodeId, additive = false) {
    const key = nodeIdKey(nodeId);
    if (!additive) {
      selectedEventIds.value = new Set([key]);
    } else {
      const next = new Set(selectedEventIds.value);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      selectedEventIds.value = next;
    }
  }

  function selectEvents(nodeIds: NodeId[], additive = false) {
    const keys = nodeIds.map(nodeIdKey);
    if (!additive) {
      selectedEventIds.value = new Set(keys);
      return;
    }
    const next = new Set(selectedEventIds.value);
    for (const key of keys) {
      if (next.has(key)) next.delete(key);
      else next.add(key);
    }
    selectedEventIds.value = next;
  }

  function isEventSelected(nodeId: NodeId): boolean {
    return selectedEventIds.value.has(nodeIdKey(nodeId));
  }

  function selectMeasure(measure: number | null) {
    selectedMeasure.value = measure;
  }

  function selectSection(sectionKey: string | null) {
    selectedSectionId.value = sectionKey;
  }

  function setActiveVoice(voiceId: number | null) {
    activeVoiceId.value = voiceId;
  }

  function setHoveredEvent(nodeId: NodeId | null) {
    hoveredEventId.value = nodeId ? nodeIdKey(nodeId) : null;
  }

  function clearSelection() {
    selectedEventIds.value = new Set();
    selectedMeasure.value = null;
    selectedSectionId.value = null;
  }

  function setZoom(zoom: number) {
    zoomX.value = Math.max(16, Math.min(128, zoom));
  }

  function setScrollX(x: number) {
    scrollX.value = Math.max(0, x);
  }

  return {
    selectedEventIds,
    selectedMeasure,
    selectedSectionId,
    hoveredEventId,
    activeVoiceId,
    voices,
    primaryEventId,
    zoomX,
    scrollX,
    selectEvent,
    selectEvents,
    isEventSelected,
    selectMeasure,
    selectSection,
    setActiveVoice,
    setHoveredEvent,
    clearSelection,
    setZoom,
    setScrollX,
    nodeIdsEqual,
  };
});
