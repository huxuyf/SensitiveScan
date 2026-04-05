<template>
  <div class="scan-page">
    <!-- 扫描配置板块 -->
    <el-card class="config-card" shadow="never">
      <template #header>
        <div class="card-header">
          <span class="title">扫描路径配置</span>
          <p class="subtitle">选择要扫描的文件夹，系统将自动识别其中的敏感信息</p>
        </div>
      </template>
      
      <div class="path-input-group">
        <el-input
          v-model="newPath"
          placeholder="手动输入路径或点击右侧选择"
          @keyup.enter="addPath"
          clearable
        >
          <template #append>
            <el-button @click="selectFolder">
              <el-icon><FolderOpened /></el-icon>
              选择文件夹
            </el-button>
          </template>
        </el-input>
        <el-button type="primary" @click="addPath" :disabled="!newPath">添加</el-button>
      </div>

      <div class="path-list" v-if="scanForm.scan_paths.length > 0">
        <el-tag
          v-for="(path, index) in scanForm.scan_paths"
          :key="index"
          closable
          @close="removePath(index)"
          class="path-tag"
          type="primary"
          effect="light"
        >
          {{ path }}
        </el-tag>
      </div>

      <div class="exclude-section">
        <el-button link @click="showExclude = !showExclude">
          {{ showExclude ? '隐藏排除路径' : '设置排除路径' }}
        </el-button>
        
        <div v-if="showExclude" class="exclude-content">
          <div class="path-input-group">
            <el-input
              v-model="newExcludePath"
              placeholder="输入要排除的路径"
              @keyup.enter="addExcludePath"
              clearable
            />
            <el-button @click="addExcludePath" :disabled="!newExcludePath">添加</el-button>
            <el-button @click="selectExcludeFolder">选择文件夹</el-button>
          </div>
          <div class="path-list" v-if="scanForm.exclude_paths.length > 0">
            <el-tag
              v-for="(path, index) in scanForm.exclude_paths"
              :key="index"
              closable
              @close="removeExcludePath(index)"
              class="path-tag"
              type="info"
            >
              {{ path }}
            </el-tag>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 中心主操作面板 -->
    <div class="action-container">
      <el-button
        type="primary"
        class="start-btn"
        @click="startScan"
        :disabled="!canStartScan || scanStore.isScanning"
        :loading="scanStore.isScanning"
      >
        <el-icon v-if="!scanStore.isScanning"><Search /></el-icon>
        <span>{{ scanStore.isScanning ? '正在扫描敏感信息...' : '开始扫描' }}</span>
      </el-button>

      <div v-if="scanStore.isScanning" class="scan-controls">
        <el-button @click="pauseScan" :icon="VideoPause" circle v-if="!scanStore.isPaused" />
        <el-button @click="resumeScan" :icon="VideoPlay" circle v-else />
        <el-button type="danger" @click="stopScan" :icon="CircleClose" circle />
      </div>
    </div>

    <!-- 扫描实时进度浮层 -->
    <transition name="el-fade-in">
      <el-card v-if="scanStore.isScanning" class="progress-card" shadow="never">
        <div class="progress-info">
          <div class="current-file">正在检查: {{ scanStore.currentFile || '准备中...' }}</div>
          <div class="progress-stats">
            <!-- 统计信息已移除 -->
          </div>
        </div>
        <el-progress
          :percentage="Math.round(scanStore.progressPercentage)"
          :stroke-width="12"
          striped
          striped-flow
        />
      </el-card>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useScanStore } from '../stores/scanStore'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import {
  FolderOpened,
  Search,
  VideoPause,
  VideoPlay,
  CircleClose
} from '@element-plus/icons-vue'

const scanStore = useScanStore()
const router = useRouter()
const showExclude = ref(false)

const scanForm = ref({
  scan_paths: [] as string[],
  exclude_paths: [] as string[],
  max_file_size: 100 * 1024 * 1024,
  sensitive_types: ['phonenumber', 'idcard', 'name', 'address']
})

const newPath = ref('')
const newExcludePath = ref('')

const canStartScan = computed(() => scanForm.value.scan_paths.length > 0)

const addScanPath = (newP: string) => {
  const normalize = (p: string) => p.replace(/\\/g, '/').replace(/\/$/, '').toLowerCase();
  const newNorm = normalize(newP);
  
  // 校验当前要添加的新目录是否已经被包含在一个更大的已存盘符或路径之下
  const isChild = scanForm.value.scan_paths.some(p => {
    const pNorm = normalize(p);
    return newNorm === pNorm || newNorm.startsWith(pNorm + '/');
  });

  if (isChild) {
    ElMessage.warning(`您添加的路径已包含在现有路径中，无需重复添加`);
    return;
  }

  // 反向检查要添加的新目录是否是现存某些子路径的父级，以便用更大的路径取代细碎的子路径
  const children = scanForm.value.scan_paths.filter(p => {
    const pNorm = normalize(p);
    return pNorm.startsWith(newNorm + '/');
  });

  if (children.length > 0) {
    scanForm.value.scan_paths = scanForm.value.scan_paths.filter(p => !children.includes(p));
    scanForm.value.scan_paths.push(newP);
    ElMessage.warning(`您添加的路径包含了已存在的子路径，已自动合并`);
  } else {
    scanForm.value.scan_paths.push(newP);
  }
}

const addPath = () => {
  if (newPath.value.trim()) {
    addScanPath(newPath.value.trim())
    newPath.value = ''
  }
}

const removePath = (index: number) => {
  scanForm.value.scan_paths.splice(index, 1)
}

const addExcludePath = () => {
  if (newExcludePath.value.trim()) {
    if (!scanForm.value.exclude_paths.includes(newExcludePath.value.trim())) {
      scanForm.value.exclude_paths.push(newExcludePath.value.trim())
    }
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
      addScanPath(selected)
    }
  } catch (error) {
    console.error('调用原生文件夹选择框失败:', error)
  }
}

const selectExcludeFolder = async () => {
  try {
    const selected = await invoke<string>('select_folder')
    if (selected) {
      scanForm.value.exclude_paths.push(selected)
    }
  } catch (error) {
    console.error('排除路径对话框调起失败:', error)
  }
}

const startScan = async () => {
  if (!canStartScan.value) return
  
  try {
    scanStore.startScan(scanForm.value)
    const result = await invoke<string>('start_scan', {
      scanPaths: scanForm.value.scan_paths,
      excludePaths: scanForm.value.exclude_paths,
      maxFileSize: scanForm.value.max_file_size,
      sensitiveTypes: scanForm.value.sensitive_types
    })
    const data = JSON.parse(result)
    if (data.status === 'started') {
      ElMessage.success('扫描任务已启动')
    } else {
      ElMessage.error('启动扫描失败')
      scanStore.stopScan()
    }
  } catch (error) {
    ElMessage.error('启动扫描失败: ' + error)
    scanStore.stopScan()
  }
}

const pauseScan = () => invoke('pause_scan').then(() => scanStore.pauseScan())
const resumeScan = () => invoke('resume_scan').then(() => scanStore.resumeScan())
const stopScan = () => invoke('stop_scan').then(() => scanStore.stopScan())

// 维护与后端的长连接事件通道
let unlistenProgress: (() => void) | null = null
let unlistenComplete: (() => void) | null = null

onMounted(async () => {
  try {
    unlistenProgress = await listen<any>('scan-progress', (event) => {
      scanStore.updateProgress(event.payload)
    })
    unlistenComplete = await listen<any>('scan-complete', () => {
      scanStore.stopScan()
      ElMessage.success('扫描已完成')
      router.push('/results')
    })
  } catch (error) {
    console.error('底座事件监听器安装失败:', error)
  }
})

onUnmounted(() => {
  unlistenProgress?.()
  unlistenComplete?.()
})
</script>

<style scoped lang="css">
.scan-page {
  display: flex;
  flex-direction: column;
  gap: 30px;
  max-width: 800px;
  margin: 0 auto;
}

.config-card {
  border-radius: 12px;
  border: 1px solid #ebeef5;
}

.card-header .title {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.card-header .subtitle {
  margin: 8px 0 0 0;
  font-size: 13px;
  color: #909399;
}

.path-input-group {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}

.path-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 16px;
}

.exclude-section {
  border-top: 1px dashed #ebeef5;
  padding-top: 16px;
}

.exclude-content {
  margin-top: 12px;
}

.action-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  margin-top: 20px;
}

.start-btn {
  height: 60px;
  width: 240px;
  font-size: 18px;
  font-weight: 600;
  border-radius: 30px;
  box-shadow: 0 4px 12px rgba(74, 144, 226, 0.3);
}

.scan-controls {
  display: flex;
  gap: 12px;
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

/* 移动端与多尺寸容器的响应式规则调整 */
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
