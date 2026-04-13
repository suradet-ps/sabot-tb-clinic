<script setup lang="ts">
import { computed, ref, watchEffect } from 'vue'
import { ChevronDown, FileX } from 'lucide-vue-next'
import DrugChip from '@/components/shared/DrugChip.vue'
import { useScreeningStore } from '@/stores/screening'
import type { PatientDrugRecord } from '@/types/patient'

const store = useScreeningStore()

// ── Header checkbox ref (needed to set indeterminate via JS property) ──────────
const headerCheckbox = ref<HTMLInputElement | null>(null)

// ── Sort state ───────────────────────────────────────────────────────────────
type SortKey =
  | 'hn'
  | 'full_name'
  | 'age'
  | 'first_dispensed'
  | 'last_dispensed'
  | 'visit_count'
type SortDir = 'asc' | 'desc'

const sortKey = ref<SortKey>('last_dispensed')
const sortDir = ref<SortDir>('desc')

function sortBy(key: SortKey) {
  if (sortKey.value === key) {
    sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortKey.value = key
    sortDir.value = 'desc'
  }
}

// ── Sorted results ────────────────────────────────────────────────────────────
const sortedResults = computed<PatientDrugRecord[]>(() => {
  const arr = [...store.results]
  const key = sortKey.value
  const dir = sortDir.value

  arr.sort((a, b) => {
    const av = a[key] as string | number | null | undefined
    const bv = b[key] as string | number | null | undefined

    if (av === null || av === undefined) return 1
    if (bv === null || bv === undefined) return -1

    let result: number
    if (typeof av === 'number' && typeof bv === 'number') {
      result = av - bv
    } else {
      result = String(av).localeCompare(String(bv), 'th')
    }
    return dir === 'asc' ? result : -result
  })

  return arr
})

// ── Selection helpers ─────────────────────────────────────────────────────────
// Selectable = not enrolled, OR enrolled but already discharged (non-active status)
const selectableRows = computed(() =>
  store.results.filter(
    (r) => !r.is_enrolled || (r.patient_status && r.patient_status !== 'active'),
  ),
)

const allSelected = computed(
  () =>
    selectableRows.value.length > 0 &&
    selectableRows.value.every((r) => store.selectedHns.has(r.hn)),
)

const someSelected = computed(
  () =>
    selectableRows.value.some((r) => store.selectedHns.has(r.hn)) && !allSelected.value,
)

// Sync indeterminate state — cannot be set via HTML attribute, must be a JS property
watchEffect(() => {
  if (headerCheckbox.value) {
    headerCheckbox.value.indeterminate = someSelected.value
  }
})

function toggleAll() {
  if (allSelected.value) {
    store.clearSelection()
  } else {
    selectableRows.value.forEach((r) => {
      if (!store.selectedHns.has(r.hn)) {
        store.toggleSelect(r.hn)
      }
    })
  }
}

function toggleRow(row: PatientDrugRecord) {
  // Actively enrolled patients cannot be re-enrolled; block only those
  if (row.is_enrolled && (!row.patient_status || row.patient_status === 'active')) return
  store.toggleSelect(row.hn)
}

// ── Formatters ────────────────────────────────────────────────────────────────
function toThaiDate(isoDate: string | null | undefined): string {
  if (!isoDate) return '-'
  try {
    const [y, m, d] = isoDate.split('-').map(Number)
    return `${String(d).padStart(2, '0')}/${String(m).padStart(2, '0')}/${y + 543}`
  } catch {
    return '-'
  }
}

function sexLabel(sex: string | null | undefined): string {
  if (sex === 'M' || sex === '1') return '♂'
  if (sex === 'F' || sex === '2') return '♀'
  return '-'
}
</script>

<template>
  <div class="table-wrapper">
    <table class="patient-table">
      <!-- ── Header ─────────────────────────────────────────────────────────── -->
      <thead>
        <tr>
          <th class="th-check">
            <input
              ref="headerCheckbox"
              type="checkbox"
              :checked="allSelected"
              :disabled="selectableRows.length === 0 || store.isLoading"
              @change="toggleAll"
              title="เลือก / ยกเลิกทั้งหมด"
            />
          </th>

          <th class="sortable" @click="sortBy('hn')">
            HN
            <ChevronDown
              :size="11"
              class="sort-icon"
              :class="{ 'sort-active': sortKey === 'hn', 'sort-asc': sortKey === 'hn' && sortDir === 'asc' }"
            />
          </th>

          <th class="sortable" @click="sortBy('full_name')">
            ชื่อ-สกุล
            <ChevronDown
              :size="11"
              class="sort-icon"
              :class="{
                'sort-active': sortKey === 'full_name',
                'sort-asc': sortKey === 'full_name' && sortDir === 'asc',
              }"
            />
          </th>

          <th class="sortable" @click="sortBy('age')">
            อายุ/เพศ
            <ChevronDown
              :size="11"
              class="sort-icon"
              :class="{
                'sort-active': sortKey === 'age',
                'sort-asc': sortKey === 'age' && sortDir === 'asc',
              }"
            />
          </th>

          <th class="sortable" @click="sortBy('first_dispensed')">
            จ่ายยาครั้งแรก
            <ChevronDown
              :size="11"
              class="sort-icon"
              :class="{
                'sort-active': sortKey === 'first_dispensed',
                'sort-asc': sortKey === 'first_dispensed' && sortDir === 'asc',
              }"
            />
          </th>

          <th class="sortable" @click="sortBy('last_dispensed')">
            จ่ายยาล่าสุด
            <ChevronDown
              :size="11"
              class="sort-icon"
              :class="{
                'sort-active': sortKey === 'last_dispensed',
                'sort-asc': sortKey === 'last_dispensed' && sortDir === 'asc',
              }"
            />
          </th>

          <th class="sortable th-center" @click="sortBy('visit_count')">
            ครั้ง
            <ChevronDown
              :size="11"
              class="sort-icon"
              :class="{
                'sort-active': sortKey === 'visit_count',
                'sort-asc': sortKey === 'visit_count' && sortDir === 'asc',
              }"
            />
          </th>

          <th>ยาที่ได้รับ</th>
          <th>สถานะ</th>
        </tr>
      </thead>

      <!-- ── Loading: 5 skeleton rows ──────────────────────────────────────── -->
      <tbody v-if="store.isLoading">
        <tr v-for="i in 5" :key="i" class="skeleton-row">
          <td>
            <div class="skeleton-line" style="width: 16px; height: 16px; border-radius: 3px" />
          </td>
          <td><div class="skeleton-line" style="width: 76px" /></td>
          <td><div class="skeleton-line" style="width: 148px" /></td>
          <td><div class="skeleton-line" style="width: 54px" /></td>
          <td><div class="skeleton-line" style="width: 88px" /></td>
          <td><div class="skeleton-line" style="width: 88px" /></td>
          <td>
            <div class="skeleton-line" style="width: 26px; margin: 0 auto" />
          </td>
          <td>
            <div style="display: flex; gap: 4px">
              <div
                class="skeleton-line"
                style="width: 26px; height: 20px; border-radius: 9999px"
              />
              <div
                class="skeleton-line"
                style="width: 26px; height: 20px; border-radius: 9999px"
              />
            </div>
          </td>
          <td><div class="skeleton-line" style="width: 96px" /></td>
        </tr>
      </tbody>

      <!-- ── Empty state ────────────────────────────────────────────────────── -->
      <tbody v-else-if="store.results.length === 0">
        <tr>
          <td colspan="9" class="empty-td">
            <div class="empty-state">
              <FileX class="empty-icon" />
              <p>ไม่พบข้อมูล กรุณาค้นหาใหม่</p>
            </div>
          </td>
        </tr>
      </tbody>

      <!-- ── Data rows ──────────────────────────────────────────────────────── -->
      <tbody v-else>
        <tr
          v-for="row in sortedResults"
          :key="row.hn"
          :class="{
            'row-enrolled': row.is_enrolled && (!row.patient_status || row.patient_status === 'active'),
            'row-discharged-selectable': row.is_enrolled && row.patient_status && row.patient_status !== 'active',
            'row-selected': store.selectedHns.has(row.hn) && (!row.is_enrolled || (row.patient_status && row.patient_status !== 'active')),
          }"
          @click="toggleRow(row)"
        >
          <td class="td-check">
            <input
              type="checkbox"
              :checked="store.selectedHns.has(row.hn)"
              :disabled="row.is_enrolled && (!row.patient_status || row.patient_status === 'active')"
              @click.stop
              @change="store.toggleSelect(row.hn)"
            />
          </td>

          <td class="hn-cell">{{ row.hn }}</td>

          <td class="name-cell">{{ row.full_name }}</td>

          <td class="age-sex-cell">
            {{ row.age !== null && row.age !== undefined ? `${row.age} ปี` : '-' }}
            <span class="sex-glyph">{{ sexLabel(row.sex) }}</span>
          </td>

          <td class="date-cell">{{ toThaiDate(row.first_dispensed) }}</td>

          <td class="date-cell">{{ toThaiDate(row.last_dispensed) }}</td>

          <td class="count-cell">{{ row.visit_count }}</td>

          <td>
            <div class="drug-chips">
              <DrugChip v-for="cls in row.drug_classes" :key="cls" :drug="cls" size="sm" />
            </div>
          </td>

          <td>
            <span v-if="row.patient_status === 'active'" class="enrolled-badge">✓ ลงทะเบียนแล้ว</span>
            <span
              v-else-if="row.patient_status && row.patient_status !== 'active'"
              class="discharged-badge"
            >✓ จำหน่ายแล้ว</span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
/* ── Wrapper ─────────────────────────────────────────────────────────────────── */
.table-wrapper {
  overflow-x: auto;
}

/* ── Table base ──────────────────────────────────────────────────────────────── */
.patient-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

/* ── Sticky header ───────────────────────────────────────────────────────────── */
thead {
  position: sticky;
  top: 0;
  background: var(--color-bg);
  z-index: 1;
}

thead th {
  padding: 10px 12px;
  text-align: left;
  font-weight: 600;
  font-size: 12px;
  color: var(--color-text-secondary);
  border-bottom: var(--border);
  white-space: nowrap;
}

.th-check {
  width: 40px;
}

.th-center {
  text-align: center;
}

/* ── Sortable column headers ─────────────────────────────────────────────────── */
.sortable {
  cursor: pointer;
  user-select: none;
}

.sortable:hover {
  color: var(--color-text);
}

.sort-icon {
  display: inline;
  vertical-align: middle;
  margin-left: 3px;
  opacity: 0;
  transition:
    opacity 0.15s,
    transform 0.2s;
  transform: rotate(0deg);
}

.sortable:hover .sort-icon {
  opacity: 0.5;
}

.sort-icon.sort-active {
  opacity: 1;
  color: var(--color-blue);
}

.sort-icon.sort-asc {
  transform: rotate(180deg);
}

/* ── Body rows ───────────────────────────────────────────────────────────────── */
tbody tr {
  border-bottom: var(--border);
  cursor: pointer;
  transition: background 0.1s;
}

tbody tr:hover:not(.row-enrolled):not(.row-discharged-selectable) {
  background: var(--color-bg-alt);
}

tbody tr.row-selected {
  background: #f0f7ff;
}

tbody tr.row-enrolled {
  opacity: 0.6;
  cursor: default;
}

/* Discharged patients are selectable for re-enrollment */
tbody tr.row-discharged-selectable {
  cursor: pointer;
}

tbody tr.row-discharged-selectable:hover:not(.row-selected) {
  background: rgba(221, 91, 0, 0.05);
}

tbody tr.row-discharged-selectable.row-selected {
  background: rgba(221, 91, 0, 0.09);
}

/* ── Cells ───────────────────────────────────────────────────────────────────── */
td {
  padding: 10px 12px;
  vertical-align: middle;
}

.td-check {
  width: 40px;
}

.hn-cell {
  font-weight: 600;
  font-family: monospace;
  font-size: 13px;
  color: var(--color-text);
  white-space: nowrap;
}

.name-cell {
  font-size: 13px;
  white-space: nowrap;
}

.age-sex-cell {
  font-size: 13px;
  white-space: nowrap;
  color: var(--color-text-secondary);
}

.sex-glyph {
  margin-left: 4px;
  font-size: 13px;
}

.date-cell {
  font-size: 12px;
  color: var(--color-text-secondary);
  white-space: nowrap;
}

.count-cell {
  font-size: 13px;
  font-weight: 600;
  text-align: center;
}

.drug-chips {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
  min-width: 60px;
}

.enrolled-badge {
  background: rgba(26, 174, 57, 0.1);
  color: #1aae39;
  padding: 2px 8px;
  border-radius: 9999px;
  font-size: 11px;
  font-weight: 600;
  white-space: nowrap;
}

.discharged-badge {
  background: rgba(163, 158, 152, 0.15);
  color: var(--color-text-muted);
  padding: 2px 8px;
  border-radius: 9999px;
  font-size: 11px;
  font-weight: 600;
  white-space: nowrap;
}

/* ── Skeleton rows ───────────────────────────────────────────────────────────── */
.skeleton-row td {
  padding: 10px 12px;
}

.skeleton-line {
  height: 14px;
  border-radius: 4px;
  background: linear-gradient(90deg, #f0f0f0 25%, #e8e8e8 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: shimmer 1.4s infinite;
}

@keyframes shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

/* ── Empty state ─────────────────────────────────────────────────────────────── */
.empty-td {
  text-align: center;
}

.empty-state {
  padding: 48px 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  color: var(--color-text-muted);
}

.empty-state p {
  margin: 0;
  font-size: 14px;
}

.empty-icon {
  width: 32px;
  height: 32px;
  margin-bottom: 12px;
  opacity: 0.4;
}
</style>