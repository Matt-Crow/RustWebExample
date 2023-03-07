namespace Admission.FrontEnd.Models;

public class Hospital
{
    public int? Id { get; set; }

    public string Name { get; set; } = string.Empty;

    public List<Patient> Patients { get; set; } = new();
}