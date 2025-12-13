# 管理员权限和开机自启功能

## 功能概述

### 1. 管理员权限检查
应用启动时会检查是否具有管理员/root权限。如果没有，应用会退出。

**Windows**: 使用 `is-elevated` 库检查是否以管理员身份运行
**Linux/macOS**: 检查 `geteuid()` 是否为0（root用户）

### 2. 开机自启配置
用户可以在设置中启用/禁用开机自启功能。

**Windows**: 在启动文件夹中创建快捷方式
- 路径: `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup`

**Linux**: 创建 `.desktop` 文件
- 路径: `~/.config/autostart/network-interface-manager.desktop`

**macOS**: 创建 LaunchAgent plist文件
- 路径: `~/Library/LaunchAgents/com.network-interface-manager.plist`

### 3. 系统托盘支持
应用支持最小化到系统托盘，可以在后台运行。

## 使用方法

### 启用管理员权限
1. 在Windows上：右键点击应用 → 以管理员身份运行
2. 在Linux/macOS上：使用 `sudo` 运行应用

### 启用开机自启
1. 打开应用
2. 点击"设置"按钮
3. 启用"开机自启"开关
4. 应用会自动在系统启动时运行

### 最小化到系统托盘
- 点击窗口的最小化按钮
- 应用会最小化到系统托盘
- 点击系统托盘图标可恢复窗口

## 技术实现

### 依赖
- `is-elevated`: Windows管理员权限检查
- `dirs`: 跨平台目录获取
- `libc`: Unix系统调用

### 命令
- `is_admin()`: 检查是否为管理员
- `set_autostart(enabled: bool)`: 设置开机自启

### 前端
- 在设置对话框中显示管理员状态
- 提供开机自启开关

## 注意事项

1. **权限要求**
   - 修改DNS需要管理员/root权限
   - 设置开机自启可能需要管理员权限

2. **跨平台兼容性**
   - Windows: 使用PowerShell创建快捷方式
   - Linux: 使用标准 `.desktop` 文件格式
   - macOS: 使用 LaunchAgent plist格式

3. **安全性**
   - 应用启动时强制检查权限
   - 如果没有足够权限，应用会立即退出

## 故障排查

### 开机自启不工作
1. 检查应用是否以管理员身份运行
2. 检查启动文件夹中是否存在相应文件
3. 查看应用日志中的错误信息

### 无法获得管理员权限
1. 确保以管理员身份运行应用
2. 在Linux/macOS上使用 `sudo` 运行
3. 检查系统权限设置
