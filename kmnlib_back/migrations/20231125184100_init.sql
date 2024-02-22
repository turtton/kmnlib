CREATE TABLE IF NOT EXISTS users
(
    id         UUID    NOT NULL PRIMARY KEY,
    name       TEXT    NOT NULL,
    rent_limit INT     NOT NULL,
    version    BIGINT  NOT NULL,
    is_deleted BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS user_events
(
    version    BIGSERIAL   NOT NULL,
    user_id    UUID        NOT NULL,
    event_name TEXT        NOT NULL,
    name       TEXT,
    rent_limit INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (version, user_id)
);

CREATE TABLE IF NOT EXISTS books
(
    id         UUID    NOT NULL PRIMARY KEY,
    title      TEXT    NOT NULL,
    amount     INT     NOT NULL,
    version    BIGINT  NOT NULL,
    is_deleted BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS book_events
(
    version    BIGSERIAL   NOT NULL,
    book_id    UUID        NOT NULL,
    event_name TEXT        NOT NULL,
    title      TEXT,
    amount     INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (version, book_id)
);

CREATE TABLE IF NOT EXISTS book_rents
(
    version BIGSERIAL NOT NULL,
    book_id UUID      NOT NULL,
    user_id UUID      NOT NULL,
    PRIMARY KEY (version, book_id, user_id),
    FOREIGN KEY (book_id) REFERENCES books (id),
    FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE TABLE IF NOT EXISTS rent_events
(
    version    BIGSERIAL   NOT NULL,
    book_id    UUID        NOT NULL,
    user_id    UUID        NOT NULL,
    event_name TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (version, book_id, user_id)
);