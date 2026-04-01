# Sensitive Scanner - 开发文档

## 项目概述

Sensitive Scanner 是一个跨平台的敏感信息扫描工具，用于检测文件中的敏感数据（手机号、身份证、姓名、地址等）。

## 技术栈

### 后端
- **Rust**: 系统编程语言，提供高性能和内存安全
- **Tauri v2**: 跨平台桌面应用框架
- **SQLite**: 轻量级关系型数据库
- **Tokio**: 异步运行时
- **Rayon**: 数据并行处理库
- **Regex**: 正则表达式库

### 前端
- **Vue 3**: 渐进式 JavaScript 框架
- **TypeScript**: 类型安全的 JavaScript 超集
- **Vite**: 快速的前端构建工具
- **Element Plus**: Vue 3 组件库
- **Pinia**: Vue 状态管理库

## 项目结构

```
SensitiveScan/
├── src/                          # 前端源代码
│   ├── components/              # Vue 组件
│   │   ├── VirtualScroll.vue   # 虚拟滚动组件
│   │   └── ...
│   ├── pages/                   # 页面组件
│   │   ├── ScanPage.vue        # 扫描页面
│   │   ├── ResultsPage.vue     # 结果页面
│   │   ├── WhitelistPage.vue   # 白名单页面
│   │   └── ...
│   ├── stores/                  # Pinia 状态管理
│   │   ├── scanStore.ts        # 扫描状态管理
│   │   └── enhancedScanStore.ts # 增强的扫描状态管理
│   ├── App.vue                  # 根组件
│   └── main.ts                  # 应用入口
├── src-tauri/                   # 后端源代码
│   ├── src/
│   │   ├── commands.rs         # Tauri 命令
│   │   ├── scanner.rs          # 扫描器实现
│   │   ├── scanner_enhanced.rs # 增强的扫描器
│   │   ├── patterns.rs         # 敏感信息检测模式
│   │   ├── patterns_optimized.rs # 优化的模式匹配
│   │   ├── models.rs           # 数据模型
│   │   ├── db.rs               # 数据库操作
│   │   ├── db_enhanced.rs      # 增强的数据库操作
│   │   ├── whitelist_manager.rs # 白名单管理
│   │   ├── error.rs            # 错误处理
│   │   ├── logger.rs           # 日志系统
│   │   └── tests.rs            # 单元测试
│   ├── Cargo.toml              # Rust 依赖配置
│   └── tauri.conf.json         # Tauri 配置
├── package.json                # 前端依赖配置
├── vite.config.ts              # Vite 配置
├── tsconfig.json               # TypeScript 配置
└── README.md                   # 项目说明文档
```

## 核心模块说明

### 1. 扫描器 (scanner.rs / scanner_enhanced.rs)

**功能**:
- 文件遍历和扫描
- 敏感信息检测
- 并发处理
- 优雅中断和恢复

**关键特性**:
- 支持多种文件格式 (Excel, CSV, TXT)
- 并发文件扫描
- 流式处理大文件
- 扫描状态管理
- 断点续扫

**主要方法**:
```rust
pub async fn start_scan(&self) -> AppResult<()>
pub fn pause_scan(&self)
pub fn resume_scan(&self)
pub async fn stop_scan(&self) -> AppResult<()>
pub async fn resume_scan(&self, snapshot: ScanSnapshot) -> AppResult<()>
```

### 2. 模式匹配 (patterns.rs / patterns_optimized.rs)

**功能**:
- 手机号检测
- 身份证号检测
- 姓名检测
- 地址检测

**优化策略**:
- 预编译正则表达式
- RegexSet 多模式匹配
- Lazy static 初始化

**主要方法**:
```rust
pub fn detect_phone_number(text: &str) -> Option<String>
pub fn detect_id_card(text: &str) -> Option<String>
pub fn detect_name(text: &str) -> Option<String>
pub fn detect_address(text: &str) -> Option<String>
pub fn batch_detect_sensitive_info(text: &str) -> Vec<(SensitiveType, String)>
```

### 3. 数据库 (db.rs / db_enhanced.rs)

**功能**:
- 扫描结果存储
- 扫描历史记录
- 白名单管理
- 数据持久化

**数据库表**:
```sql
scan_results -- 扫描结果
scan_history -- 扫描历史
whitelist -- 白名单（旧版）
whitelist_rules -- 白名单规则（新版）
```

**主要方法**:
```rust
pub fn insert_scan_result(&self, result: &ScanResult) -> SqlResult<()>
pub fn get_scan_results(&self, ...) -> SqlResult<Vec<ScanResult>>
pub fn add_whitelist_rule(&self, rule: &WhitelistRule) -> SqlResult<()>
pub fn get_whitelist_rules(&self) -> SqlResult<Vec<WhitelistRule>>
```

### 4. 白名单管理 (whitelist_manager.rs)

**功能**:
- 精确匹配规则
- 正则表达式规则
- 规则启用/禁用
- 匹配次数统计
- 导入/导出

**主要方法**:
```rust
pub fn add_rule(&self, rule: WhitelistRule) -> Result<(), String>
pub fn is_whitelisted(&self, content: &str, sensitive_type: SensitiveType) -> bool
pub fn export_rules(&self) -> Result<String, String>
pub fn import_rules(&self, rules: Vec<WhitelistRule>) -> Result<(), String>
```

### 5. 错误处理 (error.rs)

**功能**:
- 统一错误类型
- 错误分类
- 用户友好消息
- 可恢复性判断

**错误类型**:
```rust
pub enum AppError {
    FileIo(std::io::Error),
    Database(rusqlite::Error),
    Regex(regex::Error),
    // ... 其他错误类型
}
```

### 6. 日志系统 (logger.rs)

**功能**:
- 结构化日志
- 文件轮转
- 双输出（控制台 + 文件）
- 日志级别控制

**日志级别**:
- Error
- Warning
- Info
- Debug

### 7. 虚拟滚动 (VirtualScroll.vue)

**功能**:
- 大数据量渲染优化
- 只渲染视口内元素
- 动态加载
- 性能优化

**Props**:
```typescript
interface Props {
  items: any[]
  itemHeight: number
  containerHeight: number
  bufferSize?: number
  loading?: boolean
}
```

## 性能优化

### 后端优化
1. **并发处理**: 使用 Tokio 和 Rayon 实现并发
2. **正则预编译**: Lazy static 预编译所有正则表达式
3. **多模式匹配**: RegexSet 实现 O(N) 多模式匹配
4. **流式处理**: BufReader 流式读取大文件
5. **线程池**: spawn_blocking 处理阻塞 I/O

### 前端优化
1. **虚拟滚动**: 只渲染可见区域元素
2. **分页加载**: 按需加载数据
3. **缓存策略**: Pinia 状态缓存
4. **懒加载**: 组件懒加载
5. **防抖节流**: 用户输入防抖

## 测试

### 后端测试
运行 Rust 单元测试：
```bash
cd src-tauri
cargo test
```

### 前端测试
运行 Vue 单元测试：
```bash
npm run test
```

### 测试覆盖率
- 后端核心模块覆盖率 > 80%
- 前端关键组件覆盖率 > 70%

## 开发指南

### 添加新的敏感类型

1. 在 `models.rs` 中添加新的枚举值：
```rust
pub enum SensitiveType {
    PhoneNumber,
    IdCard,
    Name,
    Address,
    NewType,  // 新类型
}
```

2. 在 `patterns.rs` 中添加检测函数：
```rust
pub fn detect_new_type(text: &str) -> Option<String> {
    // 实现检测逻辑
}
```

3. 在 `scanner.rs` 中集成检测：
```rust
if let Some(detected) = detect_new_type(content) {
    return Some(ScanResult { ... });
}
```

### 添加新的文件格式支持

1. 在 `scanner.rs` 中添加文件类型判断：
```rust
match extension.as_str() {
    "xlsx" | "xls" => Self::scan_excel_file_blocking(file_path),
    "csv" | "txt" => Self::scan_text_file_blocking(file_path),
    "newformat" => Self::scan_new_format_file_blocking(file_path),  // 新格式
}
```

2. 实现解析函数：
```rust
fn scan_new_format_file_blocking(file_path: &Path) -> AppResult<u64> {
    // 实现解析逻辑
}
```

### 添加新的白名单规则类型

1. 在 `whitelist_manager.rs` 中扩展 `WhitelistRule`：
```rust
pub struct WhitelistRule {
    pub id: String,
    pub content: String,
    pub pattern: String,
    pub is_regex: bool,
    pub rule_type: RuleType,  // 新增规则类型
    // ... 其他字段
}
```

2. 实现新的匹配逻辑：
```rust
pub fn is_whitelisted(&self, content: &str, sensitive_type: SensitiveType) -> bool {
    // 根据规则类型实现匹配逻辑
}
```

## 部署

### 构建 Windows 版本
```bash
npm run tauri build -- --target x86_64-pc-windows-msvc
```

### 构建 macOS 版本
```bash
npm run tauri build -- --target x86_64-apple-darwin
```

### 构建 Linux 版本
```bash
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

## 安全考虑

1. **数据加密**: 敏感数据使用脱敏显示
2. **权限控制**: 文件访问权限检查
3. **输入验证**: 所有用户输入验证
4. **SQL 注入防护**: 使用参数化查询
5. **XSS 防护**: 前端输入转义

## 贡献指南

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 许可证

本项目采用 MIT 许可证。

## 联系方式

- 项目地址: https://github.com/yourusername/SensitiveScan
- 问题反馈: https://github.com/yourusername/SensitiveScan/issues

## 更新日志

### v1.0.0 (2024-01-01)
- 初始版本发布
- 支持手机号、身份证、姓名、地址检测
- 支持Excel、CSV、TXT文件扫描
- 白名单管理功能
- 扫描结果导出

### v0.9.0 (2023-12-15)
- 添加虚拟滚动支持
- 优化性能
- 添加单元测试
- 完善错误处理

## 常见问题

### Q: 如何添加自定义正则表达式？
A: 在白名单管理页面，选择"正则表达式"模式，输入您的正则表达式即可。

### Q: 扫描速度慢怎么办？
A: 可以在扫描配置中增加线程数，或者减少扫描的文件类型和路径。

### Q: 如何查看扫描日志？
A: 日志文件位于：
- Windows: `%APPDATA%\SensitiveScanner\scanner.log`
- macOS: `~/Library/Logs/SensitiveScanner/scanner.log`
- Linux: `~/.config/sensitive-scanner/logs/scanner.log`

### Q: 扫描中断后如何恢复？
A: 在扫描页面点击"恢复"按钮，系统会从上次中断的位置继续扫描。

## 参考资料

- [Tauri 文档](https://tauri.app/v1/guides/)
- [Vue 3 文档](https://vuejs.org/)
- [Rust 文档](https://doc.rust-lang.org/)
- [Element Plus 文档](https://element-plus.org/)
