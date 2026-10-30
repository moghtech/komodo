use std::time::Instant;

/// CPU usage of the LXC container, from its cgroup v2 `cpu.stat`.
/// lxcfs serves `/proc/stat` inside the container, but does not
/// scope CPU time to it, so `sysinfo` reports the host's cores.
pub struct CgroupCpu {
  /// Previous sample: when it was taken, and `usage_usec` at that time.
  last: (Instant, u64),
  usage_perc: Option<f32>,
}

impl CgroupCpu {
  pub fn detect() -> Option<CgroupCpu> {
    if !proc_stat_is_lxcfs() {
      return None;
    }
    Some(CgroupCpu {
      last: (Instant::now(), read_usage_usec()?),
      usage_perc: None,
    })
  }

  /// The cgroup's CPU time since the previous sample, relative to the
  /// CPUs the container may use. Keeps the previous value on a failed
  /// read.
  pub fn refresh(&mut self) {
    let Some(usage) = read_usage_usec() else {
      return;
    };
    let now = Instant::now();
    let (then, prev) = self.last;
    let elapsed_usec = now.duration_since(then).as_micros() as f64;
    // Follows the cores the container is pinned to.
    let cpus = std::thread::available_parallelism()
      .map(|n| n.get())
      .unwrap_or(1) as f64;
    if elapsed_usec > 0.0 {
      let used_usec = usage.saturating_sub(prev) as f64;
      let perc = used_usec / (elapsed_usec * cpus) * 100.0;
      self.usage_perc = Some(perc.min(100.0) as f32);
    }
    self.last = (now, usage);
  }

  /// `None` until the first refresh.
  pub fn usage_perc(&self) -> Option<f32> {
    self.usage_perc
  }
}

fn read_usage_usec() -> Option<u64> {
  parse_usage_usec(
    &std::fs::read_to_string("/sys/fs/cgroup/cpu.stat").ok()?,
  )
}

/// Parse `usage_usec` from cgroup v2 `cpu.stat` contents. The line
/// looks like: `usage_usec 123456789`.
fn parse_usage_usec(contents: &str) -> Option<u64> {
  contents.lines().find_map(|line| {
    line.strip_prefix("usage_usec ")?.trim().parse().ok()
  })
}

fn proc_stat_is_lxcfs() -> bool {
  std::fs::read_to_string("/proc/self/mountinfo")
    .map(|c| mountinfo_has_lxcfs_proc_stat(&c))
    .unwrap_or(false)
}

fn mountinfo_has_lxcfs_proc_stat(contents: &str) -> bool {
  contents.lines().any(|line| {
    // Field 5 is the mount point; fs type follows the " - "
    // separator.
    let mount_point = line.split_whitespace().nth(4);
    let fs_type = line
      .split_once(" - ")
      .and_then(|(_, rest)| rest.split_whitespace().next());
    mount_point == Some("/proc/stat")
      && fs_type.is_some_and(|t| t.ends_with("lxcfs"))
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detects_lxcfs_proc_stat() {
    let container = "462 445 0:60 /proc/stat /proc/stat rw,nosuid,nodev,relatime shared:331 master:79 - fuse.lxcfs lxcfs rw,user_id=0,group_id=0,allow_other\n";
    assert!(mountinfo_has_lxcfs_proc_stat(container));
    let host = "22 1 0:21 / /proc rw - proc proc rw\n";
    assert!(!mountinfo_has_lxcfs_proc_stat(host));
  }

  #[test]
  fn parses_usage_usec() {
    let cpu_stat = "usage_usec 123456789\nuser_usec 100\n";
    assert_eq!(parse_usage_usec(cpu_stat), Some(123456789));
    assert_eq!(parse_usage_usec(""), None);
  }
}
