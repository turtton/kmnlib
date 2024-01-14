# kmnlib_back

Backend of kmnlib

# Architecture

CQRS + Event Sourcing + Partial Clean Architecture + Minimal Cake Pattern + Actor Model

# Structure

## User

### Snapshot

```mermaid
erDiagram
    users {
        bigserial id "PK"
        text name
        int rent_limit
        bigint version
    }
```

### Event

| name        | data                                                       |
|-------------|------------------------------------------------------------|
| UserCreated | `{name: String, rent_limit: i32}`                          |
| UserUpdated | `{id: i64, name: Option<String>, rent_limit: Option<i32>}` |
| UserDeleted | `{id: i64}`                                                |

### EventStream

- `User-{id}`

## Book

### Snapshot

```mermaid
erDiagram
    books {
        bigserial id "PK"
        text title
        int amount
        bigint version
    }
    book_rents {
        bigint user_id "PK,FK"
        bigint book_id "PK,FK"
    }
    books ||--o| book_rents: "exists if rented"
```

### Event

| name        | data                                                         |
|-------------|--------------------------------------------------------------|
| BookCreated | `{title: String, amount: i32}`                               |
| BookUpdated | `{book_id: i64, title: Option<String>, amount: Option<i32>}` |
| BookDeleted | `{book_id: i64}`                                             |

| name         | data                                                  |
|--------------|-------------------------------------------------------|
| BookRented   | `{user_id: i64, book_id: i64, expected_version: i64}` |
| BookReturned | `{user_id: i64, book_id: i64, expected_version: i64}` |

### EventStream

- `Book-{id}`
- `BookRent`

## StreamVersions

```mermaid
erDiagram
    stream_versions {
        text stream_name "PK"
        bigint version
    }
```

# DB

## SnapShot

PostgreSQL

```shell
podman run --rm --name kmnlib-postgres -v ./migrations/20231125184100_init.sql:/docker-entrypoint-initdb.d/postgre.sql -e POSTGRES_PASSWORD=develop -p 5432:5432 docker.io/postgres
```

## Event

EventStoreDB

```shell
podman run --rm -it --name kmnlib-eventstore -p 2113:2113 -p 1113:1113 docker.io/eventstore/eventstore:latest --insecure --run-projections=All --enable-external-tcp --enable-atom-pub-over-http
```