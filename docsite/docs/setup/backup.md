---
sidebar_label: Back Up and Restore
---

# Back Up And Restore

Komodo can automatically back up its database on a schedule and restore from any previous
snapshot. Backups are gzip-compressed and stored on disk or a remote server. By default, the most
recent 14 backups are kept. The backup and restore operations are handled by the
[Komodo CLI](../ecosystem/cli), which is packaged in the Core image for convenience.

## Scheduled Backup

New installs (`v1.19.0+`) automatically create the
**Backup Core Database** [Procedure](../automate/procedures#procedures), scheduled daily.
If you do not have it, this is the TOML:

```toml
[[procedure]]
name = "Backup Core Database"
description = "Triggers the Core database backup at the scheduled time."
tags = ["system"]
config.schedule = "Every day at 01:00"

[[procedure.config.stage]]
name = "Stage 1"
enabled = true
executions = [
  { execution.type = "BackupCoreDatabase", execution.params = {}, enabled = true }
]
```

:::info
You are also able to integrate `BackupCoreDatabase` into other Procedures, for example to trigger
this process before launching a backup container. There is nothing special about this Procedure.
It is created by default for guidance and convenience.
:::

## Backups

When Komodo takes a database backup, it creates a **folder named for the time the backup was
taken**, and writes the gzip-compressed documents to files in that folder.
To store backups on disk, **mount a host path to `/backups`** in the Komodo Core container.

Due to its larger size and lower importance, the `Stats` collection, which contains historical
server CPU, memory, and disk usage, is not included in dated backups. Only the latest `Stats`
snapshot is kept at the top level of the backup folder.

To prevent unbounded growth, the backup process prunes older backup folders and keeps only the most
recent 14. To change this number, set `max_backups` (`KOMODO_CLI_MAX_BACKUPS`) in
`core.config.toml`, `komodo.cli.toml`, or in the Core container environment.

```text
# Folder structure
/backups
| 2025-08-12_03-00-01
| | Action.gz
| | Alerter.gz
| | ...
| 2025-08-13_03-00-01
| 2025-08-14_03-00-01
| ...
| Stats.gz
```

:::warning
Komodo does not currently provide built-in encryption. Encrypt the files before backing up
remotely if your backup solution does not support that natively.
:::

## Remote Backups

Since database backup is a function of the [Komodo CLI](../ecosystem/cli), you can also back up
directly to a remote server using the `ghcr.io/moghtech/komodo-cli` image. This service runs the
backup once and then exits, so scheduled execution should still happen through a Procedure or
Action:

```yaml
services:
  cli:
    image: ghcr.io/moghtech/komodo-cli
    command: km database backup -y
    volumes:
      - /path/to/komodo/backups:/backups
    environment:
      ## Database port must be reachable.
      KOMODO_DATABASE_ADDRESS: komodo.example.com:27017
      KOMODO_DATABASE_USERNAME: <db username>
      KOMODO_DATABASE_PASSWORD: <db password>
      KOMODO_DATABASE_DB_NAME: komodo
      KOMODO_CLI_MAX_BACKUPS: 30 # set to your preference
```

## Restore

The Komodo CLI handles database restores as well.

```yaml
services:
  cli:
    image: ghcr.io/moghtech/komodo-cli
    ## Optionally specify a specific folder with `--restore-folder`,
    ## otherwise restores the most recent backup.
    command: km database restore -y # --restore-folder 2025-08-14_03-00-01
    volumes:
      # Same mount to backup files as above
      - /path/to/komodo/backups:/backups
    environment:
      ## Database port must be reachable.
      ## Note the different env vars needed compared to backup.
      ## This is to prevent any accidental restores.
      KOMODO_CLI_DATABASE_TARGET_ADDRESS: komodo.example.com:27017
      KOMODO_CLI_DATABASE_TARGET_USERNAME: <db username>
      KOMODO_CLI_DATABASE_TARGET_PASSWORD: <db password>
      KOMODO_CLI_DATABASE_TARGET_DB_NAME: komodo-restore
```

:::warning
The restore process can be run multiple times with the same backup files and will not create extra
copies. It does **not** clear the target database beforehand. If the restore database is already
populated, those old documents will remain. In that case, you may want to drop or delete the
target database before restoring to it.
:::

## Consistency

If the backup process completes successfully, the resulting files can always be restored regardless
of how active the Komodo instance is at the time of backup. Writes that happen during the backup
process, such as updates to resource configuration, may or may not be included depending on the
timing.

This rarely causes restore problems, but if your Komodo instance sees a lot of write activity and
you need stronger consistency, you can
[lock](https://www.mongodb.com/docs/manual/reference/method/db.fsyncLock/#mongodb-method-db.fsyncLock)
Mongo before the backup. Make sure to
[unlock](https://www.mongodb.com/docs/manual/reference/method/db.fsyncUnlock/) the database
afterwards.
