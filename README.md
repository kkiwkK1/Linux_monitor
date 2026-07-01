# LinuxMonitor

Linux 桌面硬件性能监控悬浮窗，类 Windows TrafficMonitor 体验。轻量、始终置顶、可换肤、可插件。

## 特性

- 🖥 **悬浮窗** — 无边框、始终置顶、可拖拽吸边、圆角透明卡片
- 📊 **实时监控** — CPU / 内存 / 网络 / 磁盘 / GPU / 温度
- 🎨 **8 种内置皮肤** — 统一扁平设计，配色集中管理
- 🧩 **自定义皮肤** — 写一个 TOML 文件即可造自己的皮肤，无需编译
- 📈 **历史记录** — SQLite 存储 + Cairo 折线图 + CSV 导出
- 🌐 **中英双语** — 跟随系统 `LANG` 自动切换
- 🔌 **插件系统** — Rhai 脚本沙箱，可自定义告警
- 🪶 **超轻量** — 约 2MB 内存，4MB 二进制

## 安装

```bash
# 编译
cargo build --release

# 安装
sudo cp target/release/linux-monitor /usr/local/bin/

# 运行
linux-monitor
```

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| Ctrl+Shift+T | 显示 / 隐藏 |
| Ctrl+Shift+S | 设置 |
| Ctrl+Shift+H | 历史 |
| Ctrl+Q | 退出 |
| 右键 | 菜单（切皮肤 / 设置 / 历史 / 退出） |
| 左键拖拽 | 移动窗口（靠边自动吸附） |

## 皮肤

统一的扁平视觉设计，所有颜色 / 间距 / 字体集中在 `src/ui/theme.rs`（改一处，全局生效）。右键窗口 → **皮肤** 切换。内置 8 种：

| 皮肤 | 说明 |
|------|------|
| 横条模式 | 双行标签 + Cairo 折线图 |
| 竖排列表 | 各指标进度条列表（信息最全） |
| 紧凑模式 | 单行，最不占地方 |
| 精致扁平 | 圆角细进度条 + 右对齐数值 |
| 环形仪表 | CPU / MEM / GPU 环形表，数字在环心 |
| 毛玻璃 | 半透明渐变卡 + 顶部高光 |
| 大字极简 | 主指标超大显示，其余低调 |
| 极光 | 指标渲染为渐变色场 |

> 毛玻璃说明：GTK3 / X11 无法获取窗口背景像素，因此**没有真正的背景模糊 / 折射**，此皮肤用「半透明 + 渐变 + 高光」模拟玻璃质感。真模糊需合成器级支持（如 KWin 模糊、picom blur）。

## 自定义皮肤

把 `*.toml` 皮肤文件放进 `~/.config/linux-monitor/skins/`，右键 **皮肤** 菜单即可选择（首次运行会自动生成一份带注释的 `example.toml` 模板）。纯文本、可热重载、方便分享——加 / 改文件后重新打开右键菜单即刷新。

示例：

```toml
name = "我的皮肤"           # 菜单里显示的名字
layout = "vertical"         # vertical(竖排) | horizontal(横排)

# metric:  cpu | mem | net | disk | gpu | temp
# element: bar(进度条) | text(文字) | sparkline(折线) | ring(环形)
# color:   "#rrggbb" 十六进制；或 "auto" / 省略 用该指标内置配色
#          （temp 的 auto 会按温度 绿 → 琥珀 → 红 分级）

[[row]]
metric = "cpu"
element = "bar"

[[row]]
metric = "cpu"
element = "sparkline"
color = "#6cb2ff"

[[row]]
metric = "net"
element = "text"

[[row]]
metric = "temp"
element = "text"
color = "auto"
```

字段一览：

| 字段 | 取值 | 说明 |
|------|------|------|
| `name` | 任意字符串 | 皮肤菜单显示名 |
| `layout` | `vertical` / `horizontal` | 整体排列方向 |
| `[[row]].metric` | `cpu` `mem` `net` `disk` `gpu` `temp` | 该行监控的指标 |
| `[[row]].element` | `bar` `text` `sparkline` `ring` | 该行的呈现形式 |
| `[[row]].color` | `#rrggbb` / `auto` / 省略 | 颜色，`auto` 用内置配色（温度自动分级） |
| `[[row]].label` | 任意字符串（可选） | 标签文字，默认用指标名 |

## 插件

插件是放在 `~/.config/linux-monitor/plugins/*.rhai` 的 [Rhai](https://rhai.rs/) 脚本，在**沙箱**中运行（无文件系统 / 网络 / 外部命令访问），每 60 秒执行一次，可用于自定义告警。

可用变量：`cpu_percent` `mem_percent` `net_rx` `net_tx` `cpu_temp` `gpu_temp` `core_count`；可调用 `ALERT.call(类型, 阈值, 当前值, 消息)`。

```rhai
// 温度告警示例
if gpu_temp > 80.0 {
    ALERT.call("GPU温度告警", 80.0, gpu_temp, "GPU 温度过高");
}
```

## 配置

配置文件位于 `~/.config/linux-monitor/config.toml`（权限 0600），涵盖轮询间隔、外观（皮肤 / 字号 / 透明度 / 置顶 / 背景）、各监控项开关、窗口位置等。设置窗口（Ctrl+Shift+S）中的改动会写回此文件。

## 技术栈

Rust + GTK3 + Cairo + SQLite + Rhai

## License

GPL-3.0
