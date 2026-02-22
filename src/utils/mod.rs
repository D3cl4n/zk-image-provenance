use std::fs::File;
use std::io::{self, BufRead, BufReader};


// read the greyscale values from the edtior into a vector to be passed to prover
pub fn read_greyscale_values(path: &String) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // each y value is one line - convert to u8 and read into vector
    let mut y_values: Vec<u8> = vec![];
    for line in reader.lines() {
        let line = line?;
        let value: u8 = line.trim().parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid byte val"))?;
        y_values.push(value);
    }

    Ok(y_values)
}