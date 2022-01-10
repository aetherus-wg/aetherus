//! File Extensions Module
//!
//! This module contains the loader implementations for the file extensions:
//! - JSON / JSON5 Files
//! - Wavefront / .obj Files
//! - NetCDF Files
//!
//! Please see the documentation in the appropriate module for specifics on each
//! format.

pub mod json;
pub mod netcdf;
pub mod wavefront;

pub use self::{json::*, netcdf::*, wavefront::*};
