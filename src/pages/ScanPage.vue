<template>
  <div class="scan-page">
    <el-card class="scan-card">
      <template #header>
        <div class="card-header">
          <span class="title">扫描配置</span>
        </div>
      </template>

      <el-form :model="scanForm" label-width="120px" class="scan-form">
        <!-- Scan Paths -->
        <el-form-item label="扫描路径">
          <div class="path-input-group">
            <el-input
              v-model="newPath"
              placeholder="输入扫描路径"
              @keyup.enter="addPath"
            />
            <el-button @click="addPath" type="primary">添加</el-button>
            <el-button @click="selectFolder">选择文件夹</el-button>
          </div>
          <div class="path-list" v-if="scanForm.scan_paths.length > 0">
            <el-tag
              v-for="(path, index) in scanForm.scan_paths"
              :key="index"
              closable
              @close="removePath(index)"
              class="path-tag"
            >
              {{ path }}
            </el-tag>
          </div>
        </el-form-item>

        <!-- Exclude Paths -->
        <el-form-item label="排除路径">
          <div class="path-input-group">
            <el-input
              v-model="newExcludePath"
              placeholder="输入要排除的路径"
              @keyup.enter="addExcludePath"
            />
            <el-button @click="addExcludePath">添加</el-button>
            <el-button @click="selectExcludeFolder">选择文件夹</el-button>
          </div>
          <div class="path-list" v-if="scanForm.exclude_paths.length > 0">
            <el-tag
              v-for="(path, index) in scanForm.exclude_paths"
              :key="index"
              closable
              @close="removeExcludePath(index)"
              class="path-tag"
            >
              {{ path }}
            </el-tag>
          </div>
        </el-form-item>

        <!-- Sensitive Types -->
        <el-form-item label="敏感信息类型">
          <el-checkbox-group v-model="scanForm.sensitive_types">
            <el-checkbox label="phonenumber">手机号</el-checkbox>
            <el-checkbox label="idcard">身份证</el-checkbox>
            <el-checkbox label="name">姓名</el-checkbox>
            <el-checkbox label="address">地址</el-checkbox>
          </el-checkbox-group>
        </el-form-item>

        <!-- Action Buttons -->
        <el-form-item>
          <el-button
            type="primary"
            @click="startScan"
            :disabled="!canStartScan || scanStore.isScanning"
            size="large"
          >
            {{ scanStore.isScanning ? '扫描中...' : '开始扫描' }}
          </el-button>
          <el-button
            v-if="scanStore.isScanning"
            @click="pauseScan"
            :disabled="scanStore.isPaused"
          >
            暂停
          </el-button>
          <el-button
            v-if="scanStore.isScanning"
            @click="resumeScan"
            :disabled="!scanStore.isPaused"
          >
            继续
          </el-button>
          <el-button
            v-if="scanStore.isScanning"
            @click="stopScan"
            type="danger"
          >
            停止
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- Progress Panel -->
    <el-card v-if="scanStore.isScanning" class="progress-card">
      <template #header>
        <div class="card-header">
          <span class="title">扫描进度</span>
        </div>
      </template>

      <div class="progress-content">
        <div class="progress-item">
          <span class="label">当前文件:</span>
          <span class="value">{{ scanStore.currentFile }}</span>
        </div>
        <div class="progress-item">
          <span class="label">已扫描文件:</span>
          <span class="value">{{ scanStore.filesScanned }}</span>
        </div>
        <div class="progress-item">
          <span class="label">发现结果:</span>
          <span class="value">{{ scanStore.resultsFound }}</span>
        </div>
        <div class="progress-item">
          <span class="label">扫描速度:</span>
          <span class="value">{{ scanStore.scanSpeed }} 文件/秒</span>
        </div>
        <div class="progress-item">
          <span class="label">已用时间:</span>
          <span class="value">{{ formatTime(scanStore.elapsedSeconds) }}</span>
        </div>
        <div class="progress-item">
          <span class="label">预计剩余:</span>
          <span class="value">{{ formatTime(scanStore.estimatedRemaining) }}</span>
        </div>

        <!-- Progress Bar -->
        <div class="progress-bar-container">
          <el-progress
            :percentage="Math.round(scanStore.progressPercentage)"
            :format="(percentage) => `${percentage}%`"
          />
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useScanStore } from '../stores/scanStore'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/tauri'

const scanStore = useScanStore()

const scanForm = ref({
  scan_paths: [],
  exclude_paths: [],
  max_file_size: 100 * 1024 * 1024,
  sensitive_types: ['phonenumber', 'idcard', 'name', 'address']
})

const newPath = ref('')
const newExcludePath = ref('')

const canStartScan = computed(() => scanForm.value.scan_paths.length > 0)

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
    const selected = await invoke<string>('select_folder')
    if (selected) {
      scanForm.value.scan_paths.push(selected)
      ElMessage.success(`已添加路径: ${selected}`)
    }
  } catch (error) {
    ElMessage.error('选择文件夹失败')
    console.error(error)
  }
}

const selectExcludeFolder = async () => {
  try {
    const selected = await invoke<string>('select_folder')
    if (selected) {
      scanForm.value.exclude_paths.push(selected)
      ElMessage.success(`已添加排除路径: ${selected}`)
    }
  } catch (error) {
    ElMessage.error('选择文件夹失败')
    console.error(error)
  }
}

const startScan = async () => {
  if (!canStartScan.value) {
    ElMessage.warning('请至少选择一个扫描路径')
    return
  }

  try {
    scanStore.startScan(scanForm.value)
    const result = await invoke<string>('start_scan', {
      scanPaths: scanForm.value.scan_paths,
      excludePaths: scanForm.value.exclude_paths,
      maxFileSize: scanForm.value.max_file_size,
      sensitiveTypes: scanForm.value.sensitive_types
    })

    // Parse the result to get task ID
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
</script>

<style scoped lang="css">
.scan-page {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.scan-card,
.progress-card {
  background-color: #fff;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.title {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.scan-form {
  padding: 20px 0;
}

.path-input-group {
  display: flex;
  gap: 10px;
  margin-bottom: 10px;
}

.path-input-group :deep(.el-input) {
  flex: 1;
}

.path-list {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 10px;
}

.path-tag {
  max-width: 300px;
}

.size-hint {
  margin-left: 10px;
  color: #909399;
}

.progress-content {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 15px;
  margin-bottom: 20px;
}

.progress-item {
  display: flex;
  justify-content: space-between;
  padding: 10px;
  background-color: #f5f7fa;
  border-radius: 4px;
}

.label {
  font-weight: 600;
  color: #606266;
}

.value {
  color: #303133;
}

.progress-bar-container {
  grid-column: 1 / -1;
}
</style>
