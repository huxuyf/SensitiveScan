import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

interface ScanResult {
  id: string
  file_path: string
  sheet_name?: string
  row: number
  column: number
  sensitive_type: string
  content: string
  masked_content: string
  found_at: string
}

interface ScanConfig {
  scan_paths: string[]
  exclude_paths: string[]
  max_file_size: number
  sensitive_types: string[]
  thread_count: number
}

interface ScanSnapshot {
  id: string
  config: ScanConfig
  scanned_files: string[]
  pending_files: string[]
  results_count: number
  created_at: string
  last_updated: string
}

type ScanState = 'idle' | 'running' | 'paused' | 'stopping' | 'stopped' | 'completed' | 'failed'

interface ScanError {
  category: string
  message: string
  context?: string
  timestamp: string
  recoverable: boolean
}

interface Settings {
  max_file_size: number
  auto_mask_results: boolean
  export_format: string
  language: string
  enable_logging: boolean
  log_level: 'error' | 'warning' | 'info' | 'debug'
}

export const useEnhancedScanStore = defineStore('enhancedScan', () => {
  // State
  const scanState = ref<ScanState>('idle')
  const currentFile = ref('')
  const filesScanned = ref(0)
  const resultsFound = ref(0)
  const progressPercentage = ref(0)
  const elapsedSeconds = ref(0)
  const estimatedRemaining = ref(0)
  const scanSpeed = ref(0)
  const results = ref<ScanResult[]>([])
  const scanConfig = ref<ScanConfig>({
    scan_paths: [],
    exclude_paths: [],
    max_file_size: 100 * 1024 * 1024,
    sensitive_types: ['phonenumber', 'idcard', 'name', 'address'],
    thread_count: 4
  })
  const currentSnapshot = ref<ScanSnapshot | null>(null)
  const availableSnapshots = ref<ScanSnapshot[]>([])
  const scanErrors = ref<ScanError[]>([])
  const settings = ref<Settings>({
    max_file_size: 100 * 1024 * 1024,
    auto_mask_results: true,
    export_format: 'xlsx',
    language: 'zh-CN',
    enable_logging: true,
    log_level: 'info'
  })

  // Computed
  const isScanning = computed(() => scanState.value === 'running')
  const isPaused = computed(() => scanState.value === 'paused')
  const isStopping = computed(() => scanState.value === 'stopping')
  const canPause = computed(() => scanState.value === 'running')
  const canResume = computed(() => scanState.value === 'paused')
  const canStop = computed(() => ['running', 'paused'].includes(scanState.value))
  const canStart = computed(() => ['idle', 'stopped', 'completed', 'failed'].includes(scanState.value))

  const scanStats = computed(() => ({
    filesScanned: filesScanned.value,
    resultsFound: resultsFound.value,
    elapsedSeconds: elapsedSeconds.value,
    estimatedRemaining: estimatedRemaining.value,
    scanSpeed: scanSpeed.value.toFixed(2),
    state: scanState.value
  }))

  const hasErrors = computed(() => scanErrors.value.length > 0)
  const recentErrors = computed(() => scanErrors.value.slice(-10))

  // Actions
  const startScan = async (config: ScanConfig) => {
    try {
      scanConfig.value = config
      scanState.value = 'running'
      filesScanned.value = 0
      resultsFound.value = 0
      progressPercentage.value = 0
      elapsedSeconds.value = 0
      results.value = []
      scanErrors.value = []
      currentSnapshot.value = null

      // TODO: Call Tauri command
      // await invoke('start_scan', { config })

      console.log('Scan started with config:', config)
    } catch (error) {
      handleScanError('SCAN_START', error as Error)
      scanState.value = 'failed'
    }
  }

  const pauseScan = async () => {
    if (!canPause.value) return

    try {
      scanState.value = 'paused'

      // TODO: Call Tauri command
      // await invoke('pause_scan')

      console.log('Scan paused')
    } catch (error) {
      handleScanError('SCAN_PAUSE', error as Error)
    }
  }

  const resumeScan = async () => {
    if (!canResume.value) return

    try {
      scanState.value = 'running'

      // TODO: Call Tauri command
      // await invoke('resume_scan')

      console.log('Scan resumed')
    } catch (error) {
      handleScanError('SCAN_RESUME', error as Error)
    }
  }

  const stopScan = async () => {
    if (!canStop.value) return

    try {
      scanState.value = 'stopping'

      // TODO: Call Tauri command
      // const snapshot = await invoke('stop_scan')
      // currentSnapshot.value = snapshot

      console.log('Scan stopping gracefully')

      // Wait for graceful shutdown
      await new Promise(resolve => setTimeout(resolve, 2000))

      scanState.value = 'stopped'
    } catch (error) {
      handleScanError('SCAN_STOP', error as Error)
      scanState.value = 'failed'
    }
  }

  const resumeFromSnapshot = async (snapshot: ScanSnapshot) => {
    try {
      scanConfig.value = snapshot.config
      currentSnapshot.value = snapshot
      scanState.value = 'running'

      // TODO: Call Tauri command
      // await invoke('resume_scan', { snapshot })

      console.log('Resumed from snapshot:', snapshot.id)
    } catch (error) {
      handleScanError('SCAN_RESUME_SNAPSHOT', error as Error)
      scanState.value = 'failed'
    }
  }

  const updateProgress = (data: {
    current_file: string
    files_scanned: number
    results_found: number
    progress_percentage: number
    elapsed_seconds: number
    estimated_remaining_seconds: number
    scan_speed: number
  }) => {
    currentFile.value = data.current_file
    filesScanned.value = data.files_scanned
    resultsFound.value = data.results_found
    progressPercentage.value = data.progress_percentage
    elapsedSeconds.value = data.elapsed_seconds
    estimatedRemaining.value = data.estimated_remaining_seconds
    scanSpeed.value = data.scan_speed
  }

  const addResult = (result: ScanResult) => {
    results.value.push(result)
  }

  const addResults = (newResults: ScanResult[]) => {
    results.value.push(...newResults)
  }

  const clearResults = () => {
    results.value = []
  }

  const handleScanError = (category: string, error: Error) => {
    const scanError: ScanError = {
      category,
      message: error.message || 'Unknown error',
      context: error.stack,
      timestamp: new Date().toISOString(),
      recoverable: true // Default to recoverable
    }

    scanErrors.value.push(scanError)

    // Log to console if logging enabled
    if (settings.value.enable_logging) {
      console.error(`[${category}]`, error)
    }

    // TODO: Send to backend for logging
    // if (settings.value.enable_logging) {
    //   invoke('log_error', { category, message: error.message, context: error.stack })
    // }
  }

  const clearErrors = () => {
    scanErrors.value = []
  }

  const loadSnapshots = async () => {
    try {
      // TODO: Call Tauri command
      // const snapshots = await invoke('get_scan_snapshots')
      // availableSnapshots.value = snapshots

      console.log('Loaded scan snapshots')
    } catch (error) {
      handleScanError('SNAPSHOT_LOAD', error as Error)
    }
  }

  const deleteSnapshot = async (snapshotId: string) => {
    try {
      // TODO: Call Tauri command
      // await invoke('delete_scan_snapshot', { id: snapshotId })

      availableSnapshots.value = availableSnapshots.value.filter(s => s.id !== snapshotId)

      console.log('Deleted snapshot:', snapshotId)
    } catch (error) {
      handleScanError('SNAPSHOT_DELETE', error as Error)
    }
  }

  const updateSettings = (newSettings: Partial<Settings>) => {
    settings.value = {
      ...settings.value,
      ...newSettings
    }
    localStorage.setItem('enhanced-scan-settings', JSON.stringify(settings.value))
  }

  const loadSettings = () => {
    const saved = localStorage.getItem('enhanced-scan-settings')
    if (saved) {
      try {
        settings.value = { ...settings.value, ...JSON.parse(saved) }
      } catch (e) {
        console.error('Failed to load settings:', e)
      }
    }
  }

  const getRecentLogs = async (lines: number = 100) => {
    try {
      // TODO: Call Tauri command
      // const logs = await invoke('get_recent_logs', { lines })
      // return logs

      return []
    } catch (error) {
      handleScanError('LOG_FETCH', error as Error)
      return []
    }
  }

  const clearLogs = async () => {
    try {
      // TODO: Call Tauri command
      // await invoke('clear_logs')

      console.log('Logs cleared')
    } catch (error) {
      handleScanError('LOG_CLEAR', error as Error)
    }
  }

  // Initialize
  loadSettings()

  return {
    // State
    scanState,
    isScanning,
    isPaused,
    isStopping,
    currentFile,
    filesScanned,
    resultsFound,
    progressPercentage,
    elapsedSeconds,
    estimatedRemaining,
    scanSpeed,
    results,
    scanConfig,
    currentSnapshot,
    availableSnapshots,
    scanErrors,
    settings,
    // Computed
    scanStats,
    hasErrors,
    recentErrors,
    canPause,
    canResume,
    canStop,
    canStart,
    // Actions
    startScan,
    pauseScan,
    resumeScan,
    stopScan,
    resumeFromSnapshot,
    updateProgress,
    addResult,
    addResults,
    clearResults,
    handleScanError,
    clearErrors,
    loadSnapshots,
    deleteSnapshot,
    updateSettings,
    loadSettings,
    getRecentLogs,
    clearLogs
  }
})
