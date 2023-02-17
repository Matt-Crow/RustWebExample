# Complement
A web service which computes the complement of a set of hospital names.
For example, given the universal set of hospitals
    `U = {A, B, C}`
and input
    `s = {B}`
the service returns
    `s' = {A, C}`

## Usage
Ensure the `admission` project is running, then run this project.
`GET http://localhost:8081/complement` with the following body:
```
{
    "names": [
        "Atascadero",
        "Napa"
    ]
}
```
you should receive
```
{
    "names": [
        "Metropolitan",
        "Patton",
        "Coalinga"
    ]
}
```