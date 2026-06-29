// 温度告警插件: GPU > 80°C 或 CPU > 90°C 时触发
if gpu_temp > 80.0 {
    ALERT.call("GPU温度告警", 80.0, gpu_temp, "GPU 温度过高，请检查散热");
}
if cpu_temp > 90.0 {
    ALERT.call("CPU温度告警", 90.0, cpu_temp, "CPU 温度过高，请检查散热");
}
