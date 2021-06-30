# attendance-rs

This is our attendance server for RCSC. 

## Attendance Architecture

The attendance server here plays only one small (but important) role in the attendance system. The attendance server is the centralized repository to store all the attendance. 

Attendance collection systems (which *will* require authentication to upload to `attendance-rs`) will upload attendance data to the attendance server by using a mutation in the GraphQL API. After this is done, attendance viewers will visually display the attendance data, querying from the GraphQL API. We will probably implement access controls to prevent collection systems from reading the data in our attendance database and interfaces from writing to the database (more the latter than the former though).

This diagram explains where `attendance-rs` is located in the RCSC attendance pipeline.

![attendance-rs diagram](graphics/attendance-rs-diagram.png)

## Running `attendance-rs`

Setting up `attendance-rs` requires a working PostgreSQL server. Once you have this, you must complete the migrations in the `migrations` directory. These migrations will be automagically executed by SQLx in the future, as there already exists migration capabilities in SQLx.

``` sh
cd migrations
psql -d $DATABASE_NAME < *.sql
```

Once this is done, create a `.env` file that contains the following (The SQLx library requires a working PostgreSQL server to compile this code because of its compile-time SQL checks): 

```
DATABASE_URL=postgres://USER@HOST/DATABASE
```

Replace `USER`, `HOST`, and `DATABASE` accordingly.

You can set some other environment variables in the `.env` file (or wherever else you'd like). One is for the runtime database connection and one is for setting the HTTP host/port (the latter is optional and defaults to `127.0.0.1:8080`). For example, your `.env` could look like this:

``` 
DATABASE_URL=postgres://USER@HOST/DATABASE
AR_PG_CONNECTION_STR=postgres://USER@HOST/DATABASE
AR_PG_HTTP_HOST_STR=0.0.0.0:9000
```

Once `AR_PG_CONNECTION_STR` (and `AR_PG_HOST_STR`) are set, you can start the server with `cargo run`. Navigate to your `AR_PG_HTTP_HOST_STR` in a web browser to play with the API in the GraphQL playground.
