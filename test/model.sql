BEGIN;
DROP TABLE IF EXISTS users;DROP TABLE IF EXISTS tokens;DROP TABLE IF EXISTS users;
DROP TYPE IF EXISTS UserType;
CREATE TYPE UserType AS ENUM('Admin', 'User', 'Guest');CREATE TABLE users (
	id UUID NOT NULL PRIMARY KEY,
	created_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	last_login TIMESTAMPTZ,
	name TEXT NOT NULL,
	email TEXT NOT NULL,
	user_type UserType NOT NULL,
	subscription_id TEXT
);

CREATE TABLE tokens (
	id UUID NOT NULL PRIMARY KEY,
	created_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	user_id UUID NOT NULL,
	token_value UUID NOT NULL
);

CREATE TABLE users (
	user_id UUID NOT NULL,
	token_value UUID NOT NULL
);



COMMIT;
