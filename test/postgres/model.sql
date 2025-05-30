BEGIN;

DROP TABLE IF EXISTS todos;
DROP VIEW IF EXISTS AllUserList;
DROP TABLE IF EXISTS lists;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS orgs;
CREATE TABLE orgs (
	id INT4 NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY UNIQUE,
	name TEXT NOT NULL ,
	email TEXT NOT NULL 
);
CREATE TABLE users (
	id INT4 NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY UNIQUE,
	name TEXT NOT NULL ,
	email TEXT NOT NULL ,
	password TEXT NOT NULL ,
	login_count INT8 NOT NULL ,
	last_login TIMESTAMPTZ NOT NULL ,
	total_cost FLOAT8 NOT NULL ,
	org_id INT4 NOT NULL ,
	personal_org_id INT4 NOT NULL ,
	FOREIGN KEY (org_id) REFERENCES orgs(id),
	FOREIGN KEY (personal_org_id) REFERENCES orgs(id)
);
CREATE TABLE lists (
	id INT4 NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY UNIQUE,
	title TEXT NOT NULL ,
	description TEXT NOT NULL ,
	creator_user_id INT4 NOT NULL ,
	org_id INT4 NOT NULL ,
	FOREIGN KEY (creator_user_id) REFERENCES users(id),
	FOREIGN KEY (org_id) REFERENCES orgs(id)
);
CREATE VIEW AllUserList AS SELECT users.id as id, users.name as name, users.email as email, users.login_count as login_count, users.last_login as last_login, users.total_cost as total_cost, users.org_id as org_id, j_org_id.name as org_name FROM users INNER JOIN orgs j_org_id ON j_org_id.id = users.org_id;CREATE TABLE todos (
	id INT4 NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY UNIQUE,
	done BOOLEAN NOT NULL ,
	title TEXT NOT NULL ,
	description TEXT NOT NULL ,
	creator_user_id INT4 NOT NULL ,
	assigned_user_id INT4 ,
	list_id INT4 ,
	org_id INT4 NOT NULL ,
	FOREIGN KEY (creator_user_id) REFERENCES users(id),
	FOREIGN KEY (assigned_user_id) REFERENCES users(id),
	FOREIGN KEY (list_id) REFERENCES lists(id),
	FOREIGN KEY (org_id) REFERENCES orgs(id)
);

COMMIT;
