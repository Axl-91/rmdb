-- Add up migration script here

CREATE TABLE reviews(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    score INTEGER NOT NULL,
    review TEXT,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    movie_id UUID NOT NULL REFERENCES movies(id) ON DELETE CASCADE,
    UNIQUE(user_id, movie_id),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
)
