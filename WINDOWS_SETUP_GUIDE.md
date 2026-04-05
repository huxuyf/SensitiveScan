# SensitiveScan Windows 11 编译与执行指南

本指南将指导您在 Windows 11 环境下从零开始配置、编译并运行 `SensitiveScan` 项目。

## 1. 环境准备

在开始之前，请确保您的系统中已安装以下基础工具：

### 1.1 Rust 编译环境
1. 前往 [rust-lang.org](https://www.rust-lang.org/tools/install) 下载并运行 `rustup-init.exe`。
2. 在安装提示中，选择默认安装（选项 1）。
3. **关键步骤**: 安装程序会提示您安装 **Visual Studio C++ Build Tools**。请确保勾选了 "使用 C++ 的桌面开发" (Desktop development with C++) 工作负载。
4. 安装完成后，重启终端，运行 `rustc --version` 确认安装成功。

### 1.2 Node.js 环境
1. 建议安装 **Node.js 20.x 或更高版本**（从 [nodejs.org](https://nodejs.org/) 下载 LTS 版本）。
2. 项目推荐使用 `pnpm` 作为包管理器。安装 Node.js 后，在终端运行：
   ```powershell
   corepack enable
   corepack prepare pnpm@latest --activate
   ```

## 2. 项目配置与依赖安装

1. **克隆/进入项目目录**:
   打开 PowerShell 或 CMD，进入项目根目录 `SensitiveScan`。

2. **安装前端依赖**:
   ```powershell
   pnpm install
   ```

3. **预检查 Rust 依赖**:
   进入 `src-tauri` 目录（可选，用于预下载依赖）：
   ```powershell
   cd src-tauri
   cargo fetch
   cd ..
   ```

## 3. 开发模式运行

在项目根目录下执行以下命令，这将启动前端开发服务器并运行 Tauri 桌面窗口：

```powershell
pnpm tauri dev
```

> **注意**: 首次运行由于需要编译 Rust 源代码，速度会较慢，请耐心等待。后续运行将通过缓存大大加快速度。

## 4. 编译与打包 (生成安装程序)

如果您需要生成可分发的 `.exe` 安装包或绿色版程序：

```powershell
pnpm tauri build
```

编译完成后，您可以在以下目录找到生成的文件：
- **安装包**: `src-tauri\target\release\bundle\msi\*.msi`
- **可执行文件**: `src-tauri\target\release\sensitive-scanner.exe`

## 5. 常见问题排查

- **无法识别 `tauri` 命令**: 请确保您在项目根目录下运行 `pnpm tauri dev`，或者确保 `pnpm` 环境变量已正确配置。
- **Rust 编译错误**: 
  - 请检查是否安装了 **C++ 生成工具**。
  - 尝试清理缓存：`cd src-tauri; cargo clean; cd ..` 之后重新运行。
- **网络问题**: 如果 `pnpm install` 或 `cargo fetch` 极慢，请考虑配置国内镜像源。
  - npm/pnpm 镜像: `pnpm config set registry https://registry.npmmirror.com`
  - Cargo 镜像: 参考 [RustCC 镜像配置](https://rustcc.cn/article?id=83577d20-4357-4b71-9f9b-640a2335f60e)。

---

*祝您开发顺利！如有其他问题，请随时联系。*
