// When the `system-alloc` feature is used, use the System Allocator
#[cfg(feature = "system-alloc")]
use std::alloc::System;
#[cfg(feature = "system-alloc")]
#[global_allocator]
static GLOBAL: System = System;

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
        Box::<detector::SetIpDetector>::default(),
        Box::<detector::GetIpByUrlDetector>::default(),
    ]
}

fn register_drivers() -> Vec<Box<dyn driver::Driver>> {
    vec![
        Box::<driver::Cloudflare>::default(),
        Box::<driver::Dnspod>::default(),
    ]
}

async fn real_main() -> i32 {
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
            records.extend(res.iter().cloned());
        }
    }

    records.dedup();
    let mut exit_code: i32 = 0;
    for ref mut driver in &mut drivers {
        if (driver.run(&options, &records).await).is_err() {
            exit_code = 1;
        }
    }

    exit_code
}

fn main() {
    let exit_code = async_std::task::block_on(async { real_main().await });

    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}
