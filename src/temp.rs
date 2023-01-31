use std::{fs::File, io::{self, BufReader, BufRead, Error, ErrorKind}};

pub fn read() -> io::Result<()> {
    let f = File::open("./src/in.txt.secret")?;
    let br = BufReader::new(f);
    br.lines()
        .map(|line| line.and_then(|v| v.parse::<u8>().map_err(|e| Error::new(ErrorKind::InvalidData, e))))
        .filter_map(|s| {
            match s {
                Ok(x) => Some(char::from(x)),
                Err(_) => None
            }
        })
        .for_each(|e| print!("{}", e));
    Ok(())
}