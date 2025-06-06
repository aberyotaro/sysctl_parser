# sysctl_parser

`sysctl_parser` は、Linux の sysctl 設定ファイル（例: [sysctl.conf](https://man7.org/linux/man-pages/man5/sysctl.conf.5.html)）のような `key=value` 形式のテキストファイルをパースし、`HashMap<String, String>` として扱うための Rust ライブラリです。


## 使い方

Cargo.toml に以下を追加してください。

```toml
[dependencies]
sysctl_parser = { git = "https://github.com/aberyotaro/sysctl_parser.git" }
```

## サンプルコード

```rust
fn main() {
    let conf = File::open("etc/sysctl.conf").expect("Failed to open sysctl.conf");
    let scheme = File::open("etc/scheme.conf").ok(); // Optional scheme file
    match sysctl_parser::parse(conf, scheme) {
        Ok(map) => {
            for (key, value) in map {
                println!("{} = {}", key, value);
            }
        }
        Err(e) => eprintln!("Error parsing sysctl.conf: {}", e),
    }
}
```
