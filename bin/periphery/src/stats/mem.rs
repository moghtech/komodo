/// Memory breakdown, all fields in bytes.
#[derive(Clone, Copy)]
pub struct MemBreakdown {
  /// Total physical memory.
  pub total: u64,
  /// Free memory (`MemFree`).
  pub free: u64,
  /// Reclaimable page cache + buffers.
  pub buff_cache: u64,
  /// Full ZFS ARC size (`0` when ZFS is not present).
  pub zfs_arc: u64,
  /// Real application-used memory: `(total - available) - zfs_arc`.
  pub used: u64,
}

/// Read the full memory breakdown. Prefers `/proc` on Linux, falling
/// back to `sysinfo` when the files are unavailable (non-Linux, or read
/// failure).
pub fn read(system: &sysinfo::System) -> MemBreakdown {
  #[cfg(target_os = "linux")]
  if let Some(breakdown) = read_linux() {
    return breakdown;
  }
  read_sysinfo(system)
}

/// Cross-platform fallback using sysinfo. No buff/cache or ARC breakdown
/// is available here.
fn read_sysinfo(system: &sysinfo::System) -> MemBreakdown {
  let total = system.total_memory();
  let available = system.available_memory();
  MemBreakdown {
    total,
    free: system.free_memory(),
    buff_cache: 0,
    zfs_arc: 0,
    used: total.saturating_sub(available),
  }
}

/// Compute the breakdown from the Linux `/proc` files. Returns `None` if
/// `/proc/meminfo` can't be read (then we fall back to sysinfo).
///
/// `MemAvailable` does not count the ZFS ARC as reclaimable, so a large
/// ARC would inflate "used" and cause false high-memory alerts. ARC is
/// reclaimable, so we subtract it from used and report it separately.
#[cfg(target_os = "linux")]
fn read_linux() -> Option<MemBreakdown> {
  let meminfo =
    parse_meminfo(&std::fs::read_to_string("/proc/meminfo").ok()?)?;
  // Absent when ZFS isn't loaded => arc stays zero.
  let zfs_arc =
    std::fs::read_to_string("/proc/spl/kstat/zfs/arcstats")
      .ok()
      .map(|c| parse_zfs_arc_size(&c))
      .unwrap_or(0);

  // Cached + SReclaimable + Buffers - Shmem
  let buff_cache = meminfo
    .cached
    .saturating_add(meminfo.s_reclaimable)
    .saturating_add(meminfo.buffers)
    .saturating_sub(meminfo.shmem);

  let raw_used = meminfo.total.saturating_sub(meminfo.available);

  Some(MemBreakdown {
    total: meminfo.total,
    free: meminfo.free,
    buff_cache,
    zfs_arc,
    used: raw_used.saturating_sub(zfs_arc),
  })
}

#[cfg(target_os = "linux")]
#[derive(Default)]
struct MemInfo {
  total: u64,
  free: u64,
  available: u64,
  buffers: u64,
  cached: u64,
  s_reclaimable: u64,
  shmem: u64,
}

/// Parse the relevant fields from `/proc/meminfo` contents. Values there
/// are in kibibytes; we convert to bytes. Returns `None` if `MemTotal` is
/// missing (i.e. not a real meminfo file).
#[cfg(target_os = "linux")]
fn parse_meminfo(contents: &str) -> Option<MemInfo> {
  let mut info = MemInfo::default();
  let mut have_total = false;
  let mut have_available = false;

  for line in contents.lines() {
    let Some((key, rest)) = line.split_once(':') else {
      continue;
    };
    // rest looks like "  12345 kB"
    let Some(kb) = rest.split_whitespace().next() else {
      continue;
    };
    let Ok(kb) = kb.parse::<u64>() else {
      continue;
    };
    let bytes = kb.saturating_mul(1024);
    match key {
      "MemTotal" => {
        info.total = bytes;
        have_total = true;
      }
      "MemFree" => info.free = bytes,
      "MemAvailable" => {
        info.available = bytes;
        have_available = true;
      }
      "Buffers" => info.buffers = bytes,
      "Cached" => info.cached = bytes,
      "SReclaimable" => info.s_reclaimable = bytes,
      "Shmem" => info.shmem = bytes,
      _ => {}
    }
  }

  if !have_total {
    return None;
  }
  // Older kernels (<3.14) lack MemAvailable; fall back to free.
  if !have_available {
    info.available = info.free;
  }

  Some(info)
}

/// Parse the ZFS ARC `size` (bytes) from `/proc/spl/kstat/zfs/arcstats`
/// contents. The line looks like: `size  4  4294967296` (name, type, data).
/// Returns `0` if not found.
#[cfg(target_os = "linux")]
fn parse_zfs_arc_size(contents: &str) -> u64 {
  for line in contents.lines() {
    let mut fields = line.split_whitespace();
    if fields.next() == Some("size") {
      return fields.nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
    }
  }
  0
}
