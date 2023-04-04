use std::process;
use std::result;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::{atomic, Arc};
use std::time::Duration;

extern crate clap;
use clap::{Arg, ArgAction, ArgMatches, Command};

use slog;
use slog::Drain;

use reqwest::{self, ClientBuilder};

#[derive(Debug, Clone)]
pub struct ProgramOptions {
    pub timeout: Duration,
    pub insecure: bool,
    pub logger: slog::Logger,
    pub http_user_agent: String,
    pub no_proxy: bool,
    pub proxy_address: String,
    pub proxy_username: String,
    pub proxy_password: String,
}

pub type SharedProgramOptions = Arc<ProgramOptions>;
#[allow(dead_code)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
}

pub fn app<'a>() -> Command {
    let matches = command!();
    matches
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .max_term_width(120)
        // .arg(
        //     Arg::new("version")
        //         .short('v')
        //         .long("version")
        //         .action(ArgAction::SetTrue)
        //         .help("Show version"),
        // )
        .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .value_name("TIMEOUT")
                .default_value("60000")
                .help("Set timeout in miliseconds"),
        )
        .arg(
            Arg::new("insecure")
                .short('k')
                .long("insecure")
                .action(ArgAction::SetTrue)
                .help("Allow connections to SSL sites without certs"),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Output verbose log"),
        )
        .arg(
            Arg::new("http-user-agent")
                .long("http-user-agent")
                .help("Set user agent for http request"),
        )
        .arg(
            Arg::new("no-proxy")
                .long("no-proxy")
                .help("Do not use any proxy"),
        )
        .arg(
            Arg::new("proxy")
                .long("proxy")
                .help("Set http proxy(http|https|socks5|socks5h://HOST:PORT)"),
        )
        .arg(
            Arg::new("proxy-username")
                .long("proxy-username")
                .help("Set proxy username fo auth"),
        )
        .arg(
            Arg::new("proxy-password")
                .long("proxy-password")
                .help("Set proxy password fo auth"),
        )
}

pub fn unwraper_flag<S>(matches: &ArgMatches, name: S) -> bool
where
    S: AsRef<str>,
{
    if let Ok(rx) = matches.try_get_one::<bool>(name.as_ref()) {
        if let Some(x) = rx {
            return *x;
        }
    }

    false
}

fn generate_options(matches: &ArgMatches) -> ProgramOptions {
    let debug_log_on = Arc::new(atomic::AtomicBool::new(unwraper_flag(&matches, "verbose")));
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = RuntimeLevelFilter {
        drain: drain,
        on: debug_log_on.clone(),
    }
    .fuse();
    let drain = slog_async::Async::new(drain)
        .chan_size(4096)
        .overflow_strategy(slog_async::OverflowStrategy::Block)
        .build()
        .fuse();

    ProgramOptions {
        timeout: Duration::from_millis(unwraper_from_str_or(matches, "timeout", 60000)),
        insecure: unwraper_flag(&matches, "insecure"),
        logger: slog::Logger::root(drain, o!()),
        http_user_agent: unwraper_option_or(
            &matches,
            "http-user-agent",
            format!("{}/{}", crate_name!(), crate_version!()),
        ),
        no_proxy: unwraper_flag(&matches, "no-proxy"),
        proxy_address: unwraper_option_or(&matches, "proxy", String::default()),
        proxy_username: unwraper_option_or(&matches, "proxy-username", String::default()),
        proxy_password: unwraper_option_or(&matches, "proxy-password", String::default()),
    }
}

pub fn parse_options(app: Command) -> (ArgMatches, SharedProgramOptions) {
    let matches: ArgMatches = app.get_matches();
    if unwraper_flag(&matches, "version") {
        println!("{}", crate_version!());
        process::exit(0);
    }

    let options = generate_options(&matches);
    (matches, Arc::new(options))
}

pub fn unwraper_from_str_or<T, S>(matches: &ArgMatches, name: S, def: T) -> T
where
    T: FromStr,
    S: AsRef<str>,
{
    if let Ok(rx) = matches.try_get_raw(name.as_ref()) {
        if let Some(x) = rx {
            for val in x {
                if let Some(str_val) = val.to_str() {
                    if let Ok(ret) = str_val.parse::<T>() {
                        return ret;
                    }
                }
            }
        }
    }

    def
}

pub trait OptionValueWrapper<T> {
    fn pick(self, input: &str) -> Self;
}

impl<T> OptionValueWrapper<T> for T
where
    T: FromStr,
{
    fn pick(self, input: &str) -> Self {
        if let Ok(v) = input.parse::<T>() {
            v
        } else {
            self
        }
    }
}

pub fn unwraper_option_or<T, S>(matches: &ArgMatches, name: S, def: T) -> T
where
    T: OptionValueWrapper<T>,
    S: AsRef<str>,
{
    if let Ok(rx) = matches.try_get_raw(name.as_ref()) {
        if let Some(x) = rx {
            for val in x {
                if let Some(str_val) = val.to_str() {
                    return def.pick(str_val);
                }
            }
        }
    }

    def
}

pub fn unwraper_multiple_values<T, S, TN>(
    matches: &ArgMatches,
    name: S,
    logger: &slog::Logger,
    type_name: TN,
) -> Vec<T>
where
    T: FromStr,
    S: AsRef<str>,
    TN: AsRef<str>,
{
    let mut ret = vec![];
    if let Ok(rx) = matches.try_get_raw_occurrences(name.as_ref()) {
        if let Some(x) = rx {
            for val_set in x {
                for val_os_str in val_set {
                    if let Some(val) = val_os_str.to_str() {
                        if let Ok(res) = val.parse::<T>() {
                            ret.push(res);
                        } else {
                            error!(logger, "Invalid {} value {}", type_name.as_ref(), val);
                        }
                    } else {
                        error!(
                            logger,
                            "Can not convert {:?} to string for {}",
                            val_os_str,
                            type_name.as_ref()
                        );
                    }
                }
            }
        }
    }

    ret
}

/// Custom Drain logic
struct RuntimeLevelFilter<D> {
    drain: D,
    on: Arc<atomic::AtomicBool>,
}
impl<D> Drain for RuntimeLevelFilter<D>
where
    D: Drain,
{
    type Ok = Option<D::Ok>;
    type Err = Option<D::Err>;
    fn log(
        &self,
        record: &slog::Record,
        values: &slog::OwnedKVList,
    ) -> result::Result<Self::Ok, Self::Err> {
        let current_level = if self.on.load(Ordering::Relaxed) {
            slog::Level::Trace
        } else {
            slog::Level::Info
        };
        if record.level().is_at_least(current_level) {
            self.drain.log(record, values).map(Some).map_err(Some)
        } else {
            Ok(None)
        }
    }
}

impl ProgramOptions {
    pub fn create_logger(&self, tag: &str) -> slog::Logger {
        self.logger.new(o!("module" => format!("[{}]", tag)))
    }

    pub fn create_proxy(&self) -> Option<reqwest::Proxy> {
        if self.no_proxy || self.proxy_address.is_empty() {
            return None;
        }

        if let Ok(p) = reqwest::Proxy::all(&self.proxy_address) {
            if !self.proxy_username.is_empty() || !self.proxy_password.is_empty() {
                Some(p.basic_auth(&self.proxy_username, &self.proxy_password))
            } else {
                Some(p)
            }
        } else {
            None
        }
    }

    pub fn http<U>(&self, method: HttpMethod, url: U) -> reqwest::RequestBuilder
    where
        U: reqwest::IntoUrl,
    {
        let mut builder = ClientBuilder::new()
            .danger_accept_invalid_certs(!self.insecure)
            .connect_timeout(self.timeout)
            .gzip(true)
            .redirect(reqwest::redirect::Policy::limited(32))
            .timeout(self.timeout)
            //.use_rustls_tls()
            ;
        if let Some(p) = self.create_proxy() {
            builder = builder.proxy(p);
        }

        let builder = match method {
            HttpMethod::GET => builder.build().expect("Client::new()").get(url),
            HttpMethod::POST => builder.build().expect("Client::new()").post(url),
            HttpMethod::PUT => builder.build().expect("Client::new()").put(url),
            HttpMethod::PATCH => builder.build().expect("Client::new()").patch(url),
            HttpMethod::DELETE => builder.build().expect("Client::new()").delete(url),
            HttpMethod::HEAD => builder.build().expect("Client::new()").head(url),
        };
        builder.header("User-Agent", &self.http_user_agent)
    }
}
