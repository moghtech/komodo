# Database

Komodo Core depends on a database for persistent state. The database stores users, permissions,
resource definitions, configuration, audit history, and other control-plane state. If
[Core](./core.md) is the control plane, the database is the persistent store behind it.

## What The Database Is Responsible For

The database is part of Core's side of the system, not Periphery's. It does not execute workloads,
connect to managed servers, or replace the container runtime. Its job is persistence for:

- resource configuration
- user and permission data
- provider and variable configuration
- audit history and other system state

## Database Choices

Komodo supports two main database setups:

- [**MongoDB**](./setup/mongo.mdx): the default database option and the one the project is
  primarily documented around.
- [**FerretDB**](./setup/ferretdb.mdx): a MongoDB-compatible option backed by Postgres and the
  DocumentDB extension.

The database choice changes the Core deployment shape and backing services, but it does not change
the Core / Periphery execution model.

## Setup, Initialization, And Connection

The compose-based setup paths ship with database configuration already wired into the Compose files.
In both cases, the main inputs are the database credentials in `compose/compose.env`:

- `KOMODO_DATABASE_USERNAME`
- `KOMODO_DATABASE_PASSWORD`

The MongoDB path initializes a MongoDB container with those credentials. The FerretDB path uses the
same credentials for Postgres and then points FerretDB at that Postgres instance.

See:

- [Install Komodo](./setup/install-komodo.mdx)
- [MongoDB](./setup/mongo.mdx)
- [FerretDB](./setup/ferretdb.mdx)
- [`compose/compose.env`](https://github.com/moghtech/komodo/blob/main/compose/compose.env)

## Choosing Between MongoDB And FerretDB

MongoDB is the simpler default when the target system supports it cleanly. FerretDB fits systems
where MongoDB support is awkward or where the Postgres-backed path is preferred.

Existing FerretDB v1 users should also note that current Komodo docs target FerretDB v2, and
upgrades from v1 require migration. That detail is covered in the FerretDB setup page.

## Backups And Recovery

The database is also the persistence boundary for recovery. Backups and restores operate on Core
state in the database, not on the workloads running through Periphery. That is why backup and
restore live alongside setup and database choice rather than under deployment or automation.

## Operational Boundary

The database matters for Core persistence and recovery, but it does not change where workloads run.
Deployments, Stacks, Builds, and host-side operations still go through [Periphery](./periphery.md)
and the target host runtime.

## Related Pages

- [What Is Komodo](./intro.md)
- [Core](./core.md)
- [Setup](./setup)
- [Install Komodo](./setup/install-komodo.mdx)
- [MongoDB](./setup/mongo.mdx)
- [FerretDB](./setup/ferretdb.mdx)
- [Backup](./setup/backup.md)
