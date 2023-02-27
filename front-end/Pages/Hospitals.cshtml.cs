using Admission.FrontEnd.Models;
using Admission.FrontEnd.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.RazorPages;

namespace Admission.FrontEnd.Pages;

public class HospitalsModel : PageModel
{
    private readonly AdmissionsClient _client;

    public HospitalsModel(AdmissionsClient client)
    {
        _client = client;
    }

    public List<Hospital> Hospitals { get; private set; } = new ();

    public async Task OnGetAsync()
    {
        await _client.AuthenticateAs(new User()
        {
            Email = "john.doe@dsh.ca.gov"
        });
        Hospitals = await _client.GetAllHospitals();
    }
}
