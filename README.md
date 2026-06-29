# LinuxMonitor

Linux 桌面硬件性能监控悬浮窗，类 Windows TrafficMonitor 体验。

## 特性

- 🖥 **悬浮窗** — 始终置顶，可拖拽，透明背景
- 📊 **实时监控** — CPU / 内存 / 网络 / 磁盘 / GPU / 温度
- 🎨 **3 种皮肤** — 横条（含折线图）/ 竖排（进度条）/ 紧凑
- 📈 **历史记录** — SQLite 存储 + Cairo 折线图 + CSV 导出
- 🌐 **中英双语** — 根据系统 LANG 自动切换
- 🔌 **插件系统** — Rhai 脚本沙箱，可自定义告警
- 🪶 **超轻量** — 2MB 内存，0% CPU，4MB 二进制

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
| Ctrl+Shift+T | 显示/隐藏 |
| Ctrl+Shift+S | 设置 |
| Ctrl+Shift+H | 历史 |
| 右键 | 菜单 |
| 左键拖拽 | 移动窗口 |

## 插件

插件放在 `~/.config/linux-monitor/plugins/*.rhai`，每 60 秒执行一次。

```rhai
// 温度告警示例
if gpu_temp > 80.0 {
    ALERT.call("GPU温度告警", 80.0, gpu_temp, "GPU 温度过高");
}
```

## 技术栈

Rust + GTK3 + SQLite + Cairo + Rhai

## License

GPL-3.0
