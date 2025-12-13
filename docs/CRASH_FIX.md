# 应用崩溃问题修复

## 问题描述
当用户点击启用/关闭监控或修改任务时，应用会重启。

## 根本原因
后台监控线程中使用了 `.unwrap()` 方法，当Mutex锁被poisoned（中毒）时会导致panic，进而导致应用崩溃。

具体问题代码：
```rust
while *running_flag.lock().unwrap() {  // 这里会panic
    let tasks_list = tasks.lock().unwrap();  // 这里也会panic
    // ...
}
```

## 解决方案

### 1. 移除所有unwrap调用
将所有的 `.unwrap()` 替换为 `.map_err()` 或 `match` 语句，确保错误被正确处理。

### 2. 改进后台线程的错误处理
```rust
loop {
    // 检查是否应该继续运行
    let should_continue = match running_flag.lock() {
        Ok(flag) => *flag,
        Err(_) => break, // 如果锁被poisoned，退出线程
    };

    if !should_continue {
        break;
    }

    // 安全地获取任务列表
    let tasks_list = match tasks.lock() {
        Ok(list) => list.clone(),
        Err(_) => {
            // 如果锁被poisoned，跳过这次迭代
            thread::sleep(Duration::from_secs(1));
            continue;
        }
    };
    // ...
}
```

## 改进点

1. **安全的锁处理**：使用match语句而不是unwrap，避免panic
2. **优雅的错误恢复**：当锁被poisoned时，线程会跳过当前迭代而不是崩溃
3. **线程安全**：确保后台线程不会因为任何错误而导致应用崩溃

## 测试建议

1. 快速点击启用/关闭监控按钮
2. 在监控运行时修改任务
3. 在监控运行时删除任务
4. 观察应用是否保持稳定运行

## 相关文件修改
- `src-tauri/src/dns_task.rs`: 改进后台线程的错误处理
