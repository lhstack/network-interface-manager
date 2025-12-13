# 网络接口管理器 - 跨平台支持

## 平台支持

### Windows ✅
- 使用 `ipconfig` 库获取网络接口信息
- 支持所有网卡类型
- 使用 `netsh` 命令设置 DNS

### Linux ✅
- 使用 `ip` 命令获取网络接口信息
- 从 `/proc/net/dev` 读取网络统计
- 从 `/etc/resolv.conf` 读取 DNS 配置
- 使用 `sudo` 修改 DNS 设置

### macOS ✅
- 使用 `ifconfig` 命令获取网络接口信息
- 使用 `scutil` 命令读取 DNS 配置
- 使用 `networksetup` 命令设置 DNS

## 架构设计

### 模块结构
- `lib.rs`：主程序入口，统一的 Tauri 命令接口
- `network_info.rs`：跨平台网络信息获取模块

### 依赖管理
- `ipconfig`：仅在 Windows 上编译
- 其他平台使用系统命令（`ip`、`ifconfig` 等）

## 获取网络接口信息

### Windows
```rust
use ipconfig;
let adapters = ipconfig::get_adapters()?;
```

### Linux
```bash
ip link show      # 获取网卡列表
ip addr show      # 获取 IP 地址
cat /etc/resolv.conf  # 获取 DNS
```

### macOS
```bash
ifconfig          # 获取网卡信息
scutil -r State:/Network/Global/DNS  # 获取 DNS
```

## DNS 设置

### Windows
```bash
netsh interface ip set dns "interface_name" static dns_server
netsh interface ip add dns "interface_name" dns_server
```

### Linux
```bash
echo 'nameserver dns_server' | sudo tee /etc/resolv.conf
```

### macOS
```bash
sudo networksetup -setdnsservers interface_name dns_server1 dns_server2
```

## 网络接口信息字段

所有平台返回的 `NetworkInterface` 结构体包含：
- `name`：网卡名称
- `description`：网卡描述（仅 Windows）
- `mac_address`：MAC 地址
- `ipv4`：IPv4 地址列表
- `ipv6`：IPv6 地址列表
- `dns_servers`：DNS 服务器列表
- `enabled`：是否启用
- `if_type`：接口类型（仅 Windows）
- `gateways`：网关列表（仅 Windows）
- `guid`：网卡 GUID（仅 Windows）
- `mask`：子网掩码（仅 Windows）
- `receive_link_speed`：接收链接速度（仅 Windows）
- `transmit_link_speed`：发送链接速度（仅 Windows）

## 编译配置

### Cargo.toml
```toml
[target.'cfg(windows)'.dependencies]
ipconfig = "*"
```

这确保 `ipconfig` 库只在 Windows 平台编译。

## 使用示例

### 获取所有网卡信息
```javascript
const interfaces = await invoke("get_all_network_interface");
```

### 设置 DNS
```javascript
await invoke("set_dns_servers", {
  config: {
    interface_name: "eth0",
    dns_servers: ["8.8.8.8", "8.8.4.4"]
  }
});
```

## 注意事项

1. **权限要求**
   - Windows：需要管理员权限
   - Linux：需要 `sudo` 权限
   - macOS：需要 `sudo` 权限

2. **系统命令依赖**
   - Linux：需要安装 `iproute2` 包（通常预装）
   - macOS：需要 `scutil` 和 `networksetup`（系统自带）

3. **网络中断**
   - 修改网络配置可能导致网络临时中断

4. **DNS 持久化**
   - Linux：修改 `/etc/resolv.conf` 可能在重启后丢失
   - 建议使用 NetworkManager 或 systemd-resolved 进行持久化配置

## 故障排除

### Linux 上无法获取网卡信息
- 检查 `ip` 命令是否可用
- 确保 `iproute2` 包已安装

### macOS 上无法设置 DNS
- 检查是否有 `sudo` 权限
- 确保网卡名称正确

### Windows 上无法设置 DNS
- 检查是否以管理员身份运行
- 检查网卡名称是否正确
