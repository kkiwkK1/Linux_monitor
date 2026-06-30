/// Localization / 本地化字符串
///
/// All UI text is centralized here. To add a new language,
/// implement the `Locale` trait and switch at startup.
pub trait Locale: Send + Sync {
    fn app_name(&self) -> &str;
    fn app_title(&self) -> &str;

    // Menu / 菜单
    fn menu_hide(&self) -> &str;
    fn menu_skins(&self) -> &str;
    fn menu_history(&self) -> &str;
    fn menu_settings(&self) -> &str;
    fn menu_quit(&self) -> &str;

    // Skins / 皮肤
    fn skin_horizontal(&self) -> &str;
    fn skin_vertical(&self) -> &str;
    fn skin_compact(&self) -> &str;

    // Settings / 设置
    fn settings_title(&self) -> &str;
    fn settings_tab_general(&self) -> &str;
    fn settings_tab_monitors(&self) -> &str;
    fn settings_tab_appearance(&self) -> &str;
    fn settings_general_title(&self) -> &str;
    fn settings_active_interval(&self) -> &str;
    fn settings_active_desc(&self) -> &str;
    fn settings_bg_interval(&self) -> &str;
    fn settings_bg_desc(&self) -> &str;
    fn settings_idle_interval(&self) -> &str;
    fn settings_idle_desc(&self) -> &str;
    fn settings_monitors_title(&self) -> &str;
    fn settings_monitor_cpu(&self) -> &str;
    fn settings_monitor_cpu_desc(&self) -> &str;
    fn settings_monitor_memory(&self) -> &str;
    fn settings_monitor_memory_desc(&self) -> &str;
    fn settings_monitor_network(&self) -> &str;
    fn settings_monitor_network_desc(&self) -> &str;
    fn settings_monitor_disk(&self) -> &str;
    fn settings_monitor_disk_desc(&self) -> &str;
    fn settings_monitor_gpu(&self) -> &str;
    fn settings_monitor_gpu_desc(&self) -> &str;
    fn settings_monitor_thermal(&self) -> &str;
    fn settings_monitor_thermal_desc(&self) -> &str;
    fn settings_appearance_title(&self) -> &str;
    fn settings_skin(&self) -> &str;
    fn settings_font_size(&self) -> &str;
    fn settings_font_desc(&self) -> &str;
    fn settings_opacity(&self) -> &str;
    fn settings_opacity_desc(&self) -> &str;
    fn settings_always_on_top(&self) -> &str;
    fn settings_bg_type(&self) -> &str;
    fn settings_bg_none(&self) -> &str;
    fn settings_bg_color(&self) -> &str;
    fn settings_bg_image(&self) -> &str;
    fn settings_pick_color(&self) -> &str;
    fn settings_pick_image(&self) -> &str;
    fn settings_apply(&self) -> &str;
    fn settings_cancel(&self) -> &str;

    // History / 历史
    fn history_title(&self) -> &str;
    fn history_no_data(&self) -> &str;
    fn history_export(&self) -> &str;
    fn history_export_title(&self) -> &str;
    fn history_export_btn(&self) -> &str;
    fn history_records(&self) -> &str;
    fn history_db_size(&self) -> &str;
    fn history_retention(&self) -> &str;

    // Time ranges / 时间范围
    fn range_5min(&self) -> &str;
    fn range_30min(&self) -> &str;
    fn range_1hour(&self) -> &str;
    fn range_6hours(&self) -> &str;
    fn range_24hours(&self) -> &str;
    fn range_all(&self) -> &str;

    // Metrics labels / 指标标签
    fn label_cpu(&self) -> &str;
    fn label_memory(&self) -> &str;
    fn label_network(&self) -> &str;
    fn label_disk(&self) -> &str;
    fn label_gpu(&self) -> &str;
    fn label_temp(&self) -> &str;
    fn label_net_rx(&self) -> &str;
    fn label_net_tx(&self) -> &str;

    // Plugins / 插件
    fn plugins_tab(&self) -> &str;
    fn plugins_dir(&self) -> &str;
    fn plugins_scan(&self) -> &str;
    fn plugins_none(&self) -> &str;
    fn plugins_count(&self) -> &str;
}

// ============================================================
// Chinese locale / 中文
// ============================================================
pub struct ChineseLocale;

impl Locale for ChineseLocale {
    fn app_name(&self) -> &str { "LinuxMonitor" }
    fn app_title(&self) -> &str { "Linux 硬件监控" }

    fn menu_hide(&self) -> &str { "隐藏窗口" }
    fn menu_skins(&self) -> &str { "皮肤切换" }
    fn menu_history(&self) -> &str { "历史数据" }
    fn menu_settings(&self) -> &str { "系统设置" }
    fn menu_quit(&self) -> &str { "退出" }

    fn skin_horizontal(&self) -> &str { "横条模式" }
    fn skin_vertical(&self) -> &str { "竖排列表" }
    fn skin_compact(&self) -> &str { "紧凑模式" }

    fn settings_title(&self) -> &str { "LinuxMonitor — 设置" }
    fn settings_tab_general(&self) -> &str { "  常规  " }
    fn settings_tab_monitors(&self) -> &str { " 监控项 " }
    fn settings_tab_appearance(&self) -> &str { "  外观  " }
    fn settings_general_title(&self) -> &str { "轮询与行为" }
    fn settings_active_interval(&self) -> &str { "活跃轮询间隔 (ms)" }
    fn settings_active_desc(&self) -> &str { "窗口可见时的数据采集速度" }
    fn settings_bg_interval(&self) -> &str { "后台轮询间隔 (ms)" }
    fn settings_bg_desc(&self) -> &str { "窗口隐藏后的数据采集速度" }
    fn settings_idle_interval(&self) -> &str { "空闲轮询间隔 (ms)" }
    fn settings_idle_desc(&self) -> &str { "系统空闲时的数据采集速度" }
    fn settings_monitors_title(&self) -> &str { "启用的监控项" }
    fn settings_monitor_cpu(&self) -> &str { "CPU 处理器" }
    fn settings_monitor_cpu_desc(&self) -> &str { "使用率与频率" }
    fn settings_monitor_memory(&self) -> &str { "内存" }
    fn settings_monitor_memory_desc(&self) -> &str { "RAM 与 Swap 使用量" }
    fn settings_monitor_network(&self) -> &str { "网络" }
    fn settings_monitor_network_desc(&self) -> &str { "上传与下载速率" }
    fn settings_monitor_disk(&self) -> &str { "磁盘" }
    fn settings_monitor_disk_desc(&self) -> &str { "存储空间与 I/O" }
    fn settings_monitor_gpu(&self) -> &str { "GPU 显卡" }
    fn settings_monitor_gpu_desc(&self) -> &str { "显卡使用率与温度" }
    fn settings_monitor_thermal(&self) -> &str { "温度传感器" }
    fn settings_monitor_thermal_desc(&self) -> &str { "CPU/GPU/主板 温度" }
    fn settings_appearance_title(&self) -> &str { "外观与样式" }
    fn settings_skin(&self) -> &str { "皮肤:" }
    fn settings_font_size(&self) -> &str { "字体大小" }
    fn settings_font_desc(&self) -> &str { "显示文字的基础字号" }
    fn settings_opacity(&self) -> &str { "透明度" }
    fn settings_opacity_desc(&self) -> &str { "窗口透明度 (0.3–1.0)" }
    fn settings_always_on_top(&self) -> &str { "窗口置顶" }
    fn settings_bg_type(&self) -> &str { "背景类型:" }
    fn settings_bg_none(&self) -> &str { "默认" }
    fn settings_bg_color(&self) -> &str { "纯色" }
    fn settings_bg_image(&self) -> &str { "图片" }
    fn settings_pick_color(&self) -> &str { "选择颜色" }
    fn settings_pick_image(&self) -> &str { "选择图片" }
    fn settings_apply(&self) -> &str { "应用" }
    fn settings_cancel(&self) -> &str { "取消" }

    fn history_title(&self) -> &str { "性能历史" }
    fn history_no_data(&self) -> &str { "暂无历史数据。数据每 60 秒自动采集一次。" }
    fn history_export(&self) -> &str { "导出 CSV" }
    fn history_export_title(&self) -> &str { "导出历史数据" }
    fn history_export_btn(&self) -> &str { "导出" }
    fn history_records(&self) -> &str { "记录数" }
    fn history_db_size(&self) -> &str { "数据库大小" }
    fn history_retention(&self) -> &str { "保留天数: 7 天" }

    fn range_5min(&self) -> &str { "5 分钟" }
    fn range_30min(&self) -> &str { "30 分钟" }
    fn range_1hour(&self) -> &str { "1 小时" }
    fn range_6hours(&self) -> &str { "6 小时" }
    fn range_24hours(&self) -> &str { "24 小时" }
    fn range_all(&self) -> &str { "全部" }

    fn label_cpu(&self) -> &str { "CPU" }
    fn label_memory(&self) -> &str { "MEM" }
    fn label_network(&self) -> &str { "NET" }
    fn label_disk(&self) -> &str { "DSK" }
    fn label_gpu(&self) -> &str { "GPU" }
    fn label_temp(&self) -> &str { "🌡" }
    fn label_net_rx(&self) -> &str { "↓" }
    fn label_net_tx(&self) -> &str { "↑" }

    fn plugins_tab(&self) -> &str { "  插件  " }
    fn plugins_dir(&self) -> &str { "插件目录: ~/.config/linux-monitor/plugins/" }
    fn plugins_scan(&self) -> &str { "重新扫描" }
    fn plugins_none(&self) -> &str { "暂无插件" }
    fn plugins_count(&self) -> &str { "已加载 {} 个插件" }
}

// ============================================================
// English locale / 英文 (fallback)
// ============================================================
pub struct EnglishLocale;

impl Locale for EnglishLocale {
    fn app_name(&self) -> &str { "LinuxMonitor" }
    fn app_title(&self) -> &str { "Linux Hardware Monitor" }

    fn menu_hide(&self) -> &str { "Hide Window" }
    fn menu_skins(&self) -> &str { "Skins" }
    fn menu_history(&self) -> &str { "History" }
    fn menu_settings(&self) -> &str { "Settings" }
    fn menu_quit(&self) -> &str { "Quit" }

    fn skin_horizontal(&self) -> &str { "Horizontal Bar" }
    fn skin_vertical(&self) -> &str { "Vertical List" }
    fn skin_compact(&self) -> &str { "Compact Mode" }

    fn settings_title(&self) -> &str { "LinuxMonitor — Settings" }
    fn settings_tab_general(&self) -> &str { "  General  " }
    fn settings_tab_monitors(&self) -> &str { " Monitors " }
    fn settings_tab_appearance(&self) -> &str { "Appearance" }
    fn settings_general_title(&self) -> &str { "Polling & Behavior" }
    fn settings_active_interval(&self) -> &str { "Active interval (ms)" }
    fn settings_active_desc(&self) -> &str { "Polling speed when window is visible" }
    fn settings_bg_interval(&self) -> &str { "Background interval (ms)" }
    fn settings_bg_desc(&self) -> &str { "Polling speed when window is hidden" }
    fn settings_idle_interval(&self) -> &str { "Idle interval (ms)" }
    fn settings_idle_desc(&self) -> &str { "Polling speed when system is idle" }
    fn settings_monitors_title(&self) -> &str { "Enable / Disable Monitors" }
    fn settings_monitor_cpu(&self) -> &str { "CPU" }
    fn settings_monitor_cpu_desc(&self) -> &str { "Processor usage & frequency" }
    fn settings_monitor_memory(&self) -> &str { "Memory" }
    fn settings_monitor_memory_desc(&self) -> &str { "RAM & Swap usage" }
    fn settings_monitor_network(&self) -> &str { "Network" }
    fn settings_monitor_network_desc(&self) -> &str { "Upload & download speed" }
    fn settings_monitor_disk(&self) -> &str { "Disk" }
    fn settings_monitor_disk_desc(&self) -> &str { "Storage space & I/O" }
    fn settings_monitor_gpu(&self) -> &str { "GPU" }
    fn settings_monitor_gpu_desc(&self) -> &str { "Graphics card usage & temperature" }
    fn settings_monitor_thermal(&self) -> &str { "Thermal" }
    fn settings_monitor_thermal_desc(&self) -> &str { "CPU/GPU/MB temperature sensors" }
    fn settings_appearance_title(&self) -> &str { "Look & Feel" }
    fn settings_skin(&self) -> &str { "Skin:" }
    fn settings_font_size(&self) -> &str { "Font size" }
    fn settings_font_desc(&self) -> &str { "Base font size for display" }
    fn settings_opacity(&self) -> &str { "Opacity" }
    fn settings_opacity_desc(&self) -> &str { "Window transparency (0.3–1.0)" }
    fn settings_always_on_top(&self) -> &str { "Always on top" }
    fn settings_bg_type(&self) -> &str { "Background:" }
    fn settings_bg_none(&self) -> &str { "Default" }
    fn settings_bg_color(&self) -> &str { "Solid Color" }
    fn settings_bg_image(&self) -> &str { "Image" }
    fn settings_pick_color(&self) -> &str { "Pick Color" }
    fn settings_pick_image(&self) -> &str { "Pick Image" }
    fn settings_apply(&self) -> &str { "Apply" }
    fn settings_cancel(&self) -> &str { "Cancel" }

    fn history_title(&self) -> &str { "Performance History" }
    fn history_no_data(&self) -> &str { "No history data yet. Data is collected every 60 seconds." }
    fn history_export(&self) -> &str { "Export CSV" }
    fn history_export_title(&self) -> &str { "Export History Data" }
    fn history_export_btn(&self) -> &str { "Export" }
    fn history_records(&self) -> &str { "Total records" }
    fn history_db_size(&self) -> &str { "Database size" }
    fn history_retention(&self) -> &str { "Retention: 7 days" }

    fn range_5min(&self) -> &str { "5 min" }
    fn range_30min(&self) -> &str { "30 min" }
    fn range_1hour(&self) -> &str { "1 hour" }
    fn range_6hours(&self) -> &str { "6 hours" }
    fn range_24hours(&self) -> &str { "24 hours" }
    fn range_all(&self) -> &str { "All" }

    fn label_cpu(&self) -> &str { "CPU" }
    fn label_memory(&self) -> &str { "MEM" }
    fn label_network(&self) -> &str { "NET" }
    fn label_disk(&self) -> &str { "DSK" }
    fn label_gpu(&self) -> &str { "GPU" }
    fn label_temp(&self) -> &str { "🌡" }
    fn label_net_rx(&self) -> &str { "↓" }
    fn label_net_tx(&self) -> &str { "↑" }

    fn plugins_tab(&self) -> &str { " Plugins " }
    fn plugins_dir(&self) -> &str { "Plugin dir: ~/.config/linux-monitor/plugins/" }
    fn plugins_scan(&self) -> &str { "Rescan" }
    fn plugins_none(&self) -> &str { "No plugins found" }
    fn plugins_count(&self) -> &str { "Loaded {} plugins" }
}

/// Global locale instance
use std::sync::LazyLock;
pub static L: LazyLock<Box<dyn Locale>> = LazyLock::new(|| {
    // Detect language from environment
    let lang = std::env::var("LANG").unwrap_or_default();
    if lang.starts_with("zh") {
        Box::new(ChineseLocale)
    } else {
        Box::new(EnglishLocale)
    }
});
