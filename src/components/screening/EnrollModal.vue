<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { X, UserPlus, CheckCircle, AlertCircle, Loader2 } from 'lucide-vue-next'
import { usePatientStore } from '@/stores/patient'
import { useSettingsStore } from '@/stores/settings'
import type { PatientDrugRecord, EnrollmentInput } from '@/types/patient'

const props = defineProps<{
  modelValue: boolean
  patients: PatientDrugRecord[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  enrolled: []
}>()

const patientStore = usePatientStore()
const settingsStore = useSettingsStore()

// ── Form state ────────────────────────────────────────────────────────────────
const tbType = ref('pulmonary')
const diagnosisDate = ref('')
const regimen = ref('2HRZE/4HR')
const treatmentStartDate = ref('')
const enrolledBy = ref('')
const notes = ref('')
const isSubmitting = ref(false)
const error = ref<string | null>(null)
const success = ref(false)

// ── Re-enrollment detection ───────────────────────────────────────────────────
const reenrollCount = computed(() =>
  props.patients.filter(
    (p) => p.is_enrolled && p.patient_status && p.patient_status !== 'active',
  ).length,
)
const hasReenrollPatients = computed(() => reenrollCount.value > 0)

// ── Reset form whenever the modal opens ───────────────────────────────────────
watch(
  () => props.modelValue,
  (val) => {
    if (val) {
      tbType.value = 'pulmonary'
      diagnosisDate.value = ''
      regimen.value = '2HRZE/4HR'
      treatmentStartDate.value = ''
      enrolledBy.value = ''
      notes.value = ''
      error.value = null
      success.value = false
      isSubmitting.value = false
    }
  },
)

// ── Actions ───────────────────────────────────────────────────────────────────
function close() {
  if (isSubmitting.value) return
  emit('update:modelValue', false)
}

async function submit() {
  if (!treatmentStartDate.value) {
    error.value = 'กรุณาระบุวันเริ่มการรักษา'
    return
  }

  isSubmitting.value = true
  error.value = null

  try {
    for (const patient of props.patients) {
      const input: EnrollmentInput = {
        hn: patient.hn,
        tb_type: tbType.value,
        diagnosis_date: diagnosisDate.value || null,
        regimen: regimen.value,
        treatment_start_date: treatmentStartDate.value,
        enrolled_by: enrolledBy.value || null,
        notes: notes.value || null,
      }
      await patientStore.enrollPatient(input)
    }

    success.value = true
    setTimeout(() => {
      emit('enrolled')
      close()
      success.value = false
    }, 1200)
  } catch (e) {
    error.value = String(e)
  } finally {
    isSubmitting.value = false
  }
}

function unfocus(e: Event) {
  (e.target as HTMLElement)?.blur()
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div v-if="modelValue" class="modal-overlay" @click.self="close">
        <div class="modal-panel" role="dialog" aria-modal="true" aria-labelledby="modal-title">
          <!-- ── Header ──────────────────────────────────────────────────────── -->
          <div class="modal-header">
            <div class="modal-title-row">
              <UserPlus :size="18" class="modal-title-icon" />
              <h2 id="modal-title" class="modal-title">ลงทะเบียนเข้าคลินิก TB</h2>
            </div>
            <button class="close-btn" @click="close" :disabled="isSubmitting" title="ปิด">
              <X :size="18" />
            </button>
          </div>

          <!-- ── Body ───────────────────────────────────────────────────────── -->
          <div class="modal-body">
            <!-- Patient chips -->
            <div class="section-label">ผู้ป่วยที่จะลงทะเบียน ({{ patients.length }} ราย)</div>
            <div class="patient-chips">
              <span v-for="p in patients.slice(0, 3)" :key="p.hn" class="patient-chip">
                <span class="patient-chip-hn">{{ p.hn }}</span>
                <span class="patient-chip-name">{{ p.full_name }}</span>
              </span>
              <span v-if="patients.length > 3" class="patient-chip patient-chip--more">
                +{{ patients.length - 3 }} รายอื่น
              </span>
            </div>

            <!-- Re-enrollment warning -->
            <div v-if="hasReenrollPatients" class="reenroll-warning">
              <span class="reenroll-warning-icon">⚠️</span>
              <span>
                มีผู้ป่วย <strong>{{ reenrollCount }} ราย</strong>
                เคยรักษาในคลินิกมาก่อน — ระบบจะสร้างแผนการรักษาใหม่โดยเก็บประวัติเดิมไว้
              </span>
            </div>

            <div class="form-divider" />

            <!-- TB Type -->
            <div class="form-group">
              <label class="form-label">ชนิดวัณโรค</label>
              <div class="radio-group">
                <label class="radio-label">
                  <input type="radio" v-model="tbType" value="pulmonary" />
                  <span class="radio-text">ปอด (Pulmonary)</span>
                </label>
                <label class="radio-label">
                  <input type="radio" v-model="tbType" value="extra_pulmonary" />
                  <span class="radio-text">นอกปอด (Extra-pulmonary)</span>
                </label>
              </div>
            </div>

            <!-- Diagnosis date -->
            <div class="form-group">
              <label class="form-label" for="diagnosisDate">วันที่วินิจฉัยยืนยัน</label>
              <input
                id="diagnosisDate"
                type="date"
                v-model="diagnosisDate"
                class="form-input"
                placeholder="ไม่ระบุ"
              />
            </div>

            <!-- Regimen -->
            <div class="form-group">
              <label class="form-label" for="regimen">สูตรยาที่ใช้</label>
              <select id="regimen" v-model="regimen" class="form-select">
                <option
                  v-for="reg in settingsStore.regimens"
                  :key="reg"
                  :value="reg"
                >
                  {{ reg }}
                </option>
              </select>
            </div>

            <!-- Treatment start date (required) -->
            <div class="form-group">
              <label class="form-label" for="treatmentStart">
                วันเริ่มการรักษา
                <span class="required" aria-hidden="true">*</span>
              </label>
              <input
                id="treatmentStart"
                type="date"
                v-model="treatmentStartDate"
                class="form-input"
                :class="{ 'form-input--error': error && !treatmentStartDate }"
                required
              />
            </div>

            <!-- Enrolled by -->
            <div class="form-group">
              <label class="form-label" for="enrolledBy">ลงทะเบียนโดย</label>
              <input
                id="enrolledBy"
                type="text"
                v-model="enrolledBy"
                list="staff-datalist"
                class="form-input"
                placeholder="ชื่อผู้บันทึก (ไม่จำเป็น)"
                autocomplete="off"
                @change="unfocus"
              />
              <datalist id="staff-datalist">
                <option
                  v-for="name in settingsStore.staffNames"
                  :key="name"
                  :value="name"
                />
              </datalist>
            </div>

            <!-- Notes -->
            <div class="form-group">
              <label class="form-label" for="notes">หมายเหตุ</label>
              <textarea
                id="notes"
                v-model="notes"
                class="form-textarea"
                placeholder="บันทึกเพิ่มเติม (ไม่จำเป็น)"
                rows="3"
              />
            </div>

            <!-- Error alert -->
            <div v-if="error" class="error-alert" role="alert">
              <AlertCircle :size="15" class="alert-icon" />
              <span>{{ error }}</span>
            </div>

            <!-- Success alert -->
            <div v-if="success" class="success-alert" role="status">
              <CheckCircle :size="15" class="alert-icon" />
              <span>ลงทะเบียนสำเร็จ {{ patients.length }} ราย</span>
            </div>
          </div>

          <!-- ── Footer ──────────────────────────────────────────────────────── -->
          <div class="modal-footer">
            <button class="btn-ghost" type="button" @click="close" :disabled="isSubmitting">
              ยกเลิก
            </button>
            <button
              class="btn-primary"
              type="button"
              :disabled="isSubmitting || success"
              @click="submit"
            >
              <Loader2 v-if="isSubmitting" :size="14" class="spin" />
              <UserPlus v-else :size="14" />
              {{ isSubmitting ? 'กำลังบันทึก…' : 'ลงทะเบียน' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* ── Modal transition ─────────────────────────────────────────────────────────── */
.modal-fade-enter-active {
  transition: opacity 0.2s ease;
}
.modal-fade-leave-active {
  transition: opacity 0.15s ease;
}
.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}

/* ── Overlay ──────────────────────────────────────────────────────────────────── */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
}

/* ── Panel ────────────────────────────────────────────────────────────────────── */
.modal-panel {
  background: var(--color-bg);
  border-radius: var(--radius-card);
  box-shadow: var(--shadow-deep);
  width: 540px;
  max-width: 95vw;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ── Header ───────────────────────────────────────────────────────────────────── */
.modal-header {
  padding: 20px 24px 16px;
  border-bottom: var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
}

.modal-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.modal-title-icon {
  color: var(--color-blue);
  flex-shrink: 0;
}

.modal-title {
  font-size: 17px;
  font-weight: 700;
  letter-spacing: -0.25px;
  color: var(--color-text);
  margin: 0;
}

.close-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 5px;
  color: var(--color-text-muted);
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.12s, color 0.12s;
  flex-shrink: 0;
}

.close-btn:hover:not(:disabled) {
  background: var(--color-bg-alt);
  color: var(--color-text);
}

.close-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* ── Body ─────────────────────────────────────────────────────────────────────── */
.modal-body {
  padding: 20px 24px;
  overflow-y: auto;
  flex: 1;
}

/* ── Section label ────────────────────────────────────────────────────────────── */
.section-label {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.5px;
  text-transform: uppercase;
  color: var(--color-text-muted);
  margin-bottom: 8px;
}

/* ── Patient chips ────────────────────────────────────────────────────────────── */
.patient-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 12px;
  background: var(--color-bg-alt);
  border-radius: var(--radius-md);
  margin-bottom: 4px;
}

.patient-chip {
  background: var(--color-bg);
  border: var(--border);
  border-radius: var(--radius-pill);
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  white-space: nowrap;
}

.patient-chip-hn {
  font-family: monospace;
  font-weight: 700;
  color: var(--color-text);
}

.patient-chip-name {
  color: var(--color-text-secondary);
}

.patient-chip--more {
  background: var(--color-badge-bg);
  color: var(--color-badge-text);
  border-color: rgba(9, 127, 232, 0.18);
  font-weight: 600;
}

/* ── Divider ──────────────────────────────────────────────────────────────────── */
.form-divider {
  height: 1px;
  background: rgba(0, 0, 0, 0.07);
  margin: 20px 0;
}

/* ── Form groups ──────────────────────────────────────────────────────────────── */
.form-group {
  margin-bottom: 16px;
}

.form-label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: 6px;
}

.required {
  color: var(--color-orange);
  margin-left: 2px;
}

.form-input,
.form-select,
.form-textarea {
  width: 100%;
  padding: 7px 10px;
  border: 1px solid rgba(0, 0, 0, 0.15);
  border-radius: var(--radius-sm);
  font-size: 14px;
  font-family: var(--font);
  color: var(--color-text);
  background: var(--color-bg);
  outline: none;
  transition:
    border-color 0.15s,
    box-shadow 0.15s;
}

.form-input:focus,
.form-select:focus,
.form-textarea:focus {
  border-color: var(--color-blue);
  box-shadow: 0 0 0 3px rgba(0, 117, 222, 0.1);
}

.form-input--error {
  border-color: var(--color-orange);
  box-shadow: 0 0 0 3px rgba(221, 91, 0, 0.1);
}

.form-textarea {
  resize: vertical;
  min-height: 72px;
  line-height: 1.5;
}

/* ── Radio group ──────────────────────────────────────────────────────────────── */
.radio-group {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

.radio-label {
  display: flex;
  align-items: center;
  gap: 7px;
  cursor: pointer;
  user-select: none;
}

.radio-label input[type='radio'] {
  width: 15px;
  height: 15px;
  accent-color: var(--color-blue);
  cursor: pointer;
  flex-shrink: 0;
}

.radio-text {
  font-size: 14px;
  color: var(--color-text);
}

/* ── Alerts ───────────────────────────────────────────────────────────────────── */
.error-alert,
.success-alert {
  border-radius: var(--radius-sm);
  padding: 10px 12px;
  font-size: 13px;
  margin-bottom: 4px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.error-alert {
  background: rgba(221, 91, 0, 0.08);
  border: 1px solid rgba(221, 91, 0, 0.22);
  color: var(--color-orange);
}

.success-alert {
  background: rgba(26, 174, 57, 0.08);
  border: 1px solid rgba(26, 174, 57, 0.22);
  color: var(--color-green);
}

.alert-icon {
  flex-shrink: 0;
}

/* ── Re-enrollment warning ────────────────────────────────────────────────────── */
.reenroll-warning {
  display: flex;
  align-items: flex-start;
  gap: 9px;
  padding: 10px 13px;
  border-radius: var(--radius-sm);
  border: 1px solid rgba(221, 91, 0, 0.2);
  border-left: 3px solid var(--color-orange);
  background: rgba(221, 91, 0, 0.06);
  font-size: 13px;
  color: var(--color-orange);
  line-height: 1.5;
  margin-top: 10px;
}

.reenroll-warning-icon {
  flex-shrink: 0;
  font-size: 14px;
  line-height: 1.5;
}

.reenroll-warning strong {
  font-weight: 700;
  color: var(--color-orange);
}

/* ── Footer ───────────────────────────────────────────────────────────────────── */
.modal-footer {
  padding: 16px 24px;
  border-top: var(--border);
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

/* ── Buttons ──────────────────────────────────────────────────────────────────── */
.btn-ghost {
  background: transparent;
  border: none;
  padding: 7px 16px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  transition: background 0.12s;
  font-family: var(--font);
}

.btn-ghost:hover:not(:disabled) {
  background: var(--color-bg-alt);
  color: var(--color-text);
}

.btn-ghost:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--color-blue);
  color: #fff;
  border: none;
  padding: 7px 16px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  gap: 6px;
  transition: background 0.12s;
  font-family: var(--font);
}

.btn-primary:hover:not(:disabled) {
  background: var(--color-blue-active);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* ── Spinner ──────────────────────────────────────────────────────────────────── */
.spin {
  animation: spin 0.9s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>