<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

type OperationType = "install" | "uninstall" | null;
type ProtocolType = "https" | "http";

const domain = ref("");
const protocol = ref<ProtocolType>("https");
const loading = ref(false);
const message = ref("");
const messageType = ref<"success" | "error" | "info">("info");
const currentOperation = ref<OperationType>(null);

const setOperation = (op: OperationType) => {
  currentOperation.value = op;
  message.value = "";
  if (op === "uninstall") {
    domain.value = "";
  }
};

const handleInstall = async () => {
  if (!domain.value.trim()) {
    message.value = "请输入域名";
    messageType.value = "error";
    return;
  }

  loading.value = true;
  message.value = "正在安装插件...";
  messageType.value = "info";

  try {
    const result = await invoke<string>("install_plugin", { 
      domain: domain.value.trim(),
      protocol: protocol.value
    });
    message.value = result;
    messageType.value = "success";
  } catch (error) {
    message.value = `安装失败: ${error}`;
    messageType.value = "error";
  } finally {
    loading.value = false;
  }
};

const handleUninstall = async () => {
  loading.value = true;
  message.value = "正在卸载插件...";
  messageType.value = "info";

  try {
    const result = await invoke<string>("uninstall_plugin");
    message.value = result;
    messageType.value = "success";
  } catch (error) {
    message.value = `卸载失败: ${error}`;
    messageType.value = "error";
  } finally {
    loading.value = false;
  }
};
</script>

<template>
  <div class="container">
    <h1 class="title">WPS 插件管理器</h1>
    <p class="subtitle">管理 jaidoc-wps 插件</p>

    <div class="button-group">
      <button
        class="btn"
        :class="{ active: currentOperation === 'install' }"
        @click="setOperation('install')"
        :disabled="loading"
      >
        安装插件
      </button>
      <button
        class="btn btn-danger"
        :class="{ active: currentOperation === 'uninstall' }"
        @click="setOperation('uninstall')"
        :disabled="loading"
      >
        卸载插件
      </button>
    </div>

    <div v-if="currentOperation === 'install'" class="form-section">
      <div class="input-group">
        <label>协议</label>
        <div class="protocol-selector">
          <button
            class="protocol-btn"
            :class="{ active: protocol === 'https' }"
            @click="protocol = 'https'"
            :disabled="loading"
          >
            HTTPS
          </button>
          <button
            class="protocol-btn"
            :class="{ active: protocol === 'http' }"
            @click="protocol = 'http'"
            :disabled="loading"
          >
            HTTP
          </button>
        </div>
      </div>

      <div class="input-group">
        <label for="domain">域名</label>
        <div class="domain-input-wrapper">
          <span class="protocol-prefix">{{ protocol }}://</span>
          <input
            id="domain"
            v-model="domain"
            type="text"
            placeholder="例如: workin.hanweb.com"
            :disabled="loading"
            @keyup.enter="handleInstall"
          />
        </div>
        <span class="hint">输入插件服务器域名，不需要协议前缀</span>
      </div>

      <button
        class="btn btn-primary"
        @click="handleInstall"
        :disabled="loading || !domain.trim()"
      >
        {{ loading ? "安装中..." : "确认安装" }}
      </button>
    </div>

    <div v-if="currentOperation === 'uninstall'" class="form-section">
      <p class="warning-text">⚠️ 确定要卸载 jaidoc-wps 插件吗？</p>
      <button
        class="btn btn-danger"
        @click="handleUninstall"
        :disabled="loading"
      >
        {{ loading ? "卸载中..." : "确认卸载" }}
      </button>
    </div>

    <div v-if="message" class="message" :class="messageType">
      {{ message }}
    </div>
  </div>
</template>

<style scoped>
.container {
  padding: 30px;
  max-width: 500px;
  margin: 0 auto;
}

.title {
  font-size: 24px;
  font-weight: 600;
  color: #333;
  margin: 0 0 8px 0;
  text-align: center;
}

.subtitle {
  font-size: 14px;
  color: #666;
  margin: 0 0 30px 0;
  text-align: center;
}

.button-group {
  display: flex;
  gap: 12px;
  margin-bottom: 30px;
}

.btn {
  flex: 1;
  padding: 12px 20px;
  border: 2px solid #e0e0e0;
  border-radius: 8px;
  background: white;
  color: #333;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn:hover:not(:disabled) {
  border-color: #409eff;
  color: #409eff;
}

.btn.active {
  border-color: #409eff;
  background: #409eff;
  color: white;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-danger {
  border-color: #f56c6c;
  color: #f56c6c;
}

.btn-danger:hover:not(:disabled) {
  border-color: #f56c6c;
  background: #fef0f0;
}

.btn-danger.active {
  border-color: #f56c6c;
  background: #f56c6c;
  color: white;
}

.btn-primary {
  width: 100%;
  border-color: #67c23a;
  background: #67c23a;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #85ce61;
  border-color: #85ce61;
}

.form-section {
  background: #f5f7fa;
  padding: 20px;
  border-radius: 8px;
  margin-bottom: 20px;
}

.input-group {
  margin-bottom: 16px;
}

.input-group label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: #333;
  margin-bottom: 8px;
}

.protocol-selector {
  display: flex;
  gap: 8px;
}

.protocol-btn {
  flex: 1;
  padding: 8px 16px;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  background: white;
  color: #606266;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.protocol-btn:hover:not(:disabled) {
  border-color: #409eff;
  color: #409eff;
}

.protocol-btn.active {
  border-color: #409eff;
  background: #409eff;
  color: white;
}

.protocol-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.domain-input-wrapper {
  display: flex;
  align-items: center;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  background: white;
  overflow: hidden;
}

.protocol-prefix {
  padding: 10px 8px 10px 12px;
  background: #f5f7fa;
  color: #606266;
  font-size: 14px;
  border-right: 1px solid #dcdfe6;
  white-space: nowrap;
}

.domain-input-wrapper input {
  flex: 1;
  padding: 10px 12px;
  border: none;
  font-size: 14px;
  outline: none;
  background: transparent;
}

.domain-input-wrapper input:disabled {
  background: #f5f7fa;
  cursor: not-allowed;
}

.domain-input-wrapper:focus-within {
  border-color: #409eff;
}

.hint {
  display: block;
  font-size: 12px;
  color: #909399;
  margin-top: 6px;
}

.warning-text {
  color: #f56c6c;
  font-size: 14px;
  margin: 0 0 16px 0;
  text-align: center;
}

.message {
  padding: 12px 16px;
  border-radius: 4px;
  font-size: 14px;
  text-align: center;
  word-break: break-word;
}

.message.success {
  background: #f0f9eb;
  color: #67c23a;
  border: 1px solid #e1f3d8;
}

.message.error {
  background: #fef0f0;
  color: #f56c6c;
  border: 1px solid #fde2e2;
}

.message.info {
  background: #f4f4f5;
  color: #909399;
  border: 1px solid #e9e9eb;
}
</style>
