use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};

use futures::future::BoxFuture;

extern crate clap;
use crate::clap::{ArgMatches, Command};

mod get_ip_by_url_detector;
mod set_ip_detector;

pub type SetIpDetector = set_ip_detector::SetIpDetector;
pub type GetIpByUrlDetector = get_ip_by_url_detector::GetIpByUrlDetector;
pub type SharedProgramOptions = super::option::SharedProgramOptions;
pub type HttpMethod = super::option::HttpMethod;

#[derive(Debug, Clone, PartialEq)]
pub enum Record {
    A(Ipv4Addr),
    Aaaa(Ipv6Addr),
    #[allow(dead_code)]
    Cname(String),
    #[allow(dead_code)]
    Mx(String),
    #[allow(dead_code)]
    Txt(String),
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Record::A(ref v) => f.write_fmt(format_args!("A: {}", v)),
            Record::Aaaa(ref v) => f.write_fmt(format_args!("AAAA: {}", v)),
            Record::Cname(ref v) => f.write_fmt(format_args!("CNAME: {}", v)),
            Record::Mx(ref v) => f.write_fmt(format_args!("MX: {}", v)),
            Record::Txt(ref v) => f.write_fmt(format_args!("TXT: {}", v)),
        }
    }
}

pub type DetectorResult<'a> = Result<&'a Vec<Record>, ()>;

pub trait Detector {
    fn initialize(&mut self, app: Command) -> Command;
    fn parse_options(&mut self, matches: &ArgMatches, options: &mut SharedProgramOptions);

    fn run<'a, 'b>(
        &'a mut self,
        options: &mut SharedProgramOptions,
    ) -> BoxFuture<'b, DetectorResult<'a>>
    where
        'a: 'b;
}
