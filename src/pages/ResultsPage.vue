<template>
  <div class="results-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <span class="title">扫描结果</span>
          <div class="actions">
            <el-input
              v-model="searchText"
              placeholder="搜索..."
              style="width: 200px"
            />
            <el-select v-model="filterType" placeholder="筛选类型" style="width: 150px; margin-left: 10px">
              <el-option label="全部" value="" />
              <el-option label="手机号" value="PhoneNumber" />
              <el-option label="身份证" value="IdCard" />
              <el-option label="姓名" value="Name" />
              <el-option label="地址" value="Address" />
            </el-select>
            <el-button @click="exportResults" style="margin-left: 10px">导出</el-button>
            <el-button @click="clearResults" type="danger" style="margin-left: 10px">清空</el-button>
          </div>
        </div>
      </template>

      <el-table
        :data="filteredResults"
        stripe
        style="width: 100%"
        :default-sort="{ prop: 'found_at', order: 'descending' }"
        max-height="600"
      >
        <el-table-column prop="file_path" label="文件路径" width="250" show-overflow-tooltip />
        <el-table-column prop="sheet_name" label="Sheet页" width="100" />
        <el-table-column prop="row" label="行号" width="80" />
        <el-table-column prop="column" label="列号" width="80" />
        <el-table-column prop="sensitive_type" label="类型" width="100">
          <template #default="{ row }">
            <el-tag :type="getTypeColor(row.sensitive_type)">
              {{ getTypeName(row.sensitive_type) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="masked_content" label="内容（脱敏）" width="150" show-overflow-tooltip />
        <el-table-column prop="found_at" label="发现时间" width="180" sortable />
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row }">
            <el-button link type="primary" @click="showDetail(row)">查看</el-button>
            <el-button link type="danger" @click="deleteResult(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- Pagination -->
      <div class="pagination-container">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[10, 20, 50, 100]"
          :total="results.length"
          layout="total, sizes, prev, pager, next, jumper"
        />
      </div>
    </el-card>

    <!-- Detail Dialog -->
    <el-dialog v-model="showDetailDialog" title="结果详情" width="600px">
      <div v-if="selectedResult" class="detail-content">
        <el-descriptions :column="1" border>
          <el-descriptions-item label="文件路径">
            {{ selectedResult.file_path }}
          </el-descriptions-item>
          <el-descriptions-item label="Sheet页">
            {{ selectedResult.sheet_name || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="位置">
            行 {{ selectedResult.row }} 列 {{ selectedResult.column }}
          </el-descriptions-item>
          <el-descriptions-item label="敏感类型">
            {{ getTypeName(selectedResult.sensitive_type) }}
          </el-descriptions-item>
          <el-descriptions-item label="原始内容">
            <el-input
              v-model="selectedResult.content"
              type="textarea"
              readonly
              rows="3"
            />
          </el-descriptions-item>
          <el-descriptions-item label="发现时间">
            {{ selectedResult.found_at }}
          </el-descriptions-item>
        </el-descriptions>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useScanStore } from '../stores/scanStore'
import { ElMessage, ElMessageBox } from 'element-plus'

const scanStore = useScanStore()

const searchText = ref('')
const filterType = ref('')
const currentPage = ref(1)
const pageSize = ref(20)
const showDetailDialog = ref(false)
const selectedResult = ref(null)

const results = computed(() => scanStore.results)

const filteredResults = computed(() => {
  let filtered = results.value

  if (searchText.value) {
    const search = searchText.value.toLowerCase()
    filtered = filtered.filter(r =>
      r.file_path.toLowerCase().includes(search) ||
      r.content.toLowerCase().includes(search) ||
      r.masked_content.toLowerCase().includes(search)
    )
  }

  if (filterType.value) {
    filtered = filtered.filter(r => r.sensitive_type === filterType.value)
  }

  // Pagination
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filtered.slice(start, end)
})

const getTypeName = (type: string) => {
  const typeMap: Record<string, string> = {
    'PhoneNumber': '手机号',
    'IdCard': '身份证',
    'Name': '姓名',
    'Address': '地址'
  }
  return typeMap[type] || type
}

const getTypeColor = (type: string) => {
  const colorMap: Record<string, string> = {
    'PhoneNumber': 'success',
    'IdCard': 'warning',
    'Name': 'info',
    'Address': 'danger'
  }
  return colorMap[type] || 'info'
}

const showDetail = (row: any) => {
  selectedResult.value = row
  showDetailDialog.value = true
}

const deleteResult = (row: any) => {
  ElMessageBox.confirm('确定删除该结果？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(() => {
    const index = scanStore.results.indexOf(row)
    if (index > -1) {
      scanStore.results.splice(index, 1)
    }
    ElMessage.success('已删除')
  }).catch(() => {
    ElMessage.info('已取消删除')
  })
}

const exportResults = async () => {
  if (results.value.length === 0) {
    ElMessage.warning('没有结果可导出')
    return
  }

  try {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const path = await save({
      filters: [{
        name: 'CSV',
        extensions: ['csv']
      }],
      defaultPath: 'scan_results.csv'
    })

    if (path) {
      const result = await invoke<string>('export_results', {
        format: 'csv',
        filePath: path
      })
      const data = JSON.parse(result)
      if (data.status === 'exported') {
        ElMessage.success(`导出成功: ${data.file_path}`)
      }
    }
  } catch (error) {
    ElMessage.error('导出失败: ' + error)
    console.error(error)
  }
}

const clearResults = () => {
  ElMessageBox.confirm('确定要清空所有结果？此操作不可撤销', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(() => {
    scanStore.clearResults()
    ElMessage.success('已清空')
  }).catch(() => {
    ElMessage.info('已取消')
  })
}
</script>

<style scoped lang="css">
.results-page {
  background-color: #fff;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.title {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.actions {
  display: flex;
  gap: 10px;
  align-items: center;
}

.pagination-container {
  display: flex;
  justify-content: flex-end;
  margin-top: 20px;
}

.detail-content {
  padding: 20px 0;
}
</style>
