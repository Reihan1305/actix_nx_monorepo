-- Add up migration script here


CREATE TABLE IF NOT EXISTS "user"(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL UNIQUE,
    phonenumber TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);