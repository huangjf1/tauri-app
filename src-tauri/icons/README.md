# 图标文件

此目录需要包含以下图标文件用于打包：

- `32x32.png` - 32x32 像素 PNG 图标
- `128x128.png` - 128x128 像素 PNG 图标
- `128x128@2x.png` - 256x256 像素 PNG 图标（Retina）
- `icon.icns` - macOS 图标集
- `icon.ico` - Windows 图标

## 生成图标

可以使用 Tauri 的图标生成工具：

```bash
npm run tauri icon path/to/icon.png
```

或者使用在线工具生成各种尺寸的图标。

## 临时解决方案

在开发阶段，你可以使用占位图标，或者从其他项目复制图标文件到此目录。
