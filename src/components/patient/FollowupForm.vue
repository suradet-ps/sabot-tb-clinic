<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { X, Loader2, AlertTriangle } from 'lucide-vue-next'
import type { FollowupInput } from '@/types/treatment'

// ── Props & Emits ─────────────────────────────────────────────────────────

const props = defineProps<{
  modelValue: boolean
  hn: string
  monthNumber?: number
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'saved'): void
}>()

// ── Side effect options ───────────────────────────────────────────────────

interface SideEffectOption {
  key: string
  label: string
  labelEn: string
  drug: string
  isPriority?: boolean
}

const SIDE_EFFECT_OPTIONS: SideEffectOption[] = [
  {
    key: 'ชาปลายมือเท้า',
    label: 'ชาปลายมือเท้า',
    labelEn: 'Peripheral neuropathy',
    drug: 'H',
  },
  {
    key: 'ตับอักเสบ',
    label: 'ตับอักเสบ',
    labelEn: 'Hepatotoxicity',
    drug: 'H/R/Z',
  },
  {
    key: 'ตาพร่า/ตาบอดสี',
    label: 'ตาพร่า/ตาบอดสี',
    labelEn: 'Optic neuritis',
    drug: 'E',
    isPriority: true,
  },
  {
    key: 'ข้อเจ็บ/เก๊าท์',
    label: 'ข้อเจ็บ/เก๊าท์',
    labelEn: 'Hyperuricemia (Gout)',
    drug: 'Z',
  },
  {
    key: 'ไข้หนาวสั่น',
    label: 'ไข้หนาวสั่น',
    labelEn: 'Flu-like syndrome',
    drug: 'R',
  },
  {
    key: 'เลือดออกผิดปกติ',
    label: 'เลือดออกผิดปกติ',
    labelEn: 'Thrombocytopenia',
    drug: 'R',
  },
  {
    key: 'ผื่นคัน',
    label: 'ผื่นคัน',
    labelEn: 'Rash / Allergy',
    drug: '-',
  },
  {
    key: 'คลื่นไส้อาเจียน',
    label: 'คลื่นไส้อาเจียน',
    labelEn: 'Nausea / Vomiting',
    drug: '-',
  },
]

// ── Form state ────────────────────────────────────────────────────────────

function todayISO(): string {
  return new Date().toISOString().slice(0, 10)
}

interface FormState {
  followup_date: string
  month_number: number | null
  weight_kg: number | null
  sputum_result: string
  xray_result: string
  adherence: string
  notes: string
  created_by: string
}

function createEmptyForm(): FormState {
  return {
    followup_date: todayISO(),
    month_number: props.monthNumber ?? null,
    weight_kg: null,
    sputum_result: '',
    xray_result: '',
    adherence: '',
    notes: '',
    created_by: '',
  }
}

const form = ref<FormState>(createEmptyForm())
const selectedSideEffects = ref<string[]>([])
const isSubmitting = ref(false)
const submitError = ref<string | null>(null)

// Warn when optic neuritis (E side effect) is checked
const hasOpticNeuritisChecked = ref(false)

watch(selectedSideEffects, (list) => {
  hasOpticNeuritisChecked.value = list.includes('ตาพร่า/ตาบอดสี')
})

// ── Reset on open ─────────────────────────────────────────────────────────

watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      form.value = createEmptyForm()
      selectedSideEffects.value = []
      submitError.value = null
    }
  },
)

watch(
  () => props.monthNumber,
  (n) => {
    form.value.month_number = n ?? null
  },
)

// ── Submit ────────────────────────────────────────────────────────────────

async function handleSubmit() {
  submitError.value = null

  if (!form.value.followup_date) {
    submitError.value = 'กรุณาระบุวันที่ติดตามผล'
    return
  }

  isSubmitting.value = true
  try {
    const input: FollowupInput = {
      hn: props.hn,
      followup_date: form.value.followup_date,
      month_number: form.value.month_number,
      weight_kg: form.value.weight_kg,
      sputum_result: form.value.sputum_result || null,
      xray_result: form.value.xray_result || null,
      side_effects: selectedSideEffects.value.length > 0 ? [...selectedSideEffects.value] : null,
      adherence: form.value.adherence || null,
      notes: form.value.notes.trim() || null,
      created_by: form.value.created_by.trim() || null,
    }
    await invoke('add_followup', { followup: input })
    emit('saved')
    close()
  } catch (e) {
    submitError.value = String(e)
  } finally {
    isSubmitting.value = false
  }
}

// ── Close helpers ─────────────────────────────────────────────────────────

function close() {
  if (isSubmitting.value) return
  emit('update:modelValue', false)
}

function onOverlayPointerDown(e: MouseEvent) {
  if (e.target === e.currentTarget) close()
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') close()
}
</script>

<template>
  <Teleport to="body">
    <Transition name="panel">
      <div
        v-if="modelValue"
        class="panel-overlay"
        role="presentation"
        @pointerdown="onOverlayPointerDown"
        @keydown="onKeydown"
      >
        <!-- Panel -->
        <aside
          class="panel"
          role="dialog"
          aria-modal="true"
          aria-labelledby="fup-panel-title"
          tabindex="-1"
          @pointerdown.stop
        >
          <!-- ── Header ──────────────────────────────────────────── -->
          <div class="panel-header">
            <div class="panel-header-info">
              <h2 id="fup-panel-title" class="panel-title">บันทึกการติดตามผล</h2>
              <span class="panel-hn">HN {{ hn }}</span>
            </div>
            <button
              class="btn-close"
              @click="close"
              :disabled="isSubmitting"
              aria-label="ปิด"
              type="button"
            >
              <X :size="18" />
            </button>
          </div>

          <!-- ── Body ───────────────────────────────────────────── -->
          <div class="panel-body">
            <form
              id="followup-form"
              class="followup-form"
              @submit.prevent="handleSubmit"
              novalidate
            >
              <!-- Date + Month row -->
              <div class="form-row-2">
                <div class="form-group">
                  <label class="form-label" for="fup-date">
                    วันที่ติดตามผล
                    <span class="required" aria-hidden="true">*</span>
                  </label>
                  <input
                    id="fup-date"
                    type="date"
                    class="form-input"
                    v-model="form.followup_date"
                    required
                    :disabled="isSubmitting"
                  />
                </div>

                <div class="form-group">
                  <label class="form-label" for="fup-month">เดือนที่รักษา</label>
                  <input
                    id="fup-month"
                    type="number"
                    class="form-input"
                    v-model.number="form.month_number"
                    min="1"
                    max="24"
                    placeholder="เช่น 3"
                    :disabled="isSubmitting"
                  />
                </div>
              </div>

              <!-- Weight -->
              <div class="form-group">
                <label class="form-label" for="fup-weight">น้ำหนัก (กก.)</label>
                <input
                  id="fup-weight"
                  type="number"
                  class="form-input"
                  v-model.number="form.weight_kg"
                  step="0.1"
                  min="20"
                  max="200"
                  placeholder="เช่น 58.5"
                  :disabled="isSubmitting"
                />
              </div>

              <!-- Sputum + X-ray row -->
              <div class="form-row-2">
                <div class="form-group">
                  <label class="form-label" for="fup-sputum">ผลเสมหะ</label>
                  <div class="select-wrap">
                    <select
                      id="fup-sputum"
                      class="form-select"
                      v-model="form.sputum_result"
                      :disabled="isSubmitting"
                    >
                      <option value="">— เลือก —</option>
                      <option value="negative">ผลลบ (Negative)</option>
                      <option value="positive">ผลบวก (Positive)</option>
                      <option value="not_done">ไม่ได้ตรวจ</option>
                    </select>
                  </div>
                </div>

                <div class="form-group">
                  <label class="form-label" for="fup-xray">ผล X-Ray</label>
                  <div class="select-wrap">
                    <select
                      id="fup-xray"
                      class="form-select"
                      v-model="form.xray_result"
                      :disabled="isSubmitting"
                    >
                      <option value="">— เลือก —</option>
                      <option value="improved">ดีขึ้น (Improved)</option>
                      <option value="stable">คงที่ (Stable)</option>
                      <option value="worse">แย่ลง (Worse)</option>
                      <option value="not_done">ไม่ได้ตรวจ</option>
                    </select>
                  </div>
                </div>
              </div>

              <!-- Adherence -->
              <div class="form-group">
                <label class="form-label" for="fup-adherence">การรับยา (Adherence)</label>
                <div class="select-wrap">
                  <select
                    id="fup-adherence"
                    class="form-select"
                    v-model="form.adherence"
                    :disabled="isSubmitting"
                  >
                    <option value="">— เลือก —</option>
                    <option value="good">ดี (Good)</option>
                    <option value="fair">พอใช้ (Fair)</option>
                    <option value="poor">ไม่ดี (Poor)</option>
                  </select>
                </div>
              </div>

              <!-- Side effect checkboxes -->
              <div class="form-group">
                <label class="form-label">ผลข้างเคียงที่รายงาน</label>

                <!-- Optic neuritis alert -->
                <Transition name="fade">
                  <div v-if="hasOpticNeuritisChecked" class="se-priority-alert" role="alert">
                    <AlertTriangle :size="14" class="se-alert-icon" />
                    <span>
                      พบการรายงานตาพร่า/ตาบอดสี — ควรพิจารณาหยุดยา Ethambutol (E)
                      และนัดตรวจตาโดยเร็ว
                    </span>
                  </div>
                </Transition>

                <div class="checkbox-list">
                  <label
                    v-for="se in SIDE_EFFECT_OPTIONS"
                    :key="se.key"
                    class="checkbox-item"
                    :class="{
                      'checkbox-item-priority': se.isPriority,
                      'checkbox-item-checked-priority':
                        se.isPriority && selectedSideEffects.includes(se.key),
                    }"
                  >
                    <input
                      type="checkbox"
                      class="checkbox-input"
                      :value="se.key"
                      v-model="selectedSideEffects"
                      :disabled="isSubmitting"
                    />
                    <span class="checkbox-content">
                      <span class="checkbox-label-th">{{ se.label }}</span>
                      <span class="checkbox-label-en">{{ se.labelEn }}</span>
                    </span>
                    <span class="drug-tag">{{ se.drug }}</span>
                  </label>
                </div>
              </div>

              <!-- Notes -->
              <div class="form-group">
                <label class="form-label" for="fup-notes">บันทึกเพิ่มเติม</label>
                <textarea
                  id="fup-notes"
                  class="form-textarea"
                  v-model="form.notes"
                  rows="3"
                  placeholder="บันทึกอาการ ความคืบหน้า หรือข้อสังเกต..."
                  :disabled="isSubmitting"
                />
              </div>

              <!-- Created by -->
              <div class="form-group">
                <label class="form-label" for="fup-by">บันทึกโดย</label>
                <input
                  id="fup-by"
                  type="text"
                  class="form-input"
                  v-model="form.created_by"
                  placeholder="ชื่อเจ้าหน้าที่"
                  :disabled="isSubmitting"
                />
              </div>

              <!-- Submit error -->
              <div v-if="submitError" class="form-error" role="alert">
                <AlertTriangle :size="14" style="flex-shrink: 0" />
                <span>{{ submitError }}</span>
              </div>
            </form>
          </div>

          <!-- ── Footer ──────────────────────────────────────────── -->
          <div class="panel-footer">
            <button
              type="button"
              class="btn-cancel"
              @click="close"
              :disabled="isSubmitting"
            >
              ยกเลิก
            </button>
            <button
              type="submit"
              form="followup-form"
              class="btn-save"
              :disabled="isSubmitting"
            >
              <Loader2 v-if="isSubmitting" :size="14" class="spin" aria-hidden="true" />
              <span>{{ isSubmitting ? 'กำลังบันทึก...' : 'บันทึกการติดตามผล' }}</span>
            </button>
          </div>
        </aside>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* ── Overlay ──────────────────────────────────────────────────────── */
.panel-overlay {
  position: fixed;
  inset: 0;
  z-index: 900;
  background: rgba(0, 0, 0, 0.28);
  display: flex;
  align-items: stretch;
  justify-content: flex-end;
}

/* ── Panel ────────────────────────────────────────────────────────── */
.panel {
  width: 420px;
  max-width: 100vw;
  background: var(--color-bg);
  box-shadow: var(--shadow-deep);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  outline: none;
}

/* ── Header ───────────────────────────────────────────────────────── */
.panel-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 20px 20px 16px;
  border-bottom: var(--border);
  flex-shrink: 0;
  background: var(--color-bg);
}

.panel-header-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.panel-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--color-text);
  letter-spacing: -0.125px;
  margin: 0;
}

.panel-hn {
  font-size: 12px;
  color: var(--color-text-muted);
  font-weight: 600;
  letter-spacing: 0.3px;
}

.btn-close {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  cursor: pointer;
  padding: 6px;
  color: var(--color-text-muted);
  border-radius: var(--radius-sm);
  flex-shrink: 0;
  transition: color 0.15s, background 0.15s;
}

.btn-close:hover:not(:disabled) {
  color: var(--color-text);
  background: var(--color-bg-alt);
}

.btn-close:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* ── Body ─────────────────────────────────────────────────────────── */
.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

/* ── Form ─────────────────────────────────────────────────────────── */
.followup-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-row-2 {
  display: flex;
  gap: 12px;
}

.form-row-2 > .form-group {
  flex: 1;
  min-width: 0;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  user-select: none;
}

.required {
  color: var(--color-orange);
  margin-left: 1px;
}

/* ── Inputs ───────────────────────────────────────────────────────── */
.form-input,
.form-textarea {
  font-family: var(--font);
  font-size: 13px;
  color: var(--color-text);
  background: var(--color-bg);
  border: var(--border);
  border-radius: var(--radius-sm);
  padding: 8px 10px;
  width: 100%;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.form-input:focus,
.form-textarea:focus {
  outline: none;
  border-color: var(--color-blue);
  box-shadow: 0 0 0 3px rgba(0, 117, 222, 0.12);
}

.form-input:disabled,
.form-textarea:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.form-textarea {
  resize: vertical;
  min-height: 76px;
  line-height: 1.55;
}

/* ── Select wrapper ───────────────────────────────────────────────── */
.select-wrap {
  position: relative;
}

.select-wrap::after {
  content: '';
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  width: 0;
  height: 0;
  border-left: 4px solid transparent;
  border-right: 4px solid transparent;
  border-top: 5px solid var(--color-text-muted);
  pointer-events: none;
}

.form-select {
  font-family: var(--font);
  font-size: 13px;
  color: var(--color-text);
  background: var(--color-bg);
  border: var(--border);
  border-radius: var(--radius-sm);
  padding: 8px 30px 8px 10px;
  width: 100%;
  appearance: none;
  cursor: pointer;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.form-select:focus {
  outline: none;
  border-color: var(--color-blue);
  box-shadow: 0 0 0 3px rgba(0, 117, 222, 0.12);
}

.form-select:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

/* ── Side-effect priority alert ───────────────────────────────────── */
.se-priority-alert {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  background: rgba(221, 91, 0, 0.08);
  border: 1px solid rgba(221, 91, 0, 0.25);
  border-radius: var(--radius-sm);
  padding: 9px 12px;
  font-size: 12px;
  font-weight: 500;
  color: #b84a00;
  line-height: 1.5;
  margin-bottom: 2px;
}

.se-alert-icon {
  flex-shrink: 0;
  margin-top: 1px;
  color: var(--color-orange);
}

/* ── Checkbox list ────────────────────────────────────────────────── */
.checkbox-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  background: var(--color-bg-alt);
  border: var(--border);
  border-radius: var(--radius-sm);
  padding: 6px 8px;
}

.checkbox-item {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 6px 7px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background 0.12s;
}

.checkbox-item:hover {
  background: rgba(0, 0, 0, 0.04);
}

.checkbox-item-checked-priority {
  background: rgba(221, 91, 0, 0.06);
  outline: 1px solid rgba(221, 91, 0, 0.2);
}

.checkbox-input {
  flex-shrink: 0;
  width: 15px;
  height: 15px;
  cursor: pointer;
  accent-color: var(--color-blue);
  margin: 0;
}

.checkbox-content {
  display: flex;
  flex-direction: column;
  gap: 1px;
  flex: 1;
  min-width: 0;
}

.checkbox-label-th {
  font-size: 13px;
  color: var(--color-text);
  line-height: 1.3;
}

.checkbox-label-en {
  font-size: 11px;
  color: var(--color-text-muted);
  line-height: 1.3;
}

.drug-tag {
  flex-shrink: 0;
  font-size: 10px;
  font-weight: 600;
  color: var(--color-text-muted);
  background: var(--color-bg);
  border: var(--border);
  border-radius: var(--radius-pill);
  padding: 1px 6px;
  white-space: nowrap;
}

/* ── Form error ───────────────────────────────────────────────────── */
.form-error {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  background: rgba(221, 91, 0, 0.08);
  border: 1px solid rgba(221, 91, 0, 0.2);
  border-radius: var(--radius-sm);
  padding: 9px 12px;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-orange);
  line-height: 1.45;
}

/* ── Footer ───────────────────────────────────────────────────────── */
.panel-footer {
  display: flex;
  gap: 8px;
  padding: 14px 20px;
  border-top: var(--border);
  flex-shrink: 0;
  background: var(--color-bg);
}

.btn-cancel {
  flex: 1;
  padding: 9px 16px;
  background: rgba(0, 0, 0, 0.05);
  border: none;
  border-radius: var(--radius-sm);
  font-size: 13px;
  font-weight: 600;
  font-family: var(--font);
  cursor: pointer;
  color: var(--color-text-secondary);
  transition: background 0.15s;
}

.btn-cancel:hover:not(:disabled) {
  background: rgba(0, 0, 0, 0.09);
}

.btn-save {
  flex: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 9px 16px;
  background: var(--color-blue);
  border: none;
  border-radius: var(--radius-sm);
  font-size: 13px;
  font-weight: 600;
  font-family: var(--font);
  cursor: pointer;
  color: #fff;
  transition: background 0.15s;
}

.btn-save:hover:not(:disabled) {
  background: var(--color-blue-active);
}

.btn-cancel:disabled,
.btn-save:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ── Panel slide-in transition ────────────────────────────────────── */
.panel-enter-active {
  transition: opacity 0.22s ease;
}
.panel-leave-active {
  transition: opacity 0.18s ease;
}
.panel-enter-from,
.panel-leave-to {
  opacity: 0;
}

.panel-enter-active .panel {
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}
.panel-leave-active .panel {
  transition: transform 0.2s cubic-bezier(0.4, 0, 0.6, 1);
}
.panel-enter-from .panel,
.panel-leave-to .panel {
  transform: translateX(100%);
}

/* ── Alert fade transition ────────────────────────────────────────── */
.fade-enter-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

/* ── Spin animation ───────────────────────────────────────────────── */
.spin {
  animation: spin 0.9s linear infinite;
  flex-shrink: 0;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>