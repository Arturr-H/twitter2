# Twitter2
This is a replica of Twitter (X) that I've been working on. For more information check out the Readme file in the frontend of this project [here](https://github.com/Arturr-H/twitter2-frontend).

### What am I building this with?
I am using `actix_web` for all HTTP-endpoints, using `Postgresql` as the database (sql interactions made possible via crate `sqlx`). I am running the postgres database server inside of a docker container (`docker-compose.yml`), and keeping track of database migrations in `/migrations`.

### Contributions
Contributions are welcome, and If you'd like to make any huge changes, please open an Issue first. But making a pull request will always work!
