using System.Net.Http.Headers;
using Admission.FrontEnd.Models;

namespace Admission.FrontEnd.Services;

/// <summary>
/// consumes the admissions Rust API
/// </summary>
public class AdmissionsClient
{
    private readonly HttpClient _httpClient;
    private string? _token;

    public AdmissionsClient(HttpClient httpClient)
    {
        _httpClient = httpClient;
        _httpClient.BaseAddress = new Uri("http://localhost:8080/");
    }

    public async Task AuthenticateAs(User user)
    {
        var result = await _httpClient.PostAsJsonAsync("/jwt", user);
        if (result is null)
        {
            throw new Exception("authentication failed");
        }
        _token = await result.Content.ReadAsStringAsync();
        if (_token is not null)
        {
            _httpClient.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", _token);
        }
    }

    public async Task<List<Hospital>> GetAllHospitals()
    {
        var result = await _httpClient.GetFromJsonAsync<List<Hospital>>("api/v1/hospitals");
        if (result is null)
        {
            throw new Exception("get all hospitals failed to deserialize server response");
        }
        return result;
    }
}