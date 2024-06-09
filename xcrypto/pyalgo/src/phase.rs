use crate::constant::Phase;
use chrono::{DateTime, NaiveTime, Timelike, Utc};
use chrono::{TimeDelta, TimeZone};
use chrono_tz::{Asia::Shanghai, Tz};
use pyo3::prelude::*;
use std::cmp::Ordering;

#[derive(Debug)]
struct SegMap<K, V>
where
    K: Clone + PartialOrd + PartialEq,
    V: Clone,
{
    cnt: u8,
    keys: Vec<K>,
    vals: Vec<V>,
}

impl<K, V> SegMap<K, V>
where
    K: Clone + Copy + PartialOrd + Ord,
    V: Clone + Copy + PartialEq,
{
    pub fn new(min_key: K, max_key: K, defval: V) -> Self {
        let mut keys = vec![max_key; 24];
        let vals = vec![defval; 24];
        keys[0] = min_key;

        SegMap { cnt: 1, keys, vals }
    }

    pub fn keys(&self) -> Vec<K> {
        self.keys.clone()
    }

    pub fn vals(&self) -> Vec<V> {
        self.vals.clone()
    }

    pub fn capacity(&self) -> u8 {
        24
    }

    pub fn add(&mut self, key: K, val: V) {
        let mut idx = 0;

        if self.cnt >= self.capacity() {
            return;
        }

        for i in 1..self.capacity() as usize {
            idx = i;
            match self.keys[i].cmp(&key) {
                Ordering::Equal => {
                    if self.vals[i] != val {
                        self.vals[i] = val;
                        return;
                    }
                }
                Ordering::Greater => {
                    for j in (1 + i..=self.cnt as usize).rev() {
                        self.keys[j] = self.keys[j - 1];
                        self.vals[j] = self.vals[j - 1];
                    }
                    break;
                }
                Ordering::Less => (),
            }
        }

        self.keys[idx] = key;
        self.vals[idx] = val;
        self.cnt += 1;
    }

    pub fn find(&self, key: K) -> V {
        for i in 0..self.cnt as usize {
            if self.keys[i] > key {
                return self.vals[i - 1];
            }
        }
        self.vals[self.cnt as usize - 1]
    }
}

#[pyclass]
pub struct TradingPhase {
    segmap: SegMap<u32, Phase>,
}

#[pymethods]
impl TradingPhase {
    #[new]
    pub fn new() -> Self {
        TradingPhase {
            segmap: SegMap::new(0, 86400, Phase::UNDEF),
        }
    }

    pub fn keys(&self) -> Vec<u32> {
        self.segmap.keys()
    }

    pub fn vals(&self) -> Vec<Phase> {
        self.segmap.vals()
    }

    pub fn add_phase(&mut self, hour: u32, minute: u32, second: u32, phase: Phase) {
        let (naive, _) = NaiveTime::from_hms_opt(hour, minute, second)
            .unwrap()
            .overflowing_sub_signed(TimeDelta::hours(8));
        self.segmap.add(naive.num_seconds_from_midnight(), phase);
    }

    fn to_second(&self, mills: i64) -> u32 {
        let mills = Utc.timestamp_millis_opt(mills).unwrap();
        mills.num_seconds_from_midnight()
    }

    pub fn determine(&self, mills: i64) -> Phase {
        self.segmap.find(self.to_second(mills))
    }

    pub fn to_datetime(&self, mills: i64) -> DateTime<Tz> {
        DateTime::from_timestamp_millis(mills)
            .unwrap()
            .with_timezone(&Shanghai)
    }
}

impl Default for TradingPhase {
    fn default() -> Self {
        Self::new()
    }
}
