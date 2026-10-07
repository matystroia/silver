use std::sync::LazyLock;

use color_eyre::Result;
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use nonzero_ext::nonzero;
use reqwest::{
    Url,
    header::{HeaderMap, HeaderValue},
};

static URL_BASE: LazyLock<Url> = LazyLock::new(|| Url::parse("https://letterboxd.com/").unwrap());

static LIMITER: LazyLock<DefaultDirectRateLimiter> =
    LazyLock::new(|| RateLimiter::direct(Quota::per_second(nonzero!(3u32))));

pub struct LetterboxdClient {
    client: reqwest::Client,
}

impl LetterboxdClient {
    pub fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            HeaderValue::from_static(
                "Mozilla/5.0 (X11; Linux x86_64; rv:152.0) Gecko/20100101 Firefox/152.0",
            ),
        );
        headers.insert(
            "Accept",
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
        );
        headers.insert(
            "Accept-Language",
            HeaderValue::from_static("en-US,en;q=0.5"),
        );
        headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("document"));
        headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("navigate"));
        headers.insert("Sec-Fetch-Site", HeaderValue::from_static("none"));
        headers.insert("Sec-Fetch-User", HeaderValue::from_static("?1"));

        Ok(Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()?,
        })
    }

    pub async fn movie_list(&self, username: &str, list: &str) -> Result<Vec<(String, u16)>> {
        let mut movies = Vec::new();

        for page in 1.. {
            let batch = self.movie_list_page(username, list, page).await?;
            if batch.is_empty() {
                break;
            }
            movies.extend(batch);
        }

        Ok(movies)
    }

    async fn movie_list_page(
        &self,
        username: &str,
        list: &str,
        page: u32,
    ) -> Result<Vec<(String, u16)>> {
        let url = URL_BASE
            .join(&format!("{username}/list/{list}/page/{page}/"))
            .unwrap();

        LIMITER.until_ready().await;
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let dom = tl::parse(&response, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        Ok(dom
            .query_selector(r#"div.react-component[data-component-class="LazyPoster"]"#)
            .unwrap()
            .map(|node| {
                let tag = node.get(parser).unwrap().as_tag().unwrap();
                let attrs = tag.attributes();

                let full_name = attrs
                    .get("data-item-name")
                    .flatten()
                    .map(|b| b.as_utf8_str())
                    .unwrap();

                let idx = full_name.rfind('(').unwrap();
                let name = html_escape::decode_html_entities(&full_name[..idx]).to_string();
                let year = full_name[idx + 1..]
                    .trim_end_matches(')')
                    .parse::<u16>()
                    .unwrap();

                (name, year)
            })
            .collect())
    }
}
