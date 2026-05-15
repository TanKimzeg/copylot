# Changelog

本项目的所有重要变更都会记录在此文件中。

格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本号遵循 [SemVer](https://semver.org/lang/zh-CN/)。

## [0.3.0] - 2026-05-15

### Added

- 切换 `openai` 依赖至自维护 fork（`TanKimzeg/openai`），支持 `extra_body` 透传字段。
- 对 DeepSeek 模型自动注入 `thinking: { type: "disabled" }`，显式关闭深度思考以提升响应速度。

### Changed

- 重构翻译模块，抽取 `get_chat_completion_builder` 函数集中管理请求构建逻辑。
- `update_app_conf` 命令现在返回更新后的 `AppConfig`。

### Fixed

- 修复流式翻译前端逐块渲染时覆盖前文的问题（`textContent +=`）。
- 修复配置写入存储失败时错误回退为默认值的问题，现在保留原值。
- 翻译窗口关闭后不再继续推送流式数据，避免后台报错。

## [0.2.1] - 2026-04-27

### Changed

- 翻译接口从一次性请求改为流式响应（SSE），后端逐块推送，前端实时渲染翻译结果，大幅降低首字延迟。

### Fixed

- 将默认翻译模型更新为 `deepseek-v4-flash`，适配 DeepSeek 最新模型。

## [0.2.0] - 2026-04-04

### Added

- feat: 增加系统托盘。点击主窗口关闭按钮后，应用最小化到系统托盘而非完全退出。在托盘图标上右键可以打开菜单，选择退出来完全关闭应用。

- feat: 新增翻译历史记录功能。用户可以查看和管理之前的翻译记录。

- feat: 用户可以查看和管理翻译历史记录。

### Fixed

- fix: 关闭主窗口后进程仍后台驻留的问题。

## [0.1.1] - 2026-04-02

### Fixed

- fix: 修复构建配置，支持 `translator` 页面入口。

## [0.1.0] - 2026-04-01

### Added

- 初始版本发布。实现了基本的窗口管理、配置系统和翻译功能。
