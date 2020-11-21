# DDNS client

|                           | [Linux+OSX][linux-link] | [Windows MSVC+GNU][windows-link] |
|:-------------------------:|:-----------------------:|:--------------------------------:|
| Build & Publish           | ![linux-badge]          | ![windows-badge]                 |

[linux-badge]: https://travis-ci.org/owt5008137/ddns-cli.svg?branch=master "Travis build status"
[linux-link]:  https://travis-ci.org/owt5008137/ddns-cli "Travis build status"
[windows-badge]: https://ci.appveyor.com/api/projects/status/ht5pks682ehe2vkt?svg=true "AppVeyor build status"
[windows-link]:  https://ci.appveyor.com/project/owt5008137/ddns-cli "AppVeyor build status"

Docker: ```docker.io/owt5008137/ddns-cli```

## Usage

```bash
# help
./ddns-cli -h

# You can get token from https://dash.cloudflare.com/profile/api-tokens and zone id from your domian zone page
./ddns-cli --get-ip-by-url https://myip.biturl.top/ --cf-domain <DOMAIN> --cf-token <Cloudflare TOKEN> --cf-zone-id <Cloudflare ZoneID>

# You can get token and token id from https://console.dnspod.cn/account/token
./ddns-cli --get-ip-by-url https://myip.biturl.top/ --dp-name <SUB DOAMIN NAME> --dp-domain <BASE DOMAIN NAME> --dp-token <Dnspod TOKEN> --dp-token-id <Dnspod token id>
```


```bash
docker/podman pull docker.io/owt5008137/ddns-cli:latest
docker/podman run ddns-cli ddns-cli --get-ip-by-url https://myip.biturl.top/ --cf-domain <DOMAIN> --cf-token <Cloudflare TOKEN> --cf-zone-id <Cloudflare ZoneID>
docker/podman run ddns-cli ddns-cli --get-ip-by-url https://myip.biturl.top/ --dp-name <SUB DOAMIN NAME> --dp-domain <BASE DOMAIN NAME> --dp-token <Dnspod TOKEN> --dp-token-id <Dnspod token id>
```

## LICENSE

[MIT](LICENSE-MIT) or [Apache License - 2.0](LICENSE)

[1]: https://crates.io/crates/handlebars
[2]: https://docs.rs/regex/
[3]: https://github.com/Microsoft/vcpkg
