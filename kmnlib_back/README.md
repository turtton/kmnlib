# kmnlib_back

Backend of kmnlib

This project is a playground to compare with [architectured](https://github.com/HalsekiRaika/architectured).

# Architecture

CQRS + Event Sourcing + Partial Clean Architecture + Minimal Cake Pattern + Actor Model

# Structure

```mermaid
erDiagram
    users {
        uuid id "PK"
        text name
        int rent_limit
        bigint version
        boolean is_deleted
    }
    user_events {
        bigint version "PK"
        uuid user_id "PK"
        text event_name
        text name "NULL"
        int rent_limit "NULL"
        timestamp created_at
    }
    books {
        uuid id "PK"
        text title
        int amount
        bigint version
        boolean is_deleted
    }
    book_rents {
        bigint version "PK"
        uuid user_id "PK,FK"
        uuid book_id "PK,FK"
    }
    book_events {
        bigint version "PK"
        uuid book_id "PK"
        text event_name
        text title "NULL"
        int amount "NULL"
        timestamp created_at
    }
    rent_events {
        bigint version "PK"
        uuid user_id "PK"
        uuid book_id "PK"
        text event_name
        timestamp created_at
    }

    books ||--|{ book_rents: "exists if rent"
    books ||--o| book_events: "book event stream"
    users ||--o| user_events: "user event stream"
    users ||--|{ book_rents: "exists if be rented"
    book_rents ||--|| rent_events: "rent event stream"
```

### Event

| name        | data                                                        |
|-------------|-------------------------------------------------------------|
| UserCreated | `{name: String, rent_limit: i31}`                           |
| UserUpdated | `{id: UUID, name: Option<String>, rent_limit: Option<i32>}` |
| UserDeleted | `{id: UUID}`                                                |

| name        | data                                                          |
|-------------|---------------------------------------------------------------|
| BookCreated | `{title: String, amount: i32}`                                |
| BookUpdated | `{book_id: UUID, title: Option<String>, amount: Option<i32>}` |
| BookDeleted | `{book_id: UUID}`                                             |

| name         | data                                                     |
|--------------|----------------------------------------------------------|
| BookRented   | `{user_id: UUID, book_id: UUID, expected_version: UUID}` |
| BookReturned | `{user_id: UUID, book_id: UUID, expected_version: UUID}` |

# DB

PostgreSQL

```shell
podman run --rm --name kmnlib-postgres -v ./migrations/20231125184100_init.sql:/docker-entrypoint-initdb.d/postgre.sql -e POSTGRES_PASSWORD=develop -p 5432:5432 docker.io/postgres
```