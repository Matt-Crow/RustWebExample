using Admission.FrontEnd.Models;
using Admission.FrontEnd.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.RazorPages;

namespace Admission.FrontEnd.Pages.HospitalsModel;

public class DetailsModel : PageModel
{
    private readonly AdmissionsClient _client;

    public DetailsModel(AdmissionsClient client)
    {
        _client = client;
    }

    public string Name { get; set; } = "error extracting name";
    public Hospital? Hospital { get; private set; }

    public async Task OnGetAsync(string name)
    {
        Name = name;
        await _client.AuthenticateAs(new LoginRequest()
        {
            Email = "john.doe@dsh.ca.gov"
        });
        Hospital = await _client.GetHospitalByName(name);
        Hospital?.Patients.Sort(ComparePatients);
    }

    public async Task<ActionResult> OnPostUnadmitPatient(string hospital, Guid id)
    {
        await _client.AuthenticateAs(new LoginRequest()
        {
            Email = "admin@dsh.ca.gov"
        });
        await _client.Unadmit(hospital, id);
        return RedirectToPage();
    }

    private static int ComparePatients(Patient a, Patient b)
    {
        if (a.Id is null && b.Id is null)
        {
            return 0; // equal
        }
        if (a.Id is null)
        {
            return -1;
        }
        if (b.Id is null)
        {
            return 1;
        }
        return a.Id.ToString()!.CompareTo(b.Id.ToString());
    }
}