<script setup lang="ts">
import { computed } from 'vue'
import type { TreatmentPlan, Followup } from '@/types/treatment'

const props = defineProps<{
  plans: TreatmentPlan[]
  followups: Followup[]
}>()

// ── Helpers ──────────────────────────────────────────────────────────────

function addMonths(date: Date, months: number): Date {
  const d = new Date(date)
  d.setMonth(d.getMonth() + months)
  return d
}

/** Parse "2HRZE/4HR" → { intensiveMonths: 2, continuationMonths: 4 } */
function parseRegimen(regimen: string): { intensiveMonths: number; continuationMonths: number } {
  const match = regimen.match(/^(\d+)[A-Za-z]+\/(\d+)/)
  if (match) {
    return {
      intensiveMonths: parseInt(match[1], 10),
      continuationMonths: parseInt(match[2], 10),
    }
  }
  // sensible fallback for standard regimen
  return { intensiveMonths: 2, continuationMonths: 4 }
}

function toThaiDate(iso: string): string {
  try {
    const [y, m, d] = iso.split('-').map(Number)
    return `${String(d).padStart(2, '0')}/${String(m).padStart(2, '0')}/${y + 543}`
  } catch {
    return iso
  }
}

// ── Plan resolution ───────────────────────────────────────────────────────

const intensivePlan = computed(() => props.plans.find((p) => p.phase === 'intensive') ?? null)
const continuationPlan = computed(() => props.plans.find((p) => p.phase === 'continuation') ?? null)
const anyPlan = computed(() => intensivePlan.value ?? continuationPlan.value)

interface PhaseInfo {
  start: Date
  end: Date
  durationMonths: number
}

/**
 * Resolves intensive phase boundaries.
 * Prefers the explicit intensivePlan; otherwise back-computes from continuationPlan + regimen.
 */
const intensiveInfo = computed<PhaseInfo | null>(() => {
  if (intensivePlan.value) {
    const start = new Date(intensivePlan.value.phase_start)
    const end = intensivePlan.value.phase_end_expected
      ? new Date(intensivePlan.value.phase_end_expected)
      : addMonths(new Date(start), intensivePlan.value.duration_months)
    return { start, end, durationMonths: intensivePlan.value.duration_months }
  }

  // Back-compute from continuation plan
  if (continuationPlan.value) {
    const parsed = parseRegimen(continuationPlan.value.regimen)
    if (parsed.intensiveMonths === 0) return null
    const contStart = new Date(continuationPlan.value.phase_start)
    const intStart = addMonths(new Date(contStart), -parsed.intensiveMonths)
    return { start: intStart, end: contStart, durationMonths: parsed.intensiveMonths }
  }

  return null
})

/**
 * Resolves continuation phase boundaries.
 * Prefers the explicit continuationPlan; otherwise derives from intensivePlan + regimen.
 */
const continuationInfo = computed<PhaseInfo | null>(() => {
  if (continuationPlan.value) {
    const start = new Date(continuationPlan.value.phase_start)
    const end = continuationPlan.value.phase_end_expected
      ? new Date(continuationPlan.value.phase_end_expected)
      : addMonths(new Date(start), continuationPlan.value.duration_months)
    return { start, end, durationMonths: continuationPlan.value.duration_months }
  }

  // Derive from intensive plan + regimen
  if (intensivePlan.value) {
    const parsed = parseRegimen(intensivePlan.value.regimen)
    if (parsed.continuationMonths === 0) return null
    const intEnd = intensivePlan.value.phase_end_expected
      ? new Date(intensivePlan.value.phase_end_expected)
      : addMonths(new Date(intensivePlan.value.phase_start), intensivePlan.value.duration_months)
    const contEnd = addMonths(new Date(intEnd), parsed.continuationMonths)
    return { start: intEnd, end: contEnd, durationMonths: parsed.continuationMonths }
  }

  return null
})

// ── Timeline bounds ───────────────────────────────────────────────────────

const overallStart = computed(() => intensiveInfo.value?.start ?? continuationInfo.value?.start ?? null)
const overallEnd   = computed(() => continuationInfo.value?.end ?? intensiveInfo.value?.end ?? null)

const totalDays = computed(() => {
  if (!overallStart.value || !overallEnd.value) return 1
  const d = (overallEnd.value.getTime() - overallStart.value.getTime()) / 86_400_000
  return Math.max(1, d)
})

// ── Percentage helpers ────────────────────────────────────────────────────

function pctFromDate(date: Date): number {
  if (!overallStart.value) return 0
  const days = (date.getTime() - overallStart.value.getTime()) / 86_400_000
  return Math.max(0, Math.min(100, (days / totalDays.value) * 100))
}

const intensivePct = computed(() => {
  if (!intensiveInfo.value) return 0
  const dur =
    (intensiveInfo.value.end.getTime() - intensiveInfo.value.start.getTime()) / 86_400_000
  return Math.min(100, (dur / totalDays.value) * 100)
})

const continuationPct = computed(() => {
  if (!continuationInfo.value) return 0
  const dur =
    (continuationInfo.value.end.getTime() - continuationInfo.value.start.getTime()) / 86_400_000
  return Math.min(100 - intensivePct.value, (dur / totalDays.value) * 100)
})

const todayPct = computed<number | null>(() => {
  if (!overallStart.value) return null
  return pctFromDate(new Date())
})

function followupPct(date: string): number {
  return pctFromDate(new Date(date))
}

// ── Month ticks ───────────────────────────────────────────────────────────

const totalMonths = computed(
  () => (intensiveInfo.value?.durationMonths ?? 0) + (continuationInfo.value?.durationMonths ?? 0),
)

// ── Date labels ───────────────────────────────────────────────────────────

const startLabel = computed(() =>
  overallStart.value ? toThaiDate(overallStart.value.toISOString().slice(0, 10)) : '',
)
const endLabel = computed(() =>
  overallEnd.value ? toThaiDate(overallEnd.value.toISOString().slice(0, 10)) : '',
)

// Boundary between phases (intensive end / continuation start)
const showBoundary = computed(() => intensiveInfo.value !== null && continuationInfo.value !== null)

const boundaryDate = computed(() => {
  if (intensiveInfo.value) return intensiveInfo.value.end
  if (continuationInfo.value) return continuationInfo.value.start
  return null
})

const boundaryPct = computed(() =>
  boundaryDate.value ? pctFromDate(boundaryDate.value) : null
)

const boundaryPctClamped = computed(() =>
  boundaryPct.value !== null ? Math.max(1, Math.min(99, boundaryPct.value)) : null
)

const boundaryLabel = computed(() =>
  boundaryDate.value ? toThaiDate(boundaryDate.value.toISOString().slice(0, 10)) : ''
)

// ── Tooltip for each followup dot ─────────────────────────────────────────

function dotTooltip(f: Followup): string {
  const dateStr = toThaiDate(f.followup_date)
  return f.month_number ? `ติดตามผล: ${dateStr} (เดือนที่ ${f.month_number})` : `ติดตามผล: ${dateStr}`
}
</script>

<template>
  <div class="timeline-wrapper">
    <!-- Empty state -->
    <div v-if="!anyPlan" class="timeline-empty">
      ยังไม่มีแผนการรักษา
    </div>

    <div v-else class="timeline-container">
      <!-- Start / End date labels -->
      <div class="date-labels">
        <span>{{ startLabel }}</span>
        <span>{{ endLabel }}</span>
      </div>

      <!-- ── Track area ─────────────────────────────────────────── -->
      <div class="timeline-track-outer">
        <!-- Phase bars (flexbox, fills 100% width) -->
        <div class="timeline-track" role="img" aria-label="ไทม์ไลน์การรักษา">
          <!-- Intensive phase bar -->
          <div
            v-if="intensiveInfo"
            class="phase-bar intensive-bar"
            :style="{ width: intensivePct + '%' }"
            :title="`Intensive phase: ${intensiveInfo.durationMonths} เดือน`"
          >
            <span class="phase-label intensive-label">
              Intensive ({{ intensiveInfo.durationMonths }}M)
            </span>
          </div>

          <!-- Continuation phase bar -->
          <div
            v-if="continuationInfo"
            class="phase-bar continuation-bar"
            :style="{ width: continuationPct + '%' }"
            :title="`Continuation phase: ${continuationInfo.durationMonths} เดือน`"
          >
            <span class="phase-label continuation-label">
              Continuation ({{ continuationInfo.durationMonths }}M)
            </span>
          </div>

          <!-- Fallback bar when neither phase resolved -->
          <div v-if="!intensiveInfo && !continuationInfo" class="phase-bar unknown-bar" style="width: 100%">
            <span class="phase-label unknown-label">ไม่ทราบระยะ</span>
          </div>
        </div>

        <!-- ── Overlay layer: dots + today marker ──────────────── -->
        <div class="overlay-layer" aria-hidden="true">
          <!-- Phase boundary label -->
          <div
            v-if="showBoundary && boundaryPctClamped !== null"
            class="boundary-label"
            :style="{ left: boundaryPctClamped + '%' }"
            role="img"
            aria-label="วันที่เปลี่ยนเฟส"
          >
            <span>{{ boundaryLabel }}</span>
          </div>

          <!-- Follow-up visit dots -->
          <div
            v-for="f in followups"
            :key="f.id"
            class="followup-dot"
            :style="{ left: followupPct(f.followup_date) + '%' }"
            :title="dotTooltip(f)"
          />

          <!-- Today marker -->
          <div
            v-if="todayPct !== null"
            class="today-marker"
            :style="{ left: todayPct + '%' }"
          >
            <div class="today-line" />
            <span class="today-label">วันนี้</span>
          </div>
        </div>
      </div>
      <!-- ── End track area ──────────────────────────────────── -->

      <!-- Month ticks -->
      <div v-if="totalMonths > 0" class="month-ticks" aria-hidden="true">
        <span v-for="m in totalMonths" :key="m" class="month-tick">{{ m }}</span>
      </div>

      <!-- Phase legend -->
      <div class="phase-legend" aria-hidden="true">
        <span class="legend-item">
          <span class="legend-swatch legend-intensive" />
          Intensive
        </span>
        <span class="legend-item">
          <span class="legend-swatch legend-continuation" />
          Continuation
        </span>
        <span class="legend-item">
          <span class="legend-swatch legend-followup" />
          ติดตามผล
        </span>
        <span class="legend-item">
          <span class="legend-swatch legend-today" />
          วันนี้
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ── Wrapper ──────────────────────────────────────────────────────── */
.timeline-wrapper {
  padding: 4px 0;
}

/* ── Empty state ──────────────────────────────────────────────────── */
.timeline-empty {
  background: var(--color-bg-alt);
  border-radius: var(--radius-md);
  padding: 24px;
  text-align: center;
  font-size: 13px;
  color: var(--color-text-muted);
}

/* ── Container ────────────────────────────────────────────────────── */
.timeline-container {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

/* ── Date labels ──────────────────────────────────────────────────── */
.date-labels {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 500;
  color: var(--color-text-muted);
  padding: 0 2px;
}

/* ── Track outer (position: relative hosts absolute children) ─────── */
.timeline-track-outer {
  position: relative;
}

/* ── Phase bars row ───────────────────────────────────────────────── */
.timeline-track {
  display: flex;
  height: 42px;
  border-radius: var(--radius-sm);
  overflow: hidden;
  background: var(--color-bg-alt);
  /* children fill available space proportionally */
}

.phase-bar {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  transition: width 0.4s ease;
  min-width: 4px;
  flex-shrink: 0;
}

.intensive-bar {
  background: rgba(221, 91, 0, 0.13);
  border-right: 2px solid rgba(221, 91, 0, 0.3);
}

.continuation-bar {
  background: rgba(42, 157, 153, 0.13);
}

.unknown-bar {
  background: var(--color-bg-alt);
}

/* ── Phase labels (inside bars) ───────────────────────────────────── */
.phase-label {
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  padding: 0 12px;
  user-select: none;
  pointer-events: none;
}

.intensive-label {
  color: #dd5b00;
}

.continuation-label {
  color: #2a9d99;
}

.unknown-label {
  color: var(--color-text-muted);
}

/* ── Overlay layer (dots + today marker) ──────────────────────────── */
.overlay-layer {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

/* ── Follow-up dots ───────────────────────────────────────────────── */
.followup-dot {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 12px;
  height: 12px;
  background: var(--color-blue);
  border: 2.5px solid #fff;
  border-radius: 50%;
  box-shadow: 0 1px 4px rgba(0, 117, 222, 0.4);
  pointer-events: all;
  cursor: pointer;
  z-index: 2;
  transition: transform 0.15s ease;
}

.followup-dot:hover {
  transform: translate(-50%, -50%) scale(1.35);
}

/* ── Today marker ─────────────────────────────────────────────────── */
.today-marker {
  position: absolute;
  top: -6px;
  bottom: -16px;
  transform: translateX(-50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  z-index: 3;
  pointer-events: none;
}

.today-line {
  width: 2px;
  background: rgba(0, 0, 0, 0.65);
  flex: 1;
  border-radius: 1px;
}

.today-label {
  font-size: 10px;
  font-weight: 700;
  color: rgba(0, 0, 0, 0.65);
  white-space: nowrap;
  margin-top: 3px;
  letter-spacing: 0.3px;
}

/* Boundary label (phase transition date positioned above the boundary) */
.boundary-label {
  position: absolute;
  top: -18px;
  transform: translateX(-50%);
  font-size: 11px;
  font-weight: 700;
  color: var(--color-text-secondary);
  white-space: nowrap;
  z-index: 3;
  pointer-events: none;
}

/* ── Month ticks ──────────────────────────────────────────────────── */
.month-ticks {
  display: flex;
  margin-top: 16px; /* extra space so today-label doesn't overlap */
}

.month-tick {
  flex: 1;
  text-align: center;
  font-size: 11px;
  font-weight: 500;
  color: var(--color-text-muted);
  border-left: 1px dashed rgba(0, 0, 0, 0.1);
  padding-top: 2px;
  user-select: none;
}

.month-tick:first-child {
  border-left: none;
}

/* ── Legend ───────────────────────────────────────────────────────── */
.phase-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  margin-top: 10px;
  padding-top: 10px;
  border-top: var(--border);
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  color: var(--color-text-muted);
  user-select: none;
}

.legend-swatch {
  width: 11px;
  height: 11px;
  border-radius: 2px;
  flex-shrink: 0;
}

.legend-intensive {
  background: rgba(221, 91, 0, 0.5);
}

.legend-continuation {
  background: rgba(42, 157, 153, 0.5);
}

.legend-followup {
  background: var(--color-blue);
  border-radius: 50%;
}

.legend-today {
  background: rgba(0, 0, 0, 0.6);
  width: 3px;
  height: 11px;
  border-radius: 1px;
}
</style>