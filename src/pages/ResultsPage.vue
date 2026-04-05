<template>
  <div class="results-page">
    <el-card shadow="never" class="results-card">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <span class="title">扫描结果汇总</span>
            <el-tag type="info" class="count-tag">共发现 {{ scanStore.aggregatedResults.length }} 个涉敏文件</el-tag>
          </div>
          <div class="header-actions">
            <el-button type="primary" @click="exportToExcel" :disabled="!scanStore.aggregatedResults.length">
              <el-icon><Download /></el-icon>导出 Excel
            </el-button>
            <el-button
              type="danger"
              :plain="selectedRows.length === 0"
              @click="handleBatchDelete"
              :disabled="selectedRows.length === 0"
            >
              <el-icon><Delete /></el-icon>
              {{ selectedRows.length > 0 ? `删除选中 (${selectedRows.length})` : '删除' }}
            </el-button>
            <el-button type="info" plain @click="clearResults">
              <el-icon><Refresh /></el-icon>清空结果
            </el-button>
          </div>
        </div>
      </template>

      <el-table
        :data="pagedResults"
        stripe
        style="width: 100%"
        @selection-change="handleSelectionChange"
        :header-cell-style="{ background: '#f5f7fa', color: '#606266', fontWeight: 'bold' }"
      >
        <el-table-column type="selection" width="55" />
        
        <el-table-column prop="file_name" label="文件名" min-width="150" show-overflow-tooltip />
        
        <el-table-column prop="sensitive_types" label="涉敏类型" min-width="180">
          <template #default="{ row }">
            <el-tag
              v-for="type in row.sensitive_types.split('+')"
              :key="type"
              size="small"
              effect="plain"
              class="type-tag"
            >
              {{ translateType(type) }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="file_size" label="文件大小" width="100">
          <template #default="{ row }">
            {{ formatFileSize(row.file_size) }}
          </template>
        </el-table-column>

        <el-table-column prop="file_path" label="文件路径" min-width="250" show-overflow-tooltip>
          <template #default="{ row }">
            {{ formatFilePath(row.file_path) }}
          </template>
        </el-table-column>
        
        <el-table-column label="操作" width="150" fixed="right">
          <template #default="{ row }">
            <el-button link type="primary" @click="openFile(row.file_path)">打开</el-button>
            <el-button link type="danger" @click="confirmDelete(row.file_path)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="pagination-container" v-if="scanStore.aggregatedResults.length > 0">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next, jumper"
          :total="scanStore.aggregatedResults.length"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>

      <div v-if="!scanStore.aggregatedResults.length && !scanStore.isScanning" class="empty-state">
        <el-empty description="暂无符合条件的涉敏文件" />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useScanStore } from '../stores/scanStore'
import { API } from '../services/api'
import { ElMessage, ElMessageBox } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import {
  Download,
  Delete,
  Refresh
} from '@element-plus/icons-vue'

const scanStore = useScanStore()

// 分页状态管理
const currentPage = ref(1)
const pageSize = ref(10)

// 当前选中的行记录
const selectedRows = ref<any[]>([])

const pagedResults = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return scanStore.aggregatedResults.slice(start, end)
})

onMounted(async () => {
  await refreshResults()
})

const refreshResults = async () => {
  try {
    // 依然获取汇总结果，但在后端已经受熔断逻辑保护
    const results = await API.getAggregatedResults(50)
    scanStore.setAggregatedResults(results)
  } catch (error) {
    console.error('加载记录列表失败:', error)
  }
}

const handleSelectionChange = (val: any[]) => {
  selectedRows.value = val
}

const handleSizeChange = (val: number) => {
  pageSize.value = val
  currentPage.value = 1
}

const handleCurrentChange = (val: number) => {
  currentPage.value = val
}

const translateType = (type: string) => {
  const map: Record<string, string> = {
    'PhoneNumber': '手机号码',
    'IdCard': '身份证号',
    'Name': '姓名',
    'Address': '地址'
  }
  return map[type] || type
}

const formatFileSize = (bytes: number) => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const openFile = async (path: string) => {
  try {
    await API.openFile(path)
  } catch (error) {
    ElMessage.error('无法打开文件: ' + error)
  }
}

const confirmDelete = (path: string) => {
  ElMessageBox.confirm(
    `确定要从磁盘上永久删除该文件吗？\n${path}`,
    '物理删除确认',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    }
  ).then(async () => {
    try {
      await API.deleteFile(path)
      ElMessage.success('文件已物理删除')
      await refreshResults()
    } catch (error) {
      ElMessage.error('删除失败: ' + error)
    }
  })
}

const handleBatchDelete = () => {
  const count = selectedRows.value.length
  ElMessageBox.confirm(
    `确定要从磁盘上永久删除选中的 ${count} 个文件吗？此操作不可撤销！`,
    '批量物理删除确认',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'error'
    }
  ).then(async () => {
    try {
      for (const row of selectedRows.value) {
        await API.deleteFile(row.file_path)
      }
      ElMessage.success(`成功删除 ${count} 个文件`)
      selectedRows.value = []
      await refreshResults()
    } catch (error) {
      ElMessage.error('部分文件删除失败: ' + error)
      await refreshResults()
    }
  })
}

const clearResults = () => {
  ElMessageBox.confirm('确定要清空扫描结果吗？', '提示', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(async () => {
    try {
      await invoke('clear_results')
      scanStore.clearResults()
      ElMessage.success('已清空显示并清理数据库')
    } catch (error) {
      ElMessage.error('清理失败: ' + error)
    }
  })
}

const formatFilePath = (path: string) => {
  const lastSlash = Math.max(path.lastIndexOf('\\'), path.lastIndexOf('/'));
  return lastSlash >= 0 ? path.substring(0, lastSlash + 1) : path;
}

const exportToExcel = async () => {
  try {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const path = await save({
      title: '导出扫描结果',
      filters: [{ name: 'Excel', extensions: ['xlsx'] }],
      defaultPath: '涉敏文件汇总.xlsx'
    })
    
    if (path) {
      await API.exportResults('xlsx', path)
      ElMessage.success('导出成功')
    }
  } catch (error) {
    ElMessage.error('导出失败: ' + error)
  }
}
</script>

<style scoped lang="css">
.results-page {
  width: 100%;
}

.results-card {
  border-radius: 12px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.title {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.count-tag {
  font-weight: normal;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.type-tag {
  margin-right: 4px;
  margin-bottom: 4px;
  border-radius: 4px;
}

.pagination-container {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}

.empty-state {
  padding: 60px 0;
}

:deep(.el-table) {
  border-radius: 8px;
  overflow: hidden;
}

:deep(.el-table__row) {
  height: 60px;
}

:deep(.el-pagination) {
  font-weight: normal;
}
</style>
