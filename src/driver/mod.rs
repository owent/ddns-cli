use futures::future::BoxFuture;

extern crate clap;
use clap::{App, ArgMatches};

use super::detector;

pub type Record = detector::Record;
pub type DriverResult = Result<i32, ()>;

mod cloudflare;
mod dnspod;

pub type Cloudflare = cloudflare::Cloudflare;
pub type Dnspod = dnspod::Dnspod;
pub type SharedProgramOptions = super::option::SharedProgramOptions;
pub type HttpMethod = super::option::HttpMethod;

pub trait Driver {
    fn initialize<'a, 'b>(&mut self, app: App<'a, 'b>) -> App<'a, 'b>;
    fn parse_options(&mut self, matches: &ArgMatches, options: &mut SharedProgramOptions);

    fn run<'a, 'b, 'c>(
        &'a mut self,
        options: &SharedProgramOptions,
        recs: &'c Vec<Record>,
    ) -> BoxFuture<'b, DriverResult>
    where
        'a: 'b,
        'c: 'a;
}
