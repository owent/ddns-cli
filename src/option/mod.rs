use std::process;
use std::result;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::{atomic, Arc};
use std::time::Duration;

extern crate clap;
use clap::{App, Arg, ArgMatches};

use slog;
use slog::Drain;

use reqwest::{self, r#async::ClientBuilder};

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

pub fn app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .max_term_width(120)
        .arg(
            Arg::with_name("version")
                .short("v")
                .long("version")
                .help("Show version"),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .value_name("TIMEOUT")
                .takes_value(true)
                .default_value("60000")
                .help("Set timeout in miliseconds"),
        )
        .arg(
            Arg::with_name("insecure")
                .short("k")
                .long("insecure")
                .help("Allow connections to SSL sites without certs"),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .help("Output verbose log"),
        )
        .arg(
            Arg::with_name("http-user-agent")
                .long("http-user-agent")
                .help("Set user agent for http request"),
        )
        .arg(
            Arg::with_name("no-proxy")
                .long("no-proxy")
                .help("Do not use any proxy"),
        )
        .arg(
            Arg::with_name("proxy")
                .long("proxy")
                .takes_value(true)
                .help("Set http proxy(http|https|socks5|socks5h://HOST:PORT)"),
        )
        .arg(
            Arg::with_name("proxy-username")
                .long("proxy-username")
                .takes_value(true)
                .help("Set proxy username fo auth"),
        )
        .arg(
            Arg::with_name("proxy-password")
                .long("proxy-password")
                .takes_value(true)
                .help("Set proxy password fo auth"),
        )
}

fn generate_options<'a>(matches: &ArgMatches<'a>) -> ProgramOptions {
    let debug_log_on = Arc::new(atomic::AtomicBool::new(matches.is_present("verbose")));
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
        insecure: matches.is_present("insecure"),
        logger: slog::Logger::root(drain, o!("tag" => format!("[{}]", "main"))),
        http_user_agent: unwraper_option_or(
            &matches,
            "http-user-agent",
            format!("{}/{}", crate_name!(), crate_version!()),
        ),
        no_proxy: matches.is_present("no-proxy"),
        proxy_address: unwraper_option_or(&matches, "proxy", String::default()),
        proxy_username: unwraper_option_or(&matches, "proxy-username", String::default()),
        proxy_password: unwraper_option_or(&matches, "proxy-password", String::default()),
    }
}

pub fn parse_options<'a, 'b>(app: App<'a, 'b>) -> (ArgMatches<'a>, SharedProgramOptions)
where
    'a: 'b,
{
    let matches: ArgMatches<'a> = app.get_matches();
    if matches.is_present("version") {
        println!("{}", crate_version!());
        process::exit(0);
    }

    let options = generate_options(&matches);
    (matches, Arc::new(options))
}

pub fn unwraper_from_str_or<'a, T, S: AsRef<str>>(matches: &ArgMatches<'a>, name: S, def: T) -> T
where
    T: FromStr,
{
    if let Some(mut x) = matches.values_of(name) {
        if let Some(val) = x.next() {
            if let Ok(ret) = val.parse::<T>() {
                return ret;
            }
        }
    }

    def
}

pub trait OptionValueWrapper<T> {
    fn pick(self, input: &str) -> Self;
}

/**
impl OptionValueWrapper<bool> for bool {
    fn pick(self, input: &str) -> Self {
        let s = input.to_lowercase();
        if s.is_empty() {
            return false;
        }
        s != "0" && s != "false" && s != "disable" && s != "disabled" && s != "no"
    }
}
**/

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

pub fn unwraper_option_or<'a, T, S: AsRef<str>>(matches: &ArgMatches<'a>, name: S, def: T) -> T
where
    T: OptionValueWrapper<T>,
{
    if let Some(mut x) = matches.values_of(name) {
        if let Some(val) = x.next() {
            return def.pick(val);
        }
    }

    def
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
        self.logger.new(o!("tag" => format!("[{}]", tag)))
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

    pub fn http<U>(&self, method: HttpMethod, url: U) -> reqwest::r#async::RequestBuilder
    where
        U: reqwest::IntoUrl,
    {
        let mut builder = ClientBuilder::new()
            .danger_accept_invalid_certs(!self.insecure)
            .connect_timeout(self.timeout)
            .gzip(true)
            .redirect(reqwest::RedirectPolicy::limited(32))
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
