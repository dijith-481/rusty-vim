use crate::error::AppError;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines, Write},
    path::Path,
};

pub fn load_file(filename: &str) -> Result<Vec<String>, AppError> {
    let mut rows = Vec::new();
    if let Ok(lines) = read_lines(filename) {
        for line in lines {
            rows.push(line?);
        }
    }
    Ok(rows)
}
fn read_lines<P>(filename: P) -> Result<Lines<BufReader<File>>, AppError>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
pub fn write_file_to_disk(filename: &str, rows: &Vec<String>) -> Result<(), AppError> {
    let long_string = rows.join("\n");
    let mut file = File::create(&filename)?;
    file.write_all(long_string.as_bytes())?;
    Ok(())
}
