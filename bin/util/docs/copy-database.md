# Copy Database Utility

Copy the Komodo database contents between running, mongo-compatible databases.
Can be used to move between MongoDB / FerretDB, or upgrade from FerretDB v1 to v2.

```yaml
services:

  copy_database:
    image: ghcr.io/moghtech/komodo-util
    environment:
      MODE: CopyDatabase
      SOURCE_URI: mongodb://${KOMODO_DB_USERNAME}:${KOMODO_DB_PASSWORD}@source:27017
      SOURCE_DB_NAME: ${KOMODO_DATABASE_DB_NAME:-komodo}
      TARGET_URI: mongodb://${KOMODO_DB_USERNAME}:${KOMODO_DB_PASSWORD}@target:27017
      TARGET_DB_NAME: ${KOMODO_DATABASE_DB_NAME:-komodo}

```

## FerretDB v2 Update Guide

Up to Komodo 1.17.5, users who wanted to use Postgres / Sqlite were instructed to deploy FerretDB v1.
Now that v2 is out however, v1 will go largely unsupported. Users are recommended to migrate to v2 for
the best performance and ongoing support / updates, however the internal data structures
have changed and this cannot be done in-place. 

Also note that FerretDB v2 no longer supports Sqlite, and only supports 
a [customized Postgres distribution](https://docs.ferretdb.io/installation/documentdb/docker/).
Nonetheless, it remains a solid option for hosts which [do not support mongo](https://github.com/moghtech/komodo/issues/59).

Also note, the same basic process outlined below can also be used to move between MongoDB and FerretDB, just replace FerretDB v2
with the database you wish to move to.

### **Step 1**: *Add* the new database to the top of your existing Komodo compose file.

**Don't forget to also add the new volumes.**

```yaml
## In Komodo compose.yaml
services:
  postgres2:
    # Recommended: Pin to a specific version
    # https://github.com/FerretDB/documentdb/pkgs/container/postgres-documentdb
    image: ghcr.io/ferretdb/postgres-documentdb
    labels:
      komodo.skip: # Prevent Komodo from stopping with StopAllContainers
    restart: unless-stopped
    logging:
      driver: ${COMPOSE_LOGGING_DRIVER:-local}
    # ports:
    #   - 5432:5432
    volumes:
      - postgres-data:/var/lib/postgresql/data
    environment:
      POSTGRES_USER: ${KOMODO_DB_USERNAME}
      POSTGRES_PASSWORD: ${KOMODO_DB_PASSWORD}
      POSTGRES_DB: postgres

  ferretdb2:
    # Recommended: Pin to a specific version
    # https://github.com/FerretDB/FerretDB/pkgs/container/ferretdb
    image: ghcr.io/ferretdb/ferretdb
    labels:
      komodo.skip: # Prevent Komodo from stopping with StopAllContainers
    restart: unless-stopped
    depends_on:
      - postgres2
    logging:
      driver: ${COMPOSE_LOGGING_DRIVER:-local}
    # ports:
    #   - 27017:27017
    volumes:
      - ferretdb-state:/state
    environment:
      FERRETDB_POSTGRESQL_URL: postgres://${KOMODO_DB_USERNAME}:${KOMODO_DB_PASSWORD}@postgres2:5432/postgres

  ...(unchanged)

volumes:
  ...(unchanged)
  postgres-data:
  ferretdb-state:
```

### **Step 2**: *Add* the database copy utility to Komodo compose file.

The SOURCE_URI points to the existing database, ie the old FerretDB v1, and it depends
on whether it was deployed using Postgres or Sqlite. The example below uses the Postgres one,
but if you use Sqlite it should just be something like `mongodb://ferretdb:27017`.

```yaml
## In Komodo compose.yaml
services:
  ...(new database)

  copy_database:
    image: ghcr.io/moghtech/komodo-util
    environment:
      MODE: CopyDatabase
      SOURCE_URI: mongodb://${KOMODO_DB_USERNAME}:${KOMODO_DB_PASSWORD}@ferretdb:27017/${KOMODO_DATABASE_DB_NAME:-komodo}?authMechanism=PLAIN
      SOURCE_DB_NAME: ${KOMODO_DATABASE_DB_NAME:-komodo}
      TARGET_URI: mongodb://${KOMODO_DB_USERNAME}:${KOMODO_DB_PASSWORD}@ferretdb2:27017
      TARGET_DB_NAME: ${KOMODO_DATABASE_DB_NAME:-komodo}

  ...(unchanged)
```

### **Step 3**: *Compose Up* the new additions

Run `docker compose -p komodo --env-file compose.env -f xxxxx.compose.yaml up -d`, filling in the name of your compose.yaml.
This will start up both the old and new database, and copy the data to the new one.

Wait a few moments for the `copy_database` service to finish. When it exits,
confirm the logs show the data was moved successfully, and move on to the next step.

### **Step 4**: Point Komodo Core to the new database

In your Komodo compose.yaml, first *comment out* the `copy_database` service and old ferretdb v1 service/s.
Then update the `core` service environment to point to `ferretdb2`.

```yaml
services:
  ...

  core:
    ...(unchanged)
    environment:
      KOMODO_DATABASE_ADDRESS: ferretdb2:27017
      KOMODO_DATABASE_USERNAME: ${KOMODO_DB_USERNAME}
      KOMODO_DATABASE_PASSWORD: ${KOMODO_DB_PASSWORD}
```

### **Step 5**: Final *Compose Up*

Repeat the same `docker compose` command as before to apply the changes, and then try navigating to your Komodo web page.
If it works, congrats, you are done. You can clean up the compose file if you would like.

If it does not work, check the logs for any obvious issues, and if necessary you can undo the previous steps
to go back to using the previous database.
