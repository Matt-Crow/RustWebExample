namespace Admission.FrontEnd.Models;

public class AdmittedTo : AdmissionStatus
{
    public AdmittedTo() : base("AdmittedTo")
    {

    }
    
    public string AdmittedTo_ { get; set; } = string.Empty;
}