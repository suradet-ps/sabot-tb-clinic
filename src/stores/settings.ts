import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface DbConfig {
  host: string
  port: number
  database: string
  username: string
  password: string
}

const DEFAULT_CONFIG: DbConfig = {
  host: 'localhost',
  port: 3306,
  database: 'hosxp',
  username: 'root',
  password: '',
}

export const useSettingsStore = defineStore('settings', () => {
  const dbConfig = ref<DbConfig>({ ...DEFAULT_CONFIG })
  const isConnected = ref(false)
  const isConnecting = ref(false)
  const connectionError = ref<string | null>(null)

  const staffNames = ref<string[]>(['พยาบาลวิชาชีพ', 'เภสัชกร', 'แพทย์'])

  const drugCodes = ref({
    H: ['1430104'],
    R: ['1000265', '1000264'],
    E: ['1600004', '1000129'],
    Z: ['1000258'],
  })

  const regimens = ref<string[]>(['2HRZE/4HR', '2HRZE/6HR'])

  async function testConnection(config: DbConfig): Promise<boolean> {
    try {
      isConnecting.value = true
      connectionError.value = null
      const result = await invoke<boolean>('test_mysql_connection', { config })
      return result
    } catch (e) {
      connectionError.value = String(e)
      return false
    } finally {
      isConnecting.value = false
    }
  }

  async function connect(config: DbConfig): Promise<void> {
    try {
      isConnecting.value = true
      connectionError.value = null
      await invoke('connect_mysql', { config })
      dbConfig.value = config
      isConnected.value = true
      // Persist settings after every successful connection
      try {
        await invoke('save_db_config', { config })
      } catch (saveErr) {
        console.warn('Could not persist DB config:', saveErr)
      }
    } catch (e) {
      connectionError.value = String(e)
      isConnected.value = false
    } finally {
      isConnecting.value = false
    }
  }

  async function checkConnection(): Promise<void> {
    try {
      const status = await invoke<boolean>('get_mysql_status')
      isConnected.value = status
    } catch {
      isConnected.value = false
    }
  }

  /**
   * Load the previously persisted DbConfig from disk.
   * Pre-fills dbConfig so the Settings form shows the last-used values.
   * Does NOT attempt to reconnect automatically.
   */
  async function loadSavedConfig(): Promise<void> {
    try {
      const saved = await invoke<DbConfig | null>('load_db_config')
      if (saved) {
        dbConfig.value = saved
      }
    } catch {
      // Silent — no config file yet or backend not ready
    }
  }

  /**
   * Delete the persisted config from disk and reset in-memory config to defaults.
   */
  async function deleteSavedConfig(): Promise<void> {
    try {
      await invoke('delete_db_config')
    } catch (e) {
      console.warn('Could not delete saved config:', e)
    } finally {
      // Always reset in-memory state regardless of whether the backend call succeeded
      dbConfig.value = { ...DEFAULT_CONFIG }
    }
  }

  return {
    dbConfig,
    isConnected,
    isConnecting,
    connectionError,
    staffNames,
    drugCodes,
    regimens,
    testConnection,
    connect,
    checkConnection,
    loadSavedConfig,
    deleteSavedConfig,
  }
})