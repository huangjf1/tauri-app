<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

type ProtocolType = 'https' | 'http'
type TabType = 'install' | 'list'

interface PluginInfo {
  name: string
  url: string
  plugin_type: string
}

// 预设插件模板
const PRESETS = [
  { label: 'jaidoc-wps', name: 'jaidoc-wps', path: '/jaidoc-wps/' },
  { label: '自定义插件', name: '', path: '' }
]

const currentTab = ref<TabType>('install')
const protocol = ref<ProtocolType>('https')
const domain = ref('')
const pluginName = ref('jaidoc-wps')
const pluginPath = ref('/jaidoc-wps/')
const selectedPreset = ref(0)

const loading = ref(false)
const message = ref('')
const messageType = ref<'success' | 'error' | 'info'>('info')

const plugins = ref<PluginInfo[]>([])
const listLoading = ref(false)
const uninstalling = ref('')

const generatedUrl = computed(() => {
  if (!domain.value.trim()) return ''
  const path = pluginPath.value || '/'
  return `${protocol.value}://${domain.value.trim()}${path.startsWith('/') ? path : '/' + path}`
})

const canInstall = computed(() => {
  return domain.value.trim() && pluginName.value.trim() && pluginPath.value.trim()
})

const selectPreset = (idx: number) => {
  selectedPreset.value = idx
  const preset = PRESETS[idx]
  pluginName.value = preset.name
  pluginPath.value = preset.path
}

const showMsg = (text: string, type: 'success' | 'error' | 'info') => {
  message.value = text
  messageType.value = type
}

const handleInstall = async () => {
  if (!canInstall.value) {
    showMsg('请填写完整的插件信息', 'error')
    return
  }

  loading.value = true
  message.value = ''

  try {
    const result = await invoke<string>('install_plugin', {
      domain: domain.value.trim(),
      protocol: protocol.value,
      pluginName: pluginName.value.trim(),
      pluginPath: pluginPath.value.trim()
    })
    showMsg(result, 'success')
    // 安装成功后刷新列表
    if (currentTab.value === 'list') {
      await loadPlugins()
    }
  } catch (error) {
    showMsg(`${error}`, 'error')
  } finally {
    loading.value = false
  }
}

const switchToList = () => {
  currentTab.value = 'list'
  loadPlugins()
}

const loadPlugins = async () => {
  listLoading.value = true
  try {
    plugins.value = await invoke<PluginInfo[]>('list_plugins')
  } catch (error) {
    showMsg(`读取插件列表失败: ${error}`, 'error')
  } finally {
    listLoading.value = false
  }
}

const handleUninstall = async (name: string) => {
  uninstalling.value = name
  message.value = ''

  try {
    const result = await invoke<string>('uninstall_plugin', { pluginName: name })
    showMsg(result, 'success')
    await loadPlugins()
  } catch (error) {
    showMsg(`${error}`, 'error')
  } finally {
    uninstalling.value = ''
  }
}

onMounted(() => {
  loadPlugins()
})
</script>

<template>
  <div class="app">
    <header class="header">
      <h1 class="header-title">WPS 插件管理器</h1>
      <p class="header-desc">安装、卸载 WPS Office jsplugins 加载项</p>
    </header>

    <div class="divider">
      <button class="divider-tab" :class="{ active: currentTab === 'install' }" @click="currentTab = 'install'">
        安装
      </button>
      <button class="divider-tab" :class="{ active: currentTab === 'list' }" @click="switchToList">已安装</button>
    </div>

    <div class="body">
      <!-- 安装面板 -->
      <div v-if="currentTab === 'install'" class="section">
        <!-- 插件模板 -->
        <label class="label">插件类型</label>
        <div class="preset-group">
          <button
            v-for="(p, i) in PRESETS"
            :key="i"
            class="preset-btn"
            :class="{ active: selectedPreset === i }"
            @click="selectPreset(i)"
          >
            {{ p.label }}
          </button>
        </div>

        <!-- 协议 -->
        <label class="label">协议</label>
        <div class="radio-group">
          <label class="radio">
            <input type="radio" v-model="protocol" value="https" />
            HTTPS
          </label>
          <label class="radio">
            <input type="radio" v-model="protocol" value="http" />
            HTTP
          </label>
        </div>

        <!-- 域名 -->
        <label class="label">域名</label>
        <div class="input-row">
          <span class="input-addon">{{ protocol }}://</span>
          <input
            v-model="domain"
            type="text"
            placeholder="workin.hanweb.com"
            @keyup.enter="handleInstall"
            class="input"
          />
        </div>

        <!-- 自定义时显示插件名和路径 -->
        <template v-if="selectedPreset === 1">
          <label class="label">插件名称</label>
          <input v-model="pluginName" type="text" placeholder="my-plugin" class="input-full" />

          <label class="label">路径</label>
          <div class="input-row">
            <span class="input-addon">{{ protocol }}://{{ domain || 'domain' }}</span>
            <input v-model="pluginPath" type="text" placeholder="/my-plugin" class="input" />
          </div>
        </template>

        <div v-if="generatedUrl" class="preview">{{ generatedUrl }}</div>

        <button class="btn btn-primary" @click="handleInstall" :disabled="loading || !canInstall">
          {{ loading ? '安装中…' : '安装' }}
        </button>
      </div>

      <!-- 已安装列表 -->
      <div v-if="currentTab === 'list'" class="section">
        <div class="list-header">
          <span class="list-count">共 {{ plugins.length }} 个插件</span>
          <button class="btn-link" @click="loadPlugins" :disabled="listLoading">
            {{ listLoading ? '刷新中…' : '刷新' }}
          </button>
        </div>

        <div v-if="plugins.length === 0" class="empty">暂无已安装的插件</div>

        <div v-else class="plugin-list">
          <div v-for="p in plugins" :key="p.name" class="plugin-item">
            <div class="plugin-info">
              <span class="plugin-name">{{ p.name }}</span>
              <span class="plugin-url">{{ p.url }}</span>
            </div>
            <button
              class="btn btn-sm btn-danger-outline"
              @click="handleUninstall(p.name)"
              :disabled="uninstalling === p.name"
            >
              {{ uninstalling === p.name ? '卸载中…' : '卸载' }}
            </button>
          </div>
        </div>
      </div>

      <!-- 消息 -->
      <div v-if="message" class="msg" :class="messageType">{{ message }}</div>
    </div>
  </div>
</template>

<style scoped>
.app {
  min-height: 100vh;
  background: #fff;
  display: flex;
  flex-direction: column;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, sans-serif;
  color: #1d1d1f;
  font-size: 13px;
  line-height: 1.5;
}

/* header */
.header {
  padding: 20px 24px 16px;
}
.header-title {
  font-size: 15px;
  font-weight: 600;
  margin: 0;
}
.header-desc {
  margin: 4px 0 0;
  color: #86868b;
  font-size: 12px;
}

/* 分段控制 */
.divider {
  display: flex;
  border-bottom: 1px solid #e5e5e5;
  padding: 0 24px;
}
.divider-tab {
  padding: 8px 0;
  margin-right: 24px;
  border: none;
  background: none;
  font-size: 13px;
  font-weight: 500;
  color: #86868b;
  cursor: pointer;
  position: relative;
  font-family: inherit;
}
.divider-tab::after {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  bottom: -1px;
  height: 1.5px;
  background: transparent;
  transition: background 0.15s;
}
.divider-tab.active {
  color: #1d1d1f;
}
.divider-tab.active::after {
  background: #1d1d1f;
}

/* body */
.body {
  flex: 1;
  padding: 20px 24px 24px;
  display: flex;
  flex-direction: column;
}
.section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

/* label */
.label {
  font-size: 12px;
  font-weight: 500;
  color: #86868b;
  margin-bottom: -4px;
}

/* preset buttons */
.preset-group {
  display: flex;
  gap: 6px;
}
.preset-btn {
  padding: 5px 12px;
  border: 1px solid #d2d2d7;
  border-radius: 4px;
  background: #fff;
  font-size: 12px;
  color: #1d1d1f;
  cursor: pointer;
  font-family: inherit;
}
.preset-btn.active {
  background: #1d1d1f;
  color: #fff;
  border-color: #1d1d1f;
}

/* radio */
.radio-group {
  display: flex;
  gap: 16px;
}
.radio {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 13px;
  color: #1d1d1f;
  cursor: pointer;
}
.radio input {
  margin: 0;
  accent-color: #1d1d1f;
}

/* input */
.input-row {
  display: flex;
  border: 1px solid #d2d2d7;
  border-radius: 6px;
  overflow: hidden;
}
.input-row:focus-within {
  border-color: #1d1d1f;
}
.input-addon {
  padding: 7px 10px;
  background: #f5f5f7;
  color: #86868b;
  font-size: 12px;
  border-right: 1px solid #d2d2d7;
  white-space: nowrap;
  user-select: none;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 200px;
}
.input {
  flex: 1;
  padding: 7px 10px;
  border: none;
  font-size: 13px;
  outline: none;
  font-family: inherit;
  background: #fff;
  color: #1d1d1f;
  min-width: 0;
}
.input::placeholder {
  color: #c7c7cc;
}
.input-full {
  padding: 7px 10px;
  border: 1px solid #d2d2d7;
  border-radius: 6px;
  font-size: 13px;
  outline: none;
  font-family: inherit;
  background: #fff;
  color: #1d1d1f;
  width: 100%;
}
.input-full:focus {
  border-color: #1d1d1f;
}
.input-full::placeholder {
  color: #c7c7cc;
}

/* help text */
.help {
  font-size: 11px;
  color: #86868b;
  margin: 0;
}

/* url preview */
.preview {
  font-size: 11px;
  color: #86868b;
  background: #f5f5f7;
  padding: 6px 10px;
  border-radius: 4px;
  word-break: break-all;
  font-family: 'SF Mono', Menlo, Consolas, monospace;
}

/* button */
.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  font-family: inherit;
  transition: opacity 0.15s;
  width: fit-content;
  min-width: 80px;
}
.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.btn-primary {
  background: #1d1d1f;
  color: #fff;
}
.btn-primary:hover:not(:disabled) {
  opacity: 0.85;
}
.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
  min-width: 56px;
}
.btn-danger-outline {
  background: #fff;
  color: #c62828;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
}
.btn-danger-outline:hover:not(:disabled) {
  background: #fff5f5;
  border-color: #c62828;
}
.btn-link {
  border: none;
  background: none;
  color: #1d1d1f;
  font-size: 12px;
  cursor: pointer;
  font-family: inherit;
  padding: 0;
  text-decoration: underline;
  text-underline-offset: 2px;
}
.btn-link:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* list header */
.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.list-count {
  font-size: 12px;
  color: #86868b;
}

/* empty */
.empty {
  color: #86868b;
  font-size: 13px;
  text-align: center;
  padding: 32px 0;
}

/* plugin list */
.plugin-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
  background: #e5e5e5;
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  overflow: hidden;
}
.plugin-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  background: #fff;
  gap: 12px;
}
.plugin-info {
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 2px;
}
.plugin-name {
  font-size: 13px;
  font-weight: 500;
  color: #1d1d1f;
}
.plugin-url {
  font-size: 11px;
  color: #86868b;
  word-break: break-all;
  font-family: 'SF Mono', Menlo, Consolas, monospace;
}

/* message */
.msg {
  margin-top: 16px;
  padding: 10px 12px;
  border-radius: 6px;
  font-size: 12px;
  line-height: 1.5;
  word-break: break-word;
}
.msg.success {
  background: #e8f5e9;
  color: #2e7d32;
}
.msg.error {
  background: #ffebee;
  color: #c62828;
}
.msg.info {
  background: #f5f5f7;
  color: #86868b;
}
</style>
