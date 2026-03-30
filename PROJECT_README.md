# 敏感信息全盘扫描工具 - 项目说明

## 项目概述

本项目是一款轻量级、跨平台的敏感信息全盘扫描工具，使用 **Tauri + Rust + Vue 3** 技术栈开发。该工具可以在用户计算机上扫描 Excel、CSV、TXT 等文件，识别其中的手机号、身份证号、姓名、地址等敏感信息，并提供详细的扫描结果、历史记录、白名单管理等功能。

## 核心特性

### 1. 高效扫描引擎
- **异步文件遍历**：使用 Tokio 异步运行时，支持高并发文件遍历
- **流式 Excel 解析**：使用 Calamine 库进行 SAX 流式读取，防止大文件内存溢出
- **多线程并发处理**：使用 Rayon 库充分利用 CPU 多核能力，显著提升扫描速度
- **智能路径排除**：内置系统目录排除规则，用户可自定义排除路径

### 2. 精准敏感信息识别
- **手机号识别**：支持 1[3-9]\d{9} 格式，排除测试号码和连续相同数字
- **身份证号识别**：支持 18 位和 15 位格式，包含 GB 11643-1999 校验码验证
- **姓名识别**：支持 2-4 个汉字，排除常见非人名词汇
- **地址识别**：支持多种地址格式，包含省市区街道关键词匹配
- **文本预处理**：自动去除空格、横杠、不可见字符，防止绕过检测

### 3. 数据安全存储
- **本地 SQLite 数据库**：所有扫描结果存储在本地，不上传云端
- **实时落盘机制**：发现敏感信息立即写入数据库，防止意外丢失
- **内容脱敏显示**：UI 中显示脱敏内容，原始内容仅在详情页面可见
- **数据库加密**：支持密码保护导出报告

### 4. 灵活的用户界面
- **Vue 3 + Element Plus**：现代化、响应式的用户界面
- **实时进度展示**：动态显示当前扫描进度、文件数、结果数等信息
- **多维度结果筛选**：支持按文件路径、敏感类型、时间等多条件筛选
- **报告导出**：支持导出为 Excel、PDF、CSV 格式

### 5. 跨平台支持
- **Windows**：x86_64 架构
- **macOS**：x86_64 和 ARM64（Apple Silicon）架构
- **Linux**：包括国产系统（银河麒麟、统信 UOS）的 x86_64 和 ARM64 架构

## 项目结构

```
sensitive-scanner/
├── src/                          # 前端 Vue 3 源代码
│   ├── main.ts                   # 应用入口
│   ├── App.vue                   # 根组件
│   ├── index.html                # HTML 模板
│   ├── pages/                    # 页面组件
│   │   ├── ScanPage.vue          # 扫描配置页面
│   │   ├── ResultsPage.vue       # 结果展示页面
│   │   ├── HistoryPage.vue       # 历史记录页面
│   │   └── WhitelistPage.vue     # 白名单管理页面
│   ├── router/                   # Vue Router 配置
│   │   └── index.ts              # 路由定义
│   ├── stores/                   # Pinia 状态管理
│   │   └── scanStore.ts          # 扫描状态管理
│   └── services/                 # 服务层
│       └── api.ts                # Tauri API 调用接口
│
├── src-tauri/                    # 后端 Rust 源代码
│   ├── src/
│   │   ├── main.rs               # 应用入口
│   │   ├── lib.rs                # 库入口，定义 Tauri 命令
│   │   ├── models.rs             # 数据模型定义
│   │   ├── patterns.rs           # 敏感信息识别规则
│   │   ├── scanner.rs            # 文件扫描引擎
│   │   ├── db.rs                 # SQLite 数据库操作
│   │   └── commands.rs           # Tauri 命令处理
│   ├── Cargo.toml                # Rust 依赖配置
│   └── tauri.conf.json           # Tauri 应用配置
│
├── vite.config.js                # Vite 构建配置
├── package.json                  # Node.js 依赖配置
└── PROJECT_README.md             # 本文件
```

## 技术栈详解

### 前端技术
| 技术 | 版本 | 用途 |
|------|------|------|
| Vue | 3.5.31 | 前端框架 |
| Vue Router | 5.0.4 | 路由管理 |
| Pinia | 3.0.4 | 状态管理 |
| Element Plus | 2.13.6 | UI 组件库 |
| Vite | 8.0.3 | 构建工具 |
| TypeScript | - | 类型安全 |
| Axios | 1.14.0 | HTTP 客户端 |
| @tauri-apps/api | 2.10.1 | Tauri API |

### 后端技术
| 技术 | 版本 | 用途 |
|------|------|------|
| Rust | 1.94.1 | 核心语言 |
| Tauri | 2.x | 桌面框架 |
| Tokio | 1.x | 异步运行时 |
| Calamine | 0.24 | Excel 解析 |
| Regex | 1.x | 正则匹配 |
| Rayon | 1.x | 并行处理 |
| Rusqlite | 0.32 | SQLite 驱动 |
| Serde | 1.x | 序列化 |

## 核心功能模块

### 1. 扫描配置模块 (ScanPage.vue)
- **路径选择**：支持多路径选择，记忆上次扫描路径
- **排除规则**：系统内置排除项 + 用户自定义排除项
- **文件大小限制**：避免扫描过大文件浪费时间
- **敏感类型选择**：用户可选择需要识别的敏感信息类型
- **进度实时显示**：显示当前文件、已扫描文件数、发现结果数等

### 2. 扫描引擎模块 (scanner.rs)
- **异步文件遍历**：使用 async-walkdir 递归遍历目录
- **流式 Excel 解析**：使用 Calamine 库的 SAX 模式，逐行读取
- **并发处理**：使用 Rayon 库实现多线程并行扫描
- **内存管理**：确保内存占用恒定 <150MB

### 3. 敏感信息识别模块 (patterns.rs)
- **文本预处理**：去除空格、横杠等特殊字符
- **手机号检测**：正则匹配 + 测试号码排除
- **身份证检测**：正则匹配 + 校验码验证
- **姓名检测**：汉字匹配 + 常见词汇排除
- **地址检测**：关键词匹配 + 格式验证

### 4. 数据存储模块 (db.rs)
- **SQLite 操作**：使用 Rusqlite 库进行数据库操作
- **结果存储**：实时将发现的敏感信息写入数据库
- **历史管理**：记录每次扫描的配置和统计信息
- **白名单管理**：支持添加、删除、导入导出白名单

### 5. 结果展示模块 (ResultsPage.vue)
- **表格展示**：显示文件路径、位置、类型、脱敏内容等
- **多条件筛选**：按文件、类型、时间等条件筛选
- **关键词搜索**：快速查找特定结果
- **批量操作**：支持批量复制、删除等操作

### 6. 历史记录模块 (HistoryPage.vue)
- **历史列表**：显示所有扫描任务的历史记录
- **详情查看**：查看每次扫描的配置、统计信息等
- **重新执行**：支持基于历史记录重新执行扫描
- **清理功能**：支持删除指定时间范围的历史数据

### 7. 白名单管理模块 (WhitelistPage.vue)
- **添加白名单**：手动添加已知的合法敏感信息
- **导入导出**：支持 CSV/JSON 格式导入导出
- **预设模板**：提供常见白名单模板
- **自动匹配**：扫描时自动排除白名单中的内容

## 开发指南

### 环境要求
- Node.js 22.x+
- Rust 1.94.1+
- Tauri CLI 2.x+
- 系统依赖（Linux）：libwebkit2gtk-4.1-dev, libssl-dev, libgtk-3-dev

### 本地开发

1. **安装依赖**
   ```bash
   cd sensitive-scanner
   pnpm install
   cd src-tauri
   cargo build
   ```

2. **启动开发服务器**
   ```bash
   # 终端 1：启动 Vite 前端开发服务器
   pnpm dev
   
   # 终端 2：启动 Tauri 应用
   pnpm tauri dev
   ```

3. **构建生产版本**
   ```bash
   pnpm tauri build
   ```

### 代码规范

#### Rust 代码
- 遵循 Rust 官方编码规范
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 单元测试覆盖率 > 70%

#### Vue/TypeScript 代码
- 遵循 ESLint 规范
- 使用 TypeScript 进行类型检查
- 组件名使用 PascalCase
- 文件名使用 kebab-case

### Tauri 命令接口

#### 后端命令定义 (src-tauri/src/commands.rs)
```rust
#[tauri::command]
pub async fn start_scan(
    scan_paths: Vec<String>,
    exclude_paths: Vec<String>,
    max_file_size: u64,
    sensitive_types: Vec<String>,
) -> Result<String, String>
```

#### 前端调用 (src/services/api.ts)
```typescript
const result = await TauriAPI.startScan({
  scan_paths: ['/home/user/Documents'],
  exclude_paths: [],
  max_file_size: 100 * 1024 * 1024,
  sensitive_types: ['phonenumber', 'idcard']
})
```

### 事件系统

后端可以通过事件向前端发送实时更新：

```rust
// 后端发送进度事件
app_handle.emit_all("scan-progress", progress_data)?;

// 后端发送结果事件
app_handle.emit_all("scan-result", result_data)?;

// 后端发送完成事件
app_handle.emit_all("scan-complete", stats_data)?;
```

前端监听事件：

```typescript
const unlisten = await TauriAPI.onScanProgress((progress) => {
  console.log('Progress:', progress)
})

// 不需要时取消监听
unlisten()
```

## 性能指标

| 指标 | 目标 | 备注 |
|------|------|------|
| 冷启动时间 | < 2 秒 | 首次启动应用 |
| 热启动时间 | < 0.5 秒 | 应用已在内存中 |
| 单线程 100MB Excel | < 5 秒 | 主流 PC (8核 SSD) |
| 全盘扫描 (10000 文件) | < 5 分钟 | 主流 PC (8核 SSD) |
| 内存占用峰值 | < 150MB | 常规扫描 |
| 安装包大小 | < 20MB | 各平台 |

## 数据库架构

### 表结构

#### scan_results 表
```sql
CREATE TABLE scan_results (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL,
    sheet_name TEXT,
    row INTEGER NOT NULL,
    column INTEGER NOT NULL,
    sensitive_type TEXT NOT NULL,
    content TEXT NOT NULL,
    masked_content TEXT NOT NULL,
    found_at DATETIME NOT NULL
);
```

#### scan_history 表
```sql
CREATE TABLE scan_history (
    id TEXT PRIMARY KEY,
    scan_paths TEXT NOT NULL,        -- JSON 数组
    config TEXT NOT NULL,            -- JSON 对象
    stats TEXT NOT NULL,             -- JSON 对象
    created_at DATETIME NOT NULL,
    completed_at DATETIME
);
```

#### whitelist 表
```sql
CREATE TABLE whitelist (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    sensitive_type TEXT NOT NULL,
    description TEXT,
    created_at DATETIME NOT NULL
);
```

## 安全性考虑

1. **本地化处理**：所有敏感信息处理在本地进行，不上传云端
2. **权限管理**：遵守操作系统文件权限，无权限文件自动跳过
3. **数据加密**：支持导出报告时使用密码保护
4. **内容脱敏**：UI 中显示脱敏内容，防止敏感信息泄露
5. **日志记录**：详细的操作日志便于审计

## 常见问题

### Q: 为什么某些文件无法扫描？
A: 可能原因包括：
- 文件超过设定的大小限制
- 文件格式不支持（仅支持 .xlsx, .xls, .csv, .txt）
- 文件被其他程序占用
- 权限不足

### Q: 如何提高扫描速度？
A: 可以尝试以下方法：
- 增加线程数（在设置中调整）
- 排除不必要的目录
- 增加文件大小限制（跳过大文件）
- 只选择需要的敏感类型

### Q: 扫描结果存储在哪里？
A: 根据操作系统不同，存储位置为：
- Windows: `%APPDATA%\SensitiveScanner\results.db`
- macOS: `~/Library/Application Support/SensitiveScanner/results.db`
- Linux: `~/.config/sensitive-scanner/results.db`

### Q: 如何导出扫描结果？
A: 在结果页面点击"导出"按钮，选择导出格式（Excel/PDF/CSV）和保存位置即可。

## 许可证

本项目采用 MIT 许可证。详见 LICENSE 文件。

## 联系方式

- 技术支持：support@example.com
- 问题反馈：https://github.com/example/sensitive-scanner/issues
- 官方网站：https://example.com

## 更新日志

### v0.1.0 (2026-03-29)
- 初始版本发布
- 实现核心扫描功能
- 支持手机号、身份证、姓名、地址识别
- 提供 Web UI 界面
- 支持多平台部署

---

**注意**：本工具仅用于合法的数据安全检查目的。用户应确保拥有扫描文件的合法权限，并遵守相关法律法规。
