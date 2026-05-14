# WPS 插件管理器

基于 Tauri + Vue 3 + Vite 开发的 macOS 桌面应用，用于管理 WPS Office 的 jaidoc-wps 插件。

## 功能

- **安装插件**: 输入域名即可安装对应环境的 jaidoc-wps 插件
- **卸载插件**: 一键卸载已安装的插件
- **自动备份**: 操作前自动备份原配置文件

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust + Tauri
- **打包**: 支持生成 .dmg 安装包

## 开发环境要求

- Node.js >= 18.0.0
- Rust >= 1.70
- macOS (用于打包 .dmg)

## 安装依赖

```bash
npm install
```

## 开发运行

```bash
npm run tauri:dev
```

## 构建生产版本

```bash
npm run tauri:build
```

构建完成后，.dmg 文件将位于 `src-tauri/target/release/bundle/dmg/` 目录。

## 项目结构

```
wps-plugin-manager/
├── src/                    # Vue 前端代码
│   ├── App.vue            # 主界面
│   ├── main.ts            # 入口文件
│   └── vite-env.d.ts      # 类型声明
├── src-tauri/             # Tauri/Rust 后端代码
│   ├── src/
│   │   └── main.rs        # Rust 主程序
│   ├── icons/             # 应用图标
│   ├── Cargo.toml         # Rust 依赖
│   └── tauri.conf.json    # Tauri 配置
├── package.json           # Node.js 依赖
├── vite.config.ts         # Vite 配置
└── tsconfig.json          # TypeScript 配置
```

## 核心功能实现

### 安装插件

1. 用户输入域名（如 `workin.hanweb.com`）
2. 应用自动构建插件 URL: `https://{domain}/jaidoc-wps/`
3. 修改 WPS 配置文件 `~/Library/Containers/com.kingsoft.wpsoffice.mac/Data/.kingsoft/wps/jsaddons/publish.xml`
4. 添加或更新 `<jspluginonline>` 标签

### 卸载插件

1. 从 publish.xml 中移除 jaidoc-wps 插件配置
2. 保留其他插件配置不变

## 配置文件路径

```
~/Library/Containers/com.kingsoft.wpsoffice.mac/Data/.kingsoft/wps/jsaddons/publish.xml
```

## 注意事项

1. 首次运行需要授予文件访问权限
2. 操作前会自动创建 `.bak` 备份文件
3. 仅支持 macOS 系统（因为 WPS 配置文件路径是 macOS 特有的）

## 许可证

MIT
