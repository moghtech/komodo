//! Proves both halves of the Build-image TOML fix.
//!
//! 1. An empty `params` table is dropped by the sync's serializer options and
//!    the result cannot be deserialized back, because `DeploymentImage` is
//!    adjacently tagged. This is the bug.
//! 2. Restoring `image.params = {}`, the way `Deployment::push_additional` now
//!    does, makes the emitted TOML round trip.
use komodo_client::entities::deployment::DeploymentImage;

const OPTIONS: toml_pretty::Options = toml_pretty::Options {
  tab: "  ",
  skip_empty_string: true,
  skip_empty_object: true,
  max_inline_array_length: 30,
  inline_array: false,
};

#[derive(Debug, serde::Deserialize)]
struct Wrapper {
  image: DeploymentImage,
}

/// What `edit_config_object` leaves for a Build image whose build is unset (or
/// was blanked) and whose version is 0.0.0: `build_id` renamed to `build`, and
/// `version` removed by the `version.is_none()` branch.
fn emitted_config() -> String {
  let config = serde_json::json!({
    "image": { "type": "Build", "params": { "build": "" } },
  });
  toml_pretty::to_string(&config, OPTIONS).expect("serialize")
}

#[test]
fn empty_build_params_is_dropped_and_cannot_be_read_back() {
  let toml = emitted_config();
  assert_eq!(toml.trim(), "image.type = \"Build\"");
  let err = toml::from_str::<Wrapper>(&toml)
    .expect_err("a params-less Build image must not deserialize");
  assert!(
    err.to_string().contains("params"),
    "expected a missing-params error, got: {err}"
  );
}

#[test]
fn restoring_the_empty_params_table_round_trips() {
  // Exactly what `Deployment::push_additional` appends.
  let toml = format!("{}\nimage.params = {{}}", emitted_config());
  let parsed: Wrapper = toml::from_str(&toml)
    .expect("with params restored it must deserialize");
  match parsed.image {
    DeploymentImage::Build { build_id, version } => {
      assert!(build_id.is_empty());
      assert!(version.is_none());
    }
    other => panic!("wrong variant: {other:?}"),
  }
}

#[test]
fn a_populated_build_image_is_unaffected() {
  let config = serde_json::json!({
    "image": { "type": "Build", "params": { "build": "my-build" } },
  });
  let toml =
    toml_pretty::to_string(&config, OPTIONS).expect("serialize");
  assert!(toml.contains("image.params.build = \"my-build\""));
  let parsed: Wrapper = toml::from_str(&toml).expect("round trip");
  match parsed.image {
    DeploymentImage::Build { build_id, .. } => {
      assert_eq!(build_id, "my-build")
    }
    other => panic!("wrong variant: {other:?}"),
  }
}
