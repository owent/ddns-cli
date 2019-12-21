pub use core::future::Future;
use futures::future::{self, BoxFuture, FutureExt};

extern crate clap;
use clap::{App, Arg, ArgMatches};

use super::super::option;
use super::{Driver, DriverResult, Record};

type SharedProgramOptions = super::SharedProgramOptions;

pub struct Cloudflare {
    zone_id: String,
    token: String,
    domains: Vec<String>,
    logger: Option<slog::Logger>,
}

impl Driver for Cloudflare {
    fn initialize<'a, 'b>(&mut self, app: App<'a, 'b>) -> App<'a, 'b> {
        app.arg(
            Arg::with_name("cf-zone-id")
                .long("cf-zone-id")
                .value_name("ZONE_ID")
                .takes_value(true)
                .help("Set zone id of cloudflare API, you can get it from your domain zone"),
        ).arg(
            Arg::with_name("cf-token")
                .long("cf-token")
                .value_name("TOKEN")
                .takes_value(true)
                .help("Set token of cloudflare API, you can get it from https://dash.cloudflare.com/profile/api-tokens"),
        ).arg(
            Arg::with_name("cf-domain")
                .long("cf-domain")
                .value_name("DOMAIN")
                .takes_value(true)
                .help("Add domain to update using cloudflare API."),
        )
    }

    fn parse_options(&mut self, matches: &ArgMatches, options: &mut SharedProgramOptions) {
        self.zone_id = option::unwraper_option_or(&matches, "cf-zone-id", String::default());
        self.token = option::unwraper_option_or(&matches, "cf-token", String::default());
        if let Some(x) = matches.values_of("cf-domain") {
            self.domains.extend(x.map(|s| String::from(s)));
        }
        if !self.zone_id.is_empty() && !self.token.is_empty() && !self.domains.is_empty() {
            self.logger = Some(options.create_logger("Cloudflare"));
        }
    }

    fn run<'a, 'b>(
        &mut self,
        _: &SharedProgramOptions,
        recs: &Vec<Record>,
    ) -> BoxFuture<'b, DriverResult>
    where
        'a: 'b,
    {
        if let Some(ref x) = self.logger {
            info!(x, "{:?}", recs);
            info!(x, "{:?}", self.domains);
        }
        future::ready(Ok(0)).boxed()
    }
}

impl Default for Cloudflare {
    fn default() -> Self {
        Cloudflare {
            zone_id: String::default(),
            token: String::default(),
            domains: vec![],
            logger: None,
        }
    }
}
