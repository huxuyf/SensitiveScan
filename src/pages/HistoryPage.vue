<template>
  <div class="history-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <span class="title">扫描历史</span>
          <el-button @click="clearHistory" type="danger">清空历史</el-button>
        </div>
      </template>

      <el-table
        :data="history"
        stripe
        style="width: 100%"
        :default-sort="{ prop: 'created_at', order: 'descending' }"
      >
        <el-table-column prop="id" label="ID" width="100" show-overflow-tooltip />
        <el-table-column label="扫描路径" width="250">
          <template #default="{ row }">
            <div class="paths">
              <el-tag v-for="(path, index) in row.scan_paths" :key="index">
                {{ path }}
              </el-tag>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="统计信息" width="200">
          <template #default="{ row }">
            <div class="stats">
              <p>文件数: {{ row.stats.total_files_scanned }}</p>
              <p>结果数: {{ row.stats.total_results_found }}</p>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="扫描时间" width="180" sortable />
        <el-table-column label="操作" width="150" fixed="right">
          <template #default="{ row }">
            <el-button link type="primary" @click="viewDetails(row)">查看</el-button>
            <el-button link type="danger" @click="deleteHistory(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="empty-state" v-if="history.length === 0">
        <p>暂无扫描历史</p>
      </div>
    </el-card>

    <!-- Details Dialog -->
    <el-dialog v-model="showDetailsDialog" title="历史详情" width="700px">
      <div v-if="selectedHistory" class="details-content">
        <el-descriptions :column="1" border>
          <el-descriptions-item label="扫描ID">
            {{ selectedHistory.id }}
          </el-descriptions-item>
          <el-descriptions-item label="扫描路径">
            <div class="paths">
              <el-tag v-for="(path, index) in selectedHistory.scan_paths" :key="index">
                {{ path }}
              </el-tag>
            </div>
          </el-descriptions-item>
          <el-descriptions-item label="扫描时间">
            {{ selectedHistory.created_at }}
          </el-descriptions-item>
          <el-descriptions-item label="完成时间">
            {{ selectedHistory.completed_at || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="扫描文件数">
            {{ selectedHistory.stats.total_files_scanned }}
          </el-descriptions-item>
          <el-descriptions-item label="发现结果数">
            {{ selectedHistory.stats.total_results_found }}
          </el-descriptions-item>
          <el-descriptions-item label="扫描耗时">
            {{ selectedHistory.stats.scan_duration_secs }} 秒
          </el-descriptions-item>
          <el-descriptions-item label="扫描速度">
            {{ selectedHistory.stats.scan_speed.toFixed(2) }} 文件/秒
          </el-descriptions-item>
        </el-descriptions>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'

const history = ref<any[]>([])
const showDetailsDialog = ref(false)
const selectedHistory = ref(null)

const viewDetails = (row: any) => {
  selectedHistory.value = row
  showDetailsDialog.value = true
}

const deleteHistory = (row: any) => {
  ElMessageBox.confirm('确定删除该历史记录？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(() => {
    const index = history.value.indexOf(row)
    if (index > -1) {
      history.value.splice(index, 1)
    }
    ElMessage.success('已删除')
  }).catch(() => {
    ElMessage.info('已取消')
  })
}

const clearHistory = () => {
  ElMessageBox.confirm('确定要清空所有历史记录？此操作不可撤销', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(() => {
    history.value = []
    ElMessage.success('已清空')
  }).catch(() => {
    ElMessage.info('已取消')
  })
}

// Load history from backend
const loadHistory = async () => {
  try {
    // TODO: Call Tauri command to get history
    // const result = await invoke('get_history', { limit: 100 })
    // history.value = result
  } catch (error) {
    ElMessage.error('加载历史记录失败')
  }
}

// Load on component mount
loadHistory()
</script>

<style scoped lang="css">
.history-page {
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

.paths {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.stats {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.stats p {
  margin: 0;
  font-size: 12px;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: #909399;
}

.details-content {
  padding: 20px 0;
}
</style>
