import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PatientDrugRecord, SearchFilters } from '@/types/patient'

export const useScreeningStore = defineStore('screening', () => {
  const results = ref<PatientDrugRecord[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const selectedHns = ref<Set<string>>(new Set())
  const filters = ref<SearchFilters>({
    enrollment_status: 'all',
    page: 1,
    page_size: 50,
  })
  const totalCount = ref(0)

  async function search(): Promise<void> {
    try {
      isLoading.value = true
      error.value = null
      const data = await invoke<PatientDrugRecord[]>('search_tb_patients', {
        filters: filters.value,
      })
      results.value = data
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  function toggleSelect(hn: string): void {
    if (selectedHns.value.has(hn)) {
      selectedHns.value.delete(hn)
    } else {
      selectedHns.value.add(hn)
    }
  }

  function clearSelection(): void {
    selectedHns.value.clear()
  }

  const selectedRecords = computed(() =>
    results.value.filter(
      (r) =>
        selectedHns.value.has(r.hn) &&
        (!r.is_enrolled || (r.patient_status && r.patient_status !== 'active')),
    ),
  )

  return {
    results,
    isLoading,
    error,
    selectedHns,
    filters,
    totalCount,
    selectedRecords,
    search,
    toggleSelect,
    clearSelection,
  }
})