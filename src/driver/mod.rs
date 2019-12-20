use futures::future::BoxFuture;

extern crate clap;
use clap::{App, ArgMatches};

use super::option::ProgramOptions;

use super::detector;

pub type Record = detector::Record;
pub type DriverResult = Option<i32>;

mod cloudflare;
pub type Cloudflare = cloudflare::Cloudflare;

pub trait Driver {
    fn initialize<'a, 'b>(&mut self, app: App<'a, 'b>) -> App<'a, 'b>;
    fn parse_options(&mut self, matches: &ArgMatches, options: &mut ProgramOptions);

    fn run(&mut self, options: &ProgramOptions, recs: &Vec<Record>) -> BoxFuture<DriverResult>;
}
