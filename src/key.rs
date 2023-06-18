use sha2::{Digest, Sha256};

pub fn generate_flag(
  tick: i32,
  team_id: i32,
  service_name: impl AsRef<str>,
) -> String {
  let service_name = service_name.as_ref();
  let payload = (tick, team_id, service_name);
  // let payload = format!("tick={}|team={}|svc={}", tick, team_id, service_name);

  let mut hasher = Sha256::new();
  hasher.input(&payload);
  let result = hasher.result();
  format!("flag{{{}|hmac={:x}}}", payload, result)
}
