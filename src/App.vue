<template>
  <div id="app" class="app-container">
    <el-container class="app-layout">
      <!-- Header -->
      <el-header class="app-header">
        <div class="header-content">
          <div class="header-left">
            <h1 class="app-title">敏感信息扫描工具</h1>
            <span class="page-title">- {{ currentPageTitle }}</span>
          </div>
          <div class="header-actions">
            <el-dropdown trigger="click" @command="handleLanguageChange">
              <el-button class="header-btn">
                <el-icon><Globe /></el-icon>
                <span>{{ currentLanguage }}</span>
                <el-icon class="el-icon--right"><ArrowDown /></el-icon>
              </el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="zh">简体中文</el-dropdown-item>
                  <el-dropdown-item command="en">English</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
            <el-tooltip content="重置设置" placement="bottom">
              <el-button class="header-btn" @click="resetSettings">
                <el-icon><RefreshRight /></el-icon>
              </el-button>
            </el-tooltip>
            <el-dropdown trigger="click" @command="handleSaveCommand">
              <el-button class="header-btn" type="primary">
                <el-icon><DocumentCopy /></el-icon>
                <span>保存</span>
                <el-icon class="el-icon--right"><ArrowDown /></el-icon>
              </el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="save">保存当前设置</el-dropdown-item>
                  <el-dropdown-item command="export">导出配置</el-dropdown-item>
                  <el-dropdown-item command="import">导入配置</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
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
            :collapse="isCollapse"
          >
            <!-- 扫描配置 -->
            <el-sub-menu index="scan">
              <template #title>
                <el-icon><Setting /></el-icon>
                <span>扫描配置</span>
              </template>
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
            </el-sub-menu>

            <!-- 数据管理 -->
            <el-sub-menu index="data">
              <template #title>
                <el-icon><FolderOpened /></el-icon>
                <span>数据管理</span>
              </template>
              <el-menu-item index="whitelist">
                <el-icon><SuccessFilled /></el-icon>
                <span>白名单管理</span>
              </el-menu-item>
              <el-menu-item index="export">
                <el-icon><Download /></el-icon>
                <span>数据导出</span>
              </el-menu-item>
            </el-sub-menu>

            <!-- 系统设置 -->
            <el-sub-menu index="system">
              <template #title>
                <el-icon><Tools /></el-icon>
                <span>系统设置</span>
              </template>
              <el-menu-item index="settings">
                <el-icon><Setting /></el-icon>
                <span>基本设置</span>
              </el-menu-item>
              <el-menu-item index="advanced">
                <el-icon><Operation /></el-icon>
                <span>高级选项</span>
              </el-menu-item>
            </el-sub-menu>
          </el-menu>

          <!-- Version Info -->
          <div class="version-info">
            <el-tag size="small" type="info">版本 0.1.0</el-tag>
          </div>
        </el-aside>

        <!-- Main Panel -->
        <el-main class="app-main">
          <!-- Breadcrumb -->
          <div class="breadcrumb-container">
            <el-breadcrumb separator="/">
              <el-breadcrumb-item to="/">{{ appTitle }}</el-breadcrumb-item>
              <el-breadcrumb-item>{{ currentPageTitle }}</el-breadcrumb-item>
            </el-breadcrumb>
          </div>

          <!-- Router View -->
          <div class="content-container">
            <router-view />
          </div>
        </el-main>
      </el-container>
    </el-container>

    <!-- About Dialog -->
    <el-dialog v-model="showAboutDialog" title="关于" width="500px">
      <div class="about-content">
        <div class="about-header">
          <el-icon class="about-icon"><Shield /></el-icon>
          <h2>敏感信息全盘扫描工具</h2>
        </div>
        <div class="about-info">
          <p><strong>版本:</strong> 0.1.0</p>
          <p><strong>描述:</strong> 轻量级、跨平台的敏感数据识别工具</p>
          <p><strong>功能:</strong></p>
          <ul>
            <li>扫描 Excel、CSV、TXT 文件</li>
            <li>识别手机号、身份证、姓名、地址等敏感信息</li>
            <li>支持白名单管理和数据导出</li>
            <li>多线程扫描，高效快速</li>
          </ul>
        </div>
        <div class="about-footer">
          <p>© 2026 Sensitive Scanner Team</p>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import {
  Search,
  DocumentCopy,
  Clock,
  SuccessFilled,
  Setting,
  FolderOpened,
  Download,
  Tools,
  Operation,
  Globe,
  ArrowDown,
  RefreshRight,
  Shield
} from '@element-plus/icons-vue'

const router = useRouter()
const route = useRoute()
const activeMenu = ref('scan')
const isCollapse = ref(false)
const currentLanguage = ref('简体中文')
const showAboutDialog = ref(false)

const appTitle = '敏感信息扫描工具'

const currentPageTitle = computed(() => {
  const titles: Record<string, string> = {
    scan: '开始扫描',
    results: '扫描结果',
    history: '历史记录',
    whitelist: '白名单管理',
    export: '数据导出',
    settings: '基本设置',
    advanced: '高级选项'
  }
  return titles[route.path.replace('/', '')] || '开始扫描'
})

const handleMenuSelect = (key: string) => {
  activeMenu.value = key
  router.push(`/${key}`)
}

const handleLanguageChange = (lang: string) => {
  const langMap: Record<string, string> = {
    zh: '简体中文',
    en: 'English'
  }
  currentLanguage.value = langMap[lang]
  ElMessage.success(`语言已切换为 ${currentLanguage.value}`)
}

const handleSaveCommand = (command: string) => {
  switch (command) {
    case 'save':
      ElMessage.success('设置已保存')
      break
    case 'export':
      ElMessage.success('配置已导出')
      break
    case 'import':
      ElMessage.success('配置已导入')
      break
  }
}

const resetSettings = () => {
  ElMessage.warning('设置已重置为默认值')
}

// Show about dialog on double-click header
const showAbout = () => {
  showAboutDialog.value = true
}

// Expose showAbout for external access
defineExpose({ showAbout })
</script>

<style scoped lang="css">
.app-container {
  width: 100%;
  height: 100vh;
  background-color: #f5f7fa;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

.app-layout {
  height: 100%;
}

/* Header */
.app-header {
  background-color: #fff;
  border-bottom: 1px solid #e8eaf6;
  padding: 0 24px;
  display: flex;
  align-items: center;
  height: 60px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

.header-content {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.app-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #333;
}

.page-title {
  color: #666;
  font-size: 18px;
  font-weight: 400;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.header-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
}

/* Main Content */
.app-content {
  flex: 1;
  overflow: hidden;
}

/* Sidebar */
.app-sidebar {
  width: 240px;
  background-color: #fff;
  border-right: 1px solid #e8eaf6;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.sidebar-menu {
  border-right: none;
  flex: 1;
}

.sidebar-menu :deep(.el-menu-item) {
  height: 48px;
  line-height: 48px;
  margin: 0;
  padding-left: 20px !important;
}

.sidebar-menu :deep(.el-sub-menu__title) {
  height: 48px;
  line-height: 48px;
  padding-left: 20px !important;
}

.sidebar-menu :deep(.el-menu-item.is-active) {
  background-color: #e3f2fd;
  color: #4a90e2;
  font-weight: 500;
}

.sidebar-menu :deep(.el-menu-item:hover) {
  background-color: #f5f7fa;
}

.version-info {
  padding: 16px 20px;
  border-top: 1px solid #e8eaf6;
  text-align: center;
}

/* Main Panel */
.app-main {
  overflow-y: auto;
  padding: 24px;
  background-color: #f8f9fa;
}

.breadcrumb-container {
  margin-bottom: 20px;
}

.breadcrumb-container :deep(.el-breadcrumb__inner) {
  color: #666;
  font-size: 14px;
}

.breadcrumb-container :deep(.el-breadcrumb__inner.is-link) {
  color: #4a90e2;
}

.breadcrumb-container :deep(.el-breadcrumb__inner a:hover) {
  color: #357abd;
}

.content-container {
  min-height: calc(100vh - 140px);
}

/* About Dialog */
.about-content {
  padding: 20px 0;
}

.about-header {
  text-align: center;
  margin-bottom: 24px;
}

.about-icon {
  font-size: 48px;
  color: #4a90e2;
  margin-bottom: 12px;
}

.about-header h2 {
  margin: 0;
  color: #333;
  font-size: 20px;
}

.about-info {
  margin-bottom: 20px;
  line-height: 1.8;
  color: #666;
}

.about-info p {
  margin: 8px 0;
}

.about-info ul {
  margin: 8px 0;
  padding-left: 20px;
}

.about-info li {
  margin: 4px 0;
}

.about-footer {
  text-align: center;
  padding-top: 20px;
  border-top: 1px solid #ebeef5;
  color: #999;
  font-size: 13px;
}

/* Scrollbar Styling */
:deep(.el-aside::-webkit-scrollbar),
:deep(.app-main::-webkit-scrollbar) {
  width: 6px;
}

:deep(.el-aside::-webkit-scrollbar-thumb),
:deep(.app-main::-webkit-scrollbar-thumb) {
  background-color: #dcdfe6;
  border-radius: 3px;
}

:deep(.el-aside::-webkit-scrollbar-thumb:hover),
:deep(.app-main::-webkit-scrollbar-thumb:hover) {
  background-color: #c0c4cc;
}
</style>
