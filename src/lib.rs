use clap::Parser;

use crate::TakeValue::{PlusZero, TakeNum};

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

fn parse_take_value(s: &str, field_name: &str) -> Result<TakeValue, String> {
    let is_positive = s.starts_with('+');
    let val = s.trim_start_matches('+')
        .parse::<i64>()
        .map_err(|_| format!("illegal {} count -- {}", field_name, s))?;

    if is_positive && val == 0 {
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

pub fn run(cli: &Cli) -> MyResult<()> {
    println!("{:#?}", cli);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_take_value, PlusZero, TakeNum};

    #[test]
    fn test_parse_take_value() {
        let res = parse_take_value("3", "bytes");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

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
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));

        let res = parse_take_value(&(i64::MAX - 1).to_string(), "bytes");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX - 1));

        let res = parse_take_value(&(i64::MIN + 1).to_string(), "bytes");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_take_value(&i64::MIN.to_string(), "bytes");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        let res = parse_take_value(&format!("+{}", i64::MAX), "bytes");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));


        let res = parse_take_value("3.14", "bytes");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "illegal bytes count -- 3.14");

        let res = parse_take_value("foo", "lines");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "illegal lines count -- foo");
    }
}

