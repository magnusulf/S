extern crate rustc_serialize;

use std::str::FromStr;
use std::collections::HashMap;
use std::cmp::*;
use rustc_serialize::json;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::io;
use std::io::ErrorKind;
use std::fs;

type Price = i64;
const DIGITS: u32 = 2;
static MULTIPLIER: f64 = 100f64;
const FOLDER: &'static str = "stocks";

fn main() {

    let mut stocks = Vec::new();
    
    if let Err(e) = load_basic(&mut stocks) {
        println!("Error occured when loading files: {}", Error::description(&e));
        return;
        
    }
    let stocks = stocks;

    for stock in stocks {
        println!("{}", json::encode(&stock).unwrap());
    }
    
    loop {

        // Get input
        let inp = read_string();

        // Do check
        if inp.is_none() {
            continue;
        }
        let inp = inp.unwrap();

        // We should not incude newline n stuff.
        let inp = inp.trim();

        // Make it possible to stop the program
        if inp == "quit" { break; }
        
    }
}

fn load_basic(stocks: &mut Vec<Stock>) -> std::io::Result<()> {
    let folder = Path::new(FOLDER);
    let mut folderData = fs::metadata(folder);
    if (folderData.is_err()) {
        fs::create_dir(folder);
        folderData = fs::metadata(folder);
    }
    let folderData = folderData.ok().unwrap();

    let files: Vec<_> = fs::read_dir(folder).unwrap().collect();
    for file in files {
        let file = file.unwrap();
        if ! file.file_type().map(|t| t.is_file()).unwrap_or(false) { continue; }

        let name = file.file_name();
        let name = name.to_str().unwrap_or(&"");
        let name = FOLDER.to_string() + "/" + name;
        let path = Path::new(&name);
        let display = path.display();
        if name.ends_with(".txt") {
            let mut stock = Stock::new();
            
            read_txt(&path, &mut stock);
            
            let newName = name[..name.len()-"txt".len()].to_string() + &"json";
            let newPath = Path::new(&newName);
            let mut file = File::create(newPath).unwrap();
            file.write_all(json::encode(&stock).unwrap().as_bytes());

            try!((fs::remove_file(path)))
        } else if name.ends_with(".json") {
            let mut stock = Stock::new();
            try!(read_json(&path, &mut stock));
            stocks.push(stock);
        }

    }
    Ok(())
}

fn read_json(path: &Path, stock: &mut Stock) -> std::io::Result<()> {
    // Get the file
    let mut content  = String::new();
    let mut file = try!(File::open(&path));
    try!(file.read_to_string(&mut content));

    // Return
    let ret: Stock = try!(json::decode(&content).or(invalid_data("Failed decoding json".to_string())));
    stock.dates = ret.dates;
    Ok(())
    
}

fn read_txt(path: &Path, stock: &mut Stock) -> std::io::Result<()> {
    // Get the file
    let mut content  = String::new();
    let mut file = try!(File::open(&path));
    try!(file.read_to_string(&mut content));

    // Loop over all the lines in the file contents
    for line in content.lines() {

        // Generate the different parts
        let info: Vec<_> = line.split(",").collect();

        // Simple error handling
        if info.len() != 7 {
            return Err(io::Error::new(ErrorKind::InvalidData, format!("Invalid line: {}", line)));
        }

        // Fetch values from the line.
        // Note: This will crash the whole program, if the
        // file contents are invalid.
        let day: Date = try!(info[0].parse().or(invalid_data(format!("Invalid date: {}", info[0]))));
        let open: f64 = try!(info[1].parse().or(invalid_data(format!("Invalid f64: {}", info[1]))));
        let high: f64 = try!(info[2].parse().or(invalid_data(format!("Invalid f64: {}", info[2]))));
        let low: f64 = try!(info[3].parse().or(invalid_data(format!("Invalid f64: {}", info[3]))));
        let close: f64 = try!(info[4].parse().or(invalid_data(format!("Invalid f64: {}", info[4]))));
        let volume: u64 = try!(info[5].parse().or(invalid_data(format!("Invalid u64: {}", info[5]))));

        // Generate the day
        // Note we multiply, to keep decimal places,
        // and store it as integer value instead of a float value,
        // for the sake of precisison.
        let data = Period {
            start: (open * MULTIPLIER) as Price,
            high: (high * MULTIPLIER) as Price,
            low: (low * MULTIPLIER) as Price,
            end: (close * MULTIPLIER) as Price,
            volume: volume,
        };

        // Use the data
        stock.add_data(day, data);
    }
    Ok(())
}

fn invalid_data<T>(msg: String) -> std::io::Result<T> {
    Err(io::Error::new(ErrorKind::InvalidData, msg))
}

fn handle_txt(path: &Path) {

    let mut stock = Stock::new();

    match read_txt(path, &mut stock) {
        Ok(_) => {},
        Err(e) => {
            println!("Could not load stock file {}: {}", path.display(), Error::description(&e));
            return;
        }
    }
    
    // No more mutability
    let stock = stock;
    let encoded = json::encode(&stock);
    let target = read_string();
    if target.is_none() {
        println!("Failed reading input");
        return;
    }
    let target = target.unwrap();
    let slice = target.trim_right();
    let target = Path::new(&slice);
    let display = target.display();
    let out = File::create(&target);
    if (out.is_err()) {
        println!("Couldn't read {}: {}", target.display(), Error::description(&out.err().unwrap()));
        return;
    }
    match out.ok().unwrap().write_all(encoded.ok().unwrap().as_bytes()) {
        Err(why) => {
            println!("Couldn't write to {}: {}", display, Error::description(&why));
            return;
        },
        Ok(_) => println!("Successfully wrote to {}", display),
    }
}

fn read_string() -> Option<String> {
    let mut str = String::new();
    io::stdin().read_line(&mut str).ok().map(|_| str)
}

// ----------------------------- //
// PERIOD
// ----------------------------- //

#[derive(PartialEq, Eq, Hash, Clone, RustcEncodable, RustcDecodable)]
struct Period {
    start: Price,
    end: Price,
    high: Price,
    low: Price,
    volume: u64,
}

// ----------------------------- //
// STOCK
// ----------------------------- //

#[derive(RustcDecodable, RustcEncodable)]
struct Stock {
    dates: HashMap<String, Period>
}

impl Stock {
    fn new() -> Stock {
        Stock { dates: HashMap::new() }
    }

    fn add_data(&mut self, day: Date, data: Period) {
        self.dates.insert(day.to_string(), data);
    }

    fn get_day_data(&self, day: Date) -> Option<Period> {
        self.dates.get(&day.to_string()).map(|p| p.clone())
    }
    fn get_period_data(&self, from: Date, to: Date) -> Option<Period> {
        if from >= to { panic!(); }
        let days: Vec<_> = self.dates.iter()
            .map(|t| (Date::from_str(&t.0).unwrap(), t.1))
            .filter(|t| t.0 >= from && t.0 <= to).collect();
        let low = days.iter().map(|t| t.1.low).min().unwrap();
        let high = days.iter().map(|t| t.1.high).max().unwrap();
        let start = self.get_day_data(from).map(|p| p.start);
        let end = self.get_day_data(to).map(|p| p.end);
        let volume = days.iter().map(|t| t.1.volume).fold(0, |sum, x| sum + x);
        if start.is_none() || end.is_none() { None }
        else { Some(Period{start: start.unwrap(), end: end.unwrap(), high: high, low: low, volume: volume}) }
    }
}

// ----------------------------- //
// DATE
// ----------------------------- //

/*const MONTH_LENGTHS: [u32; 13] = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn is_leap_year(year: u32) -> bool {
    if (year % 400) == 0 { true }
    else if (year % 100) == 0 { false }
    else { (year % 4) == 0 }
}

fn get_month_length(year: u32, month: u32) -> u32 {
    if is_leap_year(year) && month == 2 { 29 }
    else { MONTH_LENGTHS[month as usize] }
}*/


#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
struct Date {
    year: u32,
    month: u32,
    day: u32,
}

/*impl Date {
    fn is_last_in_month(&self) -> bool {
        self.day == get_month_length(self.year, self.month)
    }
    fn is_last_in_year(&self) -> bool {
        self.is_last_in_month() && self.month == 12
    }
    fn next_day(&self) -> Date {
        if self.is_last_in_year() {
            Date {year: self.year + 1, month: 1, day: 1}
        }
        else if self.is_last_in_month() {
            Date {year: self.year, month: self.month + 1, day: 1}
        }
        else {
            Date {year: self.year, month: self.month, day: self.day + 1}
        }
    }
    
    fn is_first_in_month(&self) -> bool {
        self.day == 1
    }
    fn is_first_in_year(&self) -> bool {
        self.is_first_in_month() && self.month == 1
    }
    fn prior_day(&self) -> Date {
        if self.is_first_in_year() {
            Date {year: self.year - 1, month: 12, day: MONTH_LENGTHS[12]}
        }
        else if self.is_first_in_month() {
            Date {year: self.year, month: self.month - 1, day: MONTH_LENGTHS[(self.month - 1) as usize]}
        }
        else {
            Date {year: self.year, month: self.month, day: self.day - 1}
        }
    }
    
}*/

impl ToString for Date {
    fn to_string(&self) -> String {
        let mut ret = String::new();
        ret.push_str(&self.year.to_string());
        ret.push('-');
        ret.push_str(&to_iso_str(self.month));
        ret.push('-');
        ret.push_str(&to_iso_str(self.day));
        ret
    }
}

fn to_iso_str(i: u32) -> String {
    let ret = i.to_string();
    if (ret.len() > 1) { ret }
    else {"0".to_string() + &ret}
}

impl FromStr for Date {
    type Err = ();
    fn from_str(s: &str) -> Result<Date, ()> {
        let mut it = s.split("-");

        let year = it.next().and_then(|s| s.parse().ok());
        if year.is_none() { return Err(()); }

        let month = it.next().and_then(|s| s.parse().ok());
        if month.is_none() { return Err(()); }
        
        let day = it.next().and_then(|s| s.parse().ok());
        if day.is_none() { return Err(()); }
        
        Ok(Date {year: year.unwrap(), month: month.unwrap(), day: day.unwrap()})
                
    }
}

#[test]
fn date_cmp() {
    let d1 = Date {year: 2015, month: 1, day: 1};
    let d2 = Date {year: 2014, month: 1, day: 1};
    assert_eq!(d1.cmp(&d2), Ordering::Greater);

    let d1 = Date {year: 2015, month: 1, day: 1};
    let d2 = Date {year: 2014, month: 12, day: 1};
    assert_eq!(d1.cmp(&d2), Ordering::Greater);

    let d1 = Date {year: 2015, month: 1, day: 1};
    let d2 = Date {year: 2014, month: 4, day: 30};
    assert_eq!(d1.cmp(&d2), Ordering::Greater);

    let d1 = Date {year: 2014, month: 1, day: 1};
    let d2 = Date {year: 2014, month: 1, day: 12};
    assert_eq!(d1.cmp(&d2), Ordering::Less);

    let d1 = Date {year: 2015, month: 1, day: 1};
    let d2 = Date {year: 2015, month: 1, day: 1};
    assert_eq!(d1.cmp(&d2), Ordering::Equal);

    let d1 = Date {year: 2014, month: 5, day: 1};
    let d2 = Date {year: 2014, month: 3, day: 12};
    assert_eq!(d1.cmp(&d2), Ordering::Greater);
}
