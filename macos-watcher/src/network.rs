use std::process::Command;
use std::collections::HashMap;

pub fn check_nettop_for_pid(target: &str, cache: &mut HashMap<String, (u64, u64)>) {
  let output = Command::new("nettop")
      .args(&["-P", "-J", "bytes_in,bytes_out", "-x", "-l", "1"])
      .output()
      .expect("Failed to run nettop");

  let stdout = String::from_utf8_lossy(&output.stdout);

  for line in stdout.lines() {
      if line.to_lowercase().contains(target) {
          let parts: Vec<&str> = line.split_whitespace().collect();

          if parts.len() >= 3 {
              let key = parts[0]; // e.g. example-mac-app.17759
              let bytes_in = parts[1].parse::<u64>().unwrap_or(0);
              let bytes_out = parts[2].parse::<u64>().unwrap_or(0);

              let previous = cache.get(key).copied().unwrap_or((0, 0));
              if bytes_in != previous.0 || bytes_out != previous.1 {
                  let delta_in = bytes_in.saturating_sub(previous.0);
                  let delta_out = bytes_out.saturating_sub(previous.1);

                  println!(
                      "📡 {} ↑ {} B ↓ {} B (Δ ↑ {} ↓ {})",
                      key, bytes_out, bytes_in, delta_out, delta_in
                  );

                  cache.insert(key.to_string(), (bytes_in, bytes_out));
              }
          }
      }
  }
}
