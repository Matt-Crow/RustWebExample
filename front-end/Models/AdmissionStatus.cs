namespace Admission.FrontEnd.Models;

public class AdmissionStatus
{
    public static AdmissionStatus NEW { get; } = new AdmissionStatus("New");

    public static AdmissionStatus ON_WAITLIST { get; } = new AdmissionStatus("OnWaitlist");

    public AdmissionStatus(string name)
    {
        Name = name;
    }

    public string Name { get; }
}