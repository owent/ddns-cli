pub use core::future::Future;
use futures::future::{self, BoxFuture, FutureExt};

use std::net::IpAddr;
use std::str::FromStr;

extern crate clap;
use clap::{App, Arg, ArgMatches};

use super::super::option::ProgramOptions;
use super::{Detector, DetectorResult, Record};

pub struct SetIpDetector {
    ips: Vec<Record>,
}

impl SetIpDetector {
    pub fn default() -> Self {
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

    fn parse_options(&mut self, matches: &ArgMatches, options: &mut ProgramOptions) {
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

    fn run<'a>(&'a mut self, _: &mut ProgramOptions) -> BoxFuture<DetectorResult<'a>> {
        if self.ips.is_empty() {
            future::ready(None).boxed()
        } else {
            future::ready(Some(&self.ips)).boxed()
        }
    }
}
