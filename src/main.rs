mod athing;

use crate::athing::*;
use select::{
    document::Document,
    predicate::{Class, Name, Predicate},
};
use std::error::Error;

/// Maximum post fetch count
const MAX_POSTS: usize = 100;

fn main() {
    // handle cli args
    let args = clap::App::new("Hacker News Scraper")
        .version("0.1")
        .author("Alex Butler <alexheretic@gmail.com>")
        .about("Hacker News HTML -> JSON post scraper")
        .arg(
            clap::Arg::with_name("posts")
                .long("posts")
                .value_name("POSTS")
                .validator(|pl| pl.parse::<usize>().map(|_| ()).map_err(|err| format!("{}", err)))
                .help("Number of posts to fetch between 0 & 100, default 30"),
        )
        .get_matches();

    let n = args.value_of("posts").and_then(|p| p.parse().ok()).unwrap_or(30).min(MAX_POSTS);
    if n == 0 {
        println!("[]");
        return;
    }

    // fetch the post data
    let posts = fetch_posts(n);

    // write the posts to stdout in json format
    serde_json::to_writer_pretty(std::io::stdout(), &posts).expect("write to stdout");
}

/// Fetch `n` hacker news posts
fn fetch_posts(n: usize) -> Vec<Post> {
    let client = reqwest::Client::new();

    (1..)
        .map(|page_num| {
            fetch_news_html(&client, page_num).unwrap_or_else(|err| {
                panic!("Failed to fetch page {} from hacker news: {}", page_num, err);
            })
        })
        .flat_map(|page| {
            page.find(Name("tr").and(Class("athing")))
                .filter_map(|tr| Post::try_from(AThing(tr)))
                .collect::<Vec<_>>()
        })
        .take(n)
        .collect()
}

/// Hacker news post data
///
/// Optional fields handle occasional posts that lack data e.g. "(X is Hiring ...)"
#[derive(Debug, PartialEq, Eq, serde::Serialize)]
struct Post {
    title: String,
    uri: String,
    rank: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    points: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comments: Option<usize>,
}

impl Post {
    /// Construct from node `<tr class='athing'...` returns `None` if invalid.
    ///
    /// Truncates `title` & `author` to max 256 characters (business requirement)
    fn try_from(athing: AThing<'_>) -> Option<Self> {
        let (uri, mut title) = athing.uri_and_title().expect("uri + title");
        let line2 = athing.line2()?;

        title.truncate(256);

        Some(Post {
            title,
            uri,
            rank: athing.rank()?,
            author: line2.author().map(|mut author| {
                author.truncate(256);
                author
            }),
            points: line2.points(),
            comments: line2.comments(),
        })
    }
}

/// Blocking html fetch from hacker news website
#[cfg(not(feature = "mock-news"))]
fn fetch_news_html(client: &reqwest::Client, page: usize) -> Result<Document, Box<dyn Error>> {
    const HACKER_NEWS_URL: &str = "https://news.ycombinator.com/news";
    let response = client.get(HACKER_NEWS_URL).query(&[("p", page)]).send()?.error_for_status()?;
    Ok(Document::from_read(response)?)
}

/// Mock html fetch implemented for the first 3 pages on 28-August-2019
#[cfg(feature = "mock-news")]
fn fetch_news_html(_: &reqwest::Client, page: usize) -> Result<Document, Box<dyn Error>> {
    let html = match page {
        1 => include_str!("../tests/news-p1.html"),
        2 => include_str!("../tests/news-p2.html"),
        3 => include_str!("../tests/news-p3.html"),
        _ => unimplemented!("page {}", page),
    };

    Ok(Document::from(html))
}

#[cfg(test)]
#[cfg(feature = "mock-news")]
mod test {
    use super::*;

    #[test]
    fn fetch_top_3() {
        let posts = fetch_posts(3);
        assert_eq!(posts.len(), 3);

        let mut posts = posts.into_iter();
        assert_eq!(
            posts.next(),
            Some(Post {
                title: "Words that do Handstands".into(),
                uri: "http://hardmath123.github.io/ambigrams.html".into(),
                rank: 1,
                author: Some("hardmath123".into()),
                points: Some(82),
                comments: Some(14),
            })
        );
        assert_eq!(posts.next(), Some(Post {
            title: "Possible detection of a black hole with a mass that was thought to be impossible".into(),
            uri: "https://www.quantamagazine.org/possible-detection-of-a-black-hole-so-big-it-should-not-exist-20190828/".into(),
            rank: 2,
            author: Some("theafh".into()),
            points: Some(58),
            comments: Some(35),
        }));
        assert_eq!(
            posts.next(),
            Some(Post {
                title: "Lessons from Stripe".into(),
                uri: "https://markmcgranaghan.com/lessons-from-stripe".into(),
                rank: 3,
                author: Some("rspivak".into()),
                points: Some(110),
                comments: Some(30),
            })
        );
    }

    /// Hiring posts lack author/points/comments data
    #[test]
    fn fetch_hiring_post_12() {
        let posts = fetch_posts(12);
        assert_eq!(posts.len(), 12);

        assert_eq!(
            posts[11],
            Post {
                title: "Mimir (YC S15) Is Hiring a Product Designer to Help Us Improve CS Education".into(),
                uri: "https://hire.withgoogle.com/public/jobs/mimirhqcom/view/P_AAAAAADAACHKrbvKW9X25u".into(),
                rank: 12,
                author: None,
                points: None,
                comments: None,
            }
        );
    }

    #[test]
    fn fetch_more_than_1_page() {
        let posts = fetch_posts(74);

        let ranks: Vec<_> = posts.iter().map(|p| p.rank).collect();
        assert_eq!(ranks, (1..=74).collect::<Vec<_>>());

        assert_eq!(
            posts[72],
            Post {
                title: "Anthony Levandowski Charged with Theft of Trade Secrets".into(),
                uri: "https://www.nytimes.com/2019/08/27/technology/google-trade-secrets-levandowski.html".into(),
                rank: 73,
                author: Some("coloneltcb".into()),
                points: Some(440),
                comments: Some(339),
            }
        );
    }
}
