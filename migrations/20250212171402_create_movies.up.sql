-- Add up migration script here

CREATE TABLE movies (
    id SERIAL PRIMARY KEY,
    name VARCHAR(40),
    director VARCHAR(40)
)
