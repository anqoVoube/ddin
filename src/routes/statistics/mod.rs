use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, Utc, Datelike, TimeZone};
use scylla::IntoTypedRows;

pub mod router;
mod best;
mod product;


#[derive(Deserialize, Serialize)]
pub struct Search {
    r#type: Types,
    prev: u8
}

trait NaiveDateExt {
    fn days_in_month(&self) -> u32;
    fn days_in_year(&self) -> i32;
    fn is_leap_year(&self) -> bool;
}

impl NaiveDateExt for chrono::NaiveDate {
    fn days_in_month(&self) -> u32 {
        let month = self.month();
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => if self.is_leap_year() { 29 } else { 28 },
            _ => panic!("Invalid month: {}" , month),
        }
    }

    fn days_in_year(&self) -> i32 {
        if self.is_leap_year() { 366 } else { 365 }
    }

    fn is_leap_year(&self) -> bool {
        let year = self.year();
        return year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }
}

fn create_vector(min: u32, max: u32) -> Vec<String> {
    let mut vector: Vec<String> = Vec::new();
    for number in min..=max {
        vector.push(number.to_string());
    }
    vector
}

fn week_vector() -> Vec<String> {
    vec!("Mon".to_string(), "Tue".to_string(), "Wed".to_string(), "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string())
}

fn month_vector() -> Vec<String> {
    vec!(
        "1".to_string(),
        "2".to_string(),
        "3".to_string(),
        "4".to_string(),
        "5".to_string(),
        "6".to_string(),
        "7".to_string(),
        "8".to_string(),
        "9".to_string(),
        "10".to_string(),
        "11".to_string(),
        "12".to_string()
    )
}



#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Types{
    #[serde(rename = "W")]
    Week,
    #[serde(rename = "M")]
    Month,
    #[serde(rename = "Y")]
    Year
}

pub async fn get_date_range(r#type: Types, prev: u8) -> (NaiveDate, NaiveDate, Vec<String>){
    let current_date = Utc::now();

    let (start_date, end_date, namings) = match r#type{
        Types::Week => {
            let days_number_from_monday = current_date.weekday().number_from_monday();
            (
                (current_date + chrono::Duration::days((1 - days_number_from_monday as i32 - (7 * prev) as i32) as i64)).naive_utc().date(),
                (current_date + chrono::Duration::days((7 - days_number_from_monday as i32 - (7 * prev) as i32) as i64)).naive_utc().date(),
                week_vector()
            )

        },
        Types::Month => {
            current_date.naive_utc().date().days_in_month();
            if prev % 12 > (current_date.month() - 1) as u8{
                let working_date = NaiveDate::from_ymd_opt(current_date.year() - (prev / 12) as i32 - 1, 13 - ((prev % 12) - current_date.month() as u8 + 1) as u32, 1).expect("Couldn't convert date");
                (
                    working_date,
                    NaiveDate::from_ymd_opt(working_date.year(), working_date.month(), working_date.days_in_month()).expect("Couldn't convert date"),
                    create_vector(1, working_date.days_in_month())
                )
            } else {
                let working_date = NaiveDate::from_ymd_opt(current_date.year() - (prev / 12) as i32, current_date.month() - (prev % 12) as u32, 1).expect("Couldn't convert date");
                (
                    working_date,
                    NaiveDate::from_ymd_opt(working_date.year(), working_date.month(), working_date.days_in_month()).expect("Couldn't convert date"),
                    create_vector(1, working_date.days_in_month())
                )

            }
        },
        Types::Year => {
            let working_date = NaiveDate::from_ymd_opt(current_date.year() - prev as i32, 1, 1).expect("Couldn't convert date");
            (
                working_date,
                NaiveDate::from_ymd_opt(working_date.year(), 12, 31).expect("Couldn't convert date"),
                month_vector()
            )
        }
    };
}



