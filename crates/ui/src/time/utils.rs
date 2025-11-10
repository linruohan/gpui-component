use chrono::{Datelike, Duration, NaiveDate};

trait NaiveDateExt {
    fn days_in_month(&self) -> i32;
    fn is_leap_year(&self) -> bool;
}

impl NaiveDateExt for chrono::NaiveDate {
    fn days_in_month(&self) -> i32 {
        let month = self.month();
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if self.is_leap_year() {
                    29
                } else {
                    28
                }
            }
            _ => panic!("Invalid month: {}", month),
        }
    }

    fn is_leap_year(&self) -> bool {
        let year = self.year();
        return year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    }
}

pub(crate) fn days_in_month(year: i32, month: u32) -> Vec<Vec<NaiveDate>> {
    let mut year = year;
    let mut month = month;
    if month > 12 {
        year += 1;
        month = 1;
    }
    if month < 1 {
        year -= 1;
        month = 12;
    }

    let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let num_days = date.days_in_month();
    let start_weekday = date.weekday().num_days_from_monday();

    // Get the days in the month, 2023-02 will returns
    // "29|30|31| 1| 2| 3| 4",
    // " 5| 6| 7| 8| 9|10|11",
    // "12|13|14|15|16|17|18",
    // "19|20|21|22|23|24|25",
    // "26|27|28| 1| 2| 3| 4",
    let mut days = vec![];
    for n in 0..5 {
        let mut week_days = vec![];
        for weekday in 0..7 {
            let (mut y, mut m) = (year, month);

            // If the day is less than the start weekday, we need to go back to the previous month.
            if n == 0 && weekday < start_weekday {
                m = if m == 1 { 12 } else { m - 1 };
                y = if m == 1 { year - 1 } else { y };
            }

            // If start_weekday is 3, and n is 0 and weekday is 3, then day is 1.
            // If start_weekday is 3, and n is 1 and weekday is 4, then day is 9.
            let day = n * 7 + weekday as i32 - start_weekday as i32;

            // If the day is greater than the number of days in the month, we need to go to the next month.
            if day > num_days {
                m = if m == 12 { 1 } else { m + 1 };
                y = if m == 1 { year + 1 } else { y };
            }

            #[allow(clippy::expect_fun_call)]
            let date = date
                .checked_add_signed(Duration::days(day as i64))
                .expect(&format!("invalid date {}-{} days {}", y, m, day));
            week_days.push(date);
        }

        days.push(week_days);
    }

    days
}
pub fn get_holiday_by_lunar_rust(date: NaiveDate) -> String {
    // 普通日："15 十五"
    // 节假日："1 元旦 休"
    // 调班日："4 清明节 班"
    use chrono::Datelike;
    use lunar_rust::{
        holiday::HolidayRefHelper,
        lunar::LunarRefHelper,
        solar::{self, SolarRefHelper},
        util::holiday_util::{self, HolidayUtilRefHelper},
    };
    let solar_day = solar::from_date(date.and_hms_opt(0, 0, 0).unwrap());
    let week = solar_day.get_week();
    let lunar_day = solar_day.get_lunar();
    let jie_qi = lunar_day.get_jie_qi();
    let holiday_info = holiday_util::get().get_holiday(
        date.year() as i64,
        Some(date.month() as i64),
        Some(date.day() as i64),
    );
    match holiday_info {
        Some(holiday) => {
            let work_status = if holiday.is_work() { "班" } else { "休" };
            format!("{} {} {}", date.day(), holiday.get_name(), work_status)
        }
        None => {
            let day_in_chinese = lunar_day.get_day_in_chinese();
            let day_status = if week == 0 || week == 6 { "休" } else { "" };
            format!(
                "{} {} {}",
                date.day(),
                if jie_qi.is_empty() {
                    day_in_chinese
                } else {
                    jie_qi
                },
                day_status
            )
        }
    }
}
pub fn get_holiday_by_tyme4rs(date: NaiveDate) -> String {
    // 使用tyme4rs获取，但是节气信息不准确
    use tyme4rs::tyme::{holiday::LEGAL_HOLIDAY_NAMES, lunar::LUNAR_DAY_NAMES, solar::SolarDay};

    let solar_day = SolarDay::from_ymd(
        date.year() as isize,
        date.month() as usize,
        date.day() as usize,
    );

    if let Some(holiday) = solar_day.get_legal_holiday() {
        format!(
            "{} {} {}",
            date.day(),
            LEGAL_HOLIDAY_NAMES[holiday.get_index()],
            if holiday.is_work() { "班" } else { "休" }
        )
    } else {
        let lunar = solar_day.get_lunar_day();
        let lunar_day_name = LUNAR_DAY_NAMES[lunar.get_day() - 1].to_string();
        let jieqi = solar_day.get_term();

        format!(
            "{} {}",
            date.day(),
            lunar_day_name // if jieqi.is_jie() {
                           //     lunar_day_name
                           // } else {
                           //     jieqi.to_string()
                           // }
        )
    }
}
#[cfg(test)]
mod tests {
    use chrono::{Datelike, NaiveDate};

    use super::{days_in_month, NaiveDateExt};

    #[test]
    fn test_days_in_month() {
        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap().days_in_month(),
            29
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2023, 2, 1).unwrap().days_in_month(),
            28
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().days_in_month(),
            31
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2023, 4, 1).unwrap().days_in_month(),
            30
        );
    }

    #[test]
    fn test_days() {
        #[track_caller]
        fn assert_case(date: NaiveDate, expected: Vec<&str>) {
            let out = days_in_month(date.year(), date.month())
                .iter()
                .map(|week| {
                    week.iter()
                        .map(|d| {
                            if d.year() == date.year() && d.month() == date.month() {
                                format!("{:2}", d.day())
                            } else if d.year() == date.year() {
                                format!("{}-{}", d.month(), d.day())
                            } else {
                                format!("{}-{}-{}", d.year(), d.month(), d.day())
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("|")
                })
                .collect::<Vec<_>>();

            assert_eq!(out, expected);
        }

        assert_case(
            NaiveDate::from_ymd_opt(2024, 8, 1).unwrap(),
            vec![
                "7-28|7-29|7-30|7-31| 1| 2| 3",
                " 4| 5| 6| 7| 8| 9|10",
                "11|12|13|14|15|16|17",
                "18|19|20|21|22|23|24",
                "25|26|27|28|29|30|31",
            ],
        );
        assert_case(
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            vec![
                "2024-12-29|2024-12-30|2024-12-31| 1| 2| 3| 4",
                " 5| 6| 7| 8| 9|10|11",
                "12|13|14|15|16|17|18",
                "19|20|21|22|23|24|25",
                "26|27|28|29|30|31|2-1",
            ],
        );

        assert_case(
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            vec![
                "1-28|1-29|1-30|1-31| 1| 2| 3",
                " 4| 5| 6| 7| 8| 9|10",
                "11|12|13|14|15|16|17",
                "18|19|20|21|22|23|24",
                "25|26|27|28|29|3-1|3-2",
            ],
        );
        assert_case(
            NaiveDate::from_ymd_opt(2023, 2, 20).unwrap(),
            vec![
                "1-29|1-30|1-31| 1| 2| 3| 4",
                " 5| 6| 7| 8| 9|10|11",
                "12|13|14|15|16|17|18",
                "19|20|21|22|23|24|25",
                "26|27|28|3-1|3-2|3-3|3-4",
            ],
        );
    }
}
