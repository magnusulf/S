use std::str::FromStr;
use std::collections::HashMap;
use std::cmp::*;

type Price = i64;

fn main() {
    println!("Hello, world!");
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

struct Stock<'a> {
    dates: HashMap<&'a str, Price>
}

impl<'a> Stock<'a> {
    fn new() -> Stock<'a> {
        Stock { dates: HashMap::new() }
    }

    fn get_by_str(&self, day: &str) -> Option<Price> {
        self.dates.get(day).map(|p| p.clone())
    }
    fn get_close_price(&self, day: Date) -> Option<Price> {
        self.get_by_str(&day.to_string())
    }
    fn get_day_data(&self, day: Date) -> Option<Period> {
        let open = self.get_close_price(day.prior_day());
        let close = self.get_close_price(day);
        if open.is_none() || close.is_none() {
            None
        }
        else {
            Some(Period{start: open.unwrap().clone(), end: close.unwrap().clone()})
        }
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
