# API 文档

## 概述

本文档描述了敏感信息全盘扫描工具的 Tauri 命令接口和事件系统。

## Tauri 命令接口

### 扫描相关命令

#### 1. start_scan - 启动扫描

**功能**：启动一个新的扫描任务

**请求参数**：
```typescript
{
  scan_paths: string[]              // 扫描路径列表
  exclude_paths: string[]           // 排除路径列表
  max_file_size: number             // 最大文件大小（字节）
  sensitive_types: string[]         // 敏感类型列表
}
```

**响应数据**：
```typescript
{
  task_id: string                   // 任务 ID
  status: "started"                 // 状态
}
```

**示例**：
```typescript
const result = await TauriAPI.startScan({
  scan_paths: ['/home/user/Documents'],
  exclude_paths: ['/home/user/Documents/Temp'],
  max_file_size: 100 * 1024 * 1024,
  sensitive_types: ['phonenumber', 'idcard', 'name', 'address']
})
console.log(result.task_id)
```

#### 2. pause_scan - 暂停扫描

**功能**：暂停当前正在进行的扫描

**请求参数**：无

**响应数据**：
```typescript
{
  status: "paused"
}
```

**示例**：
```typescript
await TauriAPI.pauseScan()
```

#### 3. resume_scan - 继续扫描

**功能**：继续暂停的扫描

**请求参数**：无

**响应数据**：
```typescript
{
  status: "resumed"
}
```

**示例**：
```typescript
await TauriAPI.resumeScan()
```

#### 4. stop_scan - 停止扫描

**功能**：停止当前正在进行的扫描

**请求参数**：无

**响应数据**：
```typescript
{
  status: "stopped"
}
```

**示例**：
```typescript
await TauriAPI.stopScan()
```

### 结果相关命令

#### 5. get_scan_results - 获取扫描结果

**功能**：获取扫描结果列表

**请求参数**：
```typescript
{
  limit?: number                    // 返回数量限制（默认 1000）
  offset?: number                   // 偏移量（默认 0）
  file_path_filter?: string         // 文件路径过滤
  sensitive_type_filter?: string    // 敏感类型过滤
}
```

**响应数据**：
```typescript
Array<{
  id: string                        // 结果 ID
  file_path: string                 // 文件路径
  sheet_name?: string               // Sheet 名称（Excel 文件）
  row: number                       // 行号
  column: number                    // 列号
  sensitive_type: string            // 敏感类型
  content: string                   // 原始内容
  masked_content: string            // 脱敏内容
  found_at: string                  // 发现时间
}>
```

**示例**：
```typescript
const results = await TauriAPI.getScanResults(
  100,                              // limit
  0,                                // offset
  '/home/user/Documents',           // file_path_filter
  'PhoneNumber'                     // sensitive_type_filter
)
```

#### 6. export_results - 导出扫描结果

**功能**：导出扫描结果为指定格式

**请求参数**：
```typescript
{
  format: string                    // 导出格式：'excel' | 'pdf' | 'csv'
  file_path: string                 // 导出文件路径
}
```

**响应数据**：
```typescript
{
  status: "exported"
  file_path: string
}
```

**示例**：
```typescript
await TauriAPI.exportResults('excel', '/home/user/results.xlsx')
```

### 历史记录相关命令

#### 7. get_history - 获取扫描历史

**功能**：获取扫描历史记录

**请求参数**：
```typescript
{
  limit?: number                    // 返回数量限制（默认 100）
}
```

**响应数据**：
```typescript
Array<{
  id: string                        // 历史记录 ID
  scan_paths: string[]              // 扫描路径
  config: object                    // 扫描配置
  stats: {
    total_files_scanned: number     // 扫描文件总数
    total_results_found: number     // 发现结果总数
    scan_duration_secs: number      // 扫描耗时（秒）
    scan_speed: number              // 扫描速度（文件/秒）
    results_by_type: object         // 按类型统计结果
  }
  created_at: string                // 创建时间
  completed_at?: string             // 完成时间
}>
```

**示例**：
```typescript
const history = await TauriAPI.getHistory(50)
```

#### 8. delete_history - 删除扫描历史

**功能**：删除指定的扫描历史记录

**请求参数**：
```typescript
{
  history_id: string                // 历史记录 ID
}
```

**响应数据**：
```typescript
{
  status: "deleted"
}
```

**示例**：
```typescript
await TauriAPI.deleteHistory('history-id-123')
```

### 白名单相关命令

#### 9. add_whitelist - 添加白名单

**功能**：添加白名单项

**请求参数**：
```typescript
{
  content: string                   // 白名单内容
  sensitive_type: string            // 敏感类型
  description?: string              // 描述
}
```

**响应数据**：
```typescript
{
  id: string                        // 白名单项 ID
  content: string
  sensitive_type: string
  description?: string
  created_at: string
}
```

**示例**：
```typescript
const entry = await TauriAPI.addWhitelist(
  '13800138000',
  'phonenumber',
  '测试电话号码'
)
```

#### 10. get_whitelist - 获取白名单

**功能**：获取所有白名单项

**请求参数**：无

**响应数据**：
```typescript
Array<{
  id: string
  content: string
  sensitive_type: string
  description?: string
  created_at: string
}>
```

**示例**：
```typescript
const whitelist = await TauriAPI.getWhitelist()
```

#### 11. delete_whitelist - 删除白名单项

**功能**：删除指定的白名单项

**请求参数**：
```typescript
{
  entry_id: string                  // 白名单项 ID
}
```

**响应数据**：
```typescript
{
  status: "deleted"
}
```

**示例**：
```typescript
await TauriAPI.deleteWhitelist('entry-id-123')
```

### 统计相关命令

#### 12. get_scan_stats - 获取扫描统计

**功能**：获取当前扫描统计信息

**请求参数**：无

**响应数据**：
```typescript
{
  total_results: number             // 总结果数
  timestamp: string                 // 时间戳
}
```

**示例**：
```typescript
const stats = await TauriAPI.getScanStats()
```

## 事件系统

### 事件监听

#### 1. scan-progress - 扫描进度事件

**功能**：实时报告扫描进度

**事件数据**：
```typescript
{
  current_file: string              // 当前扫描文件
  files_scanned: number             // 已扫描文件数
  results_found: number             // 发现结果数
  progress_percentage: number       // 进度百分比（0-100）
  elapsed_seconds: number           // 已用时间（秒）
  estimated_remaining_seconds: number // 预计剩余时间（秒）
  scan_speed: number                // 扫描速度（文件/秒）
}
```

**示例**：
```typescript
const unlisten = await TauriAPI.onScanProgress((progress) => {
  console.log(`进度: ${progress.progress_percentage}%`)
  console.log(`已扫描文件: ${progress.files_scanned}`)
  console.log(`发现结果: ${progress.results_found}`)
})

// 不需要时取消监听
unlisten()
```

#### 2. scan-result - 扫描结果事件

**功能**：发现新的敏感信息时立即报告

**事件数据**：
```typescript
{
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
```

**示例**：
```typescript
const unlisten = await TauriAPI.onScanResult((result) => {
  console.log(`发现 ${result.sensitive_type}: ${result.masked_content}`)
  console.log(`位置: ${result.file_path}:${result.row}:${result.column}`)
})
```

#### 3. scan-complete - 扫描完成事件

**功能**：扫描完成时报告最终统计信息

**事件数据**：
```typescript
{
  total_files_scanned: number
  total_results_found: number
  scan_duration_secs: number
  scan_speed: number
  results_by_type: {
    [key: string]: number           // 按类型统计
  }
}
```

**示例**：
```typescript
const unlisten = await TauriAPI.onScanComplete((stats) => {
  console.log(`扫描完成！`)
  console.log(`总文件数: ${stats.total_files_scanned}`)
  console.log(`总结果数: ${stats.total_results_found}`)
  console.log(`扫描耗时: ${stats.scan_duration_secs} 秒`)
})
```

## 错误处理

所有 API 调用都可能抛出异常。建议使用 try-catch 进行处理：

```typescript
try {
  const results = await TauriAPI.getScanResults()
} catch (error) {
  console.error('获取结果失败:', error)
  ElMessage.error('获取结果失败，请重试')
}
```

## 敏感类型定义

| 类型 | 值 | 说明 |
|------|-----|------|
| 手机号 | `phonenumber` | 11 位手机号 |
| 身份证 | `idcard` | 18 位或 15 位身份证号 |
| 姓名 | `name` | 2-4 个汉字 |
| 地址 | `address` | 地址字符串 |

## 导出格式

| 格式 | 说明 |
|------|------|
| `excel` | Microsoft Excel (.xlsx) |
| `pdf` | PDF 文档 (.pdf) |
| `csv` | 逗号分隔值 (.csv) |

## 最佳实践

### 1. 错误处理
```typescript
try {
  await TauriAPI.startScan(config)
} catch (error) {
  if (error.message.includes('permission')) {
    ElMessage.error('权限不足')
  } else {
    ElMessage.error('启动扫描失败')
  }
}
```

### 2. 事件监听清理
```typescript
let unlistens: Array<() => void> = []

onMounted(async () => {
  unlistens.push(await TauriAPI.onScanProgress(...))
  unlistens.push(await TauriAPI.onScanResult(...))
})

onUnmounted(() => {
  unlistens.forEach(unlisten => unlisten())
})
```

### 3. 进度显示
```typescript
const progressPercentage = ref(0)

await TauriAPI.onScanProgress((progress) => {
  progressPercentage.value = progress.progress_percentage
})
```

### 4. 结果收集
```typescript
const results = ref([])

await TauriAPI.onScanResult((result) => {
  results.value.push(result)
})
```

## 性能考虑

1. **批量获取结果**：使用 `limit` 和 `offset` 参数进行分页查询
2. **事件节流**：避免频繁更新 UI，使用防抖或节流
3. **内存管理**：及时取消事件监听，防止内存泄漏
4. **数据库优化**：定期清理历史数据

## 版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0.0 | 2026-03-29 | 初始版本 |

## 相关资源

- [Tauri 官方文档](https://tauri.app)
- [TypeScript 文档](https://www.typescriptlang.org)
- [Vue 3 文档](https://vuejs.org)
