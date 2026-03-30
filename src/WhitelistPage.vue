<template>
  <div class="whitelist-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <span class="title">白名单管理</span>
          <el-button type="primary" @click="showAddDialog">添加白名单</el-button>
        </div>
      </template>

      <el-table
        :data="whitelist"
        stripe
        style="width: 100%"
        :default-sort="{ prop: 'created_at', order: 'descending' }"
      >
        <el-table-column prop="content" label="内容" width="200" show-overflow-tooltip />
        <el-table-column prop="sensitive_type" label="类型" width="100">
          <template #default="{ row }">
            <el-tag :type="getTypeColor(row.sensitive_type)">
              {{ getTypeName(row.sensitive_type) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="description" label="描述" width="250" show-overflow-tooltip />
        <el-table-column prop="created_at" label="添加时间" width="180" sortable />
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row }">
            <el-button link type="danger" @click="deleteEntry(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="empty-state" v-if="whitelist.length === 0">
        <p>暂无白名单项</p>
      </div>
    </el-card>

    <!-- Add Dialog -->
    <el-dialog v-model="showAddDialogFlag" title="添加白名单" width="500px">
      <el-form :model="formData" label-width="80px">
        <el-form-item label="内容" required>
          <el-input v-model="formData.content" placeholder="输入敏感信息内容" />
        </el-form-item>
        <el-form-item label="类型" required>
          <el-select v-model="formData.sensitive_type">
            <el-option label="手机号" value="phonenumber" />
            <el-option label="身份证" value="idcard" />
            <el-option label="姓名" value="name" />
            <el-option label="地址" value="address" />
          </el-select>
        </el-form-item>
        <el-form-item label="描述">
          <el-input
            v-model="formData.description"
            type="textarea"
            rows="3"
            placeholder="输入描述信息（可选）"
          />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="showAddDialogFlag = false">取消</el-button>
        <el-button type="primary" @click="addEntry">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'

const whitelist = ref<any[]>([])
const showAddDialogFlag = ref(false)
const formData = ref({
  content: '',
  sensitive_type: 'phonenumber',
  description: ''
})

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

const showAddDialog = () => {
  formData.value = {
    content: '',
    sensitive_type: 'phonenumber',
    description: ''
  }
  showAddDialogFlag.value = true
}

const addEntry = async () => {
  if (!formData.value.content.trim()) {
    ElMessage.warning('请输入内容')
    return
  }

  try {
    // TODO: Call Tauri command to add whitelist
    // const result = await invoke('add_whitelist', {
    //   content: formData.value.content,
    //   sensitive_type: formData.value.sensitive_type,
    //   description: formData.value.description
    // })

    const newEntry = {
      id: Math.random().toString(36).substr(2, 9),
      content: formData.value.content,
      sensitive_type: formData.value.sensitive_type,
      description: formData.value.description,
      created_at: new Date().toISOString()
    }

    whitelist.value.unshift(newEntry)
    showAddDialogFlag.value = false
    ElMessage.success('已添加')
  } catch (error) {
    ElMessage.error('添加失败')
  }
}

const deleteEntry = (row: any) => {
  ElMessageBox.confirm('确定删除该项？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(() => {
    const index = whitelist.value.indexOf(row)
    if (index > -1) {
      whitelist.value.splice(index, 1)
    }
    ElMessage.success('已删除')
  }).catch(() => {
    ElMessage.info('已取消')
  })
}

// Load whitelist from backend
const loadWhitelist = async () => {
  try {
    // TODO: Call Tauri command to get whitelist
    // const result = await invoke('get_whitelist')
    // whitelist.value = result
  } catch (error) {
    ElMessage.error('加载白名单失败')
  }
}

// Load on component mount
loadWhitelist()
</script>

<style scoped lang="css">
.whitelist-page {
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

.empty-state {
  text-align: center;
  padding: 40px;
  color: #909399;
}
</style>
