# DDNS client

|                           |   [Linux][linux-link]   |     [Windows][windows-link]      |     [macOS][macos-link]    |
|:-------------------------:|:-----------------------:|:--------------------------------:|:--------------------------:|
| Build & Publish           | ![linux-badge]          | ![windows-badge]                 | ![macos-badge]             |

[linux-badge]: https://github.com/owent/ddns-cli/workflows/Build%20On%20Linux/badge.svg "Linux build status"
[linux-link]:  https://github.com/owent/ddns-cli/actions?query=workflow%3A%22Build+On+Linux%22 "Linux build status"
[windows-badge]: https://github.com/owent/ddns-cli/workflows/Build%20On%20Windows/badge.svg "Windows build status"
[windows-link]:  https://github.com/owent/ddns-cli/actions?query=workflow%3A%22Build+On+Windows%22 "Windows build status"
[macos-badge]: https://github.com/owent/ddns-cli/workflows/Build%20On%20macOS/badge.svg "macOS build status"
[macos-link]:  https://github.com/owent/ddns-cli/actions?query=workflow%3A%22Build+On+macOS%22 "macOS build status"

Docker: [```docker.io/owt5008137/ddns-cli```][4]

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
[4]: https://hub.docker.com/r/owt5008137/ddns-cli
