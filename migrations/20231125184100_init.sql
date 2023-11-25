CREATE TABLE IF NOT EXISTS users
(
    id      UUID      NOT NULL PRIMARY KEY,
    name    TEXT      NOT NULL,
    version BIGSERIAL NOT NULL
);

CREATE TABLE IF NOT EXISTS books
(
    id      UUID      NOT NULL PRIMARY KEY,
    title   TEXT      NOT NULL,
    version BIGSERIAL NOT NULL
);

CREATE TABLE IF NOT EXISTS book_rents
(
    user_id UUID NOT NULL,
    book_id UUID NOT NULL,
    PRIMARY KEY (user_id, book_id),
    FOREIGN KEY (user_id) REFERENCES users (id),
    FOREIGN KEY (book_id) REFERENCES books (id)
)