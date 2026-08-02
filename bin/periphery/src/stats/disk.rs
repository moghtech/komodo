use std::process::Command;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SmartStatus {
  pub passed: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct SmartReport {
  pub smart_status: Option<SmartStatus>,
}

/// Parse smartctl JSON output string to extract health status.
pub fn parse_smart_status(json_str: &str) -> Option<bool> {
  let report: SmartReport = serde_json::from_str(json_str).ok()?;
  report.smart_status?.passed
}

/// given a device path it will try to get the SMART data about it returning true/false to indicate healthy or not otherwise it returns None
pub fn get_smart_data(device_path: &str) -> Option<bool> {
  let output = Command::new("smartctl")
    .args(["-H", device_path, "-j"])
    .output()
    .ok()?;
  if output.status.success() || !output.stdout.is_empty() {
    let json_str = String::from_utf8_lossy(&output.stdout);
    return parse_smart_status(&json_str);
  }
  None
}

#[derive(Deserialize, Debug)]
pub struct LsblkOutput {
  pub blockdevices: Vec<LsblkDevice>,
}

#[derive(Deserialize, Debug)]
pub struct LsblkDevice {
  pub name: String,
  /// 'type' is a reserved keyword in Rust, so rename it
  #[serde(rename = "type")]
  pub device_type: String,
  /// Leaf nodes will not have children, so default to empty Vec
  #[serde(default)]
  pub children: Vec<LsblkDevice>,
}

impl LsblkDevice {
  /// Recursively traverse the tree to find the physical disk name ('type' == "disk")
  pub fn find_physical_disk(&self) -> Option<String> {
    if self.device_type == "disk" {
      return Some(self.name.clone());
    }
    for child in &self.children {
      if let Some(disk_name) = child.find_physical_disk() {
        return Some(disk_name);
      }
    }
    None
  }
}

pub fn volume_to_device_mapper(mapper: &str) -> Option<String> {
  let output = Command::new("lsblk")
    .args(["-s", "-J", mapper])
    .output()
    .ok()?;
  if !output.status.success() || output.stdout.is_empty() {
    return None;
  }
  let output: Option<LsblkOutput> =
    serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
      .ok()?;
  let output = output.unwrap();
  let disk_name = output
    .blockdevices
    .iter()
    .find_map(|dev| dev.find_physical_disk())?;

  Some(format!("/dev/{disk_name}"))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_healthy_drive() {
    let json = r#"{
      "smart_status": {
        "passed": true
      }
    }"#;
    assert_eq!(parse_smart_status(json), Some(true));
  }

  #[test]
  fn test_parse_failing_drive() {
    let json = r#"{
      "smart_status": {
        "passed": false
      }
    }"#;
    assert_eq!(parse_smart_status(json), Some(false));
  }

  #[test]
  fn test_parse_nvme_drive() {
    let json = r#"{
      "smart_status": {
        "passed": true,
        "nvme": {
          "value": 0
        }
      }
    }"#;
    assert_eq!(parse_smart_status(json), Some(true));
  }

  #[test]
  fn test_parse_unsupported_or_missing_smart() {
    let json = r#"{
      "device": {
        "name": "/dev/loop0",
        "type": "loop"
      }
    }"#;
    assert_eq!(parse_smart_status(json), None);
  }

  #[test]
  fn test_parse_invalid_json() {
    assert_eq!(parse_smart_status("invalid json string"), None);
  }
}
