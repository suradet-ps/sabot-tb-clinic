import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface AppointmentRecord {
  hn: string
  full_name: string | null
  nextdate: string // YYYY-MM-DD
}

export const useAppointmentsStore = defineStore('appointments', () => {
  const appointments = ref<AppointmentRecord[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const daysAhead = ref(30)

  // Computed once at store creation — stable reference for the current day
  const todayISO = new Date().toISOString().split('T')[0]

  const todayAppointments = computed(() =>
    appointments.value.filter((a) => a.nextdate === todayISO),
  )

  const upcomingAppointments = computed(() =>
    appointments.value.filter((a) => a.nextdate > todayISO),
  )

  async function fetchAppointments(days?: number): Promise<void> {
    isLoading.value = true
    error.value = null
    try {
      const effectiveDays = days ?? daysAhead.value
      appointments.value = await invoke<AppointmentRecord[]>('get_appointments', {
        daysAhead: effectiveDays,
      })
    } catch (e) {
      error.value = String(e)
      appointments.value = []
    } finally {
      isLoading.value = false
    }
  }

  return {
    appointments,
    isLoading,
    error,
    daysAhead,
    todayISO,
    todayAppointments,
    upcomingAppointments,
    fetchAppointments,
  }
})