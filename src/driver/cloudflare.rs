pub use core::future::Future;
use futures::future::{self, BoxFuture, FutureExt};

use serde::{Deserialize, Serialize};

extern crate clap;
use clap::{App, Arg, ArgMatches};

use super::super::option;
use super::{Driver, DriverResult, Record};
use reqwest::header::CONTENT_TYPE;

type SharedProgramOptions = super::SharedProgramOptions;
type HttpMethod = super::HttpMethod;

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
                .help("Add domain to update using cloudflare API"),
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
    pub r#type: &'static str,
    pub name: String,
    pub content: String,
    pub ttl: i32,
    // pub priority: i32,
    pub proxied: bool,
}

struct CloudflareRecordAction {
    pub record: CloudflareRecord,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareGetResponseRecord {
    pub id: String,
    pub r#type: String,
    pub name: String,
    pub content: String,
    pub zone_id: String,
    pub zone_name: String,
    pub modified_on: String,
    pub created_on: String,
    pub proxiable: bool,
    pub proxied: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareResponsePage {
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
    pub count: i32,
    pub total_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareGetResponseResult {
    pub result: Vec<CloudflareGetResponseRecord>,
    pub result_info: CloudflareResponsePage,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareResponseError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareResponseResult {
    pub success: bool,
    pub errors: Vec<CloudflareResponseError>,
}

static CFHEAD_CONTENT_TYPE: &str = "application/json";
static CFRSP_EMPTY_STRING: &str = "";

impl CloudflareResponseResult {
    pub fn get_error_message<'a, 'b>(&'a self) -> &'b str
    where
        'a: 'b,
    {
        if self.errors.is_empty() {
            CFRSP_EMPTY_STRING
        } else {
            &self.errors[0].message
        }
    }
}

impl Cloudflare {
    async fn update<'a, 'b>(
        &'a mut self,
        options: SharedProgramOptions,
        recs: &'b Vec<Record>,
    ) -> DriverResult
    where
        'b: 'a,
    {
        let mut actions: Vec<CloudflareRecordAction> = recs
            .iter()
            .map(|ele| match ele {
                Record::A(r) => CloudflareRecordAction {
                    record: CloudflareRecord {
                        r#type: "A",
                        name: String::default(),
                        content: r.to_string(),
                        ttl: 1,
                        proxied: false,
                    },
                },
                Record::AAAA(r) => CloudflareRecordAction {
                    record: CloudflareRecord {
                        r#type: "AAAA",
                        name: String::default(),
                        content: r.to_string(),
                        ttl: 1,
                        proxied: false,
                    },
                },
                Record::CNAME(r) => CloudflareRecordAction {
                    record: CloudflareRecord {
                        r#type: "CNAME",
                        name: String::default(),
                        content: r.clone(),
                        ttl: 1,
                        proxied: false,
                    },
                },
                Record::MX(r) => CloudflareRecordAction {
                    record: CloudflareRecord {
                        r#type: "MX",
                        name: String::default(),
                        content: r.clone(),
                        ttl: 1,
                        proxied: false,
                    },
                },
                Record::TXT(r) => CloudflareRecordAction {
                    record: CloudflareRecord {
                        r#type: "TXT",
                        name: String::default(),
                        content: r.clone(),
                        ttl: 1,
                        proxied: false,
                    },
                },
            })
            .collect();

        for ref domain in &self.domains {
            let url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
                self.zone_id
            );
            // page=1&per_page=50&order=name&name={}
            let cli = options
                .http(HttpMethod::GET, &url)
                .bearer_auth(self.token.clone())
                .query(&[
                    ("page", "1"),
                    ("per_page", "100"),
                    ("order", "name"),
                    ("name", domain),
                ])
                .header(CONTENT_TYPE, CFHEAD_CONTENT_TYPE);

            let rsp = match cli.send().await {
                Ok(v) => v,
                Err(e) => {
                    if let Some(ref logger) = self.logger {
                        error!(logger, "Send HTTP request failed, error: {}", e);
                    }
                    continue;
                }
            };

            let rsp_text = match rsp.text().await {
                Ok(v) => v,
                Err(e) => {
                    if let Some(ref logger) = self.logger {
                        error!(logger, "Fetch response body failed, error: {}", e);
                    }
                    continue;
                }
            };

            let result = match serde_json::from_str::<CloudflareGetResponseResult>(&rsp_text) {
                Ok(v) => v,
                Err(e) => {
                    if let Some(ref logger) = self.logger {
                        error!(
                            logger,
                            "Parse response body failed, error: {}.\nbody: {}", e, rsp_text
                        );
                    }
                    continue;
                }
            };

            let mut record_no = 0;
            loop {
                record_no += 1;
                let index = record_no - 1;
                if index >= actions.len() && index >= result.result.len() {
                    break;
                }

                // Delete remain records
                if index >= actions.len() {
                    let delete_url = format!(
                        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                        self.zone_id, &result.result[index].id
                    );
                    match options
                        .http(HttpMethod::DELETE, &delete_url)
                        .bearer_auth(self.token.clone())
                        .header(CONTENT_TYPE, CFHEAD_CONTENT_TYPE)
                        .send()
                        .await
                    {
                        Ok(rsp) => match rsp.json::<CloudflareResponseResult>().await {
                            Ok(res) => {
                                if let Some(ref logger) = self.logger {
                                    debug!(
                                        logger,
                                        "Delete {} for {} {}.{}",
                                        &result.result[index].content,
                                        &result.result[index].name,
                                        if res.success { "success" } else { "failed" },
                                        res.get_error_message()
                                    );
                                }
                            }
                            Err(e) => {
                                if let Some(ref logger) = self.logger {
                                    error!(
                                        logger,
                                        "Delete {} for {} failed, error: {}",
                                        &result.result[index].content,
                                        &result.result[index].name,
                                        e
                                    );
                                }
                            }
                        },
                        Err(e) => {
                            if let Some(ref logger) = self.logger {
                                error!(
                                    logger,
                                    "Delete {} for {} failed, error: {}",
                                    &result.result[index].content,
                                    &result.result[index].name,
                                    e
                                );
                            }
                        }
                    }
                    continue;
                }

                // Create new records
                if index >= result.result.len() {
                    let create_url = format!(
                        "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
                        self.zone_id
                    );
                    actions[index].record.name = domain.to_string();
                    match options
                        .http(HttpMethod::POST, &create_url)
                        .bearer_auth(self.token.clone())
                        .json(&actions[index].record)
                        .send()
                        .await
                    {
                        Ok(rsp) => match rsp.json::<CloudflareResponseResult>().await {
                            Ok(res) => {
                                if let Some(ref logger) = self.logger {
                                    debug!(
                                        logger,
                                        "Create {} for {} {}.{}",
                                        &actions[index].record.content,
                                        &actions[index].record.name,
                                        if res.success { "success" } else { "failed" },
                                        res.get_error_message()
                                    );
                                }
                            }
                            Err(e) => {
                                if let Some(ref logger) = self.logger {
                                    error!(
                                        logger,
                                        "Create {} for {} failed, error: {}",
                                        &actions[index].record.content,
                                        &actions[index].record.name,
                                        e
                                    );
                                }
                            }
                        },
                        Err(e) => {
                            if let Some(ref logger) = self.logger {
                                error!(
                                    logger,
                                    "Create {} for {} failed, error: {}",
                                    &actions[index].record.content,
                                    &actions[index].record.name,
                                    e
                                );
                            }
                        }
                    }
                    continue;
                }

                // Update record
                if actions[index].record.content != result.result[index].content {
                    let create_url = format!(
                        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                        self.zone_id, &result.result[index].id
                    );
                    actions[index].record.name = domain.to_string();
                    match options
                        .http(HttpMethod::PUT, &create_url)
                        .bearer_auth(self.token.clone())
                        .json(&actions[index].record)
                        .send()
                        .await
                    {
                        Ok(rsp) => match rsp.json::<CloudflareResponseResult>().await {
                            Ok(res) => {
                                if let Some(ref logger) = self.logger {
                                    debug!(
                                        logger,
                                        "Update {} to {} for {} {}.{}",
                                        &result.result[index].content,
                                        &actions[index].record.content,
                                        &actions[index].record.name,
                                        if res.success { "success" } else { "failed" },
                                        res.get_error_message()
                                    );
                                }
                            }
                            Err(e) => {
                                if let Some(ref logger) = self.logger {
                                    error!(
                                        logger,
                                        "Update {} to {} for {} failed, error: {}",
                                        &result.result[index].content,
                                        &actions[index].record.content,
                                        &actions[index].record.name,
                                        e
                                    );
                                }
                            }
                        },
                        Err(e) => {
                            if let Some(ref logger) = self.logger {
                                error!(
                                    logger,
                                    "Update {} to {} for {} failed, error: {}",
                                    &result.result[index].content,
                                    &actions[index].record.content,
                                    &actions[index].record.name,
                                    e
                                );
                            }
                        }
                    }
                } else {
                    if let Some(ref logger) = self.logger {
                        debug!(
                            logger,
                            "Record {} for {} not changed, do nothing",
                            &actions[index].record.content,
                            &result.result[index].name
                        );
                    }
                }
            }

            if let Some(ref logger) = self.logger {
                let action_description: Vec<String> = recs.iter().map(|r| r.to_string()).collect();
                info!(
                    logger,
                    "Update domain name {} with {} finished",
                    domain,
                    action_description.join(",")
                );
            }
        }

        Ok(0)
    }
}
