USE RustDB;

IF OBJECT_ID(N'rust.Patients', N'U') IS NOT NULL
	DROP TABLE rust.Patients;

IF OBJECT_ID(N'rust.Hospitals', N'U') IS NOT NULL
	DROP TABLE rust.Hospitals;

CREATE TABLE rust.Hospitals (
	HospitalID int PRIMARY KEY NOT NULL,
	Name text NOT NULL
);

TRUNCATE TABLE rust.Hospitals;

INSERT INTO rust.Hospitals (HospitalID, Name)
VALUES
	(1, 'Atascadero'),
	(2, 'Coalinga'),
	(3, 'Metropolitan'),
	(4, 'Napa'),
	(5, 'Patton');

CREATE TABLE rust.Patients (
	PatientID int PRIMARY KEY NOT NULL,
	Name text NOT NULL,
	HospitalID int NOT NULL,
	CONSTRAINT FK_Patients_Hospitals FOREIGN KEY (HospitalID)
	                        REFERENCES rust.Hospitals (HospitalID)
							ON DELETE CASCADE
);

TRUNCATE TABLE rust.Patients;

INSERT INTO rust.Patients (PatientID, Name, HospitalID)
VALUES
	(1, 'John Doe', 1),
	(2, 'Jane Doe', 1),
	(3, 'Bob Smith', 2);

SELECT h.Name 'Hospital', ISNULL(p.Name, 'No patients') 'Patient' 
  FROM rust.Hospitals AS h
	LEFT OUTER JOIN 
	   rust.Patients AS p
	ON h.HospitalID = p.HospitalID;