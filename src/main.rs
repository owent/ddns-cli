// When the `system-alloc` feature is used, use the System Allocator
#[cfg(feature = "system-alloc")]
use std::alloc::System;
#[cfg(feature = "system-alloc")]
#[global_allocator]
static GLOBAL: System = System;

use std::process;

extern crate bytes;
extern crate hex;
extern crate regex;
extern crate time;

extern crate futures;
extern crate futures_core;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate slog;

extern crate async_std;
extern crate http;
extern crate tokio;
// extern crate hyper;
// extern crate hyper_tls;
extern crate reqwest;

// use tokio::prelude::*;

mod detector;
mod driver;
mod option;

fn register_detectors() -> Vec<Box<dyn detector::Detector>> {
    vec![
        Box::new(detector::SetIpDetector::default()),
        Box::new(detector::GetIpByUrlDetector::default()),
    ]
}

fn register_drivers() -> Vec<Box<dyn driver::Driver>> {
    vec![
        Box::new(driver::Cloudflare::default()),
        Box::new(driver::Dnspod::default()),
    ]
}

#[async_std::main]
async fn main() {
    let mut detectors = register_detectors();
    let mut drivers = register_drivers();
    let mut app = option::app();

    // register for command options
    for ref mut detector in &mut detectors {
        app = detector.as_mut().initialize(app);
    }

    for ref mut driver in &mut drivers {
        app = driver.initialize(app);
    }

    let (matches, mut options) = option::parse_options(app);

    // parse command options
    for ref mut detector in &mut detectors {
        detector.as_mut().parse_options(&matches, &mut options);
    }

    for ref mut driver in &mut drivers {
        driver.parse_options(&matches, &mut options);
    }

    // System::new(crate_name!()).block_on(async move {
    let mut records: Vec<detector::Record> = vec![];
    for ref mut detector in &mut detectors {
        if let Ok(res) = detector.as_mut().run(&mut options).await {
            records.extend(res.iter().map(|x| x.clone()));
        }
    }

    records.dedup();
    let mut exit_code: i32 = 0;
    for ref mut driver in &mut drivers {
        if let Err(_) = driver.run(&options, &records).await {
            exit_code = 1;
        }
    }

    if exit_code > 0 {
        process::exit(exit_code);
    }
}
