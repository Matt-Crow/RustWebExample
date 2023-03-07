using Admission.FrontEnd.Models;
using Admission.FrontEnd.Services;
using Microsoft.AspNetCore.Mvc;
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

    public async Task<ActionResult> OnPostAdmitFromWaitlist()
    {
        await _client.AuthenticateAs(new LoginRequest()
        {
            Email = "admin@dsh.ca.gov"
        });
        await _client.AdmitFromWaitlist();
        return RedirectToPage();
    }

    private static int ComparePatients(Patient a, Patient b)
    {
        return a.Name.CompareTo(b.Name);
    }
}