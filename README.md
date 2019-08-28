# Hacker News Scraper

Example Rust cli app. Scrapes posts from https://news.ycombinator.com/news into json.

```
USAGE:
    hacker-news-scraper [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --posts <POSTS>    Number of posts to fetch between 0 & 100, default 30
```

## Build
Install rust with [rustup](https://www.rustup.rs/) use the latest stable release.

```sh
# build native binary at target/release/hacker-news-scraper
cargo build --release
```

The binary may now be run directly.
```sh
target/release/hacker-news-scraper --help
```

## Run from code
```sh
cargo run --release -- --posts 30
```


## Test
Run the full test suite with
```sh
cargo test --features mock-news
```
