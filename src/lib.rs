use std::cmp::Ordering::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

use clap::Parser;

use crate::TakeValue::{PlusZero, TakeNum};

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

fn parse_take_value(s: &str, field_name: &str) -> Result<TakeValue, String> {
    let err_msg = |_| format!("illegal {} count -- {}", field_name, s);
    let is_starts_with_plus = s.starts_with('+');
    let is_starts_with_minus = s.starts_with('-');

    let is_negative = !is_starts_with_plus || is_starts_with_minus;
    if is_negative {
        let mut val = s.parse::<i64>().map_err(err_msg)?;

        if !is_starts_with_minus {
            val = -val;
        }
        return Ok(TakeNum(val));
    }

    let val = s.trim_start_matches('+')
        .parse::<i64>()
        .map_err(err_msg)?;


    if val == 0 {
        return Ok(PlusZero);
    }

    Ok(TakeNum(val))
}

fn parse_lines(s: &str) -> Result<TakeValue, String> {
    parse_take_value(s, "line")
}

fn parse_bytes(s: &str) -> Result<TakeValue, String> {
    parse_take_value(s, "byte")
}

#[derive(Parser, Debug)]
#[command(name = "tailr", version = "0.1.0", author = "Radish-Miyazaki <y.hidaka.kobe@gmail.com>", about = "Rust tail")]
pub struct Cli {
    #[arg(value_name = "FILE", help = "Input file(s)", required = true)]
    files: Vec<String>,
    #[arg(value_name = "BYTES", short = 'c', long, help = "Number of bytes", value_parser = parse_bytes, conflicts_with = "lines")]
    bytes: Option<TakeValue>,
    #[arg(value_name = "LINES", short = 'n', long, help = "Number of lines", default_value = "10", value_parser = parse_lines)]
    lines: TakeValue,
    #[arg(short, long, help = "Suppress headers")]
    quiet: bool,
}

pub fn get_cli() -> MyResult<Cli> {
    Ok(Cli::parse())
}

fn count_lines_bytes(filename: &str) -> MyResult<(i64, i64)> {
    let file = File::open(filename)?;
    let mut rdr = BufReader::new(file);

    let mut bytes: i64 = 0;
    let mut lines: i64 = 0;
    let mut buf = String::new();
    loop {
        let line_bytes = rdr.read_line(&mut buf)? as i64;
        if line_bytes == 0 {
            break;
        }

        bytes += line_bytes;
        lines += 1;
    }

    Ok((lines, bytes))
}

fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    match take_val {
        PlusZero => {
            if total == 0 {
                return None;
            }

            Some(0)
        }
        TakeNum(n) => {
            let n = *n;
            match n.cmp(&(0i64)) {
                Equal => None,
                Greater => {
                    if total < n {
                        return None;
                    }

                    Some((n - 1) as u64)
                }
                Less => {
                    if total < n.abs() {
                        return Some(0);
                    }

                    Some((total + n) as u64)
                }
            }
        }
    }
}

fn print_lines(
    mut file: impl BufRead,
    num_lines: &TakeValue,
    total_lines: i64,
) -> MyResult<()>
{
    if let Some(start) = get_start_index(num_lines, total_lines) {
        let mut line_count = 0;
        let mut buf = String::new();

        loop {
            let bytes_read = file.read_line(&mut buf)?;
            if bytes_read == 0 {
                break;
            }

            if line_count >= start {
                print!("{}", buf);
            }
            line_count += 1;
            buf.clear();
        }
    }

    Ok(())
}

fn print_bytes<T>(
    mut file: T,
    num_bytes: &TakeValue,
    total_bytes: i64,
) -> MyResult<()>
    where T: Read + Seek
{
    match get_start_index(num_bytes, total_bytes) {
        None => (),
        Some(start) => {
            file.seek(SeekFrom::Start(start))?;

            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            print!("{}", String::from_utf8_lossy(&buf));
        }
    }

    Ok(())
}

pub fn run(cli: &Cli) -> MyResult<()> {
    let file_count = cli.files.len();

    for (i, filename) in cli.files.iter().enumerate() {
        match File::open(filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(f) => {
                if file_count > 1 && !cli.quiet {
                    println!(
                        "{}==> {} <==",
                        if i > 0 { "\n" } else { "" },
                        filename,
                    );
                }

                let (total_lines, total_bytes) = count_lines_bytes(filename)?;
                if cli.bytes.is_some() {
                    print_bytes(f, &cli.bytes.as_ref().unwrap(), total_bytes)?;
                    continue;
                }

                print_lines(BufReader::new(f), &cli.lines, total_lines)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{count_lines_bytes, get_start_index, parse_take_value, PlusZero, TakeNum};

    #[test]
    fn test_parse_take_value() {
        let res = parse_take_value("3", "bytes");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        let res = parse_take_value("+3", "bytes");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        let res = parse_take_value("-3", "bytes");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        let res = parse_take_value("0", "bytes");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        let res = parse_take_value("+0", "bytes");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        let res = parse_take_value(&i64::MAX.to_string(), "bytes");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_take_value(&(i64::MIN + 1).to_string(), "bytes");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_take_value(&format!("+{}", i64::MAX), "bytes");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));

        let res = parse_take_value(&i64::MIN.to_string(), "bytes");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        let res = parse_take_value("3.14", "bytes");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "illegal bytes count -- 3.14");

        let res = parse_take_value("foo", "lines");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "illegal lines count -- foo");
    }

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));

        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }

    #[test]
    fn test_get_start_index() {
        assert_eq!(get_start_index(&PlusZero, 0), None);
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));

        assert_eq!(get_start_index(&TakeNum(0), 1), None);
        assert_eq!(get_start_index(&TakeNum(1), 0), None);

        assert_eq!(get_start_index(&TakeNum(2), 1), None);

        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));

        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));

        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }
}

