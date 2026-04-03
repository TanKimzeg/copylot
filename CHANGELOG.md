# Changelog

本项目的所有重要变更都会记录在此文件中。

格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本号遵循 [SemVer](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Added

- feat: 增加系统托盘。点击主窗口关闭按钮后，应用最小化到系统托盘而非完全退出。在托盘图标上右键可以打开菜单，选择退出来完全关闭应用。

### Fixed

- fix: 关闭主窗口后进程仍后台驻留的问题。

## [0.1.1] - 2026-04-02

### Fixed

- fix: 修复构建配置，支持 `translator` 页面入口。

## [0.1.0] - 2026-04-01

### Added

- 初始版本发布。实现了基本的窗口管理、配置系统和翻译功能。
