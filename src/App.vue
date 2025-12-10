<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
const network_interfaces = ref([]);
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
        <el-text class="mx-1" size="large" type="primary">网卡名称</el-text>
        <el-scrollbar>
          <div class="scrollbar-flex-content">
            <el-text class="mx-1 scrollbar-demo-item" type="primary">{{iface.name}}</el-text>
          </div>
        </el-scrollbar>
      </div>
      <div>
        <el-text class="mx-1" size="large" type="primary">网卡描述</el-text>
        <el-scrollbar>
          <div class="scrollbar-flex-content">
            <el-text class="mx-1 scrollbar-demo-item" type="primary">{{ iface.description }}</el-text>
          </div>
        </el-scrollbar>
      </div>
      <div>
        <el-text class="mx-1" size="large" type="primary">mac地址</el-text>
        <el-scrollbar>
          <div class="scrollbar-flex-content">
            <el-text class="mx-1 scrollbar-demo-item" type="primary">{{iface.mac_address?iface.mac_address:"无"}}</el-text>
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
            <el-text v-for="dns in iface.dns_servers" :key="ip" class="mx-1 scrollbar-demo-item" type="primary">{{ dns }}</el-text>
          </div>
          <div v-else class="scrollbar-flex-content">
            <el-text class="mx-1 scrollbar-demo-item" type="primary">无</el-text>
          </div>
        </el-scrollbar>
      </div>
      <div>
        <el-text class="mx-1" size="large" type="primary">接口类型</el-text>
        <el-scrollbar>
          <div class="scrollbar-flex-content">
            <el-text class="mx-1 scrollbar-demo-item" type="primary">{{ iface.if_type }}</el-text>
          </div>
        </el-scrollbar>
      </div>
    </el-card>

  </main>
</template>

<style scoped>
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
