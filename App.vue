<template>
  <div id="app" class="app-container">
    <el-container class="app-layout">
      <!-- Header -->
      <el-header class="app-header">
        <div class="header-content">
          <h1 class="app-title">敏感信息全盘扫描工具</h1>
          <div class="header-actions">
            <el-button type="text" @click="showAbout">关于</el-button>
            <el-button type="text" @click="showSettings">设置</el-button>
          </div>
        </div>
      </el-header>

      <!-- Main Content -->
      <el-container class="app-content">
        <!-- Sidebar -->
        <el-aside class="app-sidebar">
          <el-menu
            :default-active="activeMenu"
            @select="handleMenuSelect"
            class="sidebar-menu"
          >
            <el-menu-item index="scan">
              <el-icon><Search /></el-icon>
              <span>开始扫描</span>
            </el-menu-item>
            <el-menu-item index="results">
              <el-icon><DocumentCopy /></el-icon>
              <span>扫描结果</span>
            </el-menu-item>
            <el-menu-item index="history">
              <el-icon><Clock /></el-icon>
              <span>历史记录</span>
            </el-menu-item>
            <el-menu-item index="whitelist">
              <el-icon><SuccessFilled /></el-icon>
              <span>白名单管理</span>
            </el-menu-item>
          </el-menu>
        </el-aside>

        <!-- Main Panel -->
        <el-main class="app-main">
          <router-view />
        </el-main>
      </el-container>
    </el-container>

    <!-- Settings Dialog -->
    <el-dialog v-model="showSettingsDialog" title="系统设置" width="600px">
      <div class="settings-content">
        <el-form label-width="120px">
          <el-form-item label="主题">
            <el-select v-model="settings.theme">
              <el-option label="浅色" value="light" />
              <el-option label="深色" value="dark" />
              <el-option label="跟随系统" value="auto" />
            </el-select>
          </el-form-item>
          <el-form-item label="线程数">
            <el-input-number v-model="settings.threadCount" :min="1" :max="16" />
          </el-form-item>
          <el-form-item label="最大文件大小 (MB)">
            <el-input-number v-model="settings.maxFileSize" :min="10" :max="1000" />
          </el-form-item>
        </el-form>
      </div>
      <template #footer>
        <el-button @click="showSettingsDialog = false">取消</el-button>
        <el-button type="primary" @click="saveSettings">保存</el-button>
      </template>
    </el-dialog>

    <!-- About Dialog -->
    <el-dialog v-model="showAboutDialog" title="关于" width="500px">
      <div class="about-content">
        <p><strong>敏感信息全盘扫描工具</strong></p>
        <p>版本: 0.1.0</p>
        <p>一款轻量级、跨平台的敏感数据识别工具</p>
        <p>支持扫描 Excel、CSV、TXT 文件中的手机号、身份证、姓名、地址等敏感信息</p>
        <p style="margin-top: 20px; color: #999;">© 2026 Sensitive Scanner Team</p>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { Search, DocumentCopy, Clock, SuccessFilled } from '@element-plus/icons-vue'

const router = useRouter()
const activeMenu = ref('scan')
const showSettingsDialog = ref(false)
const showAboutDialog = ref(false)

const settings = ref({
  theme: 'auto',
  threadCount: 4,
  maxFileSize: 100,
})

const handleMenuSelect = (key: string) => {
  activeMenu.value = key
  router.push(`/${key}`)
}

const showSettings = () => {
  showSettingsDialog.value = true
}

const showAbout = () => {
  showAboutDialog.value = true
}

const saveSettings = () => {
  // Save settings to localStorage or backend
  localStorage.setItem('app-settings', JSON.stringify(settings.value))
  showSettingsDialog.value = false
  ElMessage.success('设置已保存')
}
</script>

<style scoped lang="css">
.app-container {
  width: 100%;
  height: 100vh;
  background-color: #f5f7fa;
}

.app-layout {
  height: 100%;
}

.app-header {
  background-color: #fff;
  border-bottom: 1px solid #ebeef5;
  padding: 0 20px;
  display: flex;
  align-items: center;
}

.header-content {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.app-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.app-content {
  flex: 1;
  overflow: hidden;
}

.app-sidebar {
  width: 200px;
  background-color: #fff;
  border-right: 1px solid #ebeef5;
  overflow-y: auto;
}

.sidebar-menu {
  border-right: none;
}

.app-main {
  overflow-y: auto;
  padding: 20px;
}

.settings-content {
  padding: 20px 0;
}

.about-content {
  padding: 20px 0;
  line-height: 1.8;
}
</style>
