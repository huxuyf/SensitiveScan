<template>
  <div class="whitelist-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <span class="title">白名单管理</span>
          <div class="header-actions">
            <el-button @click="handleImport" :icon="Upload">导入</el-button>
            <el-button @click="handleExport" :icon="Download">导出</el-button>
            <el-button type="primary" @click="showAddDialog" :icon="Plus">添加白名单</el-button>
          </div>
        </div>
      </template>

      <!-- Filter Bar -->
      <div class="filter-bar">
        <el-input
          v-model="searchKeyword"
          placeholder="搜索内容或描述"
          :prefix-icon="Search"
          clearable
          style="width: 300px"
        />
        <el-select
          v-model="filterType"
          placeholder="选择类型"
          clearable
          style="width: 150px"
        >
          <el-option label="手机号" value="phonenumber" />
          <el-option label="身份证" value="idcard" />
          <el-option label="姓名" value="name" />
          <el-option label="地址" value="address" />
        </el-select>
        <el-select
          v-model="filterMode"
          placeholder="匹配方式"
          clearable
          style="width: 150px"
        >
          <el-option label="精确匹配" value="exact" />
          <el-option label="正则表达式" value="regex" />
        </el-select>
        <el-select
          v-model="filterStatus"
          placeholder="状态"
          clearable
          style="width: 150px"
        >
          <el-option label="启用" value="enabled" />
          <el-option label="禁用" value="disabled" />
        </el-select>
      </div>

      <!-- Whitelist Table -->
      <el-table
        :data="filteredWhitelist"
        stripe
        style="width: 100%"
        :default-sort="{ prop: 'created_at', order: 'descending' }"
      >
        <el-table-column prop="content" label="内容/模式" width="250" show-overflow-tooltip>
          <template #default="{ row }">
            <div class="content-cell">
              <el-tag v-if="row.is_regex" type="warning" size="small" style="margin-right: 8px">
                正则
              </el-tag>
              <code>{{ row.content }}</code>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="sensitive_type" label="类型" width="100">
          <template #default="{ row }">
            <el-tag :type="getTypeColor(row.sensitive_type)">
              {{ getTypeName(row.sensitive_type) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="description" label="描述" width="200" show-overflow-tooltip />
        <el-table-column prop="match_count" label="匹配次数" width="100" sortable>
          <template #default="{ row }">
            <el-tag type="info">{{ row.match_count || 0 }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="enabled" label="状态" width="80">
          <template #default="{ row }">
            <el-switch
              v-model="row.enabled"
              @change="toggleRule(row)"
            />
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="添加时间" width="180" sortable />
        <el-table-column label="操作" width="150" fixed="right">
          <template #default="{ row }">
            <el-button link type="primary" @click="editRule(row)">编辑</el-button>
            <el-button link type="danger" @click="deleteRule(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- Empty State -->
      <div class="empty-state" v-if="filteredWhitelist.length === 0">
        <el-empty description="暂无白名单项" />
      </div>

      <!-- Statistics -->
      <div class="statistics" v-if="whitelist.length > 0">
        <el-statistic title="总规则数" :value="whitelist.length" />
        <el-statistic title="启用规则" :value="enabledCount" />
        <el-statistic title="正则规则" :value="regexCount" />
        <el-statistic title="总匹配次数" :value="totalMatches" />
      </div>
    </el-card>

    <!-- Add/Edit Dialog -->
    <el-dialog
      v-model="showAddDialogFlag"
      :title="isEditing ? '编辑白名单' : '添加白名单'"
      width="600px"
    >
      <el-form :model="formData" label-width="100px" :rules="formRules" ref="formRef">
        <el-form-item label="匹配方式" prop="match_mode">
          <el-radio-group v-model="formData.match_mode">
            <el-radio value="exact">精确匹配</el-radio>
            <el-radio value="regex">正则表达式</el-radio>
          </el-radio-group>
        </el-form-item>

        <el-form-item label="内容/模式" prop="content" required>
          <el-input
            v-model="formData.content"
            :placeholder="formData.match_mode === 'regex' ? '输入正则表达式' : '输入敏感信息内容'"
            clearable
          />
          <div class="form-tip" v-if="formData.match_mode === 'regex'">
            支持标准正则表达式语法。例如：^1[3-9]\d{9}$ 匹配手机号
          </div>
        </el-form-item>

        <el-form-item label="类型" prop="sensitive_type" required>
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

        <el-form-item label="启用状态">
          <el-switch v-model="formData.enabled" />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="showAddDialogFlag = false">取消</el-button>
        <el-button type="primary" @click="submitForm" :loading="submitting">
          {{ isEditing ? '保存' : '添加' }}
        </el-button>
      </template>
    </el-dialog>

    <!-- Import Dialog -->
    <el-dialog
      v-model="showImportDialogFlag"
      title="导入白名单"
      width="500px"
    >
      <el-upload
        ref="uploadRef"
        class="upload-demo"
        drag
        :auto-upload="false"
        accept=".json"
        :on-change="handleFileChange"
        :limit="1"
        :on-exceed="handleExceed"
      >
        <el-icon class="el-icon--upload"><UploadFilled /></el-icon>
        <div class="el-upload__text">
          将 JSON 文件拖到此处，或<em>点击上传</em>
        </div>
        <template #tip>
          <div class="el-upload__tip">
            只能上传 JSON 格式的白名单文件
          </div>
        </template>
      </el-upload>

      <template #footer>
        <el-button @click="showImportDialogFlag = false">取消</el-button>
        <el-button type="primary" @click="confirmImport" :loading="importing">
          导入
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, Search, Upload, Download, UploadFilled } from '@element-plus/icons-vue'

interface WhitelistRule {
  id: string
  content: string
  pattern: string
  is_regex: boolean
  sensitive_type: string
  description?: string
  created_at: string
  enabled: boolean
  match_count: number
}

const whitelist = ref<WhitelistRule[]>([])
const showAddDialogFlag = ref(false)
const showImportDialogFlag = ref(false)
const isEditing = ref(false)
const submitting = ref(false)
const importing = ref(false)
const searchKeyword = ref('')
const filterType = ref('')
const filterMode = ref('')
const filterStatus = ref('')
const uploadRef = ref()
const selectedFile = ref<File | null>(null)

const formData = ref({
  id: '',
  content: '',
  match_mode: 'exact',
  sensitive_type: 'phonenumber',
  description: '',
  enabled: true
})

const formRules = {
  content: [
    { required: true, message: '请输入内容或模式', trigger: 'blur' }
  ],
  sensitive_type: [
    { required: true, message: '请选择类型', trigger: 'change' }
  ]
}

const filteredWhitelist = computed(() => {
  return whitelist.value.filter(rule => {
    // Keyword filter
    if (searchKeyword.value) {
      const keyword = searchKeyword.value.toLowerCase()
      const matchesContent = rule.content.toLowerCase().includes(keyword)
      const matchesDesc = rule.description?.toLowerCase().includes(keyword) || false
      if (!matchesContent && !matchesDesc) return false
    }

    // Type filter
    if (filterType.value && rule.sensitive_type !== filterType.value) return false

    // Mode filter
    if (filterMode.value) {
      if (filterMode.value === 'regex' && !rule.is_regex) return false
      if (filterMode.value === 'exact' && rule.is_regex) return false
    }

    // Status filter
    if (filterStatus.value) {
      if (filterStatus.value === 'enabled' && !rule.enabled) return false
      if (filterStatus.value === 'disabled' && rule.enabled) return false
    }

    return true
  })
})

const enabledCount = computed(() => whitelist.value.filter(r => r.enabled).length)
const regexCount = computed(() => whitelist.value.filter(r => r.is_regex).length)
const totalMatches = computed(() => whitelist.value.reduce((sum, r) => sum + (r.match_count || 0), 0))

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
  isEditing.value = false
  formData.value = {
    id: '',
    content: '',
    match_mode: 'exact',
    sensitive_type: 'phonenumber',
    description: '',
    enabled: true
  }
  showAddDialogFlag.value = true
}

const editRule = (row: WhitelistRule) => {
  isEditing.value = true
  formData.value = {
    id: row.id,
    content: row.content,
    match_mode: row.is_regex ? 'regex' : 'exact',
    sensitive_type: row.sensitive_type,
    description: row.description || '',
    enabled: row.enabled
  }
  showAddDialogFlag.value = true
}

const submitForm = async () => {
  if (!formData.value.content.trim()) {
    ElMessage.warning('请输入内容或模式')
    return
  }

  submitting.value = true
  try {
    // TODO: Call Tauri command
    if (isEditing.value) {
      // await invoke('update_whitelist_rule', { ... })
      const index = whitelist.value.findIndex(r => r.id === formData.value.id)
      if (index > -1) {
        whitelist.value[index] = {
          ...whitelist.value[index],
          content: formData.value.content,
          is_regex: formData.value.match_mode === 'regex',
          pattern: formData.value.content,
          sensitive_type: formData.value.sensitive_type,
          description: formData.value.description,
          enabled: formData.value.enabled
        }
      }
      ElMessage.success('已更新')
    } else {
      // await invoke('add_whitelist_rule', { ... })
      const newRule: WhitelistRule = {
        id: Math.random().toString(36).substr(2, 9),
        content: formData.value.content,
        pattern: formData.value.content,
        is_regex: formData.value.match_mode === 'regex',
        sensitive_type: formData.value.sensitive_type,
        description: formData.value.description,
        created_at: new Date().toISOString(),
        enabled: formData.value.enabled,
        match_count: 0
      }
      whitelist.value.unshift(newRule)
      ElMessage.success('已添加')
    }

    showAddDialogFlag.value = false
  } catch (error) {
    ElMessage.error(isEditing.value ? '更新失败' : '添加失败')
  } finally {
    submitting.value = false
  }
}

const toggleRule = async (row: WhitelistRule) => {
  try {
    // TODO: Call Tauri command
    // await invoke('toggle_whitelist_rule', { id: row.id, enabled: row.enabled })
    ElMessage.success(row.enabled ? '已启用' : '已禁用')
  } catch (error) {
    ElMessage.error('操作失败')
    row.enabled = !row.enabled // Revert
  }
}

const deleteRule = (row: WhitelistRule) => {
  ElMessageBox.confirm('确定删除该规则？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(async () => {
    try {
      // TODO: Call Tauri command
      // await invoke('delete_whitelist_rule', { id: row.id })
      const index = whitelist.value.indexOf(row)
      if (index > -1) {
        whitelist.value.splice(index, 1)
      }
      ElMessage.success('已删除')
    } catch (error) {
      ElMessage.error('删除失败')
    }
  }).catch(() => {
    ElMessage.info('已取消')
  })
}

const handleImport = () => {
  selectedFile.value = null
  showImportDialogFlag.value = true
}

const handleFileChange = (file: any) => {
  selectedFile.value = file.raw
}

const handleExceed = () => {
  ElMessage.warning('只能上传一个文件')
}

const confirmImport = async () => {
  if (!selectedFile.value) {
    ElMessage.warning('请选择文件')
    return
  }

  importing.value = true
  try {
    const text = await selectedFile.value.text()
    const importedRules = JSON.parse(text)

    if (!Array.isArray(importedRules)) {
      throw new Error('Invalid format')
    }

    // TODO: Call Tauri command to import
    // await invoke('import_whitelist_rules', { rules: importedRules })
    whitelist.value = [...importedRules, ...whitelist.value]

    ElMessage.success(`成功导入 ${importedRules.length} 条规则`)
    showImportDialogFlag.value = false
  } catch (error) {
    ElMessage.error('导入失败：文件格式错误')
  } finally {
    importing.value = false
  }
}

const handleExport = () => {
  const data = JSON.stringify(whitelist.value, null, 2)
  const blob = new Blob([data], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `whitelist_${new Date().toISOString().slice(0, 10)}.json`
  link.click()
  URL.revokeObjectURL(url)
  ElMessage.success('导出成功')
}

// Load whitelist from backend
const loadWhitelist = async () => {
  try {
    // TODO: Call Tauri command
    // const result = await invoke('get_whitelist_rules')
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

.filter-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.content-cell {
  display: flex;
  align-items: center;
}

.content-cell code {
  background-color: #f5f7fa;
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Courier New', monospace;
  font-size: 13px;
}

.empty-state {
  text-align: center;
  padding: 40px;
}

.statistics {
  display: flex;
  gap: 40px;
  margin-top: 20px;
  padding: 20px;
  background-color: #f9fafc;
  border-radius: 8px;
}

.form-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.upload-demo {
  width: 100%;
}
</style>
