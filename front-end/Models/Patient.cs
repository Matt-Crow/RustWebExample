namespace Admission.FrontEnd.Models;

public class Patient
{
    public Guid? Id { get; set; }
    
    public string Name { get; set; } = string.Empty;

    public HashSet<string> DisallowAdmissionTo { get; set; } = new();

    public string? AdmittedTo { get; set; }
}