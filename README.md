# DDNS client

|                           | [Linux+OSX][linux-link] | [Windows MSVC+GNU][windows-link] |
|:-------------------------:|:-----------------------:|:--------------------------------:|
| Build & Publish           | ![linux-badge]          | ![windows-badge]                 |

[linux-badge]: https://travis-ci.org/owt5008137/ddns-cli.svg?branch=master "Travis build status"
[linux-link]:  https://travis-ci.org/owt5008137/ddns-cli "Travis build status"
[windows-badge]: https://ci.appveyor.com/api/projects/status/ht5pks682ehe2vkt?svg=true "AppVeyor build status"
[windows-link]:  https://ci.appveyor.com/project/owt5008137/ddns-cli "AppVeyor build status"

## Developer

1. 下载rust编译环境( https://www.rust-lang.org )
    > 在一些发行版或者软件仓库中也可以通过 pacman/apt/yum/choco 等安装 rust 目标
2. 升级rust工具链 ```rustup self update && rustup update```
3. 安装一个编译目标（默认也会安装一个的） ```rustup target install <目标架构>```
    > 可以通过 ```rustup target list``` 来查看支持的架构
4. 克隆仓库并进入主目录
5. 运行编译命令: ```cargo build```

更多详情见： https://rustup.rs/ 

## LICENSE

[MIT](LICENSE-MIT) or [Apache License - 2.0](LICENSE)

[1]: https://crates.io/crates/handlebars
[2]: https://docs.rs/regex/
[3]: https://github.com/Microsoft/vcpkg
