-- Your SQL goes here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE "server" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "last_seen" TIMESTAMP NOT NULL DEFAULT current_timestamp,
    "token" TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP
);

CREATE TABLE "server_info" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "name" TEXT NOT NULL,
    "uri" TEXT NOT NULL,
    "developer" BOOLEAN NOT NULL,
    "fallback" BOOLEAN NOT NULL,
    "full" BOOLEAN NOT NULL,
    "maintenance" BOOLEAN NOT NULL,
    "max_clients" INT,
    "server_id" UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP,
    FOREIGN KEY ("server_id") REFERENCES "server"("id")
);

CREATE TABLE "server_game" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "name" TEXT NOT NULL,
    "uri" TEXT NOT NULL,
    "rooms" INT NOT NULL,
    "max_rooms" INT,
    "clients" INT NOT NULL,
    "game_info_id" UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP,
    FOREIGN KEY ("game_info_id") REFERENCES "server_info"("id")
);