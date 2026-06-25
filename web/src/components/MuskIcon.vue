<script setup lang="ts">
// The musk deer (麝/香獐) icon — AutoMusk's mascot. Key distinguishing
// features vs other deer: NO antlers, and a pair of downward "saber tusks"
// (upper canines). This minimalist head silhouette leads with those tusks.
withDefaults(defineProps<{ size?: number }>(), { size: 28 })
</script>

<template>
  <svg
    :width="size" :height="size" viewBox="0 0 64 64" fill="none"
    xmlns="http://www.w3.org/2000/svg" class="musk-icon"
    role="img" aria-label="Auto Musk"
  >
    <!-- Head: a rounded, slightly tapered muzzle shape (no antlers!) -->
    <path
      d="M32 6
         C 22 6 16 14 16 24
         C 16 30 18 35 20 39
         L 20 44
         C 20 50 25 55 32 55
         C 39 55 44 50 44 44
         L 44 39
         C 46 35 48 30 48 24
         C 48 14 42 6 32 6 Z"
      fill="currentColor"
    />
    <!-- Ears: two leaf shapes on the sides of the head -->
    <ellipse cx="14" cy="18" rx="6" ry="9" fill="currentColor" transform="rotate(-25 14 18)" />
    <ellipse cx="50" cy="18" rx="6" ry="9" fill="currentColor" transform="rotate(25 50 18)" />
    <!-- Eyes: two small dark dots -->
    <circle cx="25" cy="27" r="2.6" fill="#1a1a1d" />
    <circle cx="39" cy="27" r="2.6" fill="#1a1a1d" />
    <!-- The signature tusks (saber canines): two thin curves curving down
         past the muzzle — the musk deer's most distinctive trait. -->
    <path d="M 27 41 Q 26 48 27.5 53" stroke="#fff" stroke-width="2.4" stroke-linecap="round" fill="none" />
    <path d="M 37 41 Q 38 48 36.5 53" stroke="#fff" stroke-width="2.4" stroke-linecap="round" fill="none" />
    <!-- Nose: a small dark triangle at the muzzle tip -->
    <path d="M 30 38 L 34 38 L 32 41 Z" fill="#1a1a1d" />
  </svg>
</template>

<style scoped>
.musk-icon { display: inline-block; color: var(--accent, #6366f1); vertical-align: middle; }
</style>
