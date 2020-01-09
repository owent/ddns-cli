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
# You can get token from https://dash.cloudflare.com/profile/api-tokens and zone id from your domian zone page
./ddns-cli.exe --get-ip-by-url https://myip.biturl.top/ --cf-domain <DOMAIN> --cf-token <Cloudflare TOKEN> --cf-zone-id <Cloudflare ZoneID>
```


```bash
docker pull docker.io/owt5008137/ddns-cli:latest
docker run ddns-cli ddns-cli --get-ip-by-url https://myip.biturl.top/ --cf-domain <DOMAIN> --cf-token <Cloudflare TOKEN> --cf-zone-id <Cloudflare ZoneID>
```

## LICENSE

[MIT](LICENSE-MIT) or [Apache License - 2.0](LICENSE)

[1]: https://crates.io/crates/handlebars
[2]: https://docs.rs/regex/
[3]: https://github.com/Microsoft/vcpkg
