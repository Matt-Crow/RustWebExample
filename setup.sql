USE RustDB;

IF OBJECT_ID(N'rust.Patients', N'U') IS NOT NULL
	DROP TABLE rust.Patients;

IF OBJECT_ID(N'rust.Hospitals', N'U') IS NOT NULL
	DROP TABLE rust.Hospitals;

IF OBJECT_ID(N'rust.User_Groups', N'U') IS NOT NULL
	DROP TABLE rust.User_Groups;
	
IF OBJECT_ID(N'rust.Users', N'U') IS NOT NULL
	DROP TABLE rust.Users;

CREATE TABLE rust.Hospitals (
	HospitalID int IDENTITY(1, 1) PRIMARY KEY NOT NULL,
	Name varchar(16) NOT NULL
);

SET IDENTITY_INSERT rust.Hospitals ON; -- allow script to set hospital IDs

INSERT INTO rust.Hospitals (HospitalID, Name)
VALUES
	(1, 'Atascadero'),
	(2, 'Coalinga'),
	(3, 'Metropolitan'),
	(4, 'Napa'),
	(5, 'Patton');

SET IDENTITY_INSERT rust.Hospitals OFF;

CREATE TABLE rust.Patients (
	PatientID int IDENTITY(1, 1) PRIMARY KEY NOT NULL,
	Name varchar(32) NOT NULL,
	HospitalID int NOT NULL,
	CONSTRAINT FK_Patients_Hospitals FOREIGN KEY (HospitalID)
	                        REFERENCES rust.Hospitals (HospitalID)
							ON DELETE CASCADE
);

INSERT INTO rust.Patients (Name, HospitalID)
VALUES
	('John Doe', 1),
	('Jane Doe', 1),
	('Bob Smith', 2)
;

CREATE TABLE rust.Users (
	UserID int IDENTITY(1, 1) PRIMARY KEY NOT NULL,
	Name varchar(32) NOT NULL,
	CONSTRAINT UK_Users_Name UNIQUE (Name)
);

SET IDENTITY_INSERT rust.Users ON;

INSERT INTO rust.Users (UserID, Name)
VALUES
	(1, 'Matt Crow'),
	(2, 'Joe Admin'),
	(3, 'Random Guy')
;

SET IDENTITY_INSERT rust.Users OFF;

CREATE TABLE rust.User_Groups (
	UserID int NOT NULL,
	GroupName varchar(16) NOT NULL,

	CONSTRAINT FK_User_Groups_Users FOREIGN KEY (UserID) 
		REFERENCES rust.Users (UserID)
		ON DELETE CASCADE,
	
	CONSTRAINT PK_User_Groups PRIMARY KEY (UserID, GroupName)
);

INSERT INTO rust.User_Groups (UserID, GroupName)
VALUES
	(1, 'Admin'),
	(1, 'Nerd'),
	(2, 'Admin')
;