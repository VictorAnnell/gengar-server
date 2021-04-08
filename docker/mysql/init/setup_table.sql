USE gengar_dev;

DROP TABLE IF EXISTS Users;
CREATE TABLE Users
(
	UserID SERIAL PRIMARY KEY,
	UserName CHAR(50) NOT NULL
);

DROP TABLE IF EXISTS Vaccines;
CREATE TABLE Vaccines
(
	VaccineID SERIAL PRIMARY KEY,
	VaccineName CHAR(50) NOT NULL
);

DROP TABLE IF EXISTS UserVaccine;
CREATE TABLE UserVaccine
(
	UserID BIGINT UNSIGNED NOT NULL,
	VaccineID BIGINT UNSIGNED NOT NULL,
	CONSTRAINT PK_UserVaccine PRIMARY KEY
	(
			UserID,
			VaccineID
	),
	FOREIGN KEY (UserID) REFERENCES Users (UserID),
	FOREIGN KEY (VaccineID) REFERENCES Vaccines (VaccineID)
);

INSERT INTO Users (UserName)
VALUES
	('user1'),
	('user2');

INSERT INTO Vaccines (VaccineName)
VALUES
	('cert1'),
	('cert2');

INSERT INTO UserVaccine (UserID, VaccineID)
VALUES
	(1, 1),
	(1, 2),
	(2, 2);
