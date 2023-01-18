USE RustDB;

IF OBJECT_ID(N'rust.Patients', N'U') IS NOT NULL
	DROP TABLE rust.Patients;

IF OBJECT_ID(N'rust.Hospitals', N'U') IS NOT NULL
	DROP TABLE rust.Hospitals;

CREATE TABLE rust.Hospitals (
	HospitalID int IDENTITY(1, 1) PRIMARY KEY NOT NULL,
	Name text NOT NULL
);

TRUNCATE TABLE rust.Hospitals;

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
	Name text NOT NULL,
	HospitalID int NOT NULL,
	CONSTRAINT FK_Patients_Hospitals FOREIGN KEY (HospitalID)
	                        REFERENCES rust.Hospitals (HospitalID)
							ON DELETE CASCADE
);

TRUNCATE TABLE rust.Patients;

INSERT INTO rust.Patients (Name, HospitalID)
VALUES
	('John Doe', 1),
	('Jane Doe', 1),
	('Bob Smith', 2);

SELECT h.Name 'Hospital', ISNULL(p.Name, 'No patients') 'Patient' 
  FROM rust.Hospitals AS h
	LEFT OUTER JOIN 
	   rust.Patients AS p
	ON h.HospitalID = p.HospitalID;