use std::process::Command;

use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq, Default)]
pub struct SmartStatus {
  pub passed: bool,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Default)]
pub struct PowerOnTime {
  #[serde(default)]
  pub hours: u64,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Default)]
pub struct Temperature {
  #[serde(default)]
  pub current: u64,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct SmartReport {
  pub smart_status: SmartStatus,
  #[serde(default)]
  pub power_on_time: PowerOnTime,
  #[serde(default)]
  pub temperature: Temperature,
}

/// Parse smartctl JSON output string to extract SMART report
fn parse_smart_report(json_str: &str) -> Option<SmartReport> {
  serde_json::from_str(json_str).ok()
}

/// given a device path it will try to get the SMART data about a disk
pub fn get_smart_data(device_path: &str) -> Option<SmartReport> {
  let output = Command::new("smartctl")
    .args(["-a", "-j", device_path])
    .output()
    .ok()?;
  if output.status.success() || !output.stdout.is_empty() {
    let json_str = String::from_utf8_lossy(&output.stdout);
    return parse_smart_report(&json_str);
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
  fn test_parse_healthy_drive_full_metrics() {
    let json = r#"{
      "smart_status": { "passed": true },
      "power_on_time": { "hours": 14200 },
      "temperature": { "current": 34 }
    }"#;
    assert_eq!(
      parse_smart_report(json),
      Some(SmartReport {
        smart_status: SmartStatus { passed: true },
        power_on_time: PowerOnTime { hours: 14200 },
        temperature: Temperature { current: 34 },
      })
    );
  }

  #[test]
  fn test_parse_missing_optional_fields() {
    let json = r#"{
      "smart_status": { "passed": true }
    }"#;
    assert_eq!(
      parse_smart_report(json),
      Some(SmartReport {
        smart_status: SmartStatus { passed: true },
        power_on_time: PowerOnTime { hours: 0 },
        temperature: Temperature { current: 0 },
      })
    );
  }

  #[test]
  fn test_parse_failing_drive() {
    let json = r#"{
      "smart_status": { "passed": false },
      "power_on_time": { "hours": 50000 },
      "temperature": { "current": 55 }
    }"#;
    let report = parse_smart_report(json).unwrap();
    assert!(!report.smart_status.passed);
  }

  #[test]
  fn test_parse_unsupported_or_invalid() {
    let json = r#"{ "device": { "name": "/dev/loop0" } }"#;
    assert_eq!(parse_smart_report(json), None);
    assert_eq!(parse_smart_report("invalid json"), None);
  }
}
