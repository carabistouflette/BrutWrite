<script setup lang="ts">
import { useWindowControls } from '../../composables/ui/useWindowControls';

const { minimize, toggleMaximize, close } = useWindowControls();
</script>

<template>
  <header class="app-header">
    <div class="logo">
      <span class="logo-text">Brut<span class="underline-target">Write</span></span>
    </div>
    <div class="window-controls">
      <button class="control-btn minimize" @click="minimize">
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
      </button>
      <button class="control-btn maximize" @click="toggleMaximize">
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
        </svg>
      </button>
      <button class="control-btn close" @click="close">
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.app-header {
  height: 48px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 20px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  position: relative;
  z-index: 100;
  transition: all 0.3s ease;
}

.logo-text {
  font-family: serif;
  font-weight: 800;
  font-style: italic;
  font-size: 1.1rem;
  letter-spacing: -0.02em;
  color: var(--color-ink);
  position: relative;
  text-shadow: 0 0 20px rgba(var(--color-accent-rgb), 0.2);
}

.underline-target {
  position: relative;
}

.underline-target::after {
  content: '';
  position: absolute;
  bottom: 0px;
  left: 0;
  width: 0;
  height: 1px;
  background: var(--color-accent);
  transition: width 0.3s ease;
  box-shadow: 0 0 6px var(--color-accent);
}

.app-header:hover .underline-target::after {
  width: 100%;
}

.window-controls {
  display: flex;
  gap: 8px;
}

.control-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  background: transparent;
  color: rgba(var(--color-ink-rgb), 0.4);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid transparent;
}

.control-btn:hover {
  background: rgba(var(--color-ink-rgb), 0.05);
  color: var(--color-ink);
  transform: translateY(-1px);
}

.control-btn.close:hover {
  background: #ef444415;
  color: #ef4444;
  border-color: #ef444430;
}

/* Shimmer Line Animation */
.app-header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 40%;
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--color-accent), transparent);
  animation: shimmer 4s infinite linear;
  will-change: transform;
}

@keyframes shimmer {
  0% {
    transform: translateX(-250%);
  }
  100% {
    transform: translateX(500%);
  }
}
</style>
