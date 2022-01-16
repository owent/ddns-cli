use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};

use futures::future::BoxFuture;

extern crate clap;
use clap::{App, ArgMatches};

mod get_ip_by_url_detector;
mod set_ip_detector;

pub type SetIpDetector = set_ip_detector::SetIpDetector;
pub type GetIpByUrlDetector = get_ip_by_url_detector::GetIpByUrlDetector;
pub type SharedProgramOptions = super::option::SharedProgramOptions;
pub type HttpMethod = super::option::HttpMethod;

#[derive(Debug, Clone, PartialEq)]
pub enum Record {
    A(Ipv4Addr),
    AAAA(Ipv6Addr),
    #[allow(dead_code)]
    CNAME(String),
    #[allow(dead_code)]
    MX(String),
    #[allow(dead_code)]
    TXT(String),
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Record::A(ref v) => f.write_fmt(format_args!("A: {}", v)),
            Record::AAAA(ref v) => f.write_fmt(format_args!("AAAA: {}", v)),
            Record::CNAME(ref v) => f.write_fmt(format_args!("CNAME: {}", v)),
            Record::MX(ref v) => f.write_fmt(format_args!("MX: {}", v)),
            Record::TXT(ref v) => f.write_fmt(format_args!("TXT: {}", v)),
        }
    }
}

pub type DetectorResult<'a> = Result<&'a Vec<Record>, ()>;

pub trait Detector {
    fn initialize<'a>(&mut self, app: App<'a>) -> App<'a>;
    fn parse_options(&mut self, matches: &ArgMatches, options: &mut SharedProgramOptions);

    fn run<'a, 'b>(
        &'a mut self,
        options: &mut SharedProgramOptions,
    ) -> BoxFuture<'b, DetectorResult<'a>>
    where
        'a: 'b;
}
