pub mod aws;

#[allow(unused)]
pub mod hetzner;

#[derive(Debug)]
pub enum BuildCleanupData {
  /// Nothing to clean up
  Server,
  /// Clean up AWS instance
  Aws { instance_id: String, region: String },
}
