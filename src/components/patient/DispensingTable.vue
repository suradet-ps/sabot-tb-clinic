<script setup lang="ts">
import { computed, ref } from 'vue'
import { ArrowUpDown, ArrowUp, ArrowDown } from 'lucide-vue-next'
import DrugChip from '@/components/shared/DrugChip.vue'
import type { DispensingRecord } from '@/types/dispensing'

const props = defineProps<{
  records: DispensingRecord[]
}>()

// ── Sorting ───────────────────────────────────────────────────────────────

type SortKey = 'date' | 'drug' | 'class' | 'qty'
type SortDir = 'asc' | 'desc'

const sortKey = ref<SortKey>('date')
const sortDir = ref<SortDir>('desc')

function toggleSort(key: SortKey) {
  if (sortKey.value === key) {
    sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortKey.value = key
    sortDir.value = key === 'date' ? 'desc' : 'asc'
  }
}

const sortedRecords = computed<DispensingRecord[]>(() => {
  const list = [...props.records]
  const dir = sortDir.value === 'asc' ? 1 : -1

  return list.sort((a, b) => {
    switch (sortKey.value) {
      case 'date':
        return dir * a.vstdate.localeCompare(b.vstdate)
      case 'drug':
        return dir * ((a.drug_name ?? a.icode).localeCompare(b.drug_name ?? b.icode, 'th'))
      case 'class': {
        const classOrder: Record<string, number> = { H: 1, R: 2, Z: 3, E: 4 }
        const aOrd = classOrder[a.drug_class ?? ''] ?? 99
        const bOrd = classOrder[b.drug_class ?? ''] ?? 99
        return dir * (aOrd - bOrd)
      }
      case 'qty':
        return dir * ((a.qty ?? 0) - (b.qty ?? 0))
      default:
        return 0
    }
  })
})

// ── Stats ─────────────────────────────────────────────────────────────────

const uniqueDates = computed(() => new Set(props.records.map((r) => r.vstdate)).size)
const uniqueClasses = computed(() => new Set(props.records.map((r) => r.drug_class).filter(Boolean)).size)

// ── Helpers ───────────────────────────────────────────────────────────────

function toThaiDate(iso: string): string {
  try {
    const [y, m, d] = iso.split('-').map(Number)
    return `${String(d).padStart(2, '0')}/${String(m).padStart(2, '0')}/${y + 543}`
  } catch {
    return iso
  }
}

function rowClass(drugClass: string | null): string {
  switch (drugClass) {
    case 'H': return 'row-H'
    case 'R': return 'row-R'
    case 'Z': return 'row-Z'
    case 'E': return 'row-E'
    default:  return 'row-unknown'
  }
}

function sortIcon(key: SortKey): 'none' | 'asc' | 'desc' {
  if (sortKey.value !== key) return 'none'
  return sortDir.value
}
</script>

<template>
  <div class="disp-table-wrapper">

    <!-- ── Summary bar ─────────────────────────────────────────────── -->
    <div v-if="records.length > 0" class="summary-bar">
      <span class="summary-item">
        <span class="summary-value">{{ records.length }}</span>
        <span class="summary-label">รายการทั้งหมด</span>
      </span>
      <span class="summary-sep" aria-hidden="true">·</span>
      <span class="summary-item">
        <span class="summary-value">{{ uniqueDates }}</span>
        <span class="summary-label">วันที่รับยา</span>
      </span>
      <span class="summary-sep" aria-hidden="true">·</span>
      <span class="summary-item">
        <span class="summary-value">{{ uniqueClasses }}</span>
        <span class="summary-label">กลุ่มยา</span>
      </span>

      <!-- Drug class legend chips -->
      <div class="legend-chips" aria-hidden="true">
        <DrugChip v-for="cls in ['H', 'R', 'Z', 'E']" :key="cls" :drug="cls" size="sm" />
      </div>
    </div>

    <!-- ── Empty state ─────────────────────────────────────────────── -->
    <div v-if="records.length === 0" class="empty-state" role="status">
      <svg class="empty-icon" xmlns="http://www.w3.org/2000/svg" width="36" height="36"
        viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"
        stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M9 11l3 3L22 4"/>
        <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>
      </svg>
      <span class="empty-title">ไม่พบประวัติการจ่ายยา TB</span>
      <span class="empty-sub">ยังไม่มีรายการยาวัณโรคในระบบ HOSxP สำหรับผู้ป่วยรายนี้</span>
    </div>

    <!-- ── Table ───────────────────────────────────────────────────── -->
    <div v-else class="table-scroll">
      <table class="disp-table" aria-label="ประวัติการจ่ายยา TB">
        <thead>
          <tr>
            <!-- วันที่ -->
            <th class="th-date">
              <button class="sort-btn" @click="toggleSort('date')" aria-label="เรียงตามวันที่">
                <span>วันที่</span>
                <ArrowUpDown v-if="sortIcon('date') === 'none'" :size="12" class="sort-icon" />
                <ArrowUp    v-else-if="sortIcon('date') === 'asc'"  :size="12" class="sort-icon sort-icon-active" />
                <ArrowDown  v-else                                   :size="12" class="sort-icon sort-icon-active" />
              </button>
            </th>

            <!-- รายการยา -->
            <th class="th-name">
              <button class="sort-btn" @click="toggleSort('drug')" aria-label="เรียงตามชื่อยา">
                <span>รายการยา</span>
                <ArrowUpDown v-if="sortIcon('drug') === 'none'" :size="12" class="sort-icon" />
                <ArrowUp    v-else-if="sortIcon('drug') === 'asc'"  :size="12" class="sort-icon sort-icon-active" />
                <ArrowDown  v-else                                   :size="12" class="sort-icon sort-icon-active" />
              </button>
            </th>

            <!-- กลุ่มยา -->
            <th class="th-class">
              <button class="sort-btn" @click="toggleSort('class')" aria-label="เรียงตามกลุ่มยา">
                <span>กลุ่มยา</span>
                <ArrowUpDown v-if="sortIcon('class') === 'none'" :size="12" class="sort-icon" />
                <ArrowUp    v-else-if="sortIcon('class') === 'asc'"  :size="12" class="sort-icon sort-icon-active" />
                <ArrowDown  v-else                                    :size="12" class="sort-icon sort-icon-active" />
              </button>
            </th>

            <!-- จำนวน -->
            <th class="th-qty">
              <button class="sort-btn sort-btn-right" @click="toggleSort('qty')" aria-label="เรียงตามจำนวน">
                <span>จำนวน</span>
                <ArrowUpDown v-if="sortIcon('qty') === 'none'" :size="12" class="sort-icon" />
                <ArrowUp    v-else-if="sortIcon('qty') === 'asc'"  :size="12" class="sort-icon sort-icon-active" />
                <ArrowDown  v-else                                   :size="12" class="sort-icon sort-icon-active" />
              </button>
            </th>
          </tr>
        </thead>

        <tbody>
          <tr
            v-for="r in sortedRecords"
            :key="r.vstdate + '-' + r.icode"
            class="data-row"
            :class="rowClass(r.drug_class)"
          >
            <!-- วันที่ -->
            <td class="td-date">
              <span class="date-text">{{ toThaiDate(r.vstdate) }}</span>
            </td>

            <!-- รายการยา -->
            <td class="td-name">
              <span class="drug-name">{{ r.drug_name ?? r.icode }}</span>
              <span v-if="r.drug_name && r.icode" class="drug-icode">{{ r.icode }}</span>
            </td>

            <!-- กลุ่มยา -->
            <td class="td-class">
              <DrugChip v-if="r.drug_class" :drug="r.drug_class" size="sm" />
              <span v-else class="no-class">—</span>
            </td>

            <!-- จำนวน -->
            <td class="td-qty">
              <span class="qty-value">{{ r.qty != null ? r.qty : '—' }}</span>
              <span v-if="r.units" class="qty-unit">{{ r.units }}</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

  </div>
</template>

<style scoped>
/* ── Wrapper ──────────────────────────────────────────────────────── */
.disp-table-wrapper {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* ── Summary bar ──────────────────────────────────────────────────── */
.summary-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  padding: 8px 12px;
  background: var(--color-bg-alt);
  border-radius: var(--radius-sm);
}

.summary-item {
  display: flex;
  align-items: baseline;
  gap: 4px;
}

.summary-value {
  font-size: 14px;
  font-weight: 700;
  color: var(--color-text);
}

.summary-label {
  font-size: 11px;
  color: var(--color-text-muted);
}

.summary-sep {
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 1;
}

.legend-chips {
  display: flex;
  gap: 4px;
  margin-left: auto;
  flex-wrap: wrap;
}

/* ── Empty state ──────────────────────────────────────────────────── */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 40px 24px;
  text-align: center;
  color: var(--color-text-muted);
}

.empty-icon {
  opacity: 0.2;
  margin-bottom: 4px;
}

.empty-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.empty-sub {
  font-size: 13px;
  color: var(--color-text-muted);
  max-width: 320px;
  line-height: 1.5;
}

/* ── Table scroll container ───────────────────────────────────────── */
.table-scroll {
  overflow-x: auto;
  border-radius: var(--radius-sm);
  border: var(--border);
}

/* ── Table ────────────────────────────────────────────────────────── */
.disp-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
  min-width: 520px;
}

/* ── Head ─────────────────────────────────────────────────────────── */
thead tr {
  background: var(--color-bg-alt);
  border-bottom: var(--border);
}

thead th {
  padding: 0;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-align: left;
  white-space: nowrap;
}

/* Sort button fills the entire <th> */
.sort-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
  padding: 9px 12px;
  background: none;
  border: none;
  font-size: 12px;
  font-weight: 600;
  font-family: var(--font);
  color: var(--color-text-secondary);
  cursor: pointer;
  text-align: left;
  white-space: nowrap;
  transition: background 0.12s, color 0.12s;
}

.sort-btn:hover {
  background: rgba(0, 0, 0, 0.03);
  color: var(--color-text);
}

.sort-btn-right {
  justify-content: flex-end;
}

.sort-icon {
  flex-shrink: 0;
  opacity: 0.35;
}

.sort-icon-active {
  opacity: 1;
  color: var(--color-blue);
}

/* Column widths */
.th-date  { width: 110px; }
.th-class { width: 90px; }
.th-qty   { width: 110px; }

/* ── Body rows ────────────────────────────────────────────────────── */
.data-row {
  border-bottom: var(--border);
  border-left: 3px solid transparent;
  transition: background 0.1s;
}

.data-row:last-child {
  border-bottom: none;
}

.data-row:hover {
  background: var(--color-bg-alt);
}

/* Drug-class color coding via left border */
.row-H { border-left-color: var(--drug-H); }
.row-R { border-left-color: var(--drug-R); }
.row-Z { border-left-color: var(--drug-Z); }
.row-E { border-left-color: var(--drug-E); }
.row-unknown { border-left-color: transparent; }

/* ── Cells ────────────────────────────────────────────────────────── */
td {
  padding: 9px 12px;
  vertical-align: middle;
}

/* Date cell */
.td-date {
  white-space: nowrap;
}

.date-text {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-secondary);
  font-variant-numeric: tabular-nums;
}

/* Drug name cell — no additional styles needed; wrapping is default block behaviour */
.drug-name {
  font-size: 13px;
  color: var(--color-text);
  font-weight: 400;
  line-height: 1.4;
}

.drug-icode {
  display: block;
  font-size: 11px;
  color: var(--color-text-muted);
  margin-top: 1px;
  font-variant-numeric: tabular-nums;
}

/* Class chip cell */
.td-class {
  white-space: nowrap;
}

.no-class {
  font-size: 12px;
  color: var(--color-text-muted);
}

/* Qty cell */
.td-qty {
  white-space: nowrap;
  text-align: right;
}

.qty-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text);
  font-variant-numeric: tabular-nums;
}

.qty-unit {
  font-size: 11px;
  color: var(--color-text-muted);
  margin-left: 3px;
}
</style>