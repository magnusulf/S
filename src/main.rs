extern crate rustc_serialize;

use std::str::FromStr;
use std::collections::HashMap;
use std::cmp::*;
use rustc_serialize::json;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

type Price = i64;

fn main() {

    let path = Path::new("stock.txt");
    let display = path.display();
    
    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   Error::description(&why)),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display,
                                                   Error::description(&why)),
        Ok(_) => print!("{} contains:\n{}", display, s),
    }
    let s = s;

    let stock: Stock = json::decode(&s).unwrap();
}

// ----------------------------- //
// PERIOD
// ----------------------------- //

struct Period {
    start: Price,
    end: Price,
}

impl Period {
    fn change(&self) -> i64 {
        self.end - self.start
    }
    fn increased(&self) -> bool {
        0 < self.change()
    }
    fn decreased(&self) -> bool {
        0 > self.change()
    }
    fn changed(&self) -> bool {
        self.change() != 0
    }
}

// ----------------------------- //
// STOCK
// ----------------------------- //

#[derive(RustcDecodable, RustcEncodable)]
struct Stock {
    dates: HashMap<String, Price>
}

impl Stock {
    fn new() -> Stock {
        Stock { dates: HashMap::new() }
    }

    fn get_close_price(&self, day: Date) -> Option<Price> {
        self.dates.get(&day.to_string()).map(|p| p.clone())
    }
    fn get_period_data(&self, from: Date, to: Date) -> Option<Period> {
        if to < from { panic!(); }
        let start = self.get_close_price(from);
        let end = self.get_close_price(to);
        if start.is_none() || end.is_none() {
            None
        } else {
            Some(Period{start: start.unwrap().clone(), end: end.unwrap().clone()})
        }
    }
    fn get_day_data(&self, day: Date) -> Option<Period> {
        self.get_period_data(day.prior_day(), day )
    }
}

// ----------------------------- //
// DATE
// ----------------------------- //

const MONTH_LENGTHS: [u32; 13] = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn is_leap_year(year: u32) -> bool {
    if (year % 400) == 0 { true }
    else if (year % 100) == 0 { false }
    else { (year % 4) == 0 }
}

fn get_month_length(year: u32, month: u32) -> u32 {
    if is_leap_year(year) && month == 2 { 29 }
    else { MONTH_LENGTHS[month as usize] }
}


#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
struct Date {
    year: u32,
    month: u32,
    day: u32,
}

impl Date {
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
    
}

impl ToString for Date {
    fn to_string(&self) -> String {
        let mut ret = String::new();
        ret.push_str(&self.year.to_string());
        ret.push('-');
        ret.push_str(&self.month.to_string());
        ret.push('-');
        ret.push_str(&self.day.to_string());
        ret
    }
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
