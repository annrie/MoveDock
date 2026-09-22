<script setup lang="ts">
withDefaults(defineProps<{ name: string; size?: number }>(), { size: 20 })
const paths: Record<string, string> = {
  dock: 'M4 7h16m-4-4 4 4-4 4M20 17H4m4-4-4 4 4 4',
  plus: 'M12 5v14M5 12h14',
  globe: 'M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0ZM3 12h18M12 3c5 5 5 13 0 18-5-5-5-13 0-18Z',
  folder: 'M3 7V5h6l2 2h10v13H3V7Z',
  clock: 'M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0ZM12 7v5l3 2',
  settings: 'M4 6h16M4 12h16M4 18h16M8 3v6m8 0v6M9 15v6',
  arrow: 'M4 12h16m-6-6 6 6-6 6',
  laptop: 'M5 4h14v12H5V4ZM2 20h20l-3-4H5l-3 4Z',
  server: 'M4 3h16v7H4V3Zm0 11h16v7H4v-7ZM7 6.5h.01M7 17.5h.01',
  check: 'm5 12 4 4L19 6',
  file: 'M14 2H5v20h14V7l-5-5Zm0 0v6h5M8 12h8M8 16h6',
  play: 'm8 4 12 8-12 8V4Z',
  refresh: 'M20 7v5h-5M4 17v-5h5M5 7a8 8 0 0 1 14-1l1 6M4 12l1 6a8 8 0 0 0 14-1',
  terminal: 'm5 7 5 5-5 5m8 0h6',
  shield: 'm12 3 8 3v6c0 5-8 9-8 9s-8-4-8-9V6l8-3Zm-4 9 3 3 5-6',
  stop: 'M6 6h12v12H6V6Z',
}
</script>
<template><svg :width="size" :height="size" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path :d="paths[name] || paths.file" /></svg></template>
