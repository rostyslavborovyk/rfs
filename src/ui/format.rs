use std::cmp;

pub fn to_readable_size(num: u64) -> String {
    let num_f64 = num as f64;
    let units = ["B", "kB", "MB", "GB"];
    let delimiter = 1000_f64;
    let exponent = cmp::min((num_f64.ln() / delimiter.ln()).floor() as i32, (units.len() - 1) as i32);
    let pretty_bytes = format!("{:.1}", num_f64 / delimiter.powi(exponent)).parse::<f64>().unwrap() * 1_f64;
    let unit = units[exponent as usize];
    format!("{} {}", pretty_bytes, unit)
}
