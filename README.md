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
    }
```

### Event

| name            | data                      | description          |
|-----------------|---------------------------|----------------------|
| UserCreated     | `{name: String}`          | User is created      |
| UserNameChanged | `{id: i64, name: String}` | User name is changed |
| UserDeleted     | `{id: i64}`               | User is deleted      |

## Book

### Snapshot

```mermaid
erDiagram
    books {
        bigserial id "PK"
        text title
    }
    bookrents {
        bigserial id "PK"
        bigint user_id "FK"
        bigint book_id "UNIQUE,FK"
    }
    books ||--o| bookrents: "exists if rented"
```

### Event

| name         | data                              | description      |
|--------------|-----------------------------------|------------------|
| BookCreated  | `{title: String}` | Book is created  |
| BookRented   | `{ book_id: i64, user_id: i64 }`  | Book is rented   |
| BookReturned | `{ book_id: i64, user_id: i64 }`  | Book is returned |
