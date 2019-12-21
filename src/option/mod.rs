use std::convert::TryFrom;
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

use http;
use hyper;

#[derive(Debug, Clone)]
pub struct ProgramOptions {
    pub timeout: Duration,
    pub insecure: bool,
    pub logger: slog::Logger,
    pub http_user_agent: String,
}

pub type SharedProgramOptions = Arc<ProgramOptions>;
#[allow(dead_code)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
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

    pub fn http<U>(
        &self,
        method: HttpMethod,
        url: U,
    ) -> (hyper::client::Builder, http::request::Builder)
    where
        http::Uri: TryFrom<U>,
        <http::Uri as TryFrom<U>>::Error: Into<http::Error>,
    {
        let mut cli = hyper::client::Builder::default();
        let _ = cli.keep_alive_timeout(self.timeout);
        let builder_l1 = match method {
            HttpMethod::GET => http::request::Request::builder().method("GET"),
            HttpMethod::POST => http::request::Request::builder().method("POST"),
            HttpMethod::PUT => http::request::Request::builder().method("PUT"),
            HttpMethod::DELETE => http::request::Request::builder().method("DELETE"),
        };
        let builder = builder_l1
            .uri(url)
            .header("User-Agent", &self.http_user_agent);

        (cli, builder)
    }
}
