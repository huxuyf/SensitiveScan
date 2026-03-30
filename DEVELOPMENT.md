# 开发指南

## 快速开始

### 前置要求
- Node.js 22.x+
- Rust 1.94.1+
- Tauri CLI 2.x+
- Git

### 安装步骤

1. **克隆项目**
   ```bash
   git clone <repository-url>
   cd sensitive-scanner
   ```

2. **安装前端依赖**
   ```bash
   pnpm install
   ```

3. **安装 Rust 工具链**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

4. **安装系统依赖（Linux 用户）**
   ```bash
   # Ubuntu/Debian
   sudo apt-get update
   sudo apt-get install -y libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev

   # Fedora
   sudo dnf install webkit2-gtk3-devel openssl-devel gtk3-devel

   # Arch
   sudo pacman -S webkit2gtk openssl gtk3
   ```

## 开发工作流

### 启动开发环境

#### 方式一：同时启动前后端（推荐）
```bash
# 在项目根目录运行
pnpm tauri dev
```

#### 方式二：分别启动前后端
```bash
# 终端 1：启动 Vite 前端开发服务器
pnpm dev

# 终端 2：启动 Tauri 应用（在项目根目录）
pnpm tauri dev
```

### 代码编辑

#### 前端开发
- 编辑 `src/` 目录下的 Vue 文件
- 修改后会自动热重载
- 使用 `pnpm lint` 检查代码质量

#### 后端开发
- 编辑 `src-tauri/src/` 目录下的 Rust 文件
- 修改后需要重新启动 Tauri 应用
- 使用 `cargo check` 快速检查编译错误
- 使用 `cargo clippy` 检查代码质量

### 构建生产版本

```bash
# 构建前端
pnpm build

# 构建 Tauri 应用
pnpm tauri build

# 输出文件位置
# - Windows: src-tauri/target/release/bundle/msi/
# - macOS: src-tauri/target/release/bundle/dmg/
# - Linux: src-tauri/target/release/bundle/deb/
```

## 项目结构详解

### 前端结构

```
src/
├── main.ts                 # Vue 应用入口
├── App.vue                 # 根组件
├── index.html              # HTML 模板
├── pages/                  # 页面组件
│   ├── ScanPage.vue        # 扫描配置页面
│   ├── ResultsPage.vue     # 结果展示页面
│   ├── HistoryPage.vue     # 历史记录页面
│   └── WhitelistPage.vue   # 白名单管理页面
├── components/             # 可复用组件
├── router/                 # 路由配置
│   └── index.ts
├── stores/                 # Pinia 状态管理
│   └── scanStore.ts
├── services/               # 服务层
│   └── api.ts              # Tauri API 封装
├── types/                  # TypeScript 类型定义
└── assets/                 # 静态资源
```

### 后端结构

```
src-tauri/src/
├── main.rs                 # 应用入口
├── lib.rs                  # 库入口，定义 Tauri 命令
├── models.rs               # 数据模型
├── patterns.rs             # 敏感信息识别规则
├── scanner.rs              # 文件扫描引擎
├── db.rs                   # 数据库操作
└── commands.rs             # Tauri 命令处理
```

## 常用命令

### 前端命令
```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm dev

# 构建生产版本
pnpm build

# 代码检查
pnpm lint

# 代码格式化
pnpm format
```

### 后端命令
```bash
# 检查编译
cargo check

# 编译
cargo build

# 编译发布版本
cargo build --release

# 运行测试
cargo test

# 代码检查和优化建议
cargo clippy

# 代码格式化
cargo fmt
```

### Tauri 命令
```bash
# 启动开发模式
pnpm tauri dev

# 构建应用
pnpm tauri build

# 查看 Tauri 版本
pnpm tauri --version

# 更新 Tauri
pnpm tauri update
```

## 调试技巧

### 前端调试
1. 在 Tauri 应用中按 `F12` 打开开发者工具
2. 使用 Chrome DevTools 进行调试
3. 在 `src/services/api.ts` 中添加日志输出

### 后端调试
1. 在 Rust 代码中使用 `println!` 或 `eprintln!` 输出日志
2. 在 `src-tauri/src/commands.rs` 中添加错误处理和日志
3. 使用 `RUST_LOG=debug` 环境变量启用调试日志

### 数据库调试
1. 使用 SQLite 客户端查看数据库内容
   ```bash
   sqlite3 ~/.config/sensitive-scanner/results.db
   ```
2. 查询扫描结果
   ```sql
   SELECT * FROM scan_results LIMIT 10;
   ```

## 测试

### 前端单元测试
```bash
# 安装测试依赖
pnpm add -D vitest @vue/test-utils

# 运行测试
pnpm test
```

### 后端单元测试
```bash
# 运行测试
cargo test

# 运行特定测试
cargo test test_phone_number_detection

# 显示输出
cargo test -- --nocapture
```

## 性能优化

### 前端优化
1. 使用 Vue DevTools 分析组件性能
2. 使用 Lighthouse 检查应用性能
3. 启用代码分割和懒加载

### 后端优化
1. 使用 `cargo flamegraph` 进行性能分析
2. 使用 `cargo bench` 进行基准测试
3. 优化正则表达式性能

## 常见问题

### 编译错误：找不到 libwebkit2gtk-4.1
**解决方案**：安装系统依赖
```bash
sudo apt-get install libwebkit2gtk-4.1-dev
```

### 编译错误：找不到 Rust 工具链
**解决方案**：确保 Rust 已正确安装
```bash
rustc --version
cargo --version
```

### 应用无法启动
**解决方案**：
1. 检查前端构建是否成功
2. 查看后端编译错误
3. 检查 Tauri 配置文件

### 数据库文件找不到
**解决方案**：
1. 检查数据库路径是否正确
2. 确保有足够的磁盘空间
3. 检查文件权限

## 贡献指南

### 提交代码
1. 创建新分支：`git checkout -b feature/your-feature`
2. 提交更改：`git commit -am 'Add new feature'`
3. 推送分支：`git push origin feature/your-feature`
4. 创建 Pull Request

### 代码规范
- 遵循 Rust 官方编码规范
- 遵循 Vue 3 最佳实践
- 添加必要的注释和文档
- 编写单元测试

### 提交信息规范
```
feat: 添加新功能
fix: 修复 bug
docs: 更新文档
style: 代码风格调整
refactor: 代码重构
test: 添加测试
chore: 构建工具或依赖更新
```

## 资源链接

- [Tauri 官方文档](https://tauri.app)
- [Vue 3 官方文档](https://vuejs.org)
- [Rust 官方文档](https://www.rust-lang.org)
- [Element Plus 文档](https://element-plus.org)
- [Pinia 文档](https://pinia.vuejs.org)

## 许可证

MIT License

## 联系方式

- 技术支持：support@example.com
- 问题反馈：https://github.com/example/sensitive-scanner/issues
