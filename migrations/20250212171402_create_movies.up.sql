-- Add up migration script here

CREATE TABLE movies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(40) NOT NULL,
    director VARCHAR(40) NOT NULL,
    synopsis TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
)
