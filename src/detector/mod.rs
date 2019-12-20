use std::net::{Ipv4Addr, Ipv6Addr};

use futures::future::BoxFuture;

extern crate clap;
use clap::{App, ArgMatches};

use super::option::ProgramOptions;

mod get_ip_by_url_detector;
mod set_ip_detector;

pub type SetIpDetector = set_ip_detector::SetIpDetector;
pub type GetIpByUrlDetector = get_ip_by_url_detector::GetIpByUrlDetector;

#[derive(Debug, Clone, PartialEq)]
pub enum Record {
    A(Ipv4Addr),
    AAAA(Ipv6Addr),
    CNAME(String),
    MX(String),
    Txt(String),
}

pub type DetectorResult<'a> = Option<&'a Vec<Record>>;

pub trait Detector {
    fn initialize<'a, 'b>(&mut self, app: App<'a, 'b>) -> App<'a, 'b>;
    fn parse_options(&mut self, matches: &ArgMatches, options: &mut ProgramOptions);

    fn run<'a>(&'a mut self, options: &mut ProgramOptions) -> BoxFuture<DetectorResult<'a>>;
}
