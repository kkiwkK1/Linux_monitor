use super::ThermalMetrics;
use std::fs;
use std::path::Path;

pub fn collect_thermal() -> Vec<ThermalMetrics> {
    let mut metrics = Vec::new();

    // Read from /sys/class/thermal
    let thermal_path = Path::new("/sys/class/thermal");
    if !thermal_path.exists() {
        return metrics;
    }

    if let Ok(entries) = fs::read_dir(thermal_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let zone_name = entry.file_name().to_string_lossy().to_string();

            // Skip cooling devices
            if zone_name.starts_with("cooling_device") {
                continue;
            }

            let mut metric = ThermalMetrics {
                name: zone_name.clone(),
                sensor_type: String::new(),
                ..Default::default()
            };

            // Read sensor type
            if let Ok(stype) = fs::read_to_string(path.join("type")) {
                metric.name = stype.trim().to_string();
                metric.sensor_type = stype.trim().to_string();
            }

            // Read temperature (millidegrees C)
            if let Ok(temp_str) = fs::read_to_string(path.join("temp")) {
                metric.temperature_c = temp_str.trim().parse::<f32>().unwrap_or(0.0) / 1000.0;
            }

            // Read trip points
            if let Ok(trip) = fs::read_to_string(path.join("trip_point_0_temp")) {
                metric.high_c = Some(trip.trim().parse::<f32>().unwrap_or(0.0) / 1000.0);
            }
            if let Ok(trip) = fs::read_to_string(path.join("trip_point_2_temp")) {
                metric.crit_c = Some(trip.trim().parse::<f32>().unwrap_or(0.0) / 1000.0);
            }

            if metric.temperature_c > 0.0 {
                metrics.push(metric);
            }
        }
    }

    // Also check hwmon sensors for more detailed readings
    let hwmon_path = Path::new("/sys/class/hwmon");
    if hwmon_path.exists() {
        if let Ok(entries) = fs::read_dir(hwmon_path) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Get hwmon name
                let hw_name = fs::read_to_string(path.join("name"))
                    .unwrap_or_default()
                    .trim()
                    .to_string();

                // Read all temp inputs
                for i in 1..=10 {
                    let temp_label_path = path.join(format!("temp{}_label", i));
                    let temp_input_path = path.join(format!("temp{}_input", i));

                    if temp_input_path.exists() {
                        if let Ok(temp_str) = fs::read_to_string(&temp_input_path) {
                            let temp = temp_str.trim().parse::<f32>().unwrap_or(0.0) / 1000.0;
                            if temp > 0.0 {
                                let label = fs::read_to_string(&temp_label_path)
                                    .unwrap_or_default()
                                    .trim()
                                    .to_string();
                                let name = if label.is_empty() {
                                    format!("{}/temp{}", hw_name, i)
                                } else {
                                    format!("{}: {}", hw_name, label)
                                };

                                metrics.push(ThermalMetrics {
                                    name,
                                    temperature_c: temp,
                                    sensor_type: hw_name.clone(),
                                    high_c: None,
                                    crit_c: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    metrics
}
