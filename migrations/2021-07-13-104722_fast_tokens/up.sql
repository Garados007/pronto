-- Your SQL goes here

CREATE TABLE "fast_token" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "token" TEXT NOT NULL UNIQUE,
    "server_id" UUID NOT NULL,
    "game" TEXT NOT NULL,
    "lobby" TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP,
    FOREIGN KEY ("server_id") REFERENCES "server"("id")
);
