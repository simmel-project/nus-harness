use std::num;
use std::iter::Iterator;

#[derive(Clone, Copy)]
pub struct SteppedRange {
    pub start: u32,
    pub end: u32,
    pub step: u32,
    offset: u32,
}

pub enum SteppedRangeError {
    NoInputProvided,
    NumberParseError(String, num::ParseIntError),
    UnparseableRange(String),
    EndLessThanStart(u32,u32),
    StepIsZero,
}

impl core::fmt::Debug for SteppedRangeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self {
            SteppedRangeError::NoInputProvided => write!(f, "no input provided"),
            SteppedRangeError::NumberParseError(s, e) => {
                write!(f, "unable to parse {}: {:?}", s, e)
            }
            SteppedRangeError::UnparseableRange(s) => write!(f, "unable to parse range {}", s),
            SteppedRangeError::EndLessThanStart(start, end) => write!(f, "range end {} is less than the start {}", end, start),
            SteppedRangeError::StepIsZero => write!(f, "step value is zero"),
        }
    }
}

fn parse_u32(value: &str) -> Result<u32, SteppedRangeError> {
    let (value, base) = get_base(value);
    match u32::from_str_radix(value, base) {
        Ok(o) => Ok(o),
        Err(e) => Err(SteppedRangeError::NumberParseError(value.to_owned(), e)),
    }
}

pub fn get_base(value: &str) -> (&str, u32) {
    if value.starts_with("0x") {
        (value.trim_start_matches("0x"), 16)
    } else if value.starts_with("0X") {
        (value.trim_start_matches("0X"), 16)
    } else if value.starts_with("0b") {
        (value.trim_start_matches("0b"), 2)
    } else if value.starts_with("0B") {
        (value.trim_start_matches("0B"), 2)
    } else if value.starts_with('0') && value != "0" {
        (value.trim_start_matches('0'), 8)
    } else {
        (value, 10)
    }
}

impl SteppedRange {
    pub fn parse(input: &str) -> Result<SteppedRange, SteppedRangeError> {
        let pieces: Vec<&str> = input.split("..").collect();
        match pieces.len() {
            0 => Err(SteppedRangeError::NoInputProvided),
            1 => {
                let n = parse_u32(pieces[0])?;
                Ok(SteppedRange {
                    start: n,
                    end: n,
                    step: 1,
                    offset: n,
                })
            }
            2 => {
                let start = parse_u32(pieces[0])?;
                let end = parse_u32(pieces[1])?;
                if end < start {
                    return Err(SteppedRangeError::EndLessThanStart(start, end));
                }
                Ok(SteppedRange {
                    start,
                    end,
                    step: 1,
                    offset: start,
                })
            }
            3 => {
                let start = parse_u32(pieces[0])?;
                let step = parse_u32(pieces[1])?;
                let end = parse_u32(pieces[2])?;
                if end < start {
                    return Err(SteppedRangeError::EndLessThanStart(start, end));
                }
                if step == 0 {
                    return Err(SteppedRangeError::StepIsZero);
                }
                Ok(SteppedRange {
                    start,
                    step,
                    end,
                    offset: start,
                })
            }
            _ => Err(SteppedRangeError::UnparseableRange(input.to_owned())),
        }
    }
}

impl Iterator for SteppedRange {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset > self.end {
            self.offset = self.start;
            None
        } else {
            let ret_val = self.offset;
            self.offset += self.step;
            Some(ret_val)
        }
    }
}
