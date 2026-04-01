<template>
  <div class="scan-page">
    <!-- Scan Enabled Card -->
    <el-card class="scan-enabled-card" shadow="hover">
      <div class="card-content">
        <div class="card-left">
          <div class="card-header">
            <h3 class="card-title">扫描功能</h3>
            <p class="card-description">
              选择要扫描的文件夹，系统将自动识别文件中的敏感信息，包括手机号、身份证、姓名、地址等
            </p>
          </div>
        </div>
        <div class="card-right">
          <el-switch
            v-model="scanEnabled"
            size="large"
            active-color="#4a90e2"
            inactive-color="#dcdfe6"
          />
        </div>
      </div>
    </el-card>

    <!-- Scan Configuration -->
    <div class="section-title">扫描配置</div>
    <el-card class="config-card" shadow="hover">
      <div class="config-grid">
        <!-- Scan Paths -->
        <div class="config-item">
          <div class="config-header">
            <h4 class="config-title">扫描路径</h4>
            <p class="config-desc">选择要扫描的文件夹路径</p>
          </div>
          <div class="path-input-group">
            <el-input
              v-model="newPath"
              placeholder="输入扫描路径"
              @keyup.enter="addPath"
              clearable
            />
            <el-button type="primary" @click="addPath">添加</el-button>
            <el-button @click="selectFolder">
              <el-icon><FolderOpened /></el-icon>
              选择文件夹
            </el-button>
          </div>
          <div class="path-list" v-if="scanForm.scan_paths.length > 0">
            <el-tag
              v-for="(path, index) in scanForm.scan_paths"
              :key="index"
              closable
              @close="removePath(index)"
              class="path-tag"
              type="info"
            >
              {{ path }}
            </el-tag>
          </div>
        </div>

        <!-- Exclude Paths -->
        <div class="config-item">
          <div class="config-header">
            <h4 class="config-title">排除路径</h4>
            <p class="config-desc">设置要排除扫描的文件夹</p>
          </div>
          <div class="path-input-group">
            <el-input
              v-model="newExcludePath"
              placeholder="输入要排除的路径"
              @keyup.enter="addExcludePath"
              clearable
            />
            <el-button @click="addExcludePath">添加</el-button>
            <el-button @click="selectExcludeFolder">
              <el-icon><FolderOpened /></el-icon>
              选择文件夹
            </el-button>
          </div>
          <div class="path-list" v-if="scanForm.exclude_paths.length > 0">
            <el-tag
              v-for="(path, index) in scanForm.exclude_paths"
              :key="index"
              closable
              @close="removeExcludePath(index)"
              class="path-tag"
              type="warning"
            >
              {{ path }}
            </el-tag>
          </div>
        </div>
      </div>
    </el-card>

    <!-- Visual Settings -->
    <div class="section-title">视觉设置</div>
    <el-card class="visual-card" shadow="hover">
      <div class="visual-grid">
        <div class="setting-item">
          <div class="setting-content">
            <h4 class="setting-title">显示进度</h4>
            <p class="setting-desc">在扫描过程中实时显示进度信息</p>
          </div>
          <el-switch
            v-model="visualSettings.showProgress"
            size="large"
            active-color="#4a90e2"
            inactive-color="#dcdfe6"
          />
        </div>

        <div class="setting-item">
          <div class="setting-content">
            <h4 class="setting-title">显示结果</h4>
            <p class="setting-desc">扫描完成后自动显示结果列表</p>
          </div>
          <el-switch
            v-model="visualSettings.showResults"
            size="large"
            active-color="#4a90e2"
            inactive-color="#dcdfe6"
          />
        </div>

        <div class="setting-item">
          <div class="setting-content">
            <h4 class="setting-title">自动保存</h4>
            <p class="setting-desc">扫描结果自动保存到历史记录</p>
          </div>
          <el-switch
            v-model="visualSettings.autoSave"
            size="large"
            active-color="#4a90e2"
            inactive-color="#dcdfe6"
          />
        </div>

        <div class="setting-item">
          <div class="setting-content">
            <h4 class="setting-title">声音提示</h4>
            <p class="setting-desc">扫描完成时播放提示音</p>
          </div>
          <el-switch
            v-model="visualSettings.soundNotification"
            size="large"
            active-color="#4a90e2"
            inactive-color="#dcdfe6"
          />
        </div>
      </div>
    </el-card>

    <!-- Sensitive Types -->
    <div class="section-title">敏感信息类型</div>
    <el-card class="types-card" shadow="hover">
      <div class="types-grid">
        <div
          v-for="type in sensitiveTypes"
          :key="type.value"
          class="type-item"
          :class="{ 'type-selected': scanForm.sensitive_types.includes(type.value) }"
          @click="toggleType(type.value)"
        >
          <div class="type-icon" :style="{ color: type.color }">
            <component :is="type.icon" />
          </div>
          <div class="type-info">
            <h4 class="type-title">{{ type.label }}</h4>
            <p class="type-desc">{{ type.description }}</p>
          </div>
          <div class="type-check">
            <el-icon v-if="scanForm.sensitive_types.includes(type.value)">
              <CircleCheck />
            </el-icon>
          </div>
        </div>
      </div>
    </el-card>

    <!-- Action Buttons -->
    <div class="action-buttons">
      <el-button
        type="primary"
        size="large"
        @click="startScan"
        :disabled="!canStartScan || scanStore.isScanning"
        :loading="scanStore.isScanning"
      >
        <el-icon><Search /></el-icon>
        <span>{{ scanStore.isScanning ? '扫描中...' : '开始扫描' }}</span>
      </el-button>
      <el-button
        v-if="scanStore.isScanning"
        size="large"
        @click="pauseScan"
        :disabled="scanStore.isPaused"
      >
        <el-icon><VideoPause /></el-icon>
        <span>暂停</span>
      </el-button>
      <el-button
        v-if="scanStore.isScanning"
        size="large"
        @click="resumeScan"
        :disabled="!scanStore.isPaused"
      >
        <el-icon><VideoPlay /></el-icon>
        <span>继续</span>
      </el-button>
      <el-button
        v-if="scanStore.isScanning"
        size="large"
        type="danger"
        @click="stopScan"
      >
        <el-icon><CircleClose /></el-icon>
        <span>停止</span>
      </el-button>
    </div>

    <!-- Progress Panel -->
    <el-card v-if="scanStore.isScanning" class="progress-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <span class="title">扫描进度</span>
          <el-button type="text" @click="scanStore.isScanning = false">关闭</el-button>
        </div>
      </template>

      <div class="progress-content">
        <div class="progress-stats">
          <div class="stat-item">
            <div class="stat-label">当前文件</div>
            <div class="stat-value">{{ scanStore.currentFile || '准备中...' }}</div>
          </div>
          <div class="stat-item">
            <div class="stat-label">已扫描文件</div>
            <div class="stat-value">{{ scanStore.filesScanned }}</div>
          </div>
          <div class="stat-item">
            <div class="stat-label">发现结果</div>
            <div class="stat-value highlight">{{ scanStore.resultsFound }}</div>
          </div>
          <div class="stat-item">
            <div class="stat-label">扫描速度</div>
            <div class="stat-value">{{ scanStore.scanSpeed }} 文件/秒</div>
          </div>
          <div class="stat-item">
            <div class="stat-label">已用时间</div>
            <div class="stat-value">{{ formatTime(scanStore.elapsedSeconds) }}</div>
          </div>
          <div class="stat-item">
            <div class="stat-label">预计剩余</div>
            <div class="stat-value">{{ formatTime(scanStore.estimatedRemaining) }}</div>
          </div>
        </div>

        <div class="progress-bar-container">
          <el-progress
            :percentage="Math.round(scanStore.progressPercentage)"
            :format="(percentage) => `${percentage}%`"
            :stroke-width="20"
            striped
            striped-flow
          />
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useScanStore } from '../stores/scanStore'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import {
  FolderOpened,
  Search,
  VideoPause,
  VideoPlay,
  CircleClose,
  CircleCheck,
  Phone,
  Grid,
  User,
  Location
} from '@element-plus/icons-vue'

const scanStore = useScanStore()

const scanEnabled = ref(true)
const scanForm = ref({
  scan_paths: [],
  exclude_paths: [],
  max_file_size: 100 * 1024 * 1024,
  sensitive_types: ['phonenumber', 'idcard', 'name', 'address']
})

const visualSettings = ref({
  showProgress: true,
  showResults: true,
  autoSave: true,
  soundNotification: false
})

const newPath = ref('')
const newExcludePath = ref('')

const sensitiveTypes = [
  {
    value: 'phonenumber',
    label: '手机号码',
    description: '识别11位手机号码',
    icon: Phone,
    color: '#4a90e2'
  },
  {
    value: 'idcard',
    label: '身份证号',
    description: '识别18位身份证号码',
    icon: Grid,
    color: '#f56c6c'
  },
  {
    value: 'name',
    label: '姓名',
    description: '识别中文姓名',
    icon: User,
    color: '#67c23a'
  },
  {
    value: 'address',
    label: '地址',
    description: '识别中文地址',
    icon: Location,
    color: '#e6a23c'
  }
]

const canStartScan = computed(() => scanForm.value.scan_paths.length > 0 && scanEnabled.value)

const addPath = () => {
  if (newPath.value.trim()) {
    scanForm.value.scan_paths.push(newPath.value.trim())
    newPath.value = ''
  }
}

const removePath = (index: number) => {
  scanForm.value.scan_paths.splice(index, 1)
}

const addExcludePath = () => {
  if (newExcludePath.value.trim()) {
    scanForm.value.exclude_paths.push(newExcludePath.value.trim())
    newExcludePath.value = ''
  }
}

const removeExcludePath = (index: number) => {
  scanForm.value.exclude_paths.splice(index, 1)
}

const selectFolder = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择扫描文件夹'
    })
    if (selected) {
      if (Array.isArray(selected)) {
        scanForm.value.scan_paths.push(selected[0])
      } else {
        scanForm.value.scan_paths.push(selected)
      }
      ElMessage.success('已添加扫描路径')
    }
  } catch (error) {
    console.error('Failed to select folder:', error)
    ElMessage.error('选择文件夹失败')
  }
}

const selectExcludeFolder = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择要排除的文件夹'
    })
    if (selected) {
      if (Array.isArray(selected)) {
        scanForm.value.exclude_paths.push(selected[0])
      } else {
        scanForm.value.exclude_paths.push(selected)
      }
      ElMessage.success('已添加排除路径')
    }
  } catch (error) {
    console.error('Failed to select exclude folder:', error)
    ElMessage.error('选择排除文件夹失败')
  }
}

const toggleType = (type: string) => {
  const index = scanForm.value.sensitive_types.indexOf(type)
  if (index > -1) {
    scanForm.value.sensitive_types.splice(index, 1)
  } else {
    scanForm.value.sensitive_types.push(type)
  }
}

const startScan = async () => {
  if (!canStartScan.value) {
    ElMessage.warning('请至少选择一个扫描路径')
    return
  }

  try {
    scanForm.value.max_file_size = scanStore.settings.max_file_size || (100 * 1024 * 1024)

    scanStore.startScan(scanForm.value)
    const result = await invoke<string>('start_scan', {
      scanPaths: scanForm.value.scan_paths,
      excludePaths: scanForm.value.exclude_paths,
      maxFileSize: scanForm.value.max_file_size,
      sensitiveTypes: scanForm.value.sensitive_types
    })

    const data = JSON.parse(result)
    if (data.status === 'started') {
      ElMessage.success('扫描已启动')
    } else {
      ElMessage.error('启动扫描失败')
      scanStore.stopScan()
    }
  } catch (error) {
    console.error('Start scan error:', error)
    ElMessage.error('启动扫描失败: ' + (error as Error).message)
    scanStore.stopScan()
  }
}

const pauseScan = async () => {
  try {
    await invoke<string>('pause_scan')
    scanStore.pauseScan()
    ElMessage.info('扫描已暂停')
  } catch (error) {
    ElMessage.error('暂停扫描失败')
    console.error(error)
  }
}

const resumeScan = async () => {
  try {
    await invoke<string>('resume_scan')
    scanStore.resumeScan()
    ElMessage.info('扫描已继续')
  } catch (error) {
    ElMessage.error('继续扫描失败')
    console.error(error)
  }
}

const stopScan = async () => {
  try {
    await invoke<string>('stop_scan')
    scanStore.stopScan()
    ElMessage.warning('扫描已停止')
  } catch (error) {
    ElMessage.error('停止扫描失败')
    console.error(error)
  }
}

const formatTime = (seconds: number) => {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60
  return `${hours}h ${minutes}m ${secs}s`
}

// Event listeners
let unlistenProgress: (() => void) | null = null
let unlistenResult: (() => void) | null = null
let unlistenComplete: (() => void) | null = null

onMounted(async () => {
  try {
    unlistenProgress = await listen<any>('scan-progress', (event) => {
      scanStore.updateProgress(event.payload)
    })

    unlistenResult = await listen<any>('scan-result', (event) => {
      scanStore.addResult(event.payload)
    })

    unlistenComplete = await listen<any>('scan-complete', (event) => {
      scanStore.stopScan()
      ElMessage.success('扫描已完成')
    })
  } catch (error) {
    console.error('Failed to setup event listeners:', error)
  }
})

onUnmounted(() => {
  unlistenProgress?.()
  unlistenResult?.()
  unlistenComplete?.()
})
</script>

<style scoped lang="css">
.scan-page {
  max-width: 1200px;
  margin: 0 auto;
}

/* Section Title */
.section-title {
  font-size: 16px;
  font-weight: 600;
  color: #333;
  margin: 24px 0 16px 0;
}

/* Scan Enabled Card */
.scan-enabled-card {
  margin-bottom: 24px;
}

.card-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-left {
  flex: 1;
}

.card-header {
  margin-bottom: 0;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
  color: #333;
  margin: 0 0 8px 0;
}

.card-description {
  color: #666;
  font-size: 14px;
  line-height: 1.5;
  margin: 0;
}

.card-right {
  margin-left: 24px;
}

/* Config Card */
.config-card {
  margin-bottom: 24px;
}

.config-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
}

.config-item {
  padding: 16px;
  background-color: #f8f9fa;
  border-radius: 8px;
}

.config-header {
  margin-bottom: 16px;
}

.config-title {
  font-size: 14px;
  font-weight: 600;
  color: #333;
  margin: 0 0 4px 0;
}

.config-desc {
  color: #666;
  font-size: 13px;
  margin: 0;
}

.path-input-group {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.path-input-group :deep(.el-input) {
  flex: 1;
}

.path-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.path-tag {
  max-width: 300px;
}

/* Visual Card */
.visual-card {
  margin-bottom: 24px;
}

.visual-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  background-color: #f8f9fa;
  border-radius: 8px;
}

.setting-content {
  flex: 1;
}

.setting-title {
  font-size: 14px;
  font-weight: 600;
  color: #333;
  margin: 0 0 4px 0;
}

.setting-desc {
  color: #666;
  font-size: 13px;
  margin: 0;
}

/* Types Card */
.types-card {
  margin-bottom: 24px;
}

.types-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.type-item {
  display: flex;
  align-items: center;
  padding: 16px;
  background-color: #f8f9fa;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s;
  border: 2px solid transparent;
}

.type-item:hover {
  background-color: #e3f2fd;
  transform: translateY(-2px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.type-selected {
  background-color: #e3f2fd;
  border-color: #4a90e2;
}

.type-icon {
  font-size: 32px;
  margin-right: 16px;
}

.type-info {
  flex: 1;
}

.type-title {
  font-size: 14px;
  font-weight: 600;
  color: #333;
  margin: 0 0 4px 0;
}

.type-desc {
  color: #666;
  font-size: 13px;
  margin: 0;
}

.type-check {
  font-size: 24px;
  color: #4a90e2;
}

/* Action Buttons */
.action-buttons {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
  justify-content: center;
}

.action-buttons :deep(.el-button) {
  min-width: 120px;
}

/* Progress Card */
.progress-card {
  margin-bottom: 24px;
}

.progress-card .card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.progress-content {
  padding: 16px 0;
}

.progress-stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  padding: 12px;
  background-color: #f8f9fa;
  border-radius: 8px;
}

.stat-label {
  font-size: 13px;
  color: #666;
  margin-bottom: 8px;
}

.stat-value {
  font-size: 18px;
  font-weight: 600;
  color: #333;
}

.stat-value.highlight {
  color: #f56c6c;
}

.progress-bar-container {
  margin-top: 16px;
}

/* Responsive */
@media (max-width: 768px) {
  .config-grid,
  .visual-grid,
  .types-grid,
  .progress-stats {
    grid-template-columns: 1fr;
  }

  .action-buttons {
    flex-wrap: wrap;
  }
}
</style>
