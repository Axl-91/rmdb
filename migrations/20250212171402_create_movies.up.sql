-- Add up migration script here

CREATE TABLE movies (
    id UUID PRIMARY KEY,
    name VARCHAR(40),
    director VARCHAR(40)
)
