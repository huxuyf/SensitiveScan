# 敏感信息全盘扫描工具

一款轻量级、跨平台的敏感信息全盘扫描工具，使用 **Tauri + Rust + Vue 3** 技术栈开发。该工具可以在用户计算机上扫描 Excel、CSV、TXT 等文件，识别其中的手机号、身份证号、姓名、地址等敏感信息，并提供详细的扫描结果、历史记录、白名单管理等功能。

## ✨ 核心特性

### 🔍 高效扫描引擎
- **异步文件遍历**：使用 Tokio 异步运行时，支持高并发文件遍历
- **流式 Excel 解析**：使用 Calamine 库进行 SAX 流式读取，防止大文件内存溢出
- **多线程并发处理**：使用 Rayon 库充分利用 CPU 多核能力，显著提升扫描速度
- **智能路径排除**：内置系统目录排除规则，用户可自定义排除路径

### 🎯 精准敏感信息识别
- **手机号识别**：支持 1[3-9]\d{9} 格式，排除测试号码和连续相同数字
- **身份证号识别**：支持 18 位和 15 位格式，包含 GB 11643-1999 校验码验证
- **姓名识别**：支持 2-4 个汉字，排除常见非人名词汇
- **地址识别**：支持多种地址格式，包含省市区街道关键词匹配
- **文本预处理**：自动去除空格、横杠、不可见字符，防止绕过检测

### 🔒 数据安全存储
- **本地 SQLite 数据库**：所有扫描结果存储在本地，不上传云端
- **实时落盘机制**：发现敏感信息立即写入数据库，防止意外丢失
- **内容脱敏显示**：UI 中显示脱敏内容，原始内容仅在详情页面可见
- **数据库加密**：支持密码保护导出报告

### 🎨 现代化用户界面
- **Vue 3 + Element Plus**：现代化、响应式的用户界面
- **实时进度展示**：动态显示当前扫描进度、文件数、结果数等信息
- **多维度结果筛选**：支持按文件路径、敏感类型、时间等多条件筛选
- **报告导出**：支持导出为 Excel、PDF、CSV 格式

### 🌐 跨平台支持
- **Windows**：x86_64 架构
- **macOS**：x86_64 和 ARM64（Apple Silicon）架构
- **Linux**：包括国产系统（银河麒麟、统信 UOS）的 x86_64 和 ARM64 架构

## 🚀 快速开始

### 环境要求

- Node.js 22.x+
- Rust 1.94.1+
- Tauri CLI 2.x+
- 系统依赖（Linux）：libwebkit2gtk-4.1-dev, libssl-dev, libgtk-3-dev

### 安装依赖

```bash
# 克隆项目
git clone <repository-url>
cd SensitiveScan

# 安装前端依赖
pnpm install

# 编译后端
cd src-tauri
cargo build
```

### 开发模式

```bash
# 启动开发服务器
pnpm tauri dev
```

### 构建生产版本

```bash
# 构建所有平台
pnpm tauri build

# 仅构建当前平台
pnpm tauri build --target [target-triple]
```

## 📁 项目结构

```
SensitiveScan/
├── src/                          # 前端 Vue 3 源代码
│   ├── main.ts                   # 应用入口
│   ├── App.vue                   # 根组件
│   ├── index.html                # HTML 模板
│   ├── ScanPage.vue              # 扫描配置页面
│   ├── ResultsPage.vue           # 结果展示页面
│   ├── HistoryPage.vue           # 历史记录页面
│   ├── WhitelistPage.vue         # 白名单管理页面
│   ├── index.ts                  # 路由配置
│   ├── scanStore.ts              # 状态管理
│   └── api.ts                    # API 接口
├── src-tauri/                    # 后端 Rust 源代码
│   ├── src/
│   │   ├── main.rs               # 应用入口
│   │   ├── lib.rs                # 库入口
│   │   ├── models.rs             # 数据模型
│   │   ├── patterns.rs           # 敏感信息识别规则
│   │   ├── scanner.rs            # 文件扫描引擎
│   │   ├── db.rs                 # 数据库操作
│   │   └── commands.rs           # Tauri 命令
│   ├── Cargo.toml                # Rust 依赖配置
│   └── tauri.conf.json           # Tauri 应用配置
├── package.json                  # Node.js 依赖配置
├── pnpm-lock.yaml                # pnpm 锁文件
├── vite.config.js                # Vite 构建配置
├── .github/workflows/            # GitHub Actions 工作流
│   └── build.yml                 # 构建配置
├── .gitignore                    # Git 忽略文件
└── README.md                     # 本文件
```

## 🔧 技术栈

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

## 📊 性能指标

| 指标 | 目标 | 备注 |
|------|------|------|
| 冷启动时间 | < 2 秒 | 首次启动应用 |
| 热启动时间 | < 0.5 秒 | 应用已在内存中 |
| 单线程 100MB Excel | < 5 秒 | 主流 PC (8核 SSD) |
| 全盘扫描 (10000 文件) | < 5 分钟 | 主流 PC (8核 SSD) |
| 内存占用峰值 | < 150MB | 常规扫描 |
| 安装包大小 | < 20MB | 各平台 |

## 🛡️ 安全性考虑

1. **本地化处理**：所有敏感信息处理在本地进行，不上传云端
2. **权限管理**：遵守操作系统文件权限，无权限文件自动跳过
3. **数据加密**：支持导出报告时使用密码保护
4. **内容脱敏**：UI 中显示脱敏内容，防止敏感信息泄露
5. **日志记录**：详细的操作日志便于审计

## 📖 使用指南

### 扫描配置

1. 选择要扫描的路径（支持多路径）
2. 设置排除路径和文件大小限制
3. 选择需要识别的敏感信息类型
4. 点击"开始扫描"按钮

### 查看结果

1. 扫描完成后自动跳转到结果页面
2. 可以按文件路径、敏感类型、时间等条件筛选
3. 点击查看按钮可以看到原始内容
4. 支持导出为 Excel、PDF、CSV 格式

### 历史记录

1. 在历史记录页面查看所有扫描任务
2. 可以重新执行历史扫描任务
3. 支持删除指定时间范围的历史数据

### 白名单管理

1. 添加已知的合法敏感信息到白名单
2. 支持导入导出白名单
3. 扫描时自动排除白名单中的内容

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

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

## 📝 许可证

本项目采用 MIT 许可证。详见 LICENSE 文件。

## 📧 联系方式

- 技术支持：support@example.com
- 问题反馈：https://github.com/example/sensitive-scanner/issues
- 官方网站：https://example.com

## 🔗 相关文档

- [项目说明](./敏感信息全盘扫描工具%20-%20项目说明.md)
- [安装与使用指南](./敏感信息全盘扫描工具%20-%20安装与使用指南.md)
- [API 文档](./API%20文档.md)
- [开发指南](./开发指南.md)
- [跨平台编译指南](./敏感信息全盘扫描工具%20-%20跨平台编译指南.md)

## ⚠️ 免责声明

本工具仅用于合法的数据安全检查目的。用户应确保拥有扫描文件的合法权限，并遵守相关法律法规。

---

**注意**：本工具会扫描本地文件系统中的敏感信息，请确保在合法授权的环境下使用。

## 📦 下载

您可以从以下地址下载最新版本：

- [Windows 版本](https://github.com/example/sensitive-scanner/releases/latest)
- [macOS 版本](https://github.com/example/sensitive-scanner/releases/latest)
- [Linux 版本](https://github.com/example/sensitive-scanner/releases/latest)

## 🔄 更新日志

### v0.1.0 (2026-03-29)
- 初始版本发布
- 实现核心扫描功能
- 支持手机号、身份证、姓名、地址识别
- 提供 Web UI 界面
- 支持多平台部署
