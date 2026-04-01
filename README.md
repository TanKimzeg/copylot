# Copylot

Windows 桌面端划词翻译工具（Tauri v2 + TypeScript）。

- 全局快捷键触发：获取当前选中文本并翻译
- 翻译结果在右侧置顶浮窗展示
- 主窗口提供配置（API Key / Base URL / Model / Hotkey），使用 store 持久化

> 当前仅支持 **Windows 桌面端**。

## 功能

- 全局快捷键（默认：`Ctrl+Alt+Q`）
- 选中文本提取（UI Automation + 剪贴板兜底）
- 翻译展示浮窗（always-on-top）
- 一键复制翻译结果

## 开发环境

- Node.js（建议 18+）
- pnpm
- Rust stable + cargo
- Windows 10/11

## 启动开发

```bash
pnpm install
pnpm dev
pnpm tauri dev
```

## 构建

```bash
pnpm build
pnpm tauri build
```

生成的安装包位于 `src-tauri/target/release/bundle`。

## 配置与数据

- 配置存储：`copylot_store.bin`（由 `tauri-plugin-store` 生成，已加入 `.gitignore`）
- 主要配置项：
  - `translation_api_key`
  - `translation_base_url`
  - `translation_model`
  - `hotkey`

## 权限（Capabilities）

本项目使用 Tauri v2 capabilities 进行鉴权：

- `src-tauri/capabilities/default.json`：最小权限（默认）
- `src-tauri/capabilities/desktop.json`：Windows 桌面端实际运行所需权限（事件、全局快捷键、剪贴板写入、store 等）

如遇到 “not allowed” 类报错，请优先检查 capability 的 `windows` 白名单与权限列表是否匹配。

## License

AGPL-3.0-only（见 `LICENSE`）。
