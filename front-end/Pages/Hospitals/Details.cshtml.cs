using Admission.FrontEnd.Models;
using Admission.FrontEnd.Services;
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
        await _client.AuthenticateAs(new User()
        {
            Email = "john.doe@dsh.ca.gov"
        });
        Hospital = await _client.GetHospitalByName(name);
        if (Hospital is not null)
        {
            Hospital.Patients.Sort(ComparePatients);
        }
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