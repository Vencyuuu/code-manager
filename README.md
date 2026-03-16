
# 代码管理器 (Code Manager)

<p align="center">
  <img src="https://img.shields.io/badge/version-1.0.0-blue" alt="Version">
  <img src="https://img.shields.io/badge/platform-Windows-green" alt="Platform">
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

- [下载 MSI 安装包](./bundle/msi/CodeManager_1.0.0_x64_en-US.msi)
- [下载 NSIS 安装包](./bundle/nsis/CodeManager_1.0.0_x64-setup.exe)

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
