<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {Setting,Location,Position} from '@element-plus/icons-vue'
const network_interfaces = ref([]);
const alert = async (msg) => {
  window.alert(msg)
}
async function get_network_interfaces() {
  network_interfaces.value = await invoke("get_all_network_interface");
}
get_network_interfaces()
setInterval(() => {
  get_network_interfaces()
},1000)
</script>

<template>
  <main class="container">
    <el-card class="card"  v-for="iface in network_interfaces">
      <div>
        <div class="header">
          <el-text class="mx-1" size="large" type="primary">网卡名称</el-text>
          <div>
            <el-dropdown trigger="click">
              <el-button type="primary" :icon="Setting" circle></el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item @click="alert('设置ip: ' + iface.name)"><el-icon><Location /></el-icon>设置ip</el-dropdown-item>
                  <el-dropdown-item @click="alert('设置dns: ' + iface.name)"><el-icon><Position /></el-icon>设置dns</el-dropdown-item>
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
      <div style="display: flex;justify-content: space-between">
        <div>
          <el-text class="mx-1" size="large" type="primary">状态</el-text>
          <div>
            <el-switch
                v-model="iface.enabled"
                size="small"
                class="ml-1"
                style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949"
            />
          </div>
        </div>
        <div>
          <el-text class="mx-1" size="large" type="primary">接口类型</el-text>
          <div>
            <el-text class="mx-1" size="large">{{iface.if_type}}</el-text>
          </div>
        </div>
      </div>

      <div>
        <el-text class="mx-1" size="large" type="primary">网卡描述</el-text>
        <el-scrollbar>
          <div class="scrollbar-flex-content">
            <el-text class="mx-1 scrollbar-demo-item" type="primary">{{ iface.description?iface.description:"无" }}</el-text>
          </div>
        </el-scrollbar>
      </div>
<!--      <div>-->
<!--        <el-text class="mx-1" size="large" type="primary">guid</el-text>-->
<!--        <el-scrollbar>-->
<!--          <div class="scrollbar-flex-content">-->
<!--            <el-text class="mx-1 scrollbar-demo-item" type="primary">{{ iface.guid?iface.guid:"无" }}</el-text>-->
<!--          </div>-->
<!--        </el-scrollbar>-->
<!--      </div>-->
<!--      <div>-->
<!--        <el-text class="mx-1" size="large" type="primary">mac地址</el-text>-->
<!--        <el-scrollbar>-->
<!--          <div class="scrollbar-flex-content">-->
<!--            <el-text class="mx-1 scrollbar-demo-item" type="primary">{{iface.mac_address?iface.mac_address:"无"}}</el-text>-->
<!--          </div>-->
<!--        </el-scrollbar>-->
<!--      </div>-->
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
            <el-text v-for="dns in iface.dns_servers" :key="ip" class="mx-1 scrollbar-demo-item" type="primary">{{ dns }}</el-text>
          </div>
          <div v-else class="scrollbar-flex-content">
            <el-text class="mx-1 scrollbar-demo-item" type="primary">无</el-text>
          </div>
        </el-scrollbar>
      </div>

      <div>
        <el-text class="mx-1" size="large" type="primary">网关</el-text>
        <el-scrollbar>
          <div v-if="iface.gateways && iface.gateways.length > 0" class="scrollbar-flex-content">
            <el-text v-for="gateway in iface.gateways" :key="gateway" class="mx-1 scrollbar-demo-item" type="primary">{{ gateway }}</el-text>
          </div>
          <div v-else class="scrollbar-flex-content">
            <el-text class="mx-1 scrollbar-demo-item" type="primary">无</el-text>
          </div>
        </el-scrollbar>
      </div>

      <div>
        <el-text class="mx-1" size="large" type="primary">子网掩码</el-text>
        <el-scrollbar>
          <div v-if="iface.mask && iface.mask.length > 0" class="scrollbar-flex-content">
            <el-text v-for="msk in iface.mask" :key="msk" class="mx-1 scrollbar-demo-item" type="primary">{{ msk }}</el-text>
          </div>
          <div v-else class="scrollbar-flex-content">
            <el-text class="mx-1 scrollbar-demo-item" type="primary">无</el-text>
          </div>
        </el-scrollbar>
      </div>
<!--      <div>-->
<!--        <el-text class="mx-1" size="large" type="primary">接口类型</el-text>-->
<!--        <el-scrollbar>-->
<!--          <div class="scrollbar-flex-content">-->
<!--            <el-text class="mx-1 scrollbar-demo-item" type="primary">{{ iface.if_type }}</el-text>-->
<!--          </div>-->
<!--        </el-scrollbar>-->
<!--      </div>-->
    </el-card>

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
