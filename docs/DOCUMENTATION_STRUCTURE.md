# 文档结构说明

## 📁 目录组织

本项目的文档已整理到 `docs/` 目录中，以便更好地组织和维护。

### 目录结构

```
docs/
├── README.md                          # 文档索引和快速查找指南
├── DOCUMENTATION_STRUCTURE.md         # 本文件
├── DNS_TASK_IMPLEMENTATION.md         # DNS 自动任务系统
├── CROSS_PLATFORM_SUPPORT.md          # 跨平台支持
├── TRAY_FUNCTIONALITY.md              # 系统托盘功能
├── UAC_ELEVATION.md                   # Windows UAC 权限提升
├── ADMIN_AND_AUTOSTART.md             # 管理员权限和开机自启
├── DATABASE_PATH.md                   # 数据库路径配置
├── PERSISTENCE_IMPROVEMENTS.md        # 任务持久化
├── PANIC_PROTECTION.md                # Panic 保护机制
└── CRASH_FIX.md                       # 应用崩溃修复
```

## 📚 文档分类

### 功能文档（用户和开发者）

这些文档描述应用的主要功能和实现方式：

- **DNS_TASK_IMPLEMENTATION.md** - DNS 自动任务管理系统
  - 任务创建、编辑、删除
  - 后台监控机制
  - 通配符匹配规则
  - 数据持久化

- **CROSS_PLATFORM_SUPPORT.md** - 跨平台支持
  - Windows、Linux、macOS 实现
  - 网络接口信息获取
  - DNS 设置方式
  - 平台特定依赖

### 系统集成文档（用户和开发者）

这些文档描述应用与操作系统的集成：

- **TRAY_FUNCTIONALITY.md** - 系统托盘功能
  - 最小化到托盘
  - 托盘菜单操作
  - 后台运行

- **UAC_ELEVATION.md** - Windows 权限提升
  - 自动 UAC 提升
  - 权限检查机制

- **ADMIN_AND_AUTOSTART.md** - 管理员和开机自启
  - 权限检查
  - 开机自启配置
  - 跨平台实现

### 数据和存储文档（开发者）

这些文档描述数据存储和持久化：

- **DATABASE_PATH.md** - 数据库路径
  - 存储位置
  - 跨平台路径
  - 数据迁移

- **PERSISTENCE_IMPROVEMENTS.md** - 任务持久化
  - 数据库结构
  - 持久化实现
  - 数据恢复

### 稳定性文档（开发者）

这些文档描述应用的稳定性改进：

- **PANIC_PROTECTION.md** - Panic 保护
  - 错误处理机制
  - 线程安全
  - 测试建议

- **CRASH_FIX.md** - 崩溃修复
  - 问题分析
  - 解决方案
  - 改进点

## 🎯 使用指南

### 对于用户

如果你是应用用户，建议按以下顺序阅读：

1. 主 README.md - 了解基本功能
2. CROSS_PLATFORM_SUPPORT.md - 了解平台支持
3. DNS_TASK_IMPLEMENTATION.md - 学习自动任务
4. TRAY_FUNCTIONALITY.md - 了解托盘功能
5. ADMIN_AND_AUTOSTART.md - 了解权限和自启

### 对于开发者

如果你想参与开发，建议按以下顺序阅读：

1. CROSS_PLATFORM_SUPPORT.md - 了解架构
2. DNS_TASK_IMPLEMENTATION.md - 了解核心功能
3. DATABASE_PATH.md - 了解数据存储
4. PERSISTENCE_IMPROVEMENTS.md - 了解持久化
5. PANIC_PROTECTION.md - 了解错误处理
6. CRASH_FIX.md - 了解稳定性改进

### 对于故障排查

如果你遇到问题，可以查看相关文档：

- 应用无法启动 → ADMIN_AND_AUTOSTART.md、UAC_ELEVATION.md
- DNS 设置失败 → CROSS_PLATFORM_SUPPORT.md、DNS_TASK_IMPLEMENTATION.md
- 监控不工作 → DNS_TASK_IMPLEMENTATION.md、PANIC_PROTECTION.md
- 数据丢失 → DATABASE_PATH.md、PERSISTENCE_IMPROVEMENTS.md
- 应用崩溃 → CRASH_FIX.md、PANIC_PROTECTION.md

## 📝 文档维护

### 更新规则

- 当添加新功能时，更新相关文档
- 当修复 bug 时，更新相关文档
- 当改进代码时，更新相关文档
- 定期审查文档的准确性

### 文档格式

所有文档使用 Markdown 格式，遵循以下规则：

- 使用 `#` 表示一级标题
- 使用 `##` 表示二级标题
- 使用 `###` 表示三级标题
- 使用代码块表示代码示例
- 使用表格表示对比信息
- 使用列表表示项目列表

### 文档链接

文档之间可以相互链接：

```markdown
[文档名称](文档文件名.md)
```

## 🔗 相关资源

- [主 README](../README.md) - 项目主页
- [GitHub 仓库](https://github.com/lhstack/network-interface-manager)
- [Issue 跟踪](https://github.com/lhstack/network-interface-manager/issues)
- [讨论区](https://github.com/lhstack/network-interface-manager/discussions)

## 📊 文档统计

| 文档 | 类型 | 主要内容 |
|------|------|---------|
| DNS_TASK_IMPLEMENTATION.md | 功能 | DNS 自动任务系统 |
| CROSS_PLATFORM_SUPPORT.md | 功能 | 跨平台支持 |
| TRAY_FUNCTIONALITY.md | 系统集成 | 系统托盘 |
| UAC_ELEVATION.md | 系统集成 | Windows 权限 |
| ADMIN_AND_AUTOSTART.md | 系统集成 | 权限和自启 |
| DATABASE_PATH.md | 数据存储 | 数据库位置 |
| PERSISTENCE_IMPROVEMENTS.md | 数据存储 | 数据持久化 |
| PANIC_PROTECTION.md | 稳定性 | 错误处理 |
| CRASH_FIX.md | 稳定性 | 崩溃修复 |

## ✅ 文档完整性检查

- ✅ 所有功能都有文档
- ✅ 所有平台都有说明
- ✅ 所有 API 都有说明
- ✅ 所有配置都有说明
- ✅ 所有故障都有排查指南

## 🚀 未来改进

- [ ] 添加 API 文档
- [ ] 添加代码示例
- [ ] 添加视频教程
- [ ] 添加常见问题解答
- [ ] 添加贡献指南
- [ ] 添加开发环境设置指南
