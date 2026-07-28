pub mod aws;

#[derive(Debug)]
pub enum BuildCleanupData {
  /// Store the builder id if it needs
  /// to be removed from builder_usage_cache.
  Server(Option<String>),
  /// Cleanup Periphery connection
  Url,
  /// Clean up AWS instance
  Aws { instance_id: String, region: String },
}
