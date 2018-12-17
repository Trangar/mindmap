-- Your SQL goes here

CREATE TABLE "user" (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    name TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);

CREATE TABLE user_token (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    user_id UUID NOT NULL REFERENCES "user"(id),
    ip TEXT NOT NULL,
    last_used TIMESTAMPTZ NOT NULL,
    active BOOL NOT NULL
);

CREATE TABLE note (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    user_id UUID NOT NULL REFERENCES "user"(id),
    view_count INT NOT NULL DEFAULT 0,
    seo_name TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    deleted BOOL NOT NULL
);

CREATE INDEX ON note(title);


CREATE TABLE note_history (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    note_id UUID NOT NULL REFERENCES "note"(id),
    created TIMESTAMPTZ NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL
);

CREATE INDEX ON note_history(created);

CREATE TABLE note_link (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    "left" UUID NOT NULL REFERENCES "note"(id),
    "right" UUID NOT NULL REFERENCES "note"(id),
    click_count INT NOT NULL DEFAULT (0)
);

