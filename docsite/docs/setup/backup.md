# Backup and Restore

:::info
Database backup and restore is actually a function of the [Komodo CLI](../ecosystem/cli),
which is packaged in with the Komodo Core image for convenience.
:::

Starting from **v1.19.0**, new Komodo installs will automatically create the
**Backup Core Database** [Procedure](../resources/procedures#procedures), scheduled daily.
If you don't have it, this is the Toml:

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
this process before launching a backup container. There is nothing special about this Procedure,
it's just created by default for guidance / convenience.
:::

## Backups

When Komodo takes a database backup, it creates a **folder named for the time the backup was taken**,
and dumps the gzip-compressed documents to files in this folder. 
In order to store the backups to disk, **mount a host path to `/backups`** in the Komodo Core container.

Due to its larger size and relative unimportance, the `Stats` collection (containing historical server cpu / mem / disk usage)
is not included in dated backups. Just latest Stats are maintained at the top level of the backup folder.

In order to prevent unbounded growth, the backup process implements a pruning feature which will ensure
only the most recent 14 backup folders are kept. To change this number, set `max_backups` (`KOMODO_CLI_MAX_BACKUPS`)
in `core.config.toml`, `komodo.cli.toml`, or in the Core container environment.

```
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
Currently no encryption is supported,
so you may want to encrypt the files before backing up remotely if your backup solution doesn't support that natively.
:::

## Remote Backups

Since database backup is actually a function of the [Komodo CLI](../ecosystem/cli), you can also backup directly to
a remote server using the `ghcr.io/moghtech/komodo-cli` image. This service will backup once and then exit, so the scheduled deployment should still happen using a Procedure or Action:

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
The restore process can be run multiple times with same backup files, and won't create any extra copies.
HOWEVER it will not "clear" the target database beforehand. If the restore database is already populated,
those old documents will also remain. You may want to drop / delete the target database
before restoring to it in this case.
:::

## Consistency

So long as the backup process completes successfully, the files produces can always be restored
no matter how active the Komodo instance is at the time of backup. However writes that happen during
the backup process, such as updates to the resource configuration, may or may not be included in the backup
depending on the timing.

While it should be rare that this causes any kind of issue when it comes to restoring, if your
Komodo undergoes a lot of usage at all hours and you are worried about consistency,
you could consider [locking](https://www.mongodb.com/docs/manual/reference/method/db.fsyncLock/#mongodb-method-db.fsyncLock)
Mongo before the backup. Just make sure to [unlock](https://www.mongodb.com/docs/manual/reference/method/db.fsyncUnlock/)
the database afterwards.