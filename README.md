
# 代码管理器 (Code Manager)

<p align="center">
  <img src="https://img.shields.io/badge/version-1.0.1-blue" alt="Version">
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux-green" alt="Platform">
  <img src="https://img.shields.io/badge/framework-Tauri 2.0-purple" alt="Framework">
</p>

一款专为开发者设计的项目管理工具，支持多 IDE 打开、Git 分支管理、NPM 脚本运行等功能。

## 功能特性

### 📁 项目管理

- 拖拽添加项目文件夹
- 项目搜索和筛选
- 项目分组管理

### 💻 多 IDE 支持

- VSCode
- Cursor
- WebStorm
- Trae

### 🔧 Git 集成

- 查看所有分支
- 快速切换分支
- 显示当前分支状态

### 📦 NPM 脚本

- 自动读取 package.json 脚本
- 一键运行脚本
- 自定义命令执行

### 🎯 产物打包

- 一键复制 dist 目录
- 自动打包成 zip
- 支持自定义命名规则（时间戳、分支名、环境名、版本号）

### 🎨 主题定制

- 8 种预设主题颜色
- 夜间模式
- Apple 风格滚动条

## 界面预览

![主界面](./docs/screenshot.png)

## 下载安装

### Windows

- [下载 MSI 安装包](./bundle/msi/CodeManager_1.0.1_x64_en-US.msi)
- [下载 NSIS 安装包](./bundle/nsis/CodeManager_1.0.1_x64-setup.exe)

### Linux

#### 依赖要求

在 Linux 上运行前，需要安装以下依赖：

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

# Fedora
sudo dnf install webkit2gtk4.1-devel libappindicator-3-devel librsvg2-devel

# Arch Linux
sudo pacman -S webkit2gtk-4.1 libappindicator librsvg2 patchelf
```

#### 运行方式

1. **下载 AppImage**（推荐）
   - 下载后添加执行权限：`chmod +x CodeManager_1.0.1_x86_64.AppImage`
   - 运行：`./CodeManager_1.0.1_x86_64.AppImage`

2. **或使用 AppImageLauncher**
   - 安装 AppImage 后会自动集成到系统应用菜单

#### 从源码构建

```bash
# 安装 Rust（如果未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装前端依赖
npm install

# 构建 Linux 版本
npm run tauri:build
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/appimage/` 目录下。

## 技术栈

- **前端**: Vue 3 + TypeScript + Ant Design Vue
- **桌面框架**: Tauri 2.0
- **后端**: Rust
- **数据库**: SQLite

## 快速开始

### 开发模式

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run tauri:dev
```

### 构建生产版本

```bash
# 构建 Windows 安装包
npm run tauri:build
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/` 目录下。

## 项目结构

```
code-manager/
├── src/                    # Vue 前端源码
│   ├── components/         # Vue 组件
│   ├── views/              # 页面视图
│   ├── stores/             # 状态管理
│   ├── types/              # TypeScript 类型
│   └── App.vue             # 根组件
├── src-tauri/              # Tauri 后端源码
│   ├── src/                # Rust 源码
│   └── capabilities/       # 权限配置
├── package.json            # 项目配置
└── README.md               # 项目文档
```

## 版本更新

### v1.0.1 (2026-03-17)

- 左侧菜单栏添加图标
- 左侧渐变改为从主题色向白色渐变
- 批量导入弹窗增加"绑定到该分组"开关
- 移除批量导入旁的"绑定到分组"按钮
- 支持微信开发者工具 IDE 打开
- 添加项目备注功能
- 添加错误日志记录功能
- 修复拖拽项目到分组时未正确绑定的问题
- 初步支持 Linux 系统

### v1.0.0 (2026-03-16)

- 初始版本发布
- 项目列表管理
- 多 IDE 支持
- Git 分支管理
- NPM 脚本运行
- 自定义脚本执行
- 产物打包功能
- 项目分组
- 自动刷新
- 主题定制与夜间模式

## License

MIT License
