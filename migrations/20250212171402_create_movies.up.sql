-- Add up migration script here

CREATE TABLE movies (
    id UUID PRIMARY KEY,
    name VARCHAR(40) NOT NULL,
    director VARCHAR(40) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
)
