// TODO: Test the fuck out of this function
pub fn calculate_round_length(tick_number: i32, interval_sec: u32) -> u64 {
  let tick = f64::from(tick_number);
  let interval = f64::from(interval_sec) / 60.0;
  let delay = 2.0 * interval * (-2.0 * tick / interval).exp() + interval;
  (delay * 60.0) as u64
}
