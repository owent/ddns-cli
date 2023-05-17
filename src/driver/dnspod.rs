use std::clone::Clone;
use std::sync::Arc;

pub use core::future::Future;
use futures::future::{self, BoxFuture, FutureExt};

use serde::{Deserialize, Serialize};

extern crate clap;
use clap::{Arg, ArgMatches, Command};

use super::super::option;
use super::{Driver, DriverResult, Record};

type SharedProgramOptions = super::SharedProgramOptions;
type HttpMethod = super::HttpMethod;

#[derive(Default)]
pub struct Dnspod {
    domain_id: String,
    domain: String,
    token: String,
    token_id: String,
    sub_domain: String,
    logger: Option<slog::Logger>,
}

static DNSPOD_RESPONSE_CODE_SUCCESS: &str = "1";

impl Driver for Dnspod {
    fn initialize(&mut self, app: Command) -> Command {
        app.arg(
            Arg::new("dp-domain-id")
                .long("dp-domain-id")
                .value_name("DOMAIN ID")
                .help("Set domain id of dnspod API, --dp-domain-id or --dp-domain must be set when using dnspod"),
        ).arg(
            Arg::new("dp-domain")
                .long("dp-domain")
                .value_name("DOMAIN")
                .help("Set domain of dnspod API, --dp-domain-id or --dp-domain must be set when using dnspod"),
        ).arg(
            Arg::new("dp-name")
                .long("dp-name")
                .value_name("SUB DOMAIN NAME")
                .help("Set sub domain name of dnspod API, using @ if it's not set"),
        ).arg(
            Arg::new("dp-token")
                .long("dp-token")
                .value_name("TOKEN")
                .help("Set token of dnspod API, you can get it from https://console.dnspod.cn/account/token"),
        ).arg(
            Arg::new("dp-token-id")
                .long("dp-token-id")
                .value_name("TOKEN_ID")
                .help("Set token ID of dnspod API, you can get it from https://console.dnspod.cn/account/token"),
        )
    }

    fn parse_options(&mut self, matches: &ArgMatches, options: &mut SharedProgramOptions) {
        self.domain_id = option::unwraper_option_or(matches, "dp-domain-id", String::default());
        self.domain = option::unwraper_option_or(matches, "dp-domain", String::default());
        self.token = option::unwraper_option_or(matches, "dp-token", String::default());
        self.token_id = option::unwraper_option_or(matches, "dp-token-id", String::default());
        self.sub_domain = option::unwraper_option_or(matches, "dp-name", String::from("@"));

        if (!self.token_id.is_empty() || !self.token.is_empty())
            && (!self.domain_id.is_empty() || !self.domain.is_empty())
        {
            self.logger = Some(options.create_logger("Dnspod"));
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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DnspodRecord {
    pub record_type: &'static str,
    pub sub_domain: String,
    pub value: String,
    pub ttl: String, // 600
    pub mx: String,  // 0-20
    pub domain_id: String,
    pub record_line: String,
    pub record_line_id: String,
}

#[derive(Debug, Clone)]
struct DnspodRecordAction {
    pub record: DnspodRecord,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DnspodGetResponseRecord {
    pub id: String,
    pub name: String,
    pub line: String,
    pub line_id: String,
    pub r#type: String,
    pub value: String,
    pub mx: String,  // 0-20
    pub ttl: String, // 600
}

impl PartialEq for DnspodRecord {
    fn eq(&self, other: &Self) -> bool {
        self.record_type == other.record_type && self.sub_domain == other.sub_domain
    }
}

impl PartialEq for DnspodRecordAction {
    fn eq(&self, other: &Self) -> bool {
        self.record == other.record
    }
}

impl PartialEq for DnspodGetResponseRecord {
    fn eq(&self, other: &Self) -> bool {
        self.r#type == other.r#type && self.name == other.name
    }
}

impl PartialEq<DnspodRecord> for DnspodGetResponseRecord {
    fn eq(&self, other: &DnspodRecord) -> bool {
        self.r#type == other.record_type && self.name == other.sub_domain
    }
}

impl PartialEq<DnspodGetResponseRecord> for DnspodRecord {
    fn eq(&self, other: &DnspodGetResponseRecord) -> bool {
        self.record_type == other.r#type && self.sub_domain == other.name
    }
}

impl PartialEq<DnspodRecordAction> for DnspodGetResponseRecord {
    fn eq(&self, other: &DnspodRecordAction) -> bool {
        self.r#type == other.record.record_type && self.name == other.record.sub_domain
    }
}

impl PartialEq<DnspodGetResponseRecord> for DnspodRecordAction {
    fn eq(&self, other: &DnspodGetResponseRecord) -> bool {
        self.record.record_type == other.r#type && self.record.sub_domain == other.name
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DnspodResponseStatus {
    pub code: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DnspodResponseDomain {
    pub id: String,
    pub name: String,
    pub punycode: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DnspodGetResponseResult {
    pub status: DnspodResponseStatus,
    pub domain: Option<DnspodResponseDomain>,
    pub records: Option<Vec<DnspodGetResponseRecord>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DnspodResponseResult {
    pub status: DnspodResponseStatus,
}

impl DnspodResponseResult {
    pub fn get_error_message(&self) -> &str {
        &self.status.message
    }

    pub fn is_success(&self) -> bool {
        let code = self.status.code.trim();
        code == DNSPOD_RESPONSE_CODE_SUCCESS
    }
}

impl Dnspod {
    fn generate_common_form(&self) -> reqwest::multipart::Form {
        let api_token_parameter = if self.token_id.is_empty() {
            self.token.clone()
        } else {
            format!("{},{}", self.token_id, self.token)
        };

        let form = reqwest::multipart::Form::new()
            .text("login_token", api_token_parameter)
            .text("format", "json");

        if !self.domain_id.is_empty() {
            form.text("domain_id", self.domain_id.clone())
        } else {
            form.text("domain", self.domain.clone())
        }
    }

    async fn update<'a, 'b>(
        &'a mut self,
        options: SharedProgramOptions,
        recs: &'b [Record],
    ) -> DriverResult
    where
        'b: 'a,
    {
        // Common parameters: login_token=LOGIN_TOKEN&format=json&lang=en
        let mut ret: i32 = 0;
        let actions: Vec<Arc<DnspodRecordAction>> = recs
            .iter()
            .map(|ele| {
                Arc::new(match ele {
                    Record::A(r) => DnspodRecordAction {
                        record: DnspodRecord {
                            record_type: "A",
                            sub_domain: String::default(),
                            value: r.to_string(),
                            ttl: String::from("600"),
                            mx: String::from("10"), // 0-20
                            domain_id: String::default(),
                            record_line: String::default(),
                            record_line_id: String::from("0"), // @see https://docs.dnspod.cn/api/5f5623f9e75cf42d25bf6776/
                        },
                    },
                    Record::Aaaa(r) => DnspodRecordAction {
                        record: DnspodRecord {
                            record_type: "AAAA",
                            sub_domain: String::default(),
                            value: r.to_string(),
                            ttl: String::from("600"),
                            mx: String::from("10"), // 0-20
                            domain_id: String::default(),
                            record_line: String::default(),
                            record_line_id: String::from("0"), // @see https://docs.dnspod.cn/api/5f5623f9e75cf42d25bf6776/
                        },
                    },
                    Record::Cname(r) => DnspodRecordAction {
                        record: DnspodRecord {
                            record_type: "CNAME",
                            sub_domain: String::default(),
                            value: r.to_string(),
                            ttl: String::from("600"),
                            mx: String::from("10"), // 0-20
                            domain_id: String::default(),
                            record_line: String::default(),
                            record_line_id: String::from("0"), // @see https://docs.dnspod.cn/api/5f5623f9e75cf42d25bf6776/
                        },
                    },
                    Record::Mx(r) => DnspodRecordAction {
                        record: DnspodRecord {
                            record_type: "MX",
                            sub_domain: String::default(),
                            value: r.to_string(),
                            ttl: String::from("600"),
                            mx: String::from("10"), // 0-20
                            domain_id: String::default(),
                            record_line: String::default(),
                            record_line_id: String::from("0"), // @see https://docs.dnspod.cn/api/5f5623f9e75cf42d25bf6776/
                        },
                    },
                    Record::Txt(r) => DnspodRecordAction {
                        record: DnspodRecord {
                            record_type: "TXT",
                            sub_domain: String::default(),
                            value: r.to_string(),
                            ttl: String::from("600"),
                            mx: String::from("10"), // 0-20
                            domain_id: String::default(),
                            record_line: String::default(),
                            record_line_id: String::from("0"), // @see https://docs.dnspod.cn/api/5f5623f9e75cf42d25bf6776/
                        },
                    },
                })
            })
            .collect();

        let mut failed_count: i32 = 0;
        failed_count += async {
            let mut current_failed_count: i32 = 0;
            let mut pending_to_delete: Vec<Arc<DnspodGetResponseRecord>> = vec![];
            let mut pending_to_create: Vec<Arc<DnspodRecordAction>> = vec![];
            let old_records = self.get_records(options.clone()).await;

            // Initialize pending delete records
            for old_record in &old_records {
                let keep = actions.iter().any(|act| {
                    act.as_ref().record.record_type == old_record.r#type
                        && act.as_ref().record.value == old_record.value
                });
                if !keep {
                    pending_to_delete.push(old_record.clone());
                }
            }

            // Initialize new records
            for action in &actions {
                let already_exists = old_records.iter().any(|res| {
                    res.r#type == action.record.record_type && res.value == action.record.value
                });
                if !already_exists {
                    let mut new_record = action.clone();
                    Arc::make_mut(&mut new_record).record.sub_domain = self.sub_domain.clone();
                    pending_to_create.push(new_record);
                }
            }

            if let Some(ref logger) = self.logger {
                if !old_records.is_empty() {
                    debug!(logger, "Old records:");
                    for ref log_item in &old_records {
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

            // Delete records no more need
            current_failed_count += self
                .remove_records(options.clone(), pending_to_delete)
                .await;

            // Create new records
            current_failed_count += self
                .create_records(options.clone(), pending_to_create)
                .await;

            current_failed_count
        }
        .await;

        if let Some(ref logger) = self.logger {
            let action_description: Vec<String> = recs.iter().map(|r| r.to_string()).collect();
            if failed_count > 0 {
                ret = 1;
                error!(
                    logger,
                    "Update domain name {} to {} with {} error(s)",
                    self.domain,
                    action_description.join(","),
                    failed_count
                );
            } else {
                info!(
                    logger,
                    "Update domain name {} to {} finished",
                    self.domain,
                    action_description.join(",")
                );
            }
        }

        if ret == 0 {
            Ok(ret)
        } else {
            Err(())
        }
    }

    async fn get_records(
        &mut self,
        options: SharedProgramOptions,
    ) -> Vec<Arc<DnspodGetResponseRecord>> {
        let mut ret: Vec<Arc<DnspodGetResponseRecord>> = vec![];

        // Records over 100 must be request by page
        let mut page_offset: usize = 0;
        let get_list_url = String::from("https://dnsapi.cn/Record.List");

        loop {
            let form = self
                .generate_common_form()
                .text("sub_domain", self.sub_domain.clone());
            let cli = options
                .http(HttpMethod::Post, &get_list_url)
                .multipart(form);

            let rsp = match cli.send().await {
                Ok(v) => v,
                Err(e) => {
                    if let Some(ref logger) = self.logger {
                        error!(logger, "Send HTTP request failed, error: {}", e);
                    }
                    break;
                }
            };

            let rsp_text = match rsp.text().await {
                Ok(v) => v,
                Err(e) => {
                    if let Some(ref logger) = self.logger {
                        error!(logger, "Fetch response body failed, error: {}", e);
                    }

                    break;
                }
            };

            let result = match serde_json::from_str::<DnspodGetResponseResult>(&rsp_text) {
                Ok(v) => v,
                Err(e) => {
                    if let Some(ref logger) = self.logger {
                        error!(
                            logger,
                            "Parse response body failed, error: {}.\nbody: {}", e, rsp_text
                        );
                    }
                    break;
                }
            };

            let records = match result.records {
                Some(x) => x,
                _ => break,
            };

            if records.is_empty() {
                break;
            }

            for old_record in &records {
                ret.push(Arc::new((*old_record).clone()));
            }

            page_offset += records.len();
            if page_offset % 100 != 0 {
                break;
            }
        }

        ret
    }

    async fn remove_records(
        &mut self,
        options: SharedProgramOptions,
        pending_to_delete: Vec<Arc<DnspodGetResponseRecord>>,
    ) -> i32 {
        let mut ret = 0;
        // Delete records no more need
        let delete_url = String::from("https://dnsapi.cn/Record.Remove");
        for ref old_record in pending_to_delete {
            let form = self
                .generate_common_form()
                .text("record_id", old_record.id.clone());

            let error_message;
            match options
                .http(HttpMethod::Post, &delete_url)
                .multipart(form)
                .send()
                .await
            {
                Ok(rsp) => {
                    if let Some(ref logger) = self.logger {
                        debug!(logger, "====== Crash checkpoint 1");
                    }
                    match rsp.json::<DnspodResponseResult>().await {
                        Ok(res) => {
                            error_message = self.check_result("Delete", &old_record.name, res)
                        }
                        Err(e) => error_message = Some(format!("{}", e)),
                    }
                }
                Err(e) => error_message = Some(format!("{}", e)),
            }

            if let Some(err_msg) = error_message {
                ret += 1;
                if let Some(ref logger) = self.logger {
                    error!(
                        logger,
                        "Delete {} for {} failed, error: {}", old_record.name, self.domain, err_msg
                    );
                }
            }
        }

        ret
    }

    async fn create_records(
        &mut self,
        options: SharedProgramOptions,
        pending_to_create: Vec<Arc<DnspodRecordAction>>,
    ) -> i32 {
        let mut ret = 0;
        let create_url = String::from("https://dnsapi.cn/Record.Create");

        for new_record_action in pending_to_create {
            let new_record = new_record_action.record.clone();
            let form = self
                .generate_common_form()
                .text("sub_domain", new_record.sub_domain.clone())
                .text("record_type", new_record.record_type)
                .text("record_line_id", new_record.record_line_id.clone())
                .text("value", new_record.value.clone())
                .text("mx", new_record.mx.clone())
                .text("ttl", new_record.ttl.clone());

            let error_message;
            match options
                .http(HttpMethod::Post, &create_url)
                .multipart(form)
                .send()
                .await
            {
                Ok(rsp) => {
                    if let Some(ref logger) = self.logger {
                        debug!(logger, "====== Crash checkpoint 1");
                    }
                    match rsp.json::<DnspodResponseResult>().await {
                        Ok(res) => {
                            error_message = self.check_result("Create", &new_record.sub_domain, res)
                        }
                        Err(e) => error_message = Some(format!("{}", e)),
                    }
                }
                Err(e) => error_message = Some(format!("{}", e)),
            }

            if let Some(err_msg) = error_message {
                ret += 1;
                if let Some(ref logger) = self.logger {
                    error!(
                        logger,
                        "Create {} for {} failed, error: {}",
                        new_record.sub_domain,
                        self.domain,
                        err_msg
                    );
                }
            }
        }

        ret
    }

    fn check_result(
        &self,
        action: &str,
        sub_domain: &String,
        res: DnspodResponseResult,
    ) -> Option<String> {
        if res.is_success() {
            if let Some(ref logger) = self.logger {
                debug!(
                    logger,
                    "{} {} for {} {}.",
                    action,
                    sub_domain,
                    self.domain,
                    res.get_error_message()
                );
            }
            None
        } else {
            Some(String::from(res.get_error_message()))
        }
    }
}
