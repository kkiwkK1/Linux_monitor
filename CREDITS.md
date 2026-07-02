# 开源协议与第三方引用 · License & Credits

## 本项目协议

**LinuxMonitor** 以 **GNU General Public License v3.0 或更新版本（`GPL-3.0-or-later`）** 授权。
完整条款见 [LICENSE](LICENSE)。

## 系统库（动态链接 / 运行时依赖）

| 库 | 用途 | 许可证 |
|----|------|--------|
| GTK 3（GDK / GLib / GObject / GIO / GdkPixbuf / ATK / Pango / Cairo） | 窗口、绘制、布局、图标 | LGPL-2.1-or-later |
| libayatana-appindicator3 | 系统托盘 / StatusNotifierItem | LGPL-2.1 / LGPL-3 |
| SQLite（经 `rusqlite` bundled 内置） | 历史记录存储 | Public Domain |

## Rust 依赖（直接）

| crate | 用途 | 许可证 |
|-------|------|--------|
| `gtk` · `gio` · `cairo-rs` · `pangocairo` | GTK3 / Cairo / Pango 的 Rust 绑定 | MIT |
| `rusqlite` | SQLite 绑定 | MIT |
| `sysinfo` | CPU / 内存 / 网络 / 磁盘等指标采集 | MIT |
| `rhai` | 插件脚本引擎（沙箱） | MIT OR Apache-2.0 |
| `libappindicator` | 托盘绑定 | MIT OR Apache-2.0 |
| `serde` · `toml` | 配置序列化 | MIT OR Apache-2.0 |
| `anyhow` · `thiserror` | 错误处理 | MIT OR Apache-2.0 |
| `log` · `env_logger` | 日志 | MIT OR Apache-2.0 |
| `libc` | 系统调用（daemon 化、文件权限等） | MIT OR Apache-2.0 |

> 完整依赖树（含传递依赖）及各自许可证可随时用 `cargo license` 查看。传递依赖以 MIT / Apache-2.0 为主，另含 `libloading`(ISC)、`smartstring`(MPL-2.0+)、`tiny-keccak`(CC0-1.0)、`unicode-ident`(Unicode-3.0) 等，均与 GPL-3.0 兼容。

## 原创与说明

- **应用图标（Pulse）** 为本项目原创，使用 Cairo 代码绘制（`src/ui/icon.rs`），随本项目以 GPL-3.0 授权。
- 界面**未内嵌第三方字体**，全部使用系统字体渲染。
- 上述 LGPL 库均以**动态链接**方式使用，符合 LGPL 与 GPL-3.0 的组合分发要求。

## 许可证兼容性

GPL-3.0 与 LGPL-2.1+/LGPL-3、MIT、Apache-2.0、ISC、MPL-2.0、Public Domain、CC0-1.0 等均兼容；本项目整体以 **GPL-3.0-or-later** 分发。
