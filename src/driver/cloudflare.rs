pub use core::future::Future;
use futures::future::{self, BoxFuture, FutureExt};

use serde::{Deserialize, Serialize};

extern crate clap;
use clap::{Arg, ArgAction, ArgMatches, Command};

use super::super::option;
use super::{Driver, DriverResult, Record};
use reqwest::header::CONTENT_TYPE;

type SharedProgramOptions = super::SharedProgramOptions;
type HttpMethod = super::HttpMethod;

#[derive(Default)]
pub struct Cloudflare {
    zone_id: String,
    token: String,
    domains: Vec<String>,
    logger: Option<slog::Logger>,
}

impl Driver for Cloudflare {
    fn initialize(&mut self, app: Command) -> Command {
        app.arg(
            Arg::new("cf-zone-id")
                .long("cf-zone-id")
                .value_name("ZONE_ID")
                .help("Set zone id of cloudflare API, you can get it from your domain zone"),
        ).arg(
            Arg::new("cf-token")
                .long("cf-token")
                .value_name("TOKEN")
                .help("Set token of cloudflare API, you can get it from https://dash.cloudflare.com/profile/api-tokens"),
        ).arg(
            Arg::new("cf-domain")
                .long("cf-domain")
                .value_name("DOMAIN")
                .num_args(1..)
                .action(ArgAction::Append)
                .help("Add domain to update using cloudflare API"),
        )
    }

    fn parse_options(&mut self, matches: &ArgMatches, options: &mut SharedProgramOptions) {
        self.zone_id = option::unwraper_option_or(matches, "cf-zone-id", String::default());
        self.token = option::unwraper_option_or(matches, "cf-token", String::default());
        if !self.zone_id.is_empty() && !self.token.is_empty() {
            self.logger = Some(options.create_logger("Cloudflare"));

            self.domains.extend(option::unwraper_multiple_values(
                matches,
                "cf-domain",
                self.logger.as_ref().unwrap(),
                "domain",
            ));
        }
    }

    fn run<'a, 'b, 'c>(
        &'a mut self,
        options: &SharedProgramOptions,
        recs: &'c [Record],
    ) -> BoxFuture<'b, DriverResult>
    where
        'a: 'b,
        'c: 'a,
    {
        if self.logger.is_none() {
            return future::ready(Ok(0)).boxed();
        }

        self.update(options.clone(), recs).boxed()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct CloudflareRecord {
    pub r#type: &'static str,
    pub name: String,
    pub content: String,
    pub ttl: i32,
    // pub priority: i32,
    pub proxied: bool,
}

#[derive(Debug)]
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

impl PartialEq for CloudflareRecord {
    fn eq(&self, other: &Self) -> bool {
        self.r#type == other.r#type && self.name == other.name
    }
}

impl PartialEq for CloudflareRecordAction {
    fn eq(&self, other: &Self) -> bool {
        self.record == other.record
    }
}

impl PartialEq for CloudflareGetResponseRecord {
    fn eq(&self, other: &Self) -> bool {
        self.r#type == other.r#type && self.name == other.name
    }
}

impl PartialEq<CloudflareRecord> for CloudflareGetResponseRecord {
    fn eq(&self, other: &CloudflareRecord) -> bool {
        self.r#type == other.r#type && self.name == other.name
    }
}

impl PartialEq<CloudflareGetResponseRecord> for CloudflareRecord {
    fn eq(&self, other: &CloudflareGetResponseRecord) -> bool {
        self.r#type == other.r#type && self.name == other.name
    }
}

impl PartialEq<CloudflareRecordAction> for CloudflareGetResponseRecord {
    fn eq(&self, other: &CloudflareRecordAction) -> bool {
        self.r#type == other.record.r#type && self.name == other.record.name
    }
}

impl PartialEq<CloudflareGetResponseRecord> for CloudflareRecordAction {
    fn eq(&self, other: &CloudflareGetResponseRecord) -> bool {
        self.record.r#type == other.r#type && self.record.name == other.name
    }
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
        recs: &'b [Record],
    ) -> DriverResult
    where
        'b: 'a,
    {
        let mut ret: i32 = 0;
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
                Record::Aaaa(r) => CloudflareRecordAction {
                    record: CloudflareRecord {
                        r#type: "AAAA",
                        name: String::default(),
                        content: r.to_string(),
                        ttl: 1,
                        proxied: false,
                    },
                },
                Record::Cname(r) => CloudflareRecordAction {
                    record: CloudflareRecord {
                        r#type: "CNAME",
                        name: String::default(),
                        content: r.clone(),
                        ttl: 1,
                        proxied: false,
                    },
                },
                Record::Mx(r) => CloudflareRecordAction {
                    record: CloudflareRecord {
                        r#type: "MX",
                        name: String::default(),
                        content: r.clone(),
                        ttl: 1,
                        proxied: false,
                    },
                },
                Record::Txt(r) => CloudflareRecordAction {
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
                .http(HttpMethod::Get, &url)
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

            let mut pending_to_delete: Vec<&CloudflareGetResponseRecord> = vec![];
            let mut pending_to_create: Vec<&mut CloudflareRecordAction> = vec![];

            for old_record in &result.result {
                let keep = actions.iter().any(|act| {
                    act.record.r#type == old_record.r#type
                        && act.record.content == old_record.content
                });
                if !keep {
                    pending_to_delete.push(old_record);
                }
            }

            for mut new_record in &mut actions {
                let already_exists = result.result.iter().any(|res| {
                    res.r#type == new_record.record.r#type
                        && res.content == new_record.record.content
                });
                if !already_exists {
                    new_record.record.name = domain.to_string();
                    pending_to_create.push(new_record);
                }
            }

            if let Some(ref logger) = self.logger {
                if !result.result.is_empty() {
                    debug!(logger, "Old records:");
                    for ref log_item in &result.result {
                        debug!(logger, "     -- {:?}", log_item);
                    }
                }

                if !pending_to_delete.is_empty() {
                    debug!(logger, "Pending to delete:");
                    for ref log_item in &pending_to_delete {
                        debug!(logger, "     -- {:?}", log_item);
                    }
                }
                if !pending_to_create.is_empty() {
                    debug!(logger, "Pending to create:");
                    for ref log_item in &pending_to_create {
                        debug!(logger, "     -- {:?}", log_item);
                    }
                }
            }

            let mut failed_count: i32 = 0;
            // Delete records no more need
            for old_record in pending_to_delete {
                let delete_url = format!(
                    "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                    self.zone_id, &old_record.id
                );
                match options
                    .http(HttpMethod::Delete, &delete_url)
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
                                    &old_record.content,
                                    &old_record.name,
                                    if res.success { "success" } else { "failed" },
                                    res.get_error_message()
                                );
                            }
                            if !res.success {
                                failed_count += 1;
                            }
                        }
                        Err(e) => {
                            failed_count += 1;
                            if let Some(ref logger) = self.logger {
                                error!(
                                    logger,
                                    "Delete {} for {} failed, error: {}",
                                    &old_record.content,
                                    &old_record.name,
                                    e
                                );
                            }
                        }
                    },
                    Err(e) => {
                        failed_count += 1;
                        if let Some(ref logger) = self.logger {
                            error!(
                                logger,
                                "Delete {} for {} failed, error: {}",
                                &old_record.content,
                                &old_record.name,
                                e
                            );
                        }
                    }
                }
            }

            // Create new records
            for ref mut new_record in pending_to_create {
                let create_url = format!(
                    "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
                    self.zone_id
                );
                match options
                    .http(HttpMethod::Post, &create_url)
                    .bearer_auth(self.token.clone())
                    .json(&new_record.record)
                    .send()
                    .await
                {
                    Ok(rsp) => match rsp.json::<CloudflareResponseResult>().await {
                        Ok(res) => {
                            if let Some(ref logger) = self.logger {
                                debug!(
                                    logger,
                                    "Create {} for {} {}.{}",
                                    &new_record.record.content,
                                    &new_record.record.name,
                                    if res.success { "success" } else { "failed" },
                                    res.get_error_message()
                                );
                            }
                            if !res.success {
                                failed_count += 1;
                            }
                        }
                        Err(e) => {
                            failed_count += 1;
                            if let Some(ref logger) = self.logger {
                                error!(
                                    logger,
                                    "Create {} for {} failed, error: {}",
                                    &new_record.record.content,
                                    &new_record.record.name,
                                    e
                                );
                            }
                        }
                    },
                    Err(e) => {
                        failed_count += 1;
                        if let Some(ref logger) = self.logger {
                            error!(
                                logger,
                                "Create {} for {} failed, error: {}",
                                &new_record.record.content,
                                &new_record.record.name,
                                e
                            );
                        }
                    }
                }
            }

            if let Some(ref logger) = self.logger {
                let action_description: Vec<String> = recs.iter().map(|r| r.to_string()).collect();
                if failed_count > 0 {
                    ret = 1;
                    error!(
                        logger,
                        "Update domain name {} to {} with {} error(s)",
                        domain,
                        action_description.join(","),
                        failed_count
                    );
                } else {
                    info!(
                        logger,
                        "Update domain name {} to {} finished",
                        domain,
                        action_description.join(",")
                    );
                }
            }
        }

        if ret == 0 {
            Ok(ret)
        } else {
            Err(())
        }
    }
}
