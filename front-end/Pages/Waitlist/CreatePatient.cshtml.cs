using Admission.FrontEnd.Models;
using Admission.FrontEnd.Models.Forms;
using Admission.FrontEnd.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.RazorPages;

namespace Admission.FrontEnd.Pages.Waitlist;

public class CreatePatientModel : PageModel
{
    private readonly AdmissionsClient _client;

    public CreatePatientModel(AdmissionsClient client)
    {
        _client = client;
    }

    [BindProperty]
    public CreatePatient Form { get; set; } = new();

    public async Task<ActionResult> OnGetAsync()
    {
        Form.AllowedHospitals.Clear();
        await _client.AuthenticateAs(new LoginRequest()
        {
            Email = "admin@dsh.ca.gov"
        });
        var allHospitals = await _client.GetAllHospitals();
        foreach (var hospital in allHospitals)
        {
            Form.AllowedHospitals.Add(new HospitalAllowed()
            {
                HospitalName = hospital.Name,
                IsAllowed = true
            });
        }
        return Page();
    }

    public async Task<ActionResult> OnPostAsync()
    {
        var patient = Form.ToPatient();
        await _client.AuthenticateAs(new LoginRequest()
        {
            Email = "admin@dsh.ca.gov"
        });
        await _client.CreatePatient(patient);

        return Redirect("~/Waitlist");
    }
}