pub mod aws;

#[derive(Debug)]
pub enum BuildCleanupData {
  /// Nothing to clean up
  Server,
  /// Cleanup Periphery connection
  Url,
  /// Clean up AWS instance
  Aws { instance_id: String, region: String },
}
