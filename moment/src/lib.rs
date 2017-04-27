extern crate chrono;
#[macro_use] extern crate enum_primitive;
extern crate vec_map;

use std::ops;

use vec_map::VecMap;

use chrono::{Datelike, Duration, TimeZone, Timelike};
use chrono::offset::local::Local;
use chrono::datetime::DateTime;

enum_from_primitive! {
    #[derive(Debug,PartialEq,Copy,Clone,Eq,Ord,PartialOrd)]
    pub enum Grain {
        Year = 0,
        Quarter = 1,
        Month = 2,
        Week = 3,
        Day = 4,
        Hour = 5,
        Minute = 6,
        Second = 7,
    }
}

#[derive(Debug,PartialEq,Clone,Eq,Default)]
pub struct Period(VecMap<i64>);

impl Period {
    fn finer_grain(&self) -> Option<Grain> {
        use enum_primitive::FromPrimitive;
        self.0.iter().max_by_key(|&(g,q)| g).and_then(|(g,q)| Grain::from_usize(g))
    }
}

impl From<PeriodComp> for Period {
    fn from(pc: PeriodComp) -> Period {
        Period::default() + pc
    }
}

impl ops::AddAssign<PeriodComp> for Period {
    fn add_assign(&mut self, rhs: PeriodComp) {
        *self.0.entry(rhs.grain as usize).or_insert(0) += rhs.quantity
    }
}

impl ops::Add<PeriodComp> for Period {
    type Output = Period;
    fn add(mut self, pc: PeriodComp) -> Period {
        self += pc;
        self
    }
}

impl ops::Add<Period> for Period {
    type Output = Period;
    fn add(self, p: Period) -> Period {
        let mut result = Period::default();
        for i in 0..8 {
            if !self.0.get(i).is_none() || ! p.0.get(i).is_none() {
                result.0.insert(i, 
                    *self.0.get(i).unwrap_or(&0) + *p.0.get(i).unwrap_or(&0));
            }
        }
        result
    }
}

impl<'a> ops::Add<&'a Period> for Period {
    type Output = Period;
    fn add(self, p: &'a Period) -> Period {
        let mut result = Period::default();
        for i in 0..8 {
            if !self.0.get(i).is_none() || ! p.0.get(i).is_none() {
                result.0.insert(i, 
                    *self.0.get(i).unwrap_or(&0) + *p.0.get(i).unwrap_or(&0));
            }
        }
        result
    }
}

impl<'a, 'b> ops::Add<&'a Period> for &'b Period {
    type Output = Period;
    fn add(self, p: &'a Period) -> Period {
        let mut result = Period::default();
        for i in 0..8 {
            if !self.0.get(i).is_none() || ! p.0.get(i).is_none() {
                result.0.insert(i, 
                    *self.0.get(i).unwrap_or(&0) + *p.0.get(i).unwrap_or(&0));
            }
        }
        result
    }
}

impl<'a> ops::Add<Period> for &'a Period {
    type Output = Period;
    fn add(self, p: Period) -> Period {
        let mut result = Period::default();
        for i in 0..8 {
            if !self.0.get(i).is_none() || ! p.0.get(i).is_none() {
                result.0.insert(i, 
                    *self.0.get(i).unwrap_or(&0) + *p.0.get(i).unwrap_or(&0));
            }
        }
        result
    }
}

impl ops::Neg for Period {
    type Output = Period;

    fn neg(self) -> Period {
        Period(self.0.iter().map(|(k,v)| (k,-*v)).collect())
    }
}


#[derive(Debug,PartialEq,Copy,Clone,Eq)]
pub struct PeriodComp {
    grain: Grain,
    quantity: i64,
}

impl PeriodComp {
    pub fn years(n: i64) -> PeriodComp {
        PeriodComp {
            grain: Grain::Year,
            quantity: n,
        }
    }
    pub fn quarters(n: i64) -> PeriodComp {
        PeriodComp {
            grain: Grain::Quarter,
            quantity: n,
        }
    }
    pub fn months(n: i64) -> PeriodComp {
        PeriodComp {
            grain: Grain::Month,
            quantity: n,
        }
    }
    pub fn weeks(n: i64) -> PeriodComp {
        PeriodComp {
            grain: Grain::Week,
            quantity: n,
        }
    }
    pub fn days(n: i64) -> PeriodComp {
        PeriodComp {
            grain: Grain::Day,
            quantity: n,
        }
    }
    pub fn hours(n: i64) -> PeriodComp {
        PeriodComp {
            grain: Grain::Hour,
            quantity: n,
        }
    }
    pub fn minutes(n: i64) -> PeriodComp {
        PeriodComp {
            grain: Grain::Minute,
            quantity: n,
        }
    }
    pub fn seconds(n: i64) -> PeriodComp {
        PeriodComp {
            grain: Grain::Second,
            quantity: n,
        }
    }
}

impl ops::Neg for PeriodComp {
    type Output = PeriodComp;

    fn neg(self) -> PeriodComp {
        PeriodComp {
            quantity: -self.quantity,
            ..self
        }
    }
}

#[derive(Debug,PartialEq,Copy,Clone,PartialOrd,Eq,Ord)]
struct Moment(DateTime<Local>);

fn last_day_in_month(y: i32, m: u32) -> i64 {
    assert!(m >= 1 && m <= 12);
    for d in 28..31 {
        if (Local.ymd_opt(y, m, d + 1)).single().is_none() {
            return d as i64;
        }
    }
    31
}

impl ops::Deref for Moment {
    type Target = DateTime<Local>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Moment {
    fn add_months(self, n: i32) -> Moment {
        let (year, month0) = if n >= 0 {
            let n = n as u32;
            let carry = ((self.month0() + n % 12) >= 12) as i32;
            (self.year() + (n / 12) as i32 + carry, (self.month0() + n) % 12)
        } else {
            let n = -n as u32;
            let borrow = (self.month0() < n % 12) as i32;
            (self.year() - (n / 12) as i32 - borrow as i32, (12 + self.month0() - (n % 12)) % 12)
        };
        let target_month_days = last_day_in_month(year, month0 + 1);
        let day = ::std::cmp::min(target_month_days, self.day() as i64) as u32;
        Moment(Local
                   .ymd(year, month0 + 1, day)
                   .and_hms(self.hour(), self.minute(), self.second()))
    }

    fn round_to(self, g: Grain) -> Moment {
        match g {
            Grain::Year => Moment(Local.ymd(self.year(), 1, 1).and_hms(0, 0, 0)),
            Grain::Month => Moment(Local.ymd(self.year(), self.month(), 1).and_hms(0, 0, 0)),
            Grain::Day => Moment(self.date().and_hms(0, 0, 0)),
            Grain::Hour => Moment(self.date().and_hms(self.hour(), 0, 0)),
            Grain::Minute => Moment(self.date().and_hms(self.hour(), self.minute(), 0)),
            Grain::Second => self,
            Grain::Week => {
                // shift to monday morning
                let day_offset = self.weekday().num_days_from_monday(); // monday is 0 here
                self.round_to(Grain::Day) - PeriodComp::days(day_offset as i64)
            }
            Grain::Quarter => {
                self.round_to(Grain::Month) - PeriodComp::months(self.month0() as i64 % 3)
            }
        }
    }
}

impl ops::Add<Period> for Moment {
    type Output = Moment;
    fn add(self, p: Period) -> Moment {
        use enum_primitive::FromPrimitive;
        let mut result = self;
        for (g, q) in p.0.iter() {
            result = result +
                     PeriodComp {
                         grain: Grain::from_usize(g).unwrap(), // checked
                         quantity: *q,
                     };
        }
        result
    }
}

impl<'a> ops::Add<&'a Period> for Moment {
    type Output = Moment;
    fn add(self, p: &'a Period) -> Moment {
        use enum_primitive::FromPrimitive;
        let mut result = self;
        for (g, q) in p.0.iter() {
            result = result +
                     PeriodComp {
                         grain: Grain::from_usize(g).unwrap(), // checked
                         quantity: *q,
                     };
        }
        result
    }
}

impl ops::Add<PeriodComp> for Moment {
    type Output = Moment;
    fn add(self, p: PeriodComp) -> Moment {
        match p.grain {
            Grain::Year => self.add_months(12 * p.quantity as i32),
            Grain::Quarter => self.add_months(3 * p.quantity as i32),
            Grain::Month => self.add_months(p.quantity as i32),
            Grain::Week => Moment(self.0 + Duration::weeks(p.quantity)),
            Grain::Day => Moment(self.0 + Duration::days(p.quantity)),
            Grain::Hour => Moment(self.0 + Duration::hours(p.quantity)),
            Grain::Minute => Moment(self.0 + Duration::minutes(p.quantity)),
            Grain::Second => Moment(self.0 + Duration::seconds(p.quantity)),
        }
    }
}

impl ops::Sub<PeriodComp> for Moment {
    type Output = Moment;
    fn sub(self, p: PeriodComp) -> Moment {
        self + -p
    }
}

#[derive(Debug,PartialEq,Copy,Clone)]
struct Interval {
    start: Moment,
    grain: Grain,
    end: Option<Moment>,
}

impl Interval {
    pub fn end_moment(self) -> Moment {
        self.end
            .unwrap_or_else(|| {
                                self.start +
                                PeriodComp {
                                    quantity: 1,
                                    grain: self.grain,
                                }
                            })
    }

    pub fn after(self) -> Interval {
        Interval {
            start: self.end_moment(),
            grain: self.grain,
            end: None,
        }
    }

    pub fn to(self, other: Interval) -> Interval {
        Interval {
            start: self.start,
            grain: ::std::cmp::max(self.grain, other.grain),
            end: Some(other.start),
        }
    }

    pub fn union(self, other: Interval) -> Interval {
        Interval {
            start: self.start,
            grain: ::std::cmp::max(self.grain, other.grain),
            end: Some(other.end_moment()),
        }
    }

    pub fn intersect(self, other: Interval) -> Option<Interval> {
        if self.start <= other.start {
            if other.start >= self.end_moment() {
                None
            } else if other.end_moment() <= self.end_moment() {
                Some(other)
            } else {
                Some(Interval {
                         start: other.start,
                         grain: ::std::cmp::max(self.grain, other.grain),
                         end: Some(self.end_moment()),
                     })
            }
        } else {
            other.intersect(self)
        }
    }

    pub fn seconds(self) -> i64 {
        self.end_moment()
            .0
            .signed_duration_since(self.start.0)
            .num_seconds()
    }
}

impl ops::Add<PeriodComp> for Interval {
    type Output = Interval;
    fn add(self, p: PeriodComp) -> Interval {
        Interval {
            start: self.start + p,
            end: self.end.map(|it| it + p),
            grain: ::std::cmp::max(self.grain, p.grain),
        }
    }
}

impl ops::Sub<PeriodComp> for Interval {
    type Output = Interval;
    fn sub(self, p: PeriodComp) -> Interval {
        self + -p
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use chrono::TimeZone;
    use chrono::offset::local::Local;
    use chrono::datetime::DateTime;

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(last_day_in_month(2015, 2), 28);
        assert_eq!(last_day_in_month(2016, 1), 31);
        assert_eq!(last_day_in_month(2016, 2), 29);
        assert_eq!(last_day_in_month(2016, 3), 31);
        assert_eq!(last_day_in_month(2016, 4), 30);
        assert_eq!(last_day_in_month(2016, 5), 31);
        assert_eq!(last_day_in_month(2016, 6), 30);
        assert_eq!(last_day_in_month(2016, 7), 31);
        assert_eq!(last_day_in_month(2016, 8), 31);
        assert_eq!(last_day_in_month(2016, 9), 30);
        assert_eq!(last_day_in_month(2016, 10), 31);
        assert_eq!(last_day_in_month(2016, 11), 30);
        assert_eq!(last_day_in_month(2016, 12), 31);
    }

    #[test]
    fn add_months_to_moment() {
        let now = Moment(Local.ymd(2017, 04, 25).and_hms(9, 10, 11));
        assert_eq!(Moment(Local.ymd(2017, 05, 25).and_hms(9, 10, 11)),
                   now.add_months(1));
        assert_eq!(Moment(Local.ymd(2018, 04, 25).and_hms(9, 10, 11)),
                   now.add_months(12));
        assert_eq!(Moment(Local.ymd(2018, 01, 25).and_hms(9, 10, 11)),
                   Moment(Local.ymd(2017, 12, 25).and_hms(9, 10, 11)).add_months(1));
        assert_eq!(Moment(Local.ymd(2017, 06, 30).and_hms(9, 10, 11)),
                   Moment(Local.ymd(2017, 05, 31).and_hms(9, 10, 11)).add_months(1));
        // daylight saving brainfuck
        assert_eq!(Moment(Local.ymd(2017, 03, 26).and_hms(3, 30, 00)),
                   Moment(Local.ymd(2017, 02, 26).and_hms(2, 30, 00)).add_months(1));
    }

    #[test]
    fn add_period_comp_to_moment() {
        let now = Moment(Local.ymd(2017, 04, 25).and_hms(9, 10, 11));
        assert_eq!(Moment(Local.ymd(2017, 04, 25).and_hms(9, 10, 21)),
                   now + PeriodComp::seconds(10));
        assert_eq!(Moment(Local.ymd(2017, 04, 25).and_hms(9, 20, 11)),
                   now + PeriodComp::minutes(10));
        assert_eq!(Moment(Local.ymd(2017, 04, 25).and_hms(19, 10, 11)),
                   now + PeriodComp::hours(10));
        assert_eq!(Moment(Local.ymd(2017, 05, 5).and_hms(9, 10, 11)),
                   now + PeriodComp::days(10));
        assert_eq!(Moment(Local.ymd(2017, 05, 2).and_hms(9, 10, 11)),
                   now + PeriodComp::weeks(1));
        assert_eq!(Moment(Local.ymd(2018, 02, 25).and_hms(9, 10, 11)),
                   now + PeriodComp::months(10));
        assert_eq!(Moment(Local.ymd(2017, 07, 25).and_hms(9, 10, 11)),
                   now + PeriodComp::quarters(1));
        assert_eq!(Moment(Local.ymd(2027, 04, 25).and_hms(9, 10, 11)),
                   now + PeriodComp::years(10));
    }

    #[test]
    fn add_period_to_moment() {
        let now = Moment(Local.ymd(2017, 04, 25).and_hms(9, 10, 11));

        let mut period = Period::default();
        period.0.insert(Grain::Year as usize, 2);
        period.0.insert(Grain::Month as usize, 3);

        assert_eq!(Moment(Local.ymd(2019, 07, 25).and_hms(9, 10, 11)),
                   now + &period);

        period.0.insert(Grain::Hour as usize, 5);

        assert_eq!(Moment(Local.ymd(2019, 07, 25).and_hms(14, 10, 11)),
                   now + &period);
    }

    #[test]
    fn sub_months_to_moment() {
        let now = Moment(Local.ymd(2017, 04, 25).and_hms(9, 10, 11));
        assert_eq!(Moment(Local.ymd(2017, 03, 25).and_hms(9, 10, 11)),
                   now.add_months(-1));
        assert_eq!(Moment(Local.ymd(2016, 04, 25).and_hms(9, 10, 11)),
                   now.add_months(-12));
        assert_eq!(Moment(Local.ymd(2017, 12, 25).and_hms(9, 10, 11)),
                   Moment(Local.ymd(2018, 01, 25).and_hms(9, 10, 11)).add_months(-1));
        assert_eq!(Moment(Local.ymd(2017, 06, 30).and_hms(9, 10, 11)),
                   Moment(Local.ymd(2017, 07, 31).and_hms(9, 10, 11)).add_months(-1));
        // daylight saving brainfuck
        assert_eq!(Moment(Local.ymd(2017, 03, 26).and_hms(3, 30, 00)),
                   Moment(Local.ymd(2017, 04, 26).and_hms(2, 30, 00)).add_months(-1));
    }

    #[test]
    fn moment_round_to() {
        let now = Moment(Local.ymd(2017, 04, 25).and_hms(9, 10, 11));
        assert_eq!(Moment(Local.ymd(2017, 01, 01).and_hms(0, 0, 0)),
                   now.round_to(Grain::Year));
        assert_eq!(Moment(Local.ymd(2017, 04, 01).and_hms(0, 0, 0)),
                   now.round_to(Grain::Month));
        assert_eq!(Moment(Local.ymd(2017, 04, 25).and_hms(0, 0, 0)),
                   now.round_to(Grain::Day));
        assert_eq!(Moment(Local.ymd(2017, 04, 25).and_hms(9, 0, 0)),
                   now.round_to(Grain::Hour));
        assert_eq!(Moment(Local.ymd(2017, 04, 25).and_hms(9, 10, 0)),
                   now.round_to(Grain::Minute));
        assert_eq!(Moment(Local.ymd(2017, 04, 25).and_hms(9, 10, 11)),
                   now.round_to(Grain::Second));
        assert_eq!(Moment(Local.ymd(2017, 04, 24).and_hms(0, 0, 0)),
                   now.round_to(Grain::Week));
        assert_eq!(Moment(Local.ymd(2017, 04, 01).and_hms(0, 0, 0)),
                   now.round_to(Grain::Quarter));
    }

    #[test]
    fn interval_add_period() {
        let now = Moment(Local.ymd(2017, 04, 25).and_hms(0, 0, 0));
        let interval = Interval {
            start: now,
            grain: Grain::Day,
            end: None,
        };
        let plus_one_hour = interval + PeriodComp::hours(1);
        assert_eq!(now + PeriodComp::hours(1), plus_one_hour.start);
        assert_eq!(Grain::Hour, plus_one_hour.grain);
    }

    #[test]
    fn interval_binary() {
        let interval = Interval {
            start: Moment(Local.ymd(2017, 04, 25).and_hms(0, 0, 0)),
            grain: Grain::Day,
            end: None,
        };
        let other = Interval {
            start: Moment(Local.ymd(2017, 04, 26).and_hms(9, 0, 0)),
            grain: Grain::Hour,
            end: None,
        };
        let result = interval.to(other);
        assert_eq!(interval.start, result.start);
        assert_eq!(Some(other.start), result.end);
        assert_eq!(Grain::Hour, result.grain);

        let result = interval.union(other);
        assert_eq!(interval.start, result.start);
        assert_eq!(Some(other.end_moment()), result.end);
        assert_eq!(Grain::Hour, result.grain);
    }

    #[test]
    fn interval_intersect() {
        let interval = Interval {
            start: Moment(Local.ymd(2017, 04, 25).and_hms(0, 0, 0)),
            grain: Grain::Day,
            end: Some(Moment(Local.ymd(2017, 04, 30).and_hms(0, 0, 0))),
        };
        assert_eq!(interval, interval.intersect(interval).unwrap());
        let other = Interval {
            start: Moment(Local.ymd(2017, 04, 26).and_hms(9, 0, 0)),
            grain: Grain::Hour,
            end: Some(Moment(Local.ymd(2017, 04, 26).and_hms(11, 0, 0))),
        };
        assert_eq!(other, interval.intersect(other).unwrap());
        let other = Interval {
            start: Moment(Local.ymd(2017, 04, 26).and_hms(9, 0, 0)),
            grain: Grain::Hour,
            end: Some(Moment(Local.ymd(2017, 05, 08).and_hms(11, 0, 0))),
        };
        assert_eq!(Interval {
                       start: Moment(Local.ymd(2017, 04, 26).and_hms(9, 0, 0)),
                       grain: Grain::Hour,
                       end: Some(Moment(Local.ymd(2017, 04, 30).and_hms(0, 0, 0))),
                   },
                   interval.intersect(other).unwrap());
        assert_eq!(Interval {
                       start: Moment(Local.ymd(2017, 04, 26).and_hms(9, 0, 0)),
                       grain: Grain::Hour,
                       end: Some(Moment(Local.ymd(2017, 04, 30).and_hms(0, 0, 0))),
                   },
                   other.intersect(interval).unwrap());
        let other = Interval {
            start: Moment(Local.ymd(2017, 05, 26).and_hms(9, 0, 0)),
            grain: Grain::Hour,
            end: Some(Moment(Local.ymd(2017, 06, 08).and_hms(11, 0, 0))),
        };
        assert_eq!(None, interval.intersect(other));
    }

    #[test]
    fn seconds() {
        let interval = Interval {
            start: Moment(Local.ymd(2017, 04, 25).and_hms(0, 0, 0)),
            grain: Grain::Day,
            end: Some(Moment(Local.ymd(2017, 04, 30).and_hms(0, 0, 0))),
        };
        assert_eq!(5 * 86400, interval.seconds());
    }

    #[test]
    fn period_comp_add_to_period() {
        assert_eq!(Some(&1), 
            (Period::default() + PeriodComp::years(1)).0.get(Grain::Year as usize));
        assert_eq!(Some(&1), 
            (Period::default() + PeriodComp::days(1)).0.get(Grain::Day as usize));
        assert_eq!(None, 
            (Period::default() + PeriodComp::days(1)).0.get(Grain::Year as usize));
    }

    #[test]
    fn period_comp_add_assign_to_period() {
        let mut period = Period::default();
        period += PeriodComp::years(1);
        assert_eq!(Some(&1), period.0.get(Grain::Year as usize));
    }

    #[test]
    fn period_add_to_period() {
        let mut a = Period::default();
        a.0.insert(Grain::Year as usize, 1);

        let mut b = Period::default();
        b.0.insert(Grain::Day as usize, 2);

        let mut c = Period::default();
        c.0.insert(Grain::Day as usize, 3);
        c.0.insert(Grain::Year as usize, 5);
        c.0.insert(Grain::Month as usize, 1);

        assert_eq!(Some(&2), (&a + &b).0.get(Grain::Day as usize));
        assert_eq!(Some(&1), (&a + &b).0.get(Grain::Year as usize));
        assert_eq!(Some(&6), (&a + &c).0.get(Grain::Year as usize));
        assert_eq!(Some(&5), (&b + &c).0.get(Grain::Day as usize));
        assert_eq!(Some(&1), (&b + &c + &a).0.get(Grain::Month as usize));
    }

    #[test]
    fn neg_period() {
        let mut a = Period::default();
        a.0.insert(Grain::Year as usize, 1);

        assert_eq!(Some(&-1), (-a).0.get(Grain::Year as usize));
    }

    #[test]
    fn finer_grain() {
        let mut a = Period::default();
        a.0.insert(Grain::Year as usize, 1);
        a.0.insert(Grain::Month as usize, 3);
        assert_eq!(a.finer_grain(), Some(Grain::Month));

        a.0.insert(Grain::Hour as usize, 4);
        assert_eq!(a.finer_grain(), Some(Grain::Hour));
    }

}