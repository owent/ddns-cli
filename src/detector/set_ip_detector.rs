pub use core::future::Future;
use futures::future::{self, BoxFuture, FutureExt};

use std::net::IpAddr;
use std::str::FromStr;

extern crate clap;
use clap::{App, Arg, ArgMatches};

use super::{Detector, DetectorResult, Record};

type SharedProgramOptions = super::SharedProgramOptions;

pub struct SetIpDetector {
    ips: Vec<Record>,
}

impl Default for SetIpDetector {
    fn default() -> Self {
        SetIpDetector { ips: vec![] }
    }
}

impl Detector for SetIpDetector {
    fn initialize<'a, 'b>(&mut self, app: App<'a, 'b>) -> App<'a, 'b> {
        app.arg(
            Arg::with_name("ip")
                .long("ip")
                .value_name("IP ADDRESS")
                .takes_value(true)
                .help("Set ip address by command line options"),
        )
    }

    fn parse_options(&mut self, matches: &ArgMatches, _: &mut SharedProgramOptions) {
        if let Some(x) = matches.values_of("ip") {
            for val in x {
                if let Ok(addr) = IpAddr::from_str(&val) {
                    let final_addr = match addr {
                        IpAddr::V4(ipv4) => Record::A(ipv4),
                        IpAddr::V6(ipv6) => Record::AAAA(ipv6),
                    };
                    self.ips.push(final_addr);
                }
            }
        }
    }

    fn run<'a, 'b>(&'a mut self, _: &mut SharedProgramOptions) -> BoxFuture<'b, DetectorResult<'a>>
    where
        'a: 'b,
    {
        if self.ips.is_empty() {
            future::ready(Err(())).boxed()
        } else {
            future::ready(Ok(&self.ips)).boxed()
        }
    }
}
