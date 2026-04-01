<template>
  <div class="results-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <span class="title">扫描结果</span>
          <div class="header-actions">
            <el-button @click="exportResults" :icon="Download">导出</el-button>
            <el-button @click="clearResults" :icon="Delete" type="danger">清空</el-button>
          </div>
        </div>
      </template>

      <!-- Statistics -->
      <div class="statistics-bar">
        <el-statistic title="总结果数" :value="results.length" />
        <el-statistic title="手机号" :value="resultsByType.phonenumber" />
        <el-statistic title="身份证" :value="resultsByType.idcard" />
        <el-statistic title="姓名" :value="resultsByType.name" />
        <el-statistic title="地址" :value="resultsByType.address" />
      </div>

      <!-- Advanced Filter -->
      <div class="filter-section">
        <el-collapse>
          <el-collapse-item title="高级筛选" name="filters">
            <el-form :inline="true" :model="filters" label-width="80px">
              <el-form-item label="关键词">
                <el-input
                  v-model="filters.keyword"
                  placeholder="搜索内容、文件路径..."
                  clearable
                  style="width: 300px"
                  @input="applyFilters"
                />
              </el-form-item>

              <el-form-item label="敏感类型">
                <el-select
                  v-model="filters.sensitiveTypes"
                  multiple
                  placeholder="选择类型"
                  style="width: 300px"
                  @change="applyFilters"
                >
                  <el-option label="手机号" value="phonenumber" />
                  <el-option label="身份证" value="idcard" />
                  <el-option label="姓名" value="name" />
                  <el-option label="地址" value="address" />
                </el-select>
              </el-form-item>

              <el-form-item label="文件路径">
                <el-input
                  v-model="filters.filePath"
                  placeholder="输入文件路径或部分"
                  clearable
                  style="width: 300px"
                  @input="applyFilters"
                />
              </el-form-item>

              <el-form-item label="日期范围">
                <el-date-picker
                  v-model="filters.dateRange"
                  type="daterange"
                  range-separator="至"
                  start-placeholder="开始日期"
                  end-placeholder="结束日期"
                  style="width: 300px"
                  @change="applyFilters"
                />
              </el-form-item>

              <el-form-item label="排序">
                <el-select v-model="filters.sortBy" style="width: 150px" @change="applyFilters">
                  <el-option label="发现时间" value="found_at" />
                  <el-option label="文件路径" value="file_path" />
                  <el-option label="敏感类型" value="sensitive_type" />
                </el-select>
                <el-button
                  :icon="filters.sortOrder === 'asc' ? SortUp : SortDown"
                  @click="toggleSortOrder"
                  circle
                  style="margin-left: 8px"
                />
              </el-form-item>

              <el-form-item>
                <el-button @click="resetFilters">重置筛选</el-button>
              </el-form-item>
            </el-form>
          </el-collapse-item>
        </el-collapse>
      </div>

      <!-- Results Table with Virtual Scroll -->
      <div class="results-container">
        <VirtualScroll
          :items="filteredResults"
          :item-height="60"
          :container-height="600"
          :loading="loading"
        >
          <template #default="{ item, index }">
            <div class="result-item" @click="showDetail(item)">
              <div class="result-cell type-cell">
                <el-tag :type="getTypeColor(item.sensitive_type)" size="small">
                  {{ getTypeName(item.sensitive_type) }}
                </el-tag>
              </div>

              <div class="result-cell content-cell">
                <div class="masked-content">{{ item.masked_content }}</div>
                <div class="original-content" v-if="showOriginalContent">
                  {{ item.content }}
                </div>
              </div>

              <div class="result-cell file-cell">
                <el-tooltip :content="item.file_path" placement="top">
                  <div class="file-path">{{ getFileName(item.file_path) }}</div>
                </el-tooltip>
              </div>

              <div class="result-cell location-cell">
                <span v-if="item.sheet_name">{{ item.sheet_name }}</span>
                <span v-if="item.sheet_name">:</span>
                <span>{{ item.row }}行{{ item.column }}列</span>
              </div>

              <div class="result-cell time-cell">
                {{ formatTime(item.found_at) }}
              </div>

              <div class="result-cell action-cell">
                <el-button link type="primary" @click.stop="showDetail(item)">
                  详情
                </el-button>
                <el-button link type="danger" @click.stop="deleteResult(item, index)">
                  删除
                </el-button>
              </div>
            </div>
          </template>
        </VirtualScroll>

        <!-- Empty State -->
        <el-empty v-if="filteredResults.length === 0" description="暂无扫描结果" />
      </div>
    </el-card>

    <!-- Result Detail Dialog -->
    <el-dialog
      v-model="showDetailDialog"
      title="结果详情"
      width="800px"
    >
      <div v-if="selectedResult" class="detail-content">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="敏感类型">
            <el-tag :type="getTypeColor(selectedResult.sensitive_type)">
              {{ getTypeName(selectedResult.sensitive_type) }}
            </el-tag>
          </el-descriptions-item>

          <el-descriptions-item label="发现时间">
            {{ formatTime(selectedResult.found_at) }}
          </el-descriptions-item>

          <el-descriptions-item label="文件路径" :span="2">
            <div class="detail-file-path">{{ selectedResult.file_path }}</div>
          </el-descriptions-item>

          <el-descriptions-item label="工作表" v-if="selectedResult.sheet_name">
            {{ selectedResult.sheet_name }}
          </el-descriptions-item>

          <el-descriptions-item label="位置">
            第 {{ selectedResult.row }} 行，第 {{ selectedResult.column }} 列
          </el-descriptions-item>

          <el-descriptions-item label="脱敏内容" :span="2">
            {{ selectedResult.masked_content }}
          </el-descriptions-item>

          <el-descriptions-item label="原始内容" :span="2">
            <el-input
              v-model="selectedResult.content"
              type="textarea"
              :rows="2"
              readonly
            />
          </el-descriptions-item>
        </el-descriptions>

        <!-- Context Preview -->
        <div class="context-preview" v-if="contextData">
          <h4>上下文预览</h4>
          <el-table :data="contextData" border size="small" max-height="300">
            <el-table-column
              v-for="(_, colIndex) in contextData[0]"
              :key="colIndex"
              :label="`列 ${colIndex + 1}`"
              :prop="String(colIndex)"
              show-overflow-tooltip
            />
          </el-table>
        </div>
      </div>

      <template #footer>
        <el-button @click="showDetailDialog = false">关闭</el-button>
        <el-button type="primary" @click="openFileLocation">打开文件位置</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Download, Delete, SortUp, SortDown } from '@element-plus/icons-vue'
import VirtualScroll from '@/components/VirtualScroll.vue'

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

interface Filters {
  keyword: string
  sensitiveTypes: string[]
  filePath: string
  dateRange: [Date, Date] | null
  sortBy: string
  sortOrder: 'asc' | 'desc'
}

// Props
const props = defineProps<{
  results: ScanResult[]
  loading?: boolean
}>()

// State
const filteredResults = ref<ScanResult[]>([...props.results])
const showOriginalContent = ref(false)
const showDetailDialog = ref(false)
const selectedResult = ref<ScanResult | null>(null)
const contextData = ref<any[][] | null>(null)
const loading = ref(props.loading || false)

const filters = ref<Filters>({
  keyword: '',
  sensitiveTypes: [],
  filePath: '',
  dateRange: null,
  sortBy: 'found_at',
  sortOrder: 'desc'
})

// Computed
const resultsByType = computed(() => {
  const counts = {
    phonenumber: 0,
    idcard: 0,
    name: 0,
    address: 0
  }
  
  props.results.forEach(result => {
    if (counts.hasOwnProperty(result.sensitive_type)) {
      counts[result.sensitive_type as keyof typeof counts]++
    }
  })
  
  return counts
})

// Methods
const getTypeName = (type: string) => {
  const typeMap: Record<string, string> = {
    'phonenumber': '手机号',
    'idcard': '身份证',
    'name': '姓名',
    'address': '地址'
  }
  return typeMap[type] || type
}

const getTypeColor = (type: string) => {
  const colorMap: Record<string, string> = {
    'phonenumber': 'success',
    'idcard': 'warning',
    'name': 'info',
    'address': 'danger'
  }
  return colorMap[type] || 'info'
}

const getFileName = (path: string) => {
  const parts = path.split(/[/\\]/)
  return parts[parts.length - 1] || path
}

const formatTime = (timeStr: string) => {
  const date = new Date(timeStr)
  return date.toLocaleString('zh-CN')
}

const applyFilters = () => {
  let results = [...props.results]
  
  // Keyword filter
  if (filters.value.keyword) {
    const keyword = filters.value.keyword.toLowerCase()
    results = results.filter(r => 
      r.content.toLowerCase().includes(keyword) ||
      r.file_path.toLowerCase().includes(keyword)
    )
  }
  
  // Sensitive types filter
  if (filters.value.sensitiveTypes.length > 0) {
    results = results.filter(r => 
      filters.value.sensitiveTypes.includes(r.sensitive_type)
    )
  }
  
  // File path filter
  if (filters.value.filePath) {
    const path = filters.value.filePath.toLowerCase()
    results = results.filter(r => 
      r.file_path.toLowerCase().includes(path)
    )
  }
  
  // Date range filter
  if (filters.value.dateRange) {
    const [start, end] = filters.value.dateRange
    results = results.filter(r => {
      const date = new Date(r.found_at)
      return date >= start && date <= end
    })
  }
  
  // Sort
  results.sort((a, b) => {
    let comparison = 0
    
    switch (filters.value.sortBy) {
      case 'found_at':
        comparison = new Date(a.found_at).getTime() - new Date(b.found_at).getTime()
        break
      case 'file_path':
        comparison = a.file_path.localeCompare(b.file_path)
        break
      case 'sensitive_type':
        comparison = a.sensitive_type.localeCompare(b.sensitive_type)
        break
    }
    
    return filters.value.sortOrder === 'asc' ? comparison : -comparison
  })
  
  filteredResults.value = results
}

const resetFilters = () => {
  filters.value = {
    keyword: '',
    sensitiveTypes: [],
    filePath: '',
    dateRange: null,
    sortBy: 'found_at',
    sortOrder: 'desc'
  }
  applyFilters()
}

const toggleSortOrder = () => {
  filters.value.sortOrder = filters.value.sortOrder === 'asc' ? 'desc' : 'asc'
  applyFilters()
}

const showDetail = async (result: ScanResult) => {
  selectedResult.value = result
  showDetailDialog.value = true
  
  // TODO: Load context data from backend
  // contextData.value = await invoke('get_result_context', { resultId: result.id })
  
  // Mock context data for now
  contextData.value = [
    ['姓名', '手机号', '身份证', '地址'],
    ['张三', result.content, '110101199001011234', '北京市朝阳区'],
    ['李四', '13900000000', '110101199001011235', '北京市海淀区']
  ]
}

const deleteResult = (result: ScanResult, index: number) => {
  ElMessageBox.confirm('确定删除该结果？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(() => {
    // TODO: Call backend to delete
    // await invoke('delete_scan_result', { id: result.id })
    
    const idx = filteredResults.value.indexOf(result)
    if (idx > -1) {
      filteredResults.value.splice(idx, 1)
    }
    
    ElMessage.success('已删除')
  }).catch(() => {
    ElMessage.info('已取消')
  })
}

const exportResults = () => {
  // TODO: Export to Excel
  ElMessage.success('导出功能开发中')
}

const clearResults = () => {
  ElMessageBox.confirm('确定清空所有结果？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(() => {
    // TODO: Call backend to clear
    // await invoke('clear_scan_results')
    
    filteredResults.value = []
    ElMessage.success('已清空')
  }).catch(() => {
    ElMessage.info('已取消')
  })
}

const openFileLocation = () => {
  if (selectedResult.value) {
    // TODO: Open file location in OS file manager
    // await invoke('open_file_location', { path: selectedResult.value.file_path })
    ElMessage.info('打开文件位置功能开发中')
  }
}

// Initialize
onMounted(() => {
  applyFilters()
})

// Watch for results changes
watch(() => props.results, () => {
  applyFilters()
}, { deep: true })
</script>

<style scoped lang="css">
.results-page {
  background-color: #f5f7fa;
  min-height: 100vh;
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.title {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.statistics-bar {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 20px;
  margin-bottom: 20px;
  padding: 20px;
  background-color: #f9fafc;
  border-radius: 8px;
}

.filter-section {
  margin-bottom: 20px;
}

.results-container {
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  overflow: hidden;
}

.result-item {
  display: grid;
  grid-template-columns: 100px 200px 1fr 150px 180px 120px;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid #e4e7ed;
  cursor: pointer;
  transition: background-color 0.2s;
  align-items: center;
}

.result-item:hover {
  background-color: #f5f7fa;
}

.result-cell {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.type-cell {
  display: flex;
  align-items: center;
}

.content-cell {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.masked-content {
  font-weight: 500;
  color: #303133;
}

.original-content {
  font-size: 12px;
  color: #909399;
}

.file-cell {
  color: #606266;
}

.file-path {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.location-cell {
  color: #909399;
  font-size: 13px;
}

.time-cell {
  color: #909399;
  font-size: 13px;
}

.action-cell {
  display: flex;
  gap: 8px;
}

.detail-content {
  padding: 10px 0;
}

.detail-file-path {
  word-break: break-all;
  line-height: 1.5;
}

.context-preview {
  margin-top: 24px;
}

.context-preview h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: #303133;
}
</style>
