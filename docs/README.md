# 文档索引

本目录包含网络接口管理器的详细文档。

## 📚 文档列表

### 核心功能文档

#### [DNS 任务实现](DNS_TASK_IMPLEMENTATION.md)
- DNS 自动任务管理系统详解
- 任务创建、编辑、删除
- 后台监控和自动修复
- 通配符匹配规则
- 数据持久化

#### [跨平台支持](CROSS_PLATFORM_SUPPORT.md)
- Windows、Linux、macOS 实现细节
- 网络接口信息获取方式
- DNS 设置命令
- 平台特定的依赖和要求

### 系统集成文档

#### [系统托盘功能](TRAY_FUNCTIONALITY.md)
- 最小化到托盘
- 托盘菜单操作
- 后台运行支持
- 用户体验说明

#### [UAC 权限提升](UAC_ELEVATION.md)
- Windows 管理员权限自动提升
- 权限检查机制
- 用户体验流程

#### [管理员和开机自启](ADMIN_AND_AUTOSTART.md)
- 管理员权限检查
- 开机自启配置
- 跨平台实现

### 数据和存储文档

#### [数据库路径](DATABASE_PATH.md)
- SQLite 数据库位置
- 跨平台路径说明
- 数据迁移指南
- 备份建议

#### [任务持久化](PERSISTENCE_IMPROVEMENTS.md)
- 数据库结构
- 任务持久化实现
- 监控状态保存
- 数据恢复机制

### 稳定性和修复文档

#### [Panic 保护](PANIC_PROTECTION.md)
- 后台线程错误处理
- catch_unwind 保护机制
- 线程安全实现
- 测试建议

#### [应用崩溃修复](CRASH_FIX.md)
- 崩溃问题根本原因
- 解决方案详解
- 改进点总结
- 测试建议

## 🔍 快速查找

### 按功能查找

| 功能 | 文档 |
|------|------|
| DNS 自动任务 | [DNS_TASK_IMPLEMENTATION.md](DNS_TASK_IMPLEMENTATION.md) |
| 网络接口管理 | [CROSS_PLATFORM_SUPPORT.md](CROSS_PLATFORM_SUPPORT.md) |
| 系统托盘 | [TRAY_FUNCTIONALITY.md](TRAY_FUNCTIONALITY.md) |
| 权限管理 | [UAC_ELEVATION.md](UAC_ELEVATION.md)、[ADMIN_AND_AUTOSTART.md](ADMIN_AND_AUTOSTART.md) |
| 数据存储 | [DATABASE_PATH.md](DATABASE_PATH.md)、[PERSISTENCE_IMPROVEMENTS.md](PERSISTENCE_IMPROVEMENTS.md) |
| 稳定性 | [PANIC_PROTECTION.md](PANIC_PROTECTION.md)、[CRASH_FIX.md](CRASH_FIX.md) |

### 按平台查找

#### Windows
- [UAC 权限提升](UAC_ELEVATION.md)
- [跨平台支持 - Windows 部分](CROSS_PLATFORM_SUPPORT.md)
- [管理员和开机自启 - Windows 部分](ADMIN_AND_AUTOSTART.md)

#### Linux
- [跨平台支持 - Linux 部分](CROSS_PLATFORM_SUPPORT.md)
- [管理员和开机自启 - Linux 部分](ADMIN_AND_AUTOSTART.md)

#### macOS
- [跨平台支持 - macOS 部分](CROSS_PLATFORM_SUPPORT.md)
- [管理员和开机自启 - macOS 部分](ADMIN_AND_AUTOSTART.md)

## 🚀 新手指南

如果你是第一次使用本应用，建议按以下顺序阅读：

1. 主 README.md - 了解应用基本功能
2. [CROSS_PLATFORM_SUPPORT.md](CROSS_PLATFORM_SUPPORT.md) - 了解平台支持
3. [DNS_TASK_IMPLEMENTATION.md](DNS_TASK_IMPLEMENTATION.md) - 学习如何使用自动任务
4. [TRAY_FUNCTIONALITY.md](TRAY_FUNCTIONALITY.md) - 了解托盘功能
5. [DATABASE_PATH.md](DATABASE_PATH.md) - 了解数据存储位置

## 🔧 开发者指南

如果你想参与开发或修改代码，建议阅读：

1. [CROSS_PLATFORM_SUPPORT.md](CROSS_PLATFORM_SUPPORT.md) - 了解架构设计
2. [DNS_TASK_IMPLEMENTATION.md](DNS_TASK_IMPLEMENTATION.md) - 了解核心功能实现
3. [PANIC_PROTECTION.md](PANIC_PROTECTION.md) - 了解错误处理机制
4. [PERSISTENCE_IMPROVEMENTS.md](PERSISTENCE_IMPROVEMENTS.md) - 了解数据持久化
5. [CRASH_FIX.md](CRASH_FIX.md) - 了解稳定性改进

## 📝 文档维护

这些文档会随着应用的更新而更新。如果你发现文档中有错误或不清楚的地方，欢迎提交 Issue 或 Pull Request。

## 🔗 相关链接

- [主 README](../README.md)
- [GitHub 仓库](https://github.com/lhstack/network-interface-manager)
- [Issue 跟踪](https://github.com/lhstack/network-interface-manager/issues)
