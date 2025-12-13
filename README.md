# 网络接口管理器 (Network Interface Manager)

一个跨平台的网络接口管理应用，支持 Windows、Linux 和 macOS。提供网络接口信息查看、DNS 配置和自动任务管理功能。

## ✨ 主要功能

### 🖥️ 网络接口管理
- 显示所有启用的网络接口信息
- 实时显示网卡名称、MAC 地址、IP 地址等信息
- 支持 IPv4 和 IPv6 地址显示
- 显示当前 DNS 配置

### 🔧 DNS 配置
- 手动设置网卡 DNS 服务器
- 支持多个 DNS 服务器配置
- 实时应用 DNS 设置
- 跨平台支持（Windows、Linux、macOS）

### 🤖 自动任务管理
- 创建 DNS 自动任务
- 支持网卡名称通配符匹配（如 `eth*`、`wlan*`、`*`）
- 后台自动监控和修复 DNS 配置
- 任务启用/禁用控制
- 实时任务执行状态显示

### 💾 数据持久化
- SQLite 数据库存储任务配置
- 应用重启后自动恢复任务和监控状态
- 数据存储在用户主目录（`~/.network interface manager/`）

### 🎯 系统集成
- 系统托盘支持（最小化到托盘）
- 开机自启配置
- 管理员权限自动提升（Windows）
- 后台运行支持

## 🚀 快速开始

### 系统要求

- **Windows**: Windows 7 或更高版本
- **Linux**: 任何现代 Linux 发行版
- **macOS**: macOS 10.13 或更高版本

### 权限要求

- **Windows**: 需要管理员权限
- **Linux**: 需要 root 权限（使用 `sudo`）
- **macOS**: 需要 sudo 权限

### 安装

1. 从 [Releases](https://github.com/lhstack/network-interface-manager/releases) 页面下载最新版本
2. 解压并运行应用

### 使用

#### 查看网络接口
1. 启动应用
2. 应用会自动显示所有启用的网络接口
3. 每张网卡显示为一个卡片，包含详细信息

#### 配置 DNS
1. 点击网卡卡片右上角的设置按钮
2. 选择"设置 DNS"
3. 输入 DNS 服务器地址（多个用逗号分隔）
4. 点击"确定"应用设置

#### 创建自动任务
1. 点击"新增任务"按钮
2. 填写任务信息：
   - 任务名称：任意名称
   - 网卡匹配规则：支持通配符（如 `eth*`、`wlan*`、`*`）
   - 目标 DNS：要设置的 DNS 服务器
3. 点击"确定"保存任务

#### 启动监控
1. 点击"启动监控"按钮
2. 系统开始后台扫描和自动修复 DNS
3. 在"任务执行状态"表格中查看实时状态

#### 最小化到托盘
1. 点击窗口关闭按钮
2. 应用最小化到系统托盘
3. 左键点击托盘图标切换显示/隐藏
4. 右键点击托盘图标显示菜单

#### 开机自启
1. 点击"设置"按钮
2. 启用"开机自启"开关
3. 应用会在系统启动时自动运行

## 📋 功能详解

### 通配符匹配规则

任务中的网卡匹配规则支持以下通配符：

| 规则 | 说明 | 示例 |
|------|------|------|
| `*` | 匹配所有网卡 | `*` |
| `eth*` | 匹配以 eth 开头的网卡 | `eth0`、`eth1` |
| `wlan*` | 匹配以 wlan 开头的网卡 | `wlan0`、`wlan1` |
| `eth0` | 精确匹配 eth0 | 仅 `eth0` |

### 任务状态说明

| 状态 | 说明 |
|------|------|
| matched | DNS 已匹配目标配置 |
| dns_mismatch | DNS 不匹配，需要修复 |
| applied | DNS 已自动应用 |

### 数据库位置

数据库文件存储在用户主目录下：

- **Windows**: `C:\Users\{username}\.network interface manager\tasks.db`
- **Linux**: `/home/{username}/.network interface manager/tasks.db`
- **macOS**: `/Users/{username}/.network interface manager/tasks.db`

## 🛠️ 开发

### 技术栈

- **前端**: Vue 3 + Element Plus + Vite
- **后端**: Rust + Tauri
- **数据库**: SQLite

### 项目结构

```
.
├── src/                          # 前端代码
│   ├── App.vue                   # 主应用组件
│   ├── main.js                   # 入口文件
│   └── assets/                   # 资源文件
├── src-tauri/                    # 后端代码
│   ├── src/
│   │   ├── lib.rs                # 主程序入口
│   │   ├── main.rs               # Tauri 入口
│   │   ├── network_info.rs       # 网络信息获取
│   │   ├── dns_task.rs           # DNS 任务管理
│   │   └── db.rs                 # 数据库操作
│   ├── Cargo.toml                # Rust 依赖
│   └── tauri.conf.json           # Tauri 配置
├── docs/                         # 文档
└── README.md                     # 本文件
```

### 构建

#### 前置要求

- Node.js 16+
- Rust 1.56+
- Tauri CLI

#### 开发模式

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm dev
```

#### 生产构建

```bash
# 构建应用
pnpm build

# 打包应用
pnpm tauri build
```

### 依赖

#### 前端
- vue@3
- element-plus
- @tauri-apps/api

#### 后端
- tauri
- serde
- rusqlite
- lazy_static
- chrono
- dirs
- is_elevated (Windows only)
- ipconfig (Windows only)

## 📚 文档

详细文档请查看 `docs/` 目录：

- [DNS 任务实现](docs/DNS_TASK_IMPLEMENTATION.md) - DNS 自动任务系统详解
- [跨平台支持](docs/CROSS_PLATFORM_SUPPORT.md) - 各平台实现细节
- [系统托盘功能](docs/TRAY_FUNCTIONALITY.md) - 托盘集成说明
- [UAC 权限提升](docs/UAC_ELEVATION.md) - Windows 权限管理
- [管理员和开机自启](docs/ADMIN_AND_AUTOSTART.md) - 权限和自启配置
- [数据库路径](docs/DATABASE_PATH.md) - 数据存储位置
- [任务持久化](docs/PERSISTENCE_IMPROVEMENTS.md) - 数据持久化实现
- [Panic 保护](docs/PANIC_PROTECTION.md) - 错误处理机制
- [应用崩溃修复](docs/CRASH_FIX.md) - 稳定性改进

## 🐛 故障排查

### 应用无法启动
- 检查是否有管理员权限
- 在 Linux/macOS 上使用 `sudo` 运行
- 查看应用日志

### DNS 设置失败
- 确保有管理员权限
- 检查网卡名称是否正确
- 检查 DNS 地址格式是否正确

### 监控不工作
- 确保监控已启动
- 检查任务是否启用
- 检查网卡匹配规则是否正确

### 数据丢失
- 检查数据库文件是否存在
- 确保应用有写入权限
- 查看应用日志中的错误信息

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

## 🙏 致谢

感谢以下开源项目的支持：

- [Tauri](https://tauri.app/) - 跨平台应用框架
- [Vue.js](https://vuejs.org/) - 前端框架
- [Element Plus](https://element-plus.org/) - UI 组件库
- [Rust](https://www.rust-lang.org/) - 系统编程语言

## 📞 联系方式

- GitHub Issues: [提交问题](https://github.com/lhstack/network-interface-manager/issues)
- 讨论: [GitHub Discussions](https://github.com/lhstack/network-interface-manager/discussions)

## 🔄 更新日志

### v0.1.0 (2024)
- ✨ 初始版本发布
- 🎯 网络接口管理功能
- 🔧 DNS 配置功能
- 🤖 自动任务管理系统
- 💾 SQLite 数据持久化
- 🎯 系统托盘集成
- 🚀 开机自启支持
- 🔐 管理员权限管理

---

**注意**: 此应用需要管理员权限才能修改网络配置。请确保以适当的权限运行应用。
