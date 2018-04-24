
pub fn compute_speed_in_mbps(total_bytes: u64, total_time_in_millis: u64) -> f64 {
    let speed = (total_bytes as f64 * 8.0) / (total_time_in_millis as f64 / 1000.0);
    speed / (1000.0 * 1000.0)
}