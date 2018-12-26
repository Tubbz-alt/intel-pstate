//! Crate for fetching and modifying the intel_pstate kernel parameters.
//!
//! # Example
//!
//! ```rust,no_run
//! use std::io;
//! use intel_pstate::PState;
//!
//! fn main() -> io::Result<()> {
//!     if let Ok(pstate) = PState::new() {
//!         pstate.set_min_perf_pct(50)?;
//!         pstate.set_max_perf_pct(100)?;
//!         pstate.set_no_turbo(false)?;
//!     }
//!
//!     Ok(())
//! }
//! ```

#[macro_use]
extern crate smart_default;

use std::io::{self, Read, Write};
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, SmartDefault)]
/// A set of pstate values that was retrieved, or is to be set.
pub struct PStateValues {
    pub min_perf_pct: u8,
    #[default = "100"]
    pub max_perf_pct: u8,
    pub no_turbo: bool
}

/// Handle for fetching and modifying Intel PState kernel parameters.
/// 
/// # Note
/// 
/// - Currently, ony Linux is supported.
/// - Setting parameters will require root permissions.
pub struct PState {
    path: PathBuf,
}

impl PState {
    /// Attempt to fetch a handle to the Intel PState sysfs kernel instance.
    pub fn new() -> io::Result<PState> {
        let path = PathBuf::from("/sys/devices/system/cpu/intel_pstate");
        if path.is_dir() {
            Ok(PState { path })
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "intel_pstate directory not found"))
        }
    }

    /// Get the minimum performance percent.
    pub fn min_perf_pct(&self) -> io::Result<u8> {
        parse_file(self.path.join("min_perf_pct"))
    }

    /// Set the minimum performance percent.
    pub fn set_min_perf_pct(&self, value: u8) -> io::Result<()> {
        write_file(self.path.join("min_perf_pct"), format!("{}", value))
    }

    /// Get the maximum performance percent.
    pub fn max_perf_pct(&self) -> io::Result<u8> {
        parse_file(self.path.join("max_perf_pct"))
    }

    /// Set the maximum performance percent.
    pub fn set_max_perf_pct(&self, value: u8) -> io::Result<()> {
        write_file(self.path.join("max_perf_pct"), format!("{}", value))
    }

    /// If true, this signifies that turbo is disabled.
    pub fn no_turbo(&self) -> io::Result<bool> {
        let value: u8 = parse_file(self.path.join("no_turbo"))?;
        Ok(value > 0)
    }

    /// Set the no_turbo value; `true` will disable turbo.
    pub fn set_no_turbo(&self, value: bool) -> io::Result<()> {
        write_file(self.path.join("no_turbo"), if value { "1" } else { "0" })
    }

    pub fn values(&self) -> io::Result<PStateValues> {
        let values = PStateValues {
            min_perf_pct: self.min_perf_pct()?,
            max_perf_pct: self.max_perf_pct()?,
            no_turbo: self.no_turbo()?
        };

        Ok(values)
    }

    /// Set all values in the given config.
    pub fn set_values(&self, values: PStateValues) -> io::Result<()> {
        self.set_min_perf_pct(values.min_perf_pct)?;
        self.set_max_perf_pct(values.max_perf_pct)?;
        self.set_no_turbo(values.no_turbo)
    }
}

fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut data = String::new();

    {
        let mut file = File::open(path.as_ref())?;
        file.read_to_string(&mut data)?;
    }

    Ok(data)
}

fn write_file<P: AsRef<Path>, S: AsRef<[u8]>>(path: P, data: S) -> io::Result<()> {
    {
        let mut file = OpenOptions::new().write(true).open(path)?;
        file.write_all(data.as_ref())?
    }

    Ok(())
}

fn parse_file<F: FromStr, P: AsRef<Path>>(path: P) -> io::Result<F> where F::Err: Display {
    read_file(path)?.trim().parse().map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}", err)
        )
    })
}
