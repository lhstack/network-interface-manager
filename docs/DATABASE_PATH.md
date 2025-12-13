# 数据库路径配置

## 概述

SQLite数据库现在存储在用户主目录下的 `.network interface manager` 目录中，而不是应用程序目录下的 `./data` 目录。

## 路径说明

### Windows
```
C:\Users\{username}\.network interface manager\tasks.db
```

### Linux
```
/home/{username}/.network interface manager/tasks.db
```

### macOS
```
/Users/{username}/.network interface manager/tasks.db
```

## 优势

1. **用户数据隔离**
   - 每个用户有自己的数据库
   - 多用户系统中数据不会混淆

2. **应用程序目录清洁**
   - 应用程序目录不会被数据文件污染
   - 便于应用程序的卸载和重新安装

3. **跨平台一致性**
   - 使用 `dirs` crate 获取用户主目录
   - 在所有平台上行为一致

4. **自动创建**
   - 如果目录不存在，会自动创建
   - 用户无需手动创建目录

## 实现细节

```rust
// 获取用户主目录
let home_dir = dirs::home_dir()
    .ok_or("Failed to get home directory")?;

// 创建.network interface manager目录
let data_dir = home_dir.join(".network interface manager");
if !data_dir.exists() {
    fs::create_dir_all(&data_dir)?;
}

// 数据库文件路径
let db_path = data_dir.join("tasks.db");
```

## 迁移说明

如果用户之前使用过旧版本，数据库文件位于 `./data/tasks.db`，需要手动迁移：

1. 找到旧的数据库文件：`./data/tasks.db`
2. 创建新目录：`~/.network interface manager/`
3. 将 `tasks.db` 复制到新目录

或者使用命令行：

### Windows
```powershell
Copy-Item -Path ".\data\tasks.db" -Destination "$env:USERPROFILE\.network interface manager\tasks.db"
```

### Linux/macOS
```bash
mkdir -p ~/.network interface manager
cp ./data/tasks.db ~/.network interface manager/tasks.db
```

## 依赖

- `dirs`: 跨平台目录获取库

## 注意事项

1. **权限**
   - 应用需要对用户主目录有写入权限
   - 通常这不是问题，因为用户对自己的主目录有完全权限

2. **隐藏目录**
   - `.network interface manager` 是隐藏目录（以点开头）
   - 在Linux/macOS上，需要使用 `ls -la` 或文件管理器的"显示隐藏文件"选项才能看到
   - 在Windows上，需要启用"显示隐藏文件"选项

3. **备份**
   - 用户可以轻松备份 `~/.network interface manager/tasks.db` 文件
   - 建议定期备份重要的任务配置

## 故障排查

### 无法创建目录
- 检查用户是否对主目录有写入权限
- 检查磁盘空间是否充足

### 找不到数据库文件
- 确保应用已经运行过一次（会自动创建目录和文件）
- 检查隐藏文件是否被显示

### 权限错误
- 确保应用以正确的用户身份运行
- 检查目录权限设置
