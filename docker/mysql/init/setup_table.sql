USE gengar_dev;

DROP TABLE IF EXISTS users;
CREATE TABLE users (
	id SERIAL PRIMARY KEY,
	name TEXT NOT NULL,
	certs TEXT NOT NULL
);

INSERT INTO users (name, certs)
VALUES
	('user1', 'cert1'),
	('user2', 'cert2')
