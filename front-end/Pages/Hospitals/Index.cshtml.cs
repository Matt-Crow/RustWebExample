using Admission.FrontEnd.Models;
using Admission.FrontEnd.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.RazorPages;

namespace Admission.FrontEnd.Pages.Hospitals;

public class IndexModel : PageModel
{
    private readonly AdmissionsClient _client;

    public IndexModel(AdmissionsClient client)
    {
        _client = client;
    }

    public List<Hospital> Hospitals { get; private set; } = new ();

    public async Task OnGetAsync()
    {
        await _client.AuthenticateAs(new LoginRequest()
        {
            Email = "john.doe@dsh.ca.gov"
        });
        Hospitals = await _client.GetAllHospitals();
        Hospitals.Sort(CompareHospitals);
    }

    private static int CompareHospitals(Hospital a, Hospital b)
    {
        return a.Name.CompareTo(b.Name);
    }
}
