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

export interface AppConfig extends DbConfig {
  staff_names: string[]
  regimens: string[]
}

const DEFAULT_STAFF_NAMES = ['พยาบาลวิชาชีพ', 'เภสัชกร', 'แพทย์']
const DEFAULT_REGIMENS = ['2HRZE/4HR', '2HRZE/6HR']

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

  const staffNames = ref<string[]>([...DEFAULT_STAFF_NAMES])

  const drugCodes = ref({
    H: ['1430104'],
    R: ['1000265', '1000264'],
    E: ['1600004', '1000129'],
    Z: ['1000258'],
  })

  const regimens = ref<string[]>([...DEFAULT_REGIMENS])

  function buildAppConfig(): AppConfig {
    return {
      ...dbConfig.value,
      staff_names: [...staffNames.value],
      regimens: [...regimens.value],
    }
  }

  async function saveAllSettings(): Promise<void> {
    await invoke('save_db_config', { config: buildAppConfig() })
  }

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
      dbConfig.value = { ...config }
      isConnected.value = true
      try {
        await saveAllSettings()
      } catch (saveErr) {
        connectionError.value = `เชื่อมต่อสำเร็จ แต่บันทึกการตั้งค่าไม่สำเร็จ: ${String(saveErr)}`
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
      const saved = await invoke<AppConfig | null>('load_db_config')
      if (saved) {
        dbConfig.value = {
          host: saved.host,
          port: saved.port,
          database: saved.database,
          username: saved.username,
          password: saved.password,
        }
        staffNames.value = saved.staff_names.length ? saved.staff_names : [...DEFAULT_STAFF_NAMES]
        regimens.value = saved.regimens.length ? saved.regimens : [...DEFAULT_REGIMENS]
      }
    } catch (e) {
      console.warn('Could not load saved config:', e)
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
      dbConfig.value = { ...DEFAULT_CONFIG }
      staffNames.value = [...DEFAULT_STAFF_NAMES]
      regimens.value = [...DEFAULT_REGIMENS]
    }
  }

  async function addStaffName(name: string): Promise<boolean> {
    const trimmedName = name.trim()
    if (!trimmedName || staffNames.value.includes(trimmedName)) {
      return false
    }

    const previous = [...staffNames.value]
    staffNames.value = [...previous, trimmedName]
    try {
      await saveAllSettings()
      return true
    } catch (e) {
      staffNames.value = previous
      throw e
    }
  }

  async function removeStaffName(name: string): Promise<boolean> {
    const previous = [...staffNames.value]
    const next = previous.filter((item) => item !== name)
    if (next.length === previous.length) {
      return false
    }

    staffNames.value = next
    try {
      await saveAllSettings()
      return true
    } catch (e) {
      staffNames.value = previous
      throw e
    }
  }

  async function addRegimen(name: string): Promise<boolean> {
    const regimen = name.trim().toUpperCase()
    if (!regimen || regimens.value.includes(regimen)) {
      return false
    }

    const previous = [...regimens.value]
    regimens.value = [...previous, regimen]
    try {
      await saveAllSettings()
      return true
    } catch (e) {
      regimens.value = previous
      throw e
    }
  }

  async function removeRegimen(name: string): Promise<boolean> {
    if (regimens.value.length <= 1) {
      return false
    }

    const previous = [...regimens.value]
    const next = previous.filter((item) => item !== name)
    if (next.length === previous.length) {
      return false
    }

    regimens.value = next
    try {
      await saveAllSettings()
      return true
    } catch (e) {
      regimens.value = previous
      throw e
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
    saveAllSettings,
    loadSavedConfig,
    deleteSavedConfig,
    addStaffName,
    removeStaffName,
    addRegimen,
    removeRegimen,
  }
})
