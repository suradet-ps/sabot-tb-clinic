<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import {
  Microscope,
  Users,
  UserMinus,
  CalendarDays,
  BarChart2,
  Settings,
  Database,
  WifiOff,
} from 'lucide-vue-next'
import TbClinicLogo from '@/components/shared/TbClinicLogo.vue'
import { useAlertStore } from '@/stores/alerts'
import { useSettingsStore } from '@/stores/settings'
import { useAppointmentsStore } from '@/stores/appointments'

const route = useRoute()
const alertStore = useAlertStore()
const settingsStore = useSettingsStore()
const appointmentsStore = useAppointmentsStore()

interface NavItem {
  path: string
  label: string
  icon: object
  showAlerts?: boolean
  showApptCount?: boolean
}

const navItems: NavItem[] = [
  { path: '/screening', label: 'คัดกรองผู้ป่วย', icon: Microscope },
  { path: '/active', label: 'ผู้ป่วยในการรักษา', icon: Users, showAlerts: true },
  { path: '/discharged', label: 'การจำหน่ายผู้ป่วย', icon: UserMinus },
  { path: '/appointments', label: 'การนัดหมาย', icon: CalendarDays, showApptCount: true },
  { path: '/reports', label: 'รายงาน', icon: BarChart2 },
  { path: '/settings', label: 'ตั้งค่า', icon: Settings },
]

function isActive(path: string): boolean {
  if (path === '/') return route.path === '/'
  return route.path === path || route.path.startsWith(path + '/')
}

const redCount = computed(() => alertStore.redCount)
const isConnected = computed(() => settingsStore.isConnected)
const todayApptCount = computed(() => appointmentsStore.todayAppointments.length)
</script>

<template>
  <aside class="sidebar">
    <!-- Branding -->
    <div class="sidebar-header">
      <div class="sidebar-logo">
        <TbClinicLogo :size="40" />
      </div>
      <div class="sidebar-brand">
        <span class="brand-title">TB Clinic</span>
        <span class="brand-sub">โรงพยาบาลสระโบสถ์</span>
      </div>
    </div>

    <div class="sidebar-divider" />

    <!-- Navigation -->
    <nav class="sidebar-nav">
      <RouterLink
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="nav-item"
        :class="{ 'nav-item--active': isActive(item.path) }"
      >
        <component :is="item.icon" :size="17" stroke-width="2" class="nav-icon" />
        <span class="nav-label">{{ item.label }}</span>
        <span
          v-if="item.showAlerts && redCount > 0"
          class="nav-badge"
          :title="`${redCount} การแจ้งเตือนเร่งด่วน`"
        >
          {{ redCount > 99 ? '99+' : redCount }}
        </span>
        <span
          v-if="item.showApptCount && todayApptCount > 0"
          class="nav-badge nav-badge--teal"
          :title="`${todayApptCount} นัดวันนี้`"
        >
          {{ todayApptCount > 99 ? '99+' : todayApptCount }}
        </span>
      </RouterLink>
    </nav>

    <!-- Spacer -->
    <div class="sidebar-spacer" />

    <!-- Connection status -->
    <div class="sidebar-footer">
      <div class="conn-row" :class="isConnected ? 'conn-row--ok' : 'conn-row--err'">
        <component
          :is="isConnected ? Database : WifiOff"
          :size="14"
          stroke-width="2"
          class="conn-icon"
        />
        <span class="conn-label">
          {{ isConnected ? 'HOSxP เชื่อมต่อแล้ว' : 'ยังไม่ได้เชื่อมต่อ' }}
        </span>
        <span class="conn-dot" />
      </div>
      <div class="sidebar-app-ver">@2026 Sabot Hospital</div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  min-width: var(--sidebar-width);
  height: 100vh;
  background: var(--color-bg-alt);
  border-right: var(--border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
  flex-shrink: 0;
}

/* ── Header / branding ── */
.sidebar-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 18px 16px 14px;
}

.sidebar-logo {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.sidebar-brand {
  display: flex;
  flex-direction: column;
  gap: 1px;
  overflow: hidden;
}

.brand-title {
  font-family: var(--font);
  font-size: 15px;
  font-weight: 600;
  letter-spacing: -0.3px;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.brand-sub {
  font-family: var(--font);
  font-size: 11px;
  font-weight: 400;
  color: var(--color-text-muted);
  white-space: nowrap;
}

/* ── Divider ── */
.sidebar-divider {
  height: 1px;
  background: rgba(0, 0, 0, 0.07);
  margin: 0 12px 8px;
}

/* ── Navigation ── */
.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  text-decoration: none;
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease;
  position: relative;
  color: var(--color-text-secondary);
  font-family: var(--font);
  font-size: 13.5px;
  font-weight: 500;
  letter-spacing: -0.1px;
  user-select: none;
}

.nav-item:hover {
  background: rgba(0, 0, 0, 0.055);
  color: var(--color-text);
}

.nav-item--active {
  background: rgba(0, 117, 222, 0.1);
  color: var(--color-blue);
}

.nav-item--active:hover {
  background: rgba(0, 117, 222, 0.13);
  color: var(--color-blue-active);
}

.nav-item--active .nav-icon {
  color: var(--color-blue);
}

.nav-icon {
  flex-shrink: 0;
  color: var(--color-text-muted);
  transition: color 120ms ease;
}

.nav-item--active .nav-icon,
.nav-item:hover .nav-icon {
  color: inherit;
}

.nav-label {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Alert count badge on nav item */
.nav-badge {
  flex-shrink: 0;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: var(--radius-pill);
  background: var(--color-orange);
  color: #ffffff;
  font-family: var(--font);
  font-size: 10.5px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  letter-spacing: 0;
  line-height: 1;
}

.nav-badge--teal {
  background: var(--color-teal);
}

/* ── Spacer ── */
.sidebar-spacer {
  flex: 1;
}

/* ── Footer / connection ── */
.sidebar-footer {
  padding: 10px 12px 14px;
  border-top: var(--border);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.conn-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  border-radius: var(--radius-sm);
}

.conn-row--ok {
  background: rgba(42, 157, 153, 0.08);
}

.conn-row--err {
  background: rgba(221, 91, 0, 0.08);
}

.conn-icon {
  flex-shrink: 0;
}

.conn-row--ok .conn-icon {
  color: var(--color-teal);
}

.conn-row--err .conn-icon {
  color: var(--color-orange);
}

.conn-label {
  flex: 1;
  font-family: var(--font);
  font-size: 11.5px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.conn-row--ok .conn-label {
  color: var(--color-teal);
}

.conn-row--err .conn-label {
  color: var(--color-orange);
}

.conn-dot {
  width: 6px;
  height: 6px;
  border-radius: var(--radius-pill);
  flex-shrink: 0;
  animation: pulse-dot 2s ease-in-out infinite;
}

.conn-row--ok .conn-dot {
  background: var(--color-teal);
}

.conn-row--err .conn-dot {
  background: var(--color-orange);
  animation: none;
}

@keyframes pulse-dot {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.sidebar-app-ver {
  font-family: var(--font);
  font-size: 10.5px;
  color: var(--color-text-muted);
  text-align: center;
  letter-spacing: 0.1px;
}
</style>
