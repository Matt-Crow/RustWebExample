USE RustDB;

IF OBJECT_ID(N'rust.Hospitals', N'U') IS NOT NULL
	DROP TABLE rust.Hospitals;

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