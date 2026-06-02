# Pake Manager

Pake 的配套桌面管理器——侧边栏统一管理多个 Web 应用。
![Uploading 图片.png…]()

## 下载使用

1. 打开 [Releases 页面](https://github.com/Xxiaoxiaoxiaox/page-manager/releases)
2. 下载最新版本的 `pake-manager.exe`
3. 双击运行，无需任何依赖

首次启动会自动生成配置文件 `~/.pake-manager/apps.json`。

## 功能

- 侧边栏管理多个 Web 应用（DeepSeek、Grok、Gemini、Claude 等）
- 双代理配置：本地代理 + 7897 代理，每个应用独立选择
- 应用拖动排序
- 窗口尺寸自动记忆
- 启动自动选中首个应用

## 开发

```bash
cd manager/src-tauri
cargo build --release
```
