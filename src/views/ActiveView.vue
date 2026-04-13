<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { RefreshCw, Users, AlertTriangle, CheckCircle, Loader2, Activity } from 'lucide-vue-next'
import PatientCard from '@/components/active/PatientCard.vue'
import { usePatientStore } from '@/stores/patient'
import { useAlertStore } from '@/stores/alerts'
import type { ActivePatientRow } from '@/types/patient'

const patientStore = usePatientStore()
const alertStore = useAlertStore()

onMounted(() => {
  patientStore.fetchActivePatients()
})

type SortKey = 'alert' | 'month' | 'name'
const sortBy = ref<SortKey>('alert')

const sortOptions: { key: SortKey; label: string }[] = [
  { key: 'alert', label: 'การแจ้งเตือน' },
  { key: 'month', label: 'เดือนการรักษา' },
  { key: 'name', label: 'ชื่อ' },
]

function alertWeight(row: ActivePatientRow): number {
  if (row.alerts.some((a) => a.severity === 'red')) return 0
  if (row.alerts.some((a) => a.severity === 'yellow')) return 1
  return 2
}

const sortedPatients = computed<ActivePatientRow[]>(() => {
  const list = [...patientStore.activePatients]
  switch (sortBy.value) {
    case 'month':
      return list.sort((a, b) => (a.current_month ?? 0) - (b.current_month ?? 0))
    case 'name':
      return list.sort((a, b) =>
        (a.demographics?.full_name ?? a.tb_patient.hn).localeCompare(
          b.demographics?.full_name ?? b.tb_patient.hn,
          'th',
        ),
      )
    case 'alert':
    default:
      return list.sort((a, b) => alertWeight(a) - alertWeight(b))
  }
})

const statsTotal = computed(() => patientStore.activePatients.length)
const statsRedAlerts = computed(() => alertStore.redCount)
const statsYellowAlerts = computed(() => alertStore.yellowAlerts.length)

const statsIntensive = computed(
  () => patientStore.activePatients.filter((p) => p.current_plan?.phase === 'intensive').length,
)
const statsContinuation = computed(
  () =>
    patientStore.activePatients.filter((p) => p.current_plan?.phase === 'continuation').length,
)

const isInitialLoad = computed(
  () => patientStore.isLoading && patientStore.activePatients.length === 0,
)
</script>

<template>
  <div class="view-root">
    <!-- ── Page header ── -->
    <div class="view-header">
      <div class="header-left">
        <h1 class="header-title">ผู้ป่วยในการรักษา</h1>
        <p class="header-sub">
          ผู้ป่วย TB ที่กำลังรับการรักษาทั้งหมด
          <strong>{{ statsTotal }}</strong> ราย
        </p>
      </div>
      <div class="header-right">
        <button
          class="btn-ghost"
          @click="patientStore.fetchActivePatients()"
          :disabled="patientStore.isLoading"
          title="รีเฟรชข้อมูล"
        >
          <Loader2 v-if="patientStore.isLoading" :size="14" class="spin" />
          <RefreshCw v-else :size="14" />
          รีเฟรช
        </button>
      </div>
    </div>

    <!-- ── Stats bar ── -->
    <div class="stats-bar">
      <div class="stat-card">
        <div class="stat-icon-wrap stat-icon-blue">
          <Users :size="15" />
        </div>
        <div class="stat-body">
          <div class="stat-num">{{ statsTotal }}</div>
          <div class="stat-label">ผู้ป่วยทั้งหมด</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon-wrap stat-icon-orange">
          <Activity :size="15" />
        </div>
        <div class="stat-body">
          <div class="stat-num stat-num-orange">{{ statsIntensive }}</div>
          <div class="stat-label">ระยะเข้มข้น</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon-wrap stat-icon-teal">
          <Activity :size="15" />
        </div>
        <div class="stat-body">
          <div class="stat-num stat-num-teal">{{ statsContinuation }}</div>
          <div class="stat-label">ระยะต่อเนื่อง</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon-wrap stat-icon-alert">
          <AlertTriangle :size="15" />
        </div>
        <div class="stat-body">
          <div class="stat-num-group">
            <span
              v-if="statsRedAlerts > 0"
              class="stat-num stat-num-red"
              title="แจ้งเตือนสีแดง"
            >{{ statsRedAlerts }}</span>
            <span v-if="statsRedAlerts > 0 && statsYellowAlerts > 0" class="stat-divider">/</span>
            <span
              v-if="statsYellowAlerts > 0"
              class="stat-num stat-num-yellow"
              title="แจ้งเตือนสีเหลือง"
            >{{ statsYellowAlerts }}</span>
            <span
              v-if="statsRedAlerts === 0 && statsYellowAlerts === 0"
              class="stat-num stat-num-ok"
            >0</span>
          </div>
          <div class="stat-label">การแจ้งเตือน</div>
        </div>
      </div>
    </div>

    <!-- ── Sort bar ── -->
    <div class="sort-bar">
      <span class="sort-label">เรียงตาม:</span>
      <div class="sort-pills">
        <button
          v-for="opt in sortOptions"
          :key="opt.key"
          class="sort-btn"
          :class="{ 'sort-active': sortBy === opt.key }"
          @click="sortBy = opt.key"
        >
          {{ opt.label }}
        </button>
      </div>
    </div>

    <!-- ── Loading state (initial) ── -->
    <div v-if="isInitialLoad" class="state-container">
      <div class="loading-state">
        <Loader2 :size="28" class="spin loading-icon" />
        <span class="state-title">กำลังโหลดข้อมูล...</span>
        <span class="state-sub">กำลังดึงข้อมูลผู้ป่วยจากฐานข้อมูล</span>
      </div>
    </div>

    <!-- ── Empty state ── -->
    <div
      v-else-if="!patientStore.isLoading && patientStore.activePatients.length === 0"
      class="state-container"
    >
      <div class="empty-state">
        <CheckCircle :size="44" class="empty-icon" />
        <span class="state-title">ยังไม่มีผู้ป่วยที่กำลังรับการรักษา</span>
        <span class="state-sub">ไปที่หน้าคัดกรองเพื่อลงทะเบียนผู้ป่วย</span>
        <RouterLink to="/screening" class="empty-cta">ไปที่การคัดกรอง</RouterLink>
      </div>
    </div>

    <!-- ── Error state ── -->
    <div
      v-else-if="patientStore.error && patientStore.activePatients.length === 0"
      class="state-container"
    >
      <div class="error-state">
        <AlertTriangle :size="44" class="error-icon" />
        <span class="state-title">ไม่สามารถโหลดข้อมูลได้</span>
        <span class="state-sub">{{ patientStore.error }}</span>
        <button class="empty-cta" @click="patientStore.fetchActivePatients()">ลองใหม่</button>
      </div>
    </div>

    <!-- ── Patient grid ── -->
    <div v-else class="patient-grid">
      <TransitionGroup name="card" tag="div" class="patient-grid-inner">
        <PatientCard
          v-for="patient in sortedPatients"
          :key="patient.tb_patient.hn"
          :patient="patient"
          @view-detail="(hn) => $router.push(`/patient/${hn}`)"
          @add-followup="(hn) => $router.push(`/patient/${hn}`)"
          @discharge="(hn) => $router.push(`/patient/${hn}`)"
        />
      </TransitionGroup>
    </div>
  </div>
</template>

<style scoped>
/* ── Page root ── */
.view-root {
  padding: 32px 32px 48px;
  max-width: 1440px;
}

/* ── Header ── */
.view-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 24px;
  gap: 16px;
}

.header-title {
  font-size: 22px;
  font-weight: 700;
  letter-spacing: -0.3px;
  color: var(--color-text);
  margin: 0 0 4px;
}

.header-sub {
  font-size: 14px;
  color: var(--color-text-secondary);
  margin: 0;
  line-height: 1.4;
}

.header-sub strong {
  font-weight: 700;
  color: var(--color-text);
}

.header-right {
  flex-shrink: 0;
}

/* ── Ghost button ── */
.btn-ghost {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: 1px solid rgba(0, 0, 0, 0.15);
  padding: 7px 13px;
  font-size: 13px;
  font-weight: 600;
  font-family: var(--font);
  cursor: pointer;
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  transition:
    background 0.15s,
    border-color 0.15s;
}

.btn-ghost:hover:not(:disabled) {
  background: var(--color-bg-alt);
  border-color: rgba(0, 0, 0, 0.22);
}

.btn-ghost:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ── Stats bar ── */
.stats-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.stat-card {
  background: var(--color-bg);
  border: var(--border);
  border-radius: var(--radius-card);
  padding: 14px 18px;
  display: flex;
  align-items: center;
  gap: 12px;
  box-shadow: var(--shadow-card);
  min-width: 140px;
}

.stat-icon-wrap {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stat-icon-blue {
  background: rgba(0, 117, 222, 0.1);
  color: var(--color-blue);
}

.stat-icon-orange {
  background: rgba(221, 91, 0, 0.1);
  color: var(--color-orange);
}

.stat-icon-teal {
  background: rgba(42, 157, 153, 0.1);
  color: var(--color-teal);
}

.stat-icon-alert {
  background: rgba(245, 166, 35, 0.1);
  color: #c78b00;
}

.stat-body {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.stat-num {
  font-size: 24px;
  font-weight: 700;
  line-height: 1;
  letter-spacing: -0.5px;
  color: var(--color-text);
}

.stat-num-group {
  display: flex;
  align-items: baseline;
  gap: 4px;
  line-height: 1;
}

.stat-num-group .stat-num {
  font-size: 22px;
}

.stat-divider {
  font-size: 16px;
  font-weight: 400;
  color: var(--color-text-muted);
}

.stat-num-orange {
  color: var(--color-orange);
}

.stat-num-teal {
  color: var(--color-teal);
}

.stat-num-red {
  color: #dd5b00;
}

.stat-num-yellow {
  color: #c78b00;
}

.stat-num-ok {
  color: var(--color-text-muted);
}

.stat-label {
  font-size: 12px;
  color: var(--color-text-muted);
  margin-top: 1px;
}

/* ── Sort bar ── */
.sort-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 20px;
}

.sort-label {
  font-size: 12px;
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.sort-pills {
  display: flex;
  gap: 6px;
}

.sort-btn {
  background: var(--color-bg);
  border: var(--border);
  border-radius: var(--radius-pill);
  padding: 5px 13px;
  font-size: 12px;
  font-weight: 500;
  font-family: var(--font);
  cursor: pointer;
  color: var(--color-text-secondary);
  transition:
    background 0.15s,
    border-color 0.15s,
    color 0.15s;
}

.sort-btn:hover {
  background: var(--color-bg-alt);
}

.sort-active {
  background: var(--color-badge-bg);
  color: var(--color-blue);
  border-color: rgba(0, 117, 222, 0.3);
  font-weight: 600;
}

/* ── State containers ── */
.state-container {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}

.loading-state,
.empty-state,
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  color: var(--color-text-muted);
  text-align: center;
}

.loading-icon {
  color: var(--color-blue);
  opacity: 0.7;
  margin-bottom: 4px;
}

.empty-icon {
  color: var(--color-teal);
  opacity: 0.25;
  margin-bottom: 4px;
}

.error-icon {
  color: var(--color-orange);
  opacity: 0.4;
  margin-bottom: 4px;
}

.state-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.state-sub {
  font-size: 13px;
  color: var(--color-text-muted);
  max-width: 320px;
}

.empty-cta {
  margin-top: 6px;
  display: inline-flex;
  align-items: center;
  padding: 7px 16px;
  background: var(--color-blue);
  color: #fff;
  border: none;
  border-radius: var(--radius-sm);
  font-size: 13px;
  font-weight: 600;
  font-family: var(--font);
  cursor: pointer;
  text-decoration: none;
  transition: background 0.15s;
}

.empty-cta:hover {
  background: var(--color-blue-active);
}

/* ── Patient grid ── */
.patient-grid-inner {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

/* ── Card transition ── */
.card-enter-active {
  transition:
    opacity 0.25s ease,
    transform 0.25s ease;
}

.card-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.card-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.card-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

.card-move {
  transition: transform 0.3s ease;
}

/* ── Spin animation ── */
.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>