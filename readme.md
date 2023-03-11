# Make quote

[<img alt="crates.io" src="https://img.shields.io/crates/v/make-quote.svg?style=flat&color=fd7726&labelColor=252535&logo=rust" height="20">](https://crates.io/crates/make-quote)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/make-quote?color=2b5a28&logo=rust&labelColor=252535" height="20">](https://docs.rs/make-quote/)

This library provides a single function that can generate a quote image from user input.
It is still under development, further document and crates.io dependencies will be released
someday.

## Usage

```rust
// First of all, load an font into memory
let font = load_font("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc").unwrap();

// Create a configuration about one image
let config = Configuration::builder()
    .output_size(1920, 1080)
    .font(&font)
    .avatar_path("./assets/avatar.png")
    .quote("大家好，今天来点大家想看的东西。")
    .username("V5电竞俱乐部中单选手 Otto")
    .build();

// Then generate the image and get the image buffer
let buffer = make_quote_image(&config).unwrap();

// You can do anything you like to the buffer, save it or just send it through the net.
std::fs::write("./assets/test.jpg", buffer).unwrap();
```

This will provide the below example output:

![img](./assets/test.jpg)
