<script setup lang="ts">
import { onMounted } from 'vue'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import { useAlertStore } from '@/stores/alerts'
import { useSettingsStore } from '@/stores/settings'
import { useAppointmentsStore } from '@/stores/appointments'

const alertStore = useAlertStore()
const settingsStore = useSettingsStore()
const appointmentsStore = useAppointmentsStore()

onMounted(async () => {
  // Load persisted config first so the Settings form is pre-filled
  await settingsStore.loadSavedConfig()
  // Check whether the backend already has an active connection
  await settingsStore.checkConnection()
  // Start alert polling loop
  alertStore.startAutoRefresh()
  // Fetch upcoming appointments (silently no-ops when not connected)
  appointmentsStore.fetchAppointments()
})
</script>

<template>
  <div class="app-shell">
    <AppSidebar />
    <main class="app-main">
      <RouterView />
    </main>
  </div>
</template>

<style>
/* ─── Design System CSS Variables ─────────────────────────────────── */
:root {
  /* Colors — text */
  --color-text: rgba(0, 0, 0, 0.95);
  --color-text-secondary: #615d59;
  --color-text-muted: #a39e98;

  /* Colors — surface */
  --color-bg: #ffffff;
  --color-bg-alt: #f6f5f4;
  --color-bg-dark: #31302e;

  /* Colors — brand / interactive */
  --color-blue: #0075de;
  --color-blue-active: #005bab;
  --color-blue-focus: #097fe8;

  /* Colors — badge */
  --color-badge-bg: #f2f9ff;
  --color-badge-text: #097fe8;

  /* Colors — semantic */
  --color-teal: #2a9d99;
  --color-green: #1aae39;
  --color-orange: #dd5b00;
  --color-pink: #ff64c8;
  --color-purple: #391c57;
  --color-brown: #523410;

  /* Drug class accent colors */
  --drug-H: #2a9d99;
  --drug-R: #dd5b00;
  --drug-Z: #523410;
  --drug-E: #0075de;

  /* Borders */
  --border: 1px solid rgba(0, 0, 0, 0.1);
  --border-color: rgba(0, 0, 0, 0.1);

  /* Shadows */
  --shadow-card:
    rgba(0, 0, 0, 0.04) 0px 4px 18px,
    rgba(0, 0, 0, 0.027) 0px 2.025px 7.84688px,
    rgba(0, 0, 0, 0.02) 0px 0.8px 2.925px,
    rgba(0, 0, 0, 0.01) 0px 0.175px 1.04062px;
  --shadow-deep:
    rgba(0, 0, 0, 0.01) 0px 1px 3px,
    rgba(0, 0, 0, 0.02) 0px 3px 7px,
    rgba(0, 0, 0, 0.02) 0px 7px 15px,
    rgba(0, 0, 0, 0.04) 0px 14px 28px,
    rgba(0, 0, 0, 0.05) 0px 23px 52px;

  /* Border radius */
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-card: 12px;
  --radius-pill: 9999px;

  /* Typography */
  --font: 'Inter', -apple-system, system-ui, 'Segoe UI', Helvetica, Arial, sans-serif;

  /* Layout */
  --sidebar-width: 240px;
}

/* ─── Global Reset & Base ──────────────────────────────────────────── */
*,
*::before,
*::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html,
body {
  height: 100%;
  width: 100%;
  overflow: hidden;
}

body {
  font-family: var(--font);
  font-size: 14px;
  line-height: 1.5;
  color: var(--color-text);
  background-color: var(--color-bg-alt);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

#app {
  height: 100%;
  width: 100%;
}

/* ─── Typography scale ─────────────────────────────────────────────── */
h1 {
  font-size: 24px;
  font-weight: 700;
  letter-spacing: -0.03em;
  line-height: 1.2;
}

h2 {
  font-size: 20px;
  font-weight: 600;
  letter-spacing: -0.02em;
  line-height: 1.3;
}

h3 {
  font-size: 16px;
  font-weight: 600;
  letter-spacing: -0.01em;
  line-height: 1.4;
}

h4 {
  font-size: 14px;
  font-weight: 600;
  line-height: 1.4;
}

p {
  line-height: 1.6;
}

/* ─── Scrollbar ─────────────────────────────────────────────────────── */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.15);
  border-radius: var(--radius-pill);
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.25);
}

/* ─── Focus ring ────────────────────────────────────────────────────── */
:focus-visible {
  outline: 2px solid var(--color-blue);
  outline-offset: 2px;
}

/* ─── Utility classes ───────────────────────────────────────────────── */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}

.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>

<style scoped>
/* ─── App Shell Layout ──────────────────────────────────────────────── */
.app-shell {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background-color: var(--color-bg-alt);
}

.app-main {
  flex: 1;
  min-width: 0;
  height: 100vh;
  overflow-y: auto;
  background-color: var(--color-bg);
}
</style>