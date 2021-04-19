USE gengar_dev;

DROP TABLE IF EXISTS Users;
CREATE TABLE Users
(
	UserID SERIAL PRIMARY KEY,
	GoogleUserID CHAR(21) NOT NULL
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
	FOREIGN KEY (VaccineID) REFERENCES Vaccines (VaccineID),
	RegisterDate DATE NOT NULL,
	ExpirationDate DATE NOT NULL
);

INSERT INTO Users (GoogleUserID)
VALUES
	('234385785823438578589'),
	('418446744073709551615');

INSERT INTO Vaccines (VaccineName)
VALUES
	('cert1'),
	('cert2');

INSERT INTO UserVaccine (UserID, VaccineID, RegisterDate, ExpirationDate)
VALUES
	(1, 1, '1988-12-30', '2022-03-30'),
	(1, 2, '2015-02-19', '2021-06-02'),
	(2, 2, '2020-12-12', '2122-11-01');
