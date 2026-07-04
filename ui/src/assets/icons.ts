/** Iconify icon IDs — Material Symbols (Apache 2.0) via @iconify/vue */
export const icons = {
  play: 'material-symbols:play-arrow-rounded',
  stop: 'material-symbols:stop-rounded',
  pause: 'material-symbols:pause-rounded',
  pointer: 'material-symbols:near-me-rounded',
  boxSelect: 'material-symbols:select-rounded',
  brush: 'tabler:brush',
  eraser: 'material-symbols:ink-eraser-rounded',
  split: 'material-symbols:content-cut-rounded',
  delete: 'material-symbols:delete-outline-rounded',
  duplicate: 'material-symbols:content-copy-rounded',
  paste: 'material-symbols:content-paste-rounded',
  zoomIn: 'material-symbols:zoom-in-rounded',
  zoomOut: 'material-symbols:zoom-out-rounded',
  zoomFit: 'material-symbols:fit-screen-rounded',
  pan: 'material-symbols:pan-tool-rounded',
  pagePrev: 'material-symbols:chevron-left-rounded',
  pageNext: 'material-symbols:chevron-right-rounded',
  refresh: 'material-symbols:refresh-rounded',
  download: 'material-symbols:download-rounded',
  pianoRoll: 'tabler:piano',
  playlist: 'material-symbols:view-list-rounded',
  pattern: 'material-symbols:grid-view-rounded',
  snap: 'material-symbols:grid-on-rounded',
  dice: 'tabler:dice-5',
} as const;

export type IconName = keyof typeof icons;
