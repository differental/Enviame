# Migration Instructions

This page is for users migrating from a direct deployment to a docker containerised deployment.

## Database

After running `docker compose up --build`, a postgres data volume should be created automatically. In order to then migrate the old postgres database (either remote or local) into the new database, you should first map the postgres container to an unused port on your server. For example, you can enable access to your "docker postgres" by adding the following to `compose.yaml`:

```yaml
# compose.yaml 

# ...
#   volumes:
#     - pgdata:/var/lib/postgresql/data
    ports:
      - 127.0.0.1:5433:5432

# server:
#   build:
# ...
```

This maps the postgres db running inside port 5432 (by default) of your postgres container to `127.0.0.1:5433` on your server. You can then use the following commands to create data dumps from your old database:

```bash
pg_dump -Fc -f old_enviame_db_data.dump 
```

`-Fc` specifies a custom file format, and `-f old_enviame_db_data.dump` means output dump data to a file path.

You might wish to add in `-h`, `-p`, `-U` or `-d` typically used to specify connection options. For a db running on localhost:5432 with the name `enviame_prod`, accessible by user `enviame`: 

```bash
pg_dump -h localhost -p 5432 -U enviame -d enviame_prod -Fc -f old_enviame_db_data.dump
```

Afterwards, we can start by restoring the database schema in the newly created database. Copy `/scripts/schema.sql` and run:

```bash
psql -h localhost -p 5433 -U enviame_user -d enviame_db -f schema.sql 
```

This runs `schema.sql` on the new postgres database (assuming database name is `enviame_db`, user name is `enviame_user` and it's been mapped to localhost port 5433).

Then we can restore data from our dump:

```bash 
pg_restore -h localhost -p 5433 -U enviame_user -d enviame_db --data-only --disable-triggers old_enviame_db_data.dump
```

All done! You can then remove the port mapping in your `compose.yaml` and redeploy. Database migration complete.

