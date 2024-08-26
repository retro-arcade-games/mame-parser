use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::error::Error;
use url::Url;

/// Extracts the data source URL from a given web page based on a matching string.
///
/// This function downloads the HTML of the provided URL, parses it to find an anchor (`<a>`) tag
/// that contains a matching substring in the href attribute and ends with either `.zip` or `.7z`.
/// It then constructs and returns the full URL of that data source.
///
/// # Parameters
/// - `url`: A string slice (`&str`) representing the URL of the page to parse. For example:
///   `https://www.progettosnaps.net/dats/MAME`.
/// - `matching`: A string slice (`&str`) representing the substring to search for in the href attribute of anchor tags. For example:
///   `download/?tipo=dat_mame&file=/dats/MAME/packs/MAME_Dats`.
///
/// # Returns
/// Returns a `Result<String, Box<dyn Error + Send + Sync>>`:
/// - On success: Contains the full URL of the matching data source.
/// - On failure: Contains an error if no matching source is found or if there was an issue with parsing.
///
/// # Errors
/// This function will return an error if:
/// - The HTML cannot be retrieved or parsed.
/// - No matching link is found.
/// - There is an issue constructing the final URL (e.g., missing scheme or host).
///
/// # Example
/// ```rust
/// use mame_parser::helpers::data_source_helper::get_data_source;
///
/// let url = "https://www.progettosnaps.net/dats/MAME";
/// let matching = "download/?tipo=dat_mame&file=/dats/MAME/packs/MAME_Dats";
/// let data_source = get_data_source(url, matching);
/// assert!(data_source.is_ok());
/// ```
///
pub(crate) fn get_data_source(
    url: &str,
    matching: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    // Download the HTML
    let client = Client::new();
    let response = client.get(url).send()?;
    let body = response.text()?;

    // Parse the HTML
    let document = Html::parse_document(&body);
    let selector = Selector::parse("a").unwrap();

    // Find the matching source
    let mut source: Option<String> = None;
    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if href.contains(matching) && (href.ends_with("zip") || href.ends_with("7z")) {
                source = Some(href.to_string());
            }
        }
    }

    // If a source was found, return it
    if let Some(mut source) = source {
        if !source.starts_with("http") {
            let url_obj = Url::parse(url)?;
            let base = format!("{}://{}", url_obj.scheme(), url_obj.host_str().unwrap());
            let slash = if !url_obj.path().ends_with('/') && !source.starts_with('/') {
                "/"
            } else {
                ""
            };
            source = format!("{}{}{}", base, slash, source);
        }
        Ok(source)
    } else {
        Err("No matching source found".into())
    }
}
