BEGIN;

DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS orgs;
CREATE TABLE orgs (
	id INT4 NOT NULL,
	name TEXT NOT NULL,
	email TEXT NOT NULL
);
CREATE TABLE users (
	id INT4 NOT NULL,
	name TEXT NOT NULL,
	email TEXT NOT NULL,
	password TEXT NOT NULL,
	org_id INT4 NOT NULL,
	personal_org_id INT4 NOT NULL,
	FOREIGN KEY (org_id) REFERENCES orgs(id),
	FOREIGN KEY (personal_org_id) REFERENCES orgs(id)
);

COMMIT;
