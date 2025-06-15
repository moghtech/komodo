# Version Upgrades

Most version upgrades only require a redeployment of the Core container after pulling the latest version, and are fully backward compatible with the periphery clients, which may be updated later on as convenient. This is the default, and will be the case unless specifically mentioned in the [version release notes](https://github.com/moghtech/komodo/releases).

Some Core API upgrades may change behavior such as building / cloning, and require updating the Periphery binaries to match the Core version before this functionality can be restored. This will be specifically mentioned in the release notes.