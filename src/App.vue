<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Setting, Position, Delete, Plus, Edit, Document, Refresh } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';

// 当前激活的标签页
const activeTab = ref('tasks')

const network_interfaces = ref([]);
const showDnsDialog = ref(false);
const selectedInterface = ref(null);
const showTaskDialog = ref(false);
const tasks = ref([]);
const taskStatuses = ref([]);
const isMonitoring = ref(false);
const editingTaskId = ref(null);
const showSettingsDialog = ref(false);
const isAdmin = ref(false);
const logs = ref([]);
const dnsForm = ref({
  dns_servers: ''
});
const isAutoStartEnabled = ref(false);

// 网络配置对话框
const showNetworkConfigDialog = ref(false);
const networkConfigForm = ref({
  interface_name: '',
  dhcp: true,
  ip_address: '',
  subnet_mask: '255.255.255.0',
  gateway: '',
  dns: ''
});

const taskForm = ref({
  name: '',
  interface_pattern: '',
  target_dns: '',
  interval: 1,
  enabled: true
});

async function get_network_interfaces() {
  try {
    const all_interfaces = await invoke("get_all_network_interface");
    // 只显示启用的网卡
    network_interfaces.value = all_interfaces.filter(iface => iface.enabled);
  } catch (error) {
    console.error('Failed to get interfaces:', error);
  }
}

function openDnsDialog(iface) {
  selectedInterface.value = iface;
  dnsForm.value = {
    dns_servers: (iface.dns_servers && iface.dns_servers.length > 0) ? iface.dns_servers.join(', ') : ''
  };
  showDnsDialog.value = true;
}

function openNetworkConfigDialog(iface) {
  selectedInterface.value = iface;
  networkConfigForm.value = {
    interface_name: iface.name,
    dhcp: iface.dhcp || false,
    ip_address: iface.ipv4?.[0] || '',
    subnet_mask: iface.subnet_mask || '255.255.255.0',
    gateway: iface.gateways?.[0] || '',
    dns: iface.dns_servers?.join(', ') || ''
  };
  showNetworkConfigDialog.value = true;
}

async function handleSetNetworkConfig() {
  try {
    const config = {
      interface_name: networkConfigForm.value.interface_name,
      dhcp: networkConfigForm.value.dhcp,
      ip_address: networkConfigForm.value.ip_address,
      subnet_mask: networkConfigForm.value.subnet_mask,
      gateway: networkConfigForm.value.gateway,
      dns: networkConfigForm.value.dns ? networkConfigForm.value.dns.split(',').map(d => d.trim()).filter(d => d) : []
    };

    // 静态IP模式下验证必填项
    if (!config.dhcp) {
      if (!config.ip_address || !config.subnet_mask) {
        ElMessage.warning('静态IP模式需要填写IP地址和子网掩码');
        return;
      }
    }

    await invoke("set_network_config", { config });
    ElMessage.success('网络配置成功');
    showNetworkConfigDialog.value = false;
    setTimeout(() => get_network_interfaces(), 1000);
  } catch (error) {
    ElMessage.error(`网络配置失败: ${error}`);
  }
}

async function handleSetDns() {
  if (!dnsForm.value.dns_servers) {
    ElMessage.warning('请输入DNS服务器地址');
    return;
  }

  try {
    const dns_list = dnsForm.value.dns_servers.split(',').map(d => d.trim()).filter(d => d);
    if (dns_list.length === 0) {
      ElMessage.warning('请输入有效的DNS服务器地址');
      return;
    }

    await invoke("set_dns_servers", {
      config: {
        interface_name: selectedInterface.value.name,
        dns_servers: dns_list
      }
    });

    ElMessage.success('DNS配置成功');
    showDnsDialog.value = false;
    setTimeout(() => get_network_interfaces(), 1000);
  } catch (error) {
    ElMessage.error(`DNS配置失败: ${error}`);
  }
}

function openTaskDialog() {
  editingTaskId.value = null;
  taskForm.value = {
    name: '',
    interface_pattern: '',
    target_dns: '',
    interval: 1,
    enabled: true
  };
  showTaskDialog.value = true;
}

function openEditTaskDialog(task) {
  editingTaskId.value = task.id;
  taskForm.value = {
    name: task.name,
    interface_pattern: task.interface_pattern,
    target_dns: task.target_dns.join(', '),
    interval: task.interval || 1,
    enabled: task.enabled
  };
  showTaskDialog.value = true;
}

async function handleAddTask() {
  if (!taskForm.value.name || !taskForm.value.interface_pattern || !taskForm.value.target_dns) {
    ElMessage.warning('请填写所有必填项');
    return;
  }

  try {
    const dns_list = taskForm.value.target_dns.split(',').map(d => d.trim()).filter(d => d);
    if (dns_list.length === 0) {
      ElMessage.warning('请输入有效的DNS服务器地址');
      return;
    }

    const interval = Math.max(1, parseInt(taskForm.value.interval) || 1);

    if (editingTaskId.value) {
      // 编辑模式
      const task = {
        id: editingTaskId.value,
        name: taskForm.value.name,
        interface_pattern: taskForm.value.interface_pattern,
        target_dns: dns_list,
        interval: interval,
        enabled: taskForm.value.enabled,
        created_at: Math.floor(Date.now() / 1000)
      };
      await invoke("update_dns_task", { task });
      ElMessage.success('任务更新成功');
    } else {
      // 新增模式
      const task = {
        id: Date.now().toString(),
        name: taskForm.value.name,
        interface_pattern: taskForm.value.interface_pattern,
        target_dns: dns_list,
        interval: interval,
        enabled: taskForm.value.enabled,
        created_at: Math.floor(Date.now() / 1000)
      };
      await invoke("add_dns_task", { task });
      ElMessage.success('任务添加成功');
    }
    showTaskDialog.value = false;
    await loadTasks();
  } catch (error) {
    ElMessage.error(`操作失败: ${error}`);
  }
}

async function loadTasks() {
  try {
    tasks.value = await invoke("get_dns_tasks");
  } catch (error) {
    console.error('Failed to load tasks:', error);
  }
}

async function loadTaskStatuses() {
  try {
    taskStatuses.value = await invoke("get_task_statuses");
  } catch (error) {
    console.error('Failed to load task statuses:', error);
  }
}

async function loadLogs() {
  try {
    logs.value = await invoke("get_logs");
  } catch (error) {
    console.error('Failed to load logs:', error);
  }
}

async function clearLogs() {
  try {
    await invoke("clear_logs");
    logs.value = [];
    ElMessage.success('日志已清空');
  } catch (error) {
    ElMessage.error(`清空日志失败: ${error}`);
  }
}

async function handleRemoveTask(taskId) {
  try {
    await invoke("remove_dns_task", { taskId });
    ElMessage.success('任务删除成功');
    await loadTasks();
  } catch (error) {
    ElMessage.error(`任务删除失败: ${error}`);
  }
}

async function handleToggleMonitoring() {
  try {
    if (isMonitoring.value) {
      await invoke("stop_dns_monitoring");
      isMonitoring.value = false;
      ElMessage.success('监控已停止');
    } else {
      await invoke("start_dns_monitoring");
      isMonitoring.value = true;
      ElMessage.success('监控已启动');
    }
  } catch (error) {
    ElMessage.error(`操作失败: ${error}`);
  }
}

async function handleUpdateTask(task) {
  try {
    await invoke("update_dns_task", { task });
    ElMessage.success('任务更新成功');
  } catch (error) {
    ElMessage.error(`任务更新失败: ${error}`);
  }
}

async function checkMonitoringStatus() {
  try {
    isMonitoring.value = await invoke("is_dns_monitoring_running");
  } catch (error) {
    console.error('Failed to check monitoring status:', error);
  }
}

async function checkAdminStatus() {
  try {
    isAdmin.value = await invoke("is_admin");
  } catch (error) {
    console.error('Failed to check admin status:', error);
  }
}

async function handleAutostartChange() {
  try {
    let enabled = await isEnabled()
    if (enabled) {
      await disable()
      isAutoStartEnabled.value = false
      ElMessage.success('开机自启已禁用');
    } else {
      await enable()
      isAutoStartEnabled.value = true
      ElMessage.success('开机自启已启用');
    }
  } catch (error) {
    ElMessage.error(`设置开机自启失败: ${error}`);
  }
}

function getStatusType(status) {
  switch (status) {
    case 'matched': return 'success';
    case 'applied': return 'warning';
    case 'running': return 'primary';
    case 'stopped': return 'info';
    default: return 'danger';
  }
}

function getStatusText(status) {
  switch (status) {
    case 'matched': return '已匹配';
    case 'applied': return '已应用';
    case 'running': return '运行中';
    case 'stopped': return '已停止';
    case 'dns_mismatch': return '不匹配';
    default: return status;
  }
}

async function initializeApp() {
  try {
    await invoke("init_app");
    await checkAdminStatus();
    await get_network_interfaces();
    await loadTasks();
    await checkMonitoringStatus();
    await loadLogs();
    isAutoStartEnabled.value = await isEnabled()
  } catch (error) {
    console.error('Failed to initialize app:', error);
  }
}

initializeApp()

setInterval(() => {
  get_network_interfaces()
  loadTaskStatuses()
  loadLogs()
}, 500)
</script>

<template>
  <main class="app-container">
    <!-- 顶部标签栏 -->
    <el-tabs v-model="activeTab" class="app-tabs">
      <el-tab-pane label="DNS任务" name="tasks">
        <!-- 任务管理面板 -->
        <el-card class="panel-card">
          <template #header>
            <div class="card-header">
              <span>DNS自动任务管理</span>
              <div class="header-actions">
                <el-button type="primary" :icon="Plus" @click="openTaskDialog">新增任务</el-button>
                <el-button :type="isMonitoring ? 'danger' : 'success'" @click="handleToggleMonitoring">
                  {{ isMonitoring ? '停止监控' : '启动监控' }}
                </el-button>
                <el-button type="info" :icon="Setting" @click="showSettingsDialog = true">设置</el-button>
              </div>
            </div>
          </template>

          <el-table :data="tasks" stripe style="width: 100%">
            <el-table-column prop="name" label="任务名称" min-width="120" />
            <el-table-column prop="interface_pattern" label="网卡匹配" min-width="120" />
            <el-table-column label="目标DNS" min-width="150">
              <template #default="{ row }">
                {{ row.target_dns.join(', ') }}
              </template>
            </el-table-column>
            <el-table-column prop="interval" label="间隔(秒)" width="90" />
            <el-table-column label="启用" width="70">
              <template #default="{ row }">
                <el-switch v-model="row.enabled" size="small" @change="() => handleUpdateTask(row)" />
              </template>
            </el-table-column>
            <el-table-column label="操作" width="120">
              <template #default="{ row }">
                <el-button type="primary" :icon="Edit" size="small" circle @click="openEditTaskDialog(row)" />
                <el-button type="danger" :icon="Delete" size="small" circle @click="handleRemoveTask(row.id)" />
              </template>
            </el-table-column>
          </el-table>

          <el-divider />

          <div class="status-section">
            <el-text size="large" type="primary">任务执行状态</el-text>
            <el-table :data="taskStatuses" stripe style="width: 100%; margin-top: 10px;" max-height="300">
              <el-table-column prop="task_name" label="任务" min-width="100" />
              <el-table-column prop="interface_name" label="网卡" min-width="120" />
              <el-table-column label="状态" width="90">
                <template #default="{ row }">
                  <el-tag :type="getStatusType(row.status)" size="small">
                    {{ getStatusText(row.status) }}
                  </el-tag>
                </template>
              </el-table-column>
              <el-table-column label="当前DNS" min-width="140">
                <template #default="{ row }">
                  {{ row.current_dns?.join(', ') || '-' }}
                </template>
              </el-table-column>
              <el-table-column prop="message" label="消息" min-width="150" />
              <el-table-column prop="last_check" label="检查时间" width="160" />
            </el-table>
          </div>
        </el-card>
      </el-tab-pane>

      <el-tab-pane label="网卡列表" name="interfaces">
        <el-card class="panel-card">
          <template #header>
            <div class="card-header">
              <span>网卡信息</span>
              <el-button :icon="Refresh" @click="get_network_interfaces">刷新</el-button>
            </div>
          </template>
          
          <div class="interface-grid">
            <el-card v-for="iface in network_interfaces" :key="iface.name" class="interface-card" shadow="hover">
              <template #header>
                <div class="iface-header">
                  <div class="iface-title">
                    <span class="iface-name">{{ iface.name }}</span>
                    <el-tag :type="iface.dhcp ? 'success' : 'warning'" size="small" style="margin-left: 8px;">
                      {{ iface.dhcp ? 'DHCP' : '静态' }}
                    </el-tag>
                  </div>
                  <el-dropdown trigger="click">
                    <el-button type="primary" :icon="Setting" size="small" circle />
                    <template #dropdown>
                      <el-dropdown-menu>
                        <el-dropdown-item @click="openNetworkConfigDialog(iface)">
                          <el-icon><Setting /></el-icon>网络配置
                        </el-dropdown-item>
                        <el-dropdown-item @click="openDnsDialog(iface)">
                          <el-icon><Position /></el-icon>设置DNS
                        </el-dropdown-item>
                      </el-dropdown-menu>
                    </template>
                  </el-dropdown>
                </div>
              </template>
              
              <div class="iface-info">
                <div class="info-row">
                  <span class="label">IPv4:</span>
                  <span class="value">{{ iface.ipv4?.join(', ') || '无' }}</span>
                </div>
                <div class="info-row">
                  <span class="label">掩码:</span>
                  <span class="value">{{ iface.subnet_mask || '无' }}</span>
                </div>
                <div class="info-row">
                  <span class="label">网关:</span>
                  <span class="value">{{ iface.gateways?.join(', ') || '无' }}</span>
                </div>
                <div class="info-row">
                  <span class="label">DNS:</span>
                  <span class="value">{{ iface.dns_servers?.join(', ') || '无' }}</span>
                </div>
              </div>
            </el-card>
          </div>
        </el-card>
      </el-tab-pane>

      <el-tab-pane label="日志" name="logs">
        <el-card class="panel-card">
          <template #header>
            <div class="card-header">
              <span>执行日志</span>
              <el-button type="danger" :icon="Delete" @click="clearLogs">清空日志</el-button>
            </div>
          </template>
          
          <el-table :data="logs" stripe style="width: 100%" max-height="500">
            <el-table-column prop="time" label="时间" width="180" />
            <el-table-column prop="task_name" label="任务" width="150" />
            <el-table-column prop="message" label="消息" min-width="300" />
          </el-table>
          
          <div v-if="logs.length === 0" class="empty-logs">
            <el-empty description="暂无日志" />
          </div>
        </el-card>
      </el-tab-pane>
    </el-tabs>

    <!-- DNS配置对话框 -->
    <el-dialog v-model="showDnsDialog" title="设置DNS服务器" width="500px">
      <el-form :model="dnsForm" label-width="120px">
        <el-form-item label="DNS服务器">
          <el-input 
            v-model="dnsForm.dns_servers" 
            type="textarea" 
            rows="4"
            placeholder="输入DNS服务器地址，多个用逗号分隔&#10;例如: 8.8.8.8, 8.8.4.4" 
            clearable
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showDnsDialog = false">取消</el-button>
        <el-button type="primary" @click="handleSetDns">确定</el-button>
      </template>
    </el-dialog>

    <!-- 新增/编辑任务对话框 -->
    <el-dialog v-model="showTaskDialog" :title="editingTaskId ? '编辑DNS自动任务' : '新增DNS自动任务'" width="600px">
      <el-form :model="taskForm" label-width="150px">
        <el-form-item label="任务名称" required>
          <el-input v-model="taskForm.name" placeholder="例如: 公司DNS配置" clearable />
        </el-form-item>
        <el-form-item label="网卡匹配规则" required>
          <el-input 
            v-model="taskForm.interface_pattern" 
            placeholder="支持通配符，例如: eth*, wlan*, * (匹配所有)" 
            clearable 
          />
          <div class="form-tip">支持通配符: * 匹配任意字符</div>
        </el-form-item>
        <el-form-item label="目标DNS" required>
          <el-input 
            v-model="taskForm.target_dns" 
            type="textarea" 
            rows="2"
            placeholder="输入DNS服务器地址，多个用逗号分隔&#10;例如: 8.8.8.8, 8.8.4.4" 
            clearable
          />
        </el-form-item>
        <el-form-item label="检查间隔(秒)">
          <el-input-number v-model="taskForm.interval" :min="1" :max="3600" />
          <div class="form-tip">DNS检查和修复的时间间隔，最小1秒</div>
        </el-form-item>
        <el-form-item label="启用">
          <el-switch v-model="taskForm.enabled" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showTaskDialog = false">取消</el-button>
        <el-button type="primary" @click="handleAddTask">{{ editingTaskId ? '更新' : '确定' }}</el-button>
      </template>
    </el-dialog>

    <!-- 网络配置对话框 -->
    <el-dialog v-model="showNetworkConfigDialog" title="网络配置" width="550px">
      <el-form :model="networkConfigForm" label-width="100px">
        <el-form-item label="网卡">
          <el-input :value="networkConfigForm.interface_name" disabled />
        </el-form-item>
        <el-form-item label="配置方式">
          <el-radio-group v-model="networkConfigForm.dhcp">
            <el-radio :value="true">自动获取(DHCP)</el-radio>
            <el-radio :value="false">手动配置(静态IP)</el-radio>
          </el-radio-group>
        </el-form-item>
        
        <template v-if="!networkConfigForm.dhcp">
          <el-form-item label="IP地址" required>
            <el-input v-model="networkConfigForm.ip_address" placeholder="例如: 192.168.1.100" clearable />
          </el-form-item>
          <el-form-item label="子网掩码" required>
            <el-input v-model="networkConfigForm.subnet_mask" placeholder="例如: 255.255.255.0" clearable />
          </el-form-item>
          <el-form-item label="默认网关">
            <el-input v-model="networkConfigForm.gateway" placeholder="例如: 192.168.1.1" clearable />
          </el-form-item>
        </template>
        
        <el-form-item label="DNS服务器">
          <el-input 
            v-model="networkConfigForm.dns" 
            type="textarea" 
            rows="2"
            placeholder="输入DNS服务器地址，多个用逗号分隔&#10;留空则自动获取(DHCP模式)" 
            clearable
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showNetworkConfigDialog = false">取消</el-button>
        <el-button type="primary" @click="handleSetNetworkConfig">应用</el-button>
      </template>
    </el-dialog>

    <!-- 设置对话框 -->
    <el-dialog v-model="showSettingsDialog" title="应用设置" width="500px">
      <el-form label-width="150px">
        <el-form-item label="管理员状态">
          <el-tag :type="isAdmin ? 'success' : 'danger'">
            {{ isAdmin ? '已获得管理员权限' : '未获得管理员权限' }}
          </el-tag>
        </el-form-item>
        <el-form-item label="监控状态">
          <el-tag :type="isMonitoring ? 'success' : 'info'">
            {{ isMonitoring ? '监控运行中' : '监控已停止' }}
          </el-tag>
        </el-form-item>
        <el-form-item label="开机自启">
          <el-switch v-model="isAutoStartEnabled" @change="handleAutostartChange" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button type="primary" @click="showSettingsDialog = false">关闭</el-button>
      </template>
    </el-dialog>
  </main>
</template>

<style scoped>
.app-container {
  padding: 10px;
  height: 100vh;
  box-sizing: border-box;
}

.app-tabs {
  height: 100%;
}

.panel-card {
  height: calc(100vh - 80px);
  overflow: auto;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.status-section {
  margin-top: 20px;
}

.interface-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

.interface-card {
  min-width: 320px;
}

.iface-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.iface-title {
  display: flex;
  align-items: center;
}

.iface-name {
  font-weight: bold;
  font-size: 14px;
}

.iface-info {
  font-size: 13px;
}

.info-row {
  display: flex;
  margin-bottom: 8px;
}

.info-row .label {
  width: 50px;
  color: #909399;
  flex-shrink: 0;
}

.info-row .value {
  color: #303133;
  word-break: break-all;
}

.form-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.empty-logs {
  padding: 40px 0;
}
</style>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 14px;
  line-height: 1.5;
  font-weight: 400;
  color: #0f0f0f;
  background-color: #f6f6f6;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  margin: 0;
  padding: 0;
}
</style>
