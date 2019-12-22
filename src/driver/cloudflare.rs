pub use core::future::Future;
use futures::future::{self, BoxFuture, FutureExt};

use serde::{Deserialize, Serialize};
use serde_json::Result;

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

    fn run<'a, 'b, 'c>(
        &'a mut self,
        options: &SharedProgramOptions,
        recs: &'c Vec<Record>,
    ) -> BoxFuture<'b, DriverResult>
    where
        'a: 'b,
        'c: 'a,
    {
        if self.logger.is_none() {
            return future::ready(Ok(0)).boxed();
        }

        self.update(options.clone(), &recs).boxed()
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

#[derive(Serialize, Deserialize)]
struct CloudflareRecord {
    r#type: &'static str,
    name: String,
    content: String,
    // ttl:
    priority: i32,
    proxied: bool,
}

struct CloudflareRecordAction {
    record: CloudflareRecord,
    is_create: bool,
}

impl Cloudflare {
    async fn update<'a, 'b>(
        &'a mut self,
        _: SharedProgramOptions,
        recs: &'b Vec<Record>,
    ) -> DriverResult
    where
        'b: 'a,
    {
        let get_by_page_url = "https://api.cloudflare.com/client/v4/zones/{0}/dns_records?page=1&per_page=10&order=name&name={1}";
        /*
        {
            "result": [],
            "result_info": {
                "page": 1,
                "per_page": 50,
                "total_pages": 0,
                "count": 0,
                "total_count": 0
            },
            "success": true,
            "errors": [],
            "messages": []
        }
        */
        /*
        {
            "result": [
                {
                "id": "ac280f010d11ab1f5d6fe696d98ab4dd",
                "type": "A",
                "name": "vr-m.ouri.app",
                "content": "119.28.178.13",
                "proxiable": true,
                "proxied": false,
                "ttl": 1,
                "locked": false,
                "zone_id": "afb9539c0ceca4df01185f3f00351514",
                "zone_name": "ouri.app",
                "modified_on": "2019-11-02T02:48:59.550587Z",
                "created_on": "2019-11-02T02:48:59.550587Z",
                "meta": {
                    "auto_added": false,
                    "managed_by_apps": false,
                    "managed_by_argo_tunnel": false
                }
                },
                {
                "id": "5aa23d33fd0cf7016438604fd445b056",
                "type": "A",
                "name": "vr-m.ouri.app",
                "content": "119.28.56.48",
                "proxiable": true,
                "proxied": false,
                "ttl": 1,
                "locked": false,
                "zone_id": "afb9539c0ceca4df01185f3f00351514",
                "zone_name": "ouri.app",
                "modified_on": "2019-06-02T00:52:46.987069Z",
                "created_on": "2019-06-02T00:52:46.987069Z",
                "meta": {
                    "auto_added": false,
                    "managed_by_apps": false,
                    "managed_by_argo_tunnel": false
                }
                }
            ],
            "result_info": {
                "page": 1,
                "per_page": 50,
                "total_pages": 1,
                "count": 2,
                "total_count": 2
            },
            "success": true,
            "errors": [],
            "messages": []
            }
        */
        let create_record_url = "https://api.cloudflare.com/client/v4/zones/{0}/dns_records";
        let update_record_url = "https://api.cloudflare.com/client/v4/zones/{0}/dns_records/{1}";
        let delete_record_url = "https://api.cloudflare.com/client/v4/zones/{0}/dns_records/{1}";
        let token_header = ("Authorization", "Bearer {0}");
        let content_type = ("Content-Type", "application/json");

        let actions: Vec<CloudflareRecordAction> = recs
            .iter()
            .map(|ele| match ele {
                Record::A(r) => CloudflareRecordAction {
                    record: CloudflareRecord {
                        r#type: "A",
                        name: String::default(),
                        content: r.to_string(),
                        priority: 10,
                        proxied: false,
                    },
                    is_create: true,
                },
                Record::AAAA(r) => CloudflareRecordAction {
                    record: CloudflareRecord {
                        r#type: "AAAA",
                        name: String::default(),
                        content: r.to_string(),
                        priority: 10,
                        proxied: false,
                    },
                    is_create: true,
                },
                Record::CNAME(r) => CloudflareRecordAction {
                    record: CloudflareRecord {
                        r#type: "CNAME",
                        name: String::default(),
                        content: r.clone(),
                        priority: 10,
                        proxied: false,
                    },
                    is_create: true,
                },
                Record::MX(r) => CloudflareRecordAction {
                    record: CloudflareRecord {
                        r#type: "MX",
                        name: String::default(),
                        content: r.clone(),
                        priority: 10,
                        proxied: false,
                    },
                    is_create: true,
                },
                Record::TXT(r) => CloudflareRecordAction {
                    record: CloudflareRecord {
                        r#type: "TXT",
                        name: String::default(),
                        content: r.clone(),
                        priority: 10,
                        proxied: false,
                    },
                    is_create: true,
                },
            })
            .collect();

        if let Some(ref logger) = self.logger {
            for x in actions {
                let serialized = serde_json::to_string(&x.record).unwrap();
                info!(logger, "{}", serialized);
            }
        }
        Ok(0)
    }
}
