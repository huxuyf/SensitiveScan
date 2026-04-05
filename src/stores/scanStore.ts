import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

interface AggregatedResult {
  file_path: string
  file_name: string
  file_size: number
  file_type: string
  sensitive_types: string
  count: number
}

interface ScanConfig {
  scan_paths: string[]
  exclude_paths: string[]
  max_file_size: number
  sensitive_types: string[]
}

interface Settings {
  max_file_size: number
  auto_mask_results: boolean
  export_format: string
  language: string
}

export const useScanStore = defineStore('scan', () => {
  // 核心状态数据
  const isScanning = ref(false)
  const isPaused = ref(false)
  const currentFile = ref('')
  const filesScanned = ref(0)
  const resultsFound = ref(0)
  const progressPercentage = ref(0)
  const elapsedSeconds = ref(0)
  const estimatedRemaining = ref(0)
  const scanSpeed = ref(0)
  const aggregatedResults = ref<AggregatedResult[]>([])
  const scanConfig = ref<ScanConfig>({
    scan_paths: [],
    exclude_paths: [],
    max_file_size: 100 * 1024 * 1024,
    sensitive_types: ['phonenumber', 'idcard', 'name', 'address']
  })
  const settings = ref<Settings>({
    max_file_size: 100 * 1024 * 1024,
    auto_mask_results: true,
    export_format: 'xlsx',
    language: 'zh-CN'
  })

  // 依赖派生的计算属性
  const scanStats = computed(() => ({
    filesScanned: filesScanned.value,
    resultsFound: resultsFound.value,
    elapsedSeconds: elapsedSeconds.value,
    estimatedRemaining: estimatedRemaining.value,
    scanSpeed: scanSpeed.value.toFixed(2)
  }))

  const startScan = (config: ScanConfig) => {
    scanConfig.value = config
    isScanning.value = true
    isPaused.value = false
    filesScanned.value = 0
    resultsFound.value = 0
    progressPercentage.value = 0
    elapsedSeconds.value = 0
    aggregatedResults.value = []
  }

  const pauseScan = () => { isPaused.value = true }
  const resumeScan = () => { isPaused.value = false }
  const stopScan = () => {
    isScanning.value = false
    isPaused.value = false
  }

  const updateProgress = (data: any) => {
    currentFile.value = data.current_file
    filesScanned.value = data.files_scanned
    resultsFound.value = data.results_found
    progressPercentage.value = data.progress_percentage
    elapsedSeconds.value = data.elapsed_seconds
    estimatedRemaining.value = data.estimated_remaining
    scanSpeed.value = data.scan_speed
  }

  const clearResults = () => {
    aggregatedResults.value = []
  }

  const setAggregatedResults = (data: AggregatedResult[]) => {
    aggregatedResults.value = data
  }

  // 利用 LocalStorage 将偏好设置本地持久化
  const updateSettings = (newSettings: Partial<Settings>) => {
    settings.value = { ...settings.value, ...newSettings }
    localStorage.setItem('scan-settings', JSON.stringify(settings.value))
  }

  const loadSettings = () => {
    const saved = localStorage.getItem('scan-settings')
    if (saved) {
      try {
        settings.value = { ...settings.value, ...JSON.parse(saved) }
      } catch (e) {
        console.error('加载本地设置数据失败:', e)
      }
    }
  }

  loadSettings()

  return {
    isScanning,
    isPaused,
    currentFile,
    filesScanned,
    resultsFound,
    progressPercentage,
    elapsedSeconds,
    estimatedRemaining,
    scanSpeed,
    aggregatedResults,
    scanConfig,
    settings,
    scanStats,
    startScan,
    pauseScan,
    resumeScan,
    stopScan,
    updateProgress,
    clearResults,
    setAggregatedResults,
    updateSettings,
    loadSettings
  }
})
