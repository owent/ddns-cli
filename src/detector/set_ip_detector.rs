pub use core::future::Future;
use futures::future::{self, BoxFuture, FutureExt};

use std::net::IpAddr;

extern crate clap;
use clap::{Arg, ArgAction, ArgMatches, Command};

use super::super::option;
use super::{Detector, DetectorResult, Record};

type SharedProgramOptions = super::SharedProgramOptions;

pub struct SetIpDetector {
    ips: Vec<Record>,
    ignore_link_local: bool,
    ignore_shared: bool,
    ignore_loopback: bool,
    ignore_private: bool,
    ignore_multicast: bool,
}

impl Default for SetIpDetector {
    fn default() -> Self {
        SetIpDetector {
            ips: vec![],
            ignore_link_local: false,
            ignore_shared: false,
            ignore_loopback: false,
            ignore_private: false,
            ignore_multicast: false,
        }
    }
}

impl Detector for SetIpDetector {
    fn initialize(&mut self, app: Command) -> Command {
        app.arg(
            Arg::new("ip")
                .long("ip")
                .value_name("IP ADDRESS")
                .num_args(1..)
                .action(ArgAction::Append)
                .help("Set ip address by command line options"),
        )
        .arg(
            Arg::new("ip-no-link-local")
                .long("ip-no-link-local")
                .action(ArgAction::SetTrue)
                .help("Ignore link local address"),
        )
        .arg(
            Arg::new("ip-no-shared")
                .long("ip-no-shared")
                .action(ArgAction::SetTrue)
                .help("Ignore shared address(100.64.0.0/10)"),
        )
        .arg(
            Arg::new("ip-no-loopback")
                .long("ip-no-loopback")
                .action(ArgAction::SetTrue)
                .help("Ignore loopback address"),
        )
        .arg(
            Arg::new("ip-no-private")
                .long("ip-no-private")
                .action(ArgAction::SetTrue)
                .help("Ignore private address"),
        )
        .arg(
            Arg::new("ip-no-multicast")
                .long("ip-no-multicast")
                .action(ArgAction::SetTrue)
                .help("Ignore multicast address"),
        )
    }

    fn parse_options(&mut self, matches: &ArgMatches, options: &mut SharedProgramOptions) {
        self.ignore_link_local = option::unwraper_flag(&matches, "ip-no-link-local");
        self.ignore_shared = option::unwraper_flag(&matches, "ip-no-shared");
        self.ignore_loopback = option::unwraper_flag(&matches, "ip-no-loopback");
        self.ignore_private = option::unwraper_flag(&matches, "ip-no-private");
        self.ignore_multicast = option::unwraper_flag(&matches, "ip-no-multicast");

        {
            let logger = options.create_logger("SetIpDetector");
            for addr in option::unwraper_multiple_values(&matches, "ip", &logger, "ip address") {
                let final_addr = match addr {
                    IpAddr::V4(ipv4) => {
                        let res;
                        if self.ignore_link_local && ipv4.is_link_local() {
                            res = None
                        } else if self.ignore_shared
                            && ipv4.octets()[0] == 100
                            && (ipv4.octets()[1] & 0b1100_0000 == 0b0100_0000)
                        {
                            res = None
                        } else if self.ignore_loopback && ipv4.is_loopback() {
                            res = None
                        } else if self.ignore_private && ipv4.is_private() {
                            res = None
                        } else if self.ignore_multicast && ipv4.is_multicast() {
                            res = None
                        } else {
                            res = Some(Record::A(ipv4))
                        }
                        res
                    }
                    IpAddr::V6(ipv6) => {
                        let res;
                        if self.ignore_link_local && (ipv6.segments()[0] & 0xffc0) == 0xfe80 {
                            res = None
                        } else if self.ignore_loopback && ipv6.is_loopback() {
                            res = None
                        } else if self.ignore_private && (ipv6.segments()[0] & 0xfe00) == 0xfc00 {
                            res = None
                        } else if self.ignore_multicast && ipv6.is_multicast() {
                            res = None
                        } else {
                            res = Some(Record::AAAA(ipv6))
                        }
                        res
                    }
                };

                if let Some(ipaddr) = final_addr {
                    self.ips.push(ipaddr);
                    debug!(logger, "Add ip address {}", addr.to_string());
                } else {
                    debug!(logger, "Ignore ip address {}", addr.to_string());
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
