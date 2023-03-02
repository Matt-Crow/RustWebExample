using Admission.FrontEnd.Models;
using Admission.FrontEnd.Services;
using Microsoft.AspNetCore.Mvc.RazorPages;

namespace Admission.FrontEnd.Pages;

public class WaitlistModel : PageModel
{
    private readonly AdmissionsClient _client;

    public WaitlistModel(AdmissionsClient client)
    {
        _client = client;
    }

    public List<Patient> Patients { get; private set; } = new();

    public async Task OnGetAsync()
    {
        await _client.AuthenticateAs(new LoginRequest()
        {
            Email = "john.doe@dsh.ca.gov"
        });
        Patients = await _client.GetWaitlist();
        Patients.Sort(ComparePatients);
    }

    private static int ComparePatients(Patient a, Patient b)
    {
        return a.Name.CompareTo(b.Name);
    }
}