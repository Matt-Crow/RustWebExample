namespace Admission.FrontEnd.Models;

public class User
{
    public string Email { get; set; } = string.Empty;

    public HashSet<string> Groups { get; set; } = new ();
}