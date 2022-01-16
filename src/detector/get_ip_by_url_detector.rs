pub use core::future::Future;
use futures::future::{self, BoxFuture, FutureExt, TryFutureExt};

use std::net::IpAddr;
use std::str::FromStr;

extern crate clap;
use clap::{App, Arg, ArgMatches};

use super::super::option;
use super::{Detector, DetectorResult, Record};

type SharedProgramOptions = super::SharedProgramOptions;
type HttpMethod = super::HttpMethod;

pub struct GetIpByUrlDetector {
    url: String,
    ips: Vec<Record>,
}

impl GetIpByUrlDetector {
    // #[actix_rt::main]
    pub async fn pull_request_content<'a>(
        &'a mut self,
        options: SharedProgramOptions,
    ) -> DetectorResult<'a> {
        let logger = options.create_logger("GetIpByUrlDetector");

        let cli = options.http(HttpMethod::GET, &self.url);

        debug!(logger, "Sending request to {} to get my address", self.url);

        let response = cli
            .send()
            .map_err(|e| {
                error!(logger, "Send HTTP request failed {}", e);
                debug!(logger, "{:?}", e);
                ()
            })
            .await?;

        let body_text = response
            .text()
            .map_err(|e| {
                error!(logger, "Get HTTP response failed {}", e);
                debug!(logger, "{:?}", e);
                ()
            })
            .await?;

        debug!(logger, "Detect my address: {}", body_text);

        match IpAddr::from_str(&body_text) {
            Ok(addr) => {
                let final_addr = match addr {
                    IpAddr::V4(ipv4) => Record::A(ipv4),
                    IpAddr::V6(ipv6) => Record::AAAA(ipv6),
                };
                self.ips.push(final_addr);
                Ok(&self.ips)
            }
            Err(e) => {
                error!(logger, "Parse ip address from HTTP body failed {}", e);
                debug!(logger, "{:?}", e);
                Err(())
            }
        }
    }
}

impl Detector for GetIpByUrlDetector {
    fn initialize<'a>(&mut self, app: App<'a>) -> App<'a> {
        app.arg(
            Arg::new("get-ip-by-url")
                .long("get-ip-by-url")
                .value_name("URL TO VISIT")
                .takes_value(true)
                .help("Get ip by visit specify url(https://myip.biturl.top/ for example)"),
        )
    }

    fn parse_options(&mut self, matches: &ArgMatches, _: &mut SharedProgramOptions) {
        self.url = option::unwraper_option_or(&matches, "get-ip-by-url", String::default());
    }

    fn run<'a, 'b>(
        &'a mut self,
        options: &mut SharedProgramOptions,
    ) -> BoxFuture<'b, DetectorResult<'a>>
    where
        'a: 'b,
    {
        if self.url.is_empty() {
            future::ready(Err(())).boxed()
        } else {
            self.pull_request_content(options.clone()).boxed()
        }
    }
}

impl Default for GetIpByUrlDetector {
    fn default() -> Self {
        GetIpByUrlDetector {
            url: String::default(),
            ips: vec![],
        }
    }
}
