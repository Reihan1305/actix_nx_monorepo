-- Add up migration script here
CREATE TABLE IF NOT EXISTS "refresh_token"(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    refreshtoken TEXT NOT NULL,
    userid UUID NOT NULL,
    CONSTRAINT fk_user Foreign Key (userid) REFERENCES "user" (id) ON DELETE CASCADE
)