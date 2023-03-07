namespace Admission.FrontEnd.Models.Forms;

public class HospitalAllowed
{
    public string HospitalName { get; set; } = string.Empty;

    public bool IsAllowed { get; set; } = true;
}