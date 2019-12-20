pub use core::future::Future;
use futures::future::{self, BoxFuture, FutureExt, TryFutureExt};
use futures::{Stream, TryFuture};

use std::net::IpAddr;
use std::str::FromStr;

use awc::{Client, ClientRequest, MessageBody};

extern crate clap;
use clap::{App, Arg, ArgMatches};

use super::super::option;
use super::{Detector, DetectorResult, Record};

pub struct GetIpByUrlDetector {
    url: String,
    ips: Vec<Record>,
}

impl GetIpByUrlDetector {
    pub fn default() -> Self {
        GetIpByUrlDetector {
            url: String::default(),
            ips: vec![],
        }
    }

    pub async fn pull_request_content<'a>(
        &'a mut self,
        request: option::HttpClient,
        logger: slog::Logger,
    ) -> Result<(String, slog::Logger), ()> {
        let response = request
            .send()
            .map_err(|e| {
                error!(logger, "Send HTTP request failed {}", e);
                debug!(logger, "{:?}", e);
                ()
            })
            .await?;

        let body = response
            .body()
            .map_err(|e| {
                error!(logger, "Fetch HTTP body failed {}", e);
                debug!(logger, "{:?}", e);
                ()
            })
            .await?;

        let ip_addr_str = String::from_utf8((&body).to_vec()).map_err(|e| {
            error!(logger, "Convert HTTP content to UTF-8 string failed {}", e);
            debug!(logger, "{:?}", e);
            ()
        })?;

        Ok((ip_addr_str, logger))
    }

    pub async fn make_request<'a>(
        &'a mut self,
        request: option::HttpClient,
        logger: slog::Logger,
    ) -> DetectorResult<'a> {
        let response = self.pull_request_content(request, logger).await;
        match response {
            Ok((ip_addr_str, logger)) => {
                if let Ok(addr) = IpAddr::from_str(&ip_addr_str) {
                    let final_addr = match addr {
                        IpAddr::V4(ipv4) => Record::A(ipv4),
                        IpAddr::V6(ipv6) => Record::AAAA(ipv6),
                    };
                    self.ips.push(final_addr);
                    Some(&self.ips)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

impl Detector for GetIpByUrlDetector {
    fn initialize<'a, 'b>(&mut self, app: App<'a, 'b>) -> App<'a, 'b> {
        app.arg(
            Arg::with_name("get-ip-by-url")
                .long("get-ip-by-url")
                .value_name("URL TO VISIT")
                .takes_value(true)
                .help("Get ip by visit specify url"),
        )
    }

    fn parse_options(&mut self, matches: &ArgMatches, options: &mut option::ProgramOptions) {
        self.url = option::unwraper_string_or(&matches, "get-ip-by-url", String::default());
    }

    fn run<'a>(
        &'a mut self,
        options: &mut option::ProgramOptions,
    ) -> BoxFuture<DetectorResult<'a>> {
        if self.url.is_empty() {
            future::ready(None).boxed()
        } else {
            let logger = option::create_logger(&options, "GetIpByUrlDetector");
            let client_request = option::create_http_client(&self.url, &options);
            self.make_request(client_request, logger).boxed()
        }
    }
}
