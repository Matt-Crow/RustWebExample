namespace Admission.FrontEnd.Models.Forms;

public class CreatePatient
{
    public string PatientName { get; set; } = string.Empty;

    public List<HospitalAllowed> AllowedHospitals { get; set; } = new();

    public Patient ToPatient()
    {
        var disallowedHospitals = AllowedHospitals
            .Where(h => !h.IsAllowed)
            .Select(h => h.HospitalName);

        var patient = new Patient
        {
            Name = PatientName,
            DisallowAdmissionTo = new HashSet<string>(disallowedHospitals)
        };

        return patient;
    }
}