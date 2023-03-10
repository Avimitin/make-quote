# Make quote

This library provides a single function that can generate a quote image from user input.
It is still under development, further document and crates.io dependencies will be released
someday.

## Usage

```rust
let config = Configuration {
    output_size: (1920, 1080),
    output_path: "./assets/test.jpg".to_string(),
    font_path: "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc".to_string(),
    avatar_path: "./assets/avatar.jpg".to_string(),
    quote: "我超，OP！".to_string(),
    username: "嘉然今天吃什么".to_string(),
};

make_quote_image(&config);
```

This will provide the below example output:

![img](./assets/test.jpg)
