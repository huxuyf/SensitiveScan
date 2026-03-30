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
}

export const useScanStore = defineStore('scan', () => {
  // State
  const isScanning = ref(false)
  const isPaused = ref(false)
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
    sensitive_types: ['phonenumber', 'idcard', 'name', 'address']
  })

  // Computed
  const scanStats = computed(() => ({
    filesScanned: filesScanned.value,
    resultsFound: resultsFound.value,
    elapsedSeconds: elapsedSeconds.value,
    estimatedRemaining: estimatedRemaining.value,
    scanSpeed: scanSpeed.value.toFixed(2)
  }))

  // Actions
  const startScan = (config: ScanConfig) => {
    scanConfig.value = config
    isScanning.value = true
    isPaused.value = false
    filesScanned.value = 0
    resultsFound.value = 0
    progressPercentage.value = 0
    elapsedSeconds.value = 0
    results.value = []
  }

  const pauseScan = () => {
    isPaused.value = true
  }

  const resumeScan = () => {
    isPaused.value = false
  }

  const stopScan = () => {
    isScanning.value = false
    isPaused.value = false
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

  const clearResults = () => {
    results.value = []
  }

  return {
    // State
    isScanning,
    isPaused,
    currentFile,
    filesScanned,
    resultsFound,
    progressPercentage,
    elapsedSeconds,
    estimatedRemaining,
    scanSpeed,
    results,
    scanConfig,
    // Computed
    scanStats,
    // Actions
    startScan,
    pauseScan,
    resumeScan,
    stopScan,
    updateProgress,
    addResult,
    clearResults
  }
})
