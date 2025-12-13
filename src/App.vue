<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {Setting,Position,Delete,Plus,Edit} from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';

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
const dnsForm = ref({
  dns_servers: ''
});
const isAutoStartEnabled = ref(false);

const taskForm = ref({
  name: '',
  interface_pattern: '',
  target_dns: '',
  enabled: true
});

async function get_network_interfaces() {
  const all_interfaces = await invoke("get_all_network_interface");
  // 只显示启用的网卡
  network_interfaces.value = all_interfaces.filter(iface => iface.enabled);
}

function openDnsDialog(iface) {
  selectedInterface.value = iface;
  dnsForm.value = {
    dns_servers: (iface.dns_servers && iface.dns_servers.length > 0) ? iface.dns_servers.join(', ') : ''
  };
  showDnsDialog.value = true;
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

    const result = await invoke("set_dns_servers", {
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

    if (editingTaskId.value) {
      // 编辑模式
      const task = {
        id: editingTaskId.value,
        name: taskForm.value.name,
        interface_pattern: taskForm.value.interface_pattern,
        target_dns: dns_list,
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

async function handleRemoveTask(taskId) {
  try {
    console.log('Deleting task with id:', taskId);
    const result = await invoke("remove_dns_task", { taskId });
    console.log('Delete result:', result);
    ElMessage.success('任务删除成功');
    await loadTasks();
  } catch (error) {
    console.error('Delete error:', error);
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
    if(enabled){
      await disable()
      isAutoStartEnabled.value = false
      ElMessage.success('开机自启已禁用');
    }else {
      await enable()
      isAutoStartEnabled.value = true
      ElMessage.success('开机自启已启用');
    }
  } catch (error) {
    ElMessage.error(`设置开机自启失败: ${error}`);
  }
}


async function initializeApp() {
  try {
    // 首先初始化应用（加载数据库）
    await invoke("init_app");
    // 检查管理员状态
    await checkAdminStatus();
    // 然后加载数据
    await get_network_interfaces();
    await loadTasks();
    await checkMonitoringStatus();
    isAutoStartEnabled.value = await isEnabled()
  } catch (error) {
    console.error('Failed to initialize app:', error);
  }
}

initializeApp()

setInterval(() => {
  get_network_interfaces()
  loadTaskStatuses()
}, 1000)
</script>

<template>
  <main class="container">
    <!-- 任务管理面板 -->
    <el-card class="task-panel">
      <template #header>
        <div class="card-header">
          <span>DNS自动任务管理</span>
          <div>
            <el-button type="primary" :icon="Plus" @click="openTaskDialog">新增任务</el-button>
            <el-button :type="isMonitoring ? 'danger' : 'success'" @click="handleToggleMonitoring">
              {{ isMonitoring ? '停止监控' : '启动监控' }}
            </el-button>
            <el-button type="info" :icon="Setting" @click="showSettingsDialog = true">设置</el-button>
          </div>
        </div>
      </template>

      <el-table :data="tasks" stripe style="width: 100%">
        <el-table-column prop="name" label="任务名称" width="150" />
        <el-table-column prop="interface_pattern" label="网卡匹配规则" width="150" />
        <el-table-column prop="target_dns" label="目标DNS" width="200">
          <template #default="{ row }">
            {{ row.target_dns.join(', ') }}
          </template>
        </el-table-column>
        <el-table-column prop="enabled" label="启用" width="80">
          <template #default="{ row }">
            <el-switch v-model="row.enabled" @change="() => handleUpdateTask(row)" />
          </template>
        </el-table-column>
        <el-table-column label="操作" width="150">
          <template #default="{ row }">
            <el-button type="primary" :icon="Edit" size="small" @click="openEditTaskDialog(row)" />
            <el-button type="danger" :icon="Delete" size="small" @click="handleRemoveTask(row.id)" />
          </template>
        </el-table-column>
      </el-table>

      <el-divider />

      <div class="status-section">
        <el-text size="large" type="primary">任务执行状态</el-text>
        <el-table :data="taskStatuses" stripe style="width: 100%; margin-top: 10px;">
          <el-table-column prop="task_id" label="任务ID" width="150" />
          <el-table-column prop="interface_name" label="网卡名称" width="150" />
          <el-table-column prop="status" label="状态" width="120">
            <template #default="{ row }">
              <el-tag :type="row.status === 'matched' ? 'success' : row.status === 'applied' ? 'warning' : 'danger'">
                {{ row.status }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="current_dns" label="当前DNS" width="200">
            <template #default="{ row }">
              {{ row.current_dns.join(', ') }}
            </template>
          </el-table-column>
          <el-table-column prop="target_dns" label="目标DNS" width="200">
            <template #default="{ row }">
              {{ row.target_dns.join(', ') }}
            </template>
          </el-table-column>
        </el-table>
      </div>
    </el-card>

    <!-- 网卡信息面板 -->
    <el-card class="card" v-for="iface in network_interfaces" :key="iface.name">
      <div>
        <div class="header">
          <el-text class="mx-1" size="large" type="primary">网卡名称</el-text>
          <div>
            <el-dropdown trigger="click">
              <el-button type="primary" :icon="Setting" circle></el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item @click="openDnsDialog(iface)"><el-icon><Position /></el-icon>设置DNS</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
        </div>
        <el-scrollbar>
          <div class="scrollbar-flex-content">
            <el-text class="mx-1 scrollbar-demo-item" type="primary">{{iface.name?iface.name:"无"}}</el-text>
          </div>
        </el-scrollbar>
      </div>

      <div>
        <el-text class="mx-1" size="large" type="primary">ipv4</el-text>
        <el-scrollbar>
          <div v-if="iface.ipv4 && iface.ipv4.length > 0" class="scrollbar-flex-content">
            <el-text v-for="ip in iface.ipv4" :key="ip" class="mx-1 scrollbar-demo-item" type="primary">{{ ip }}</el-text>
          </div>
          <div v-else class="scrollbar-flex-content">
            <el-text class="mx-1 scrollbar-demo-item" type="primary">无</el-text>
          </div>
        </el-scrollbar>
      </div>
      <div>
        <el-text class="mx-1" size="large" type="primary">ipv6</el-text>
        <el-scrollbar>
          <div v-if="iface.ipv6 && iface.ipv6.length > 0" class="scrollbar-flex-content">
            <el-text v-for="ip in iface.ipv6" :key="ip" class="mx-1 scrollbar-demo-item" type="primary">{{ ip }}</el-text>
          </div>
          <div v-else class="scrollbar-flex-content">
            <el-text class="mx-1 scrollbar-demo-item" type="primary">无</el-text>
          </div>
        </el-scrollbar>
      </div>
      <div>
        <el-text class="mx-1" size="large" type="primary">dns</el-text>
        <el-scrollbar>
          <div v-if="iface.dns_servers && iface.dns_servers.length > 0" class="scrollbar-flex-content">
            <el-text v-for="dns in iface.dns_servers" :key="dns" class="mx-1 scrollbar-demo-item" type="primary">{{ dns }}</el-text>
          </div>
          <div v-else class="scrollbar-flex-content">
            <el-text class="mx-1 scrollbar-demo-item" type="primary">无</el-text>
          </div>
        </el-scrollbar>
      </div>
    </el-card>

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
        <el-form-item label="任务名称">
          <el-input v-model="taskForm.name" placeholder="例如: 公司DNS配置" clearable />
        </el-form-item>
        <el-form-item label="网卡匹配规则">
          <el-input 
            v-model="taskForm.interface_pattern" 
            placeholder="支持通配符，例如: eth*, wlan*, * (匹配所有)" 
            clearable 
          />
        </el-form-item>
        <el-form-item label="目标DNS">
          <el-input 
            v-model="taskForm.target_dns" 
            type="textarea" 
            rows="3"
            placeholder="输入DNS服务器地址，多个用逗号分隔&#10;例如: 8.8.8.8, 8.8.4.4" 
            clearable
          />
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

    <!-- 设置对话框 -->
    <el-dialog v-model="showSettingsDialog" title="应用设置" width="500px">
      <el-form label-width="150px">
        <el-form-item label="管理员状态">
          <el-text :type="isAdmin ? 'success' : 'danger'">
            {{ isAdmin ? '已获得管理员权限' : '未获得管理员权限' }}
          </el-text>
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
.header{
  display: flex;
  justify-content: space-between;
}
.card {
  text-align: left;
  max-width: 400px;
  min-width: 400px;
}
.task-panel {
  width: 100%;
  margin-bottom: 20px;
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}
.status-section {
  margin-top: 20px;
}
.scrollbar-flex-content {
  display: flex;
  width: fit-content;
}
.scrollbar-demo-item {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 35px;
  margin: 10px;
  text-align: center;
  border-radius: 4px;
  background: var(--el-color-danger-light-9);
  color: var(--el-color-danger);
}
</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 0;
  width: 100%;
  height: 100%;
  display: flex;
  flex-wrap: wrap; /* 关键属性：允许换行 */
  justify-content: space-between;
  gap: 5px;
}

</style>
