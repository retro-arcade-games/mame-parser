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

/// Extracts the file name from a given URL.
///
/// This function takes a URL string and extracts the last part of the path, then further processes it to obtain the file name
/// if it is part of a query parameter. The function is useful for URLs that include file names either at the end of the path
/// or as part of a query string.
///
/// # Parameters
/// - `url`: A string slice (`&str`) representing the URL from which to extract the file name. For example:
///   `https://example.com/download?file=my_document.pdf`.
///
/// # Returns
/// Returns a `String` containing the extracted file name:
/// - On success: The extracted file name (e.g., `"my_document.pdf"`).
/// - If the URL does not have a valid structure or does not contain a recognizable file name, an empty string is returned.
///
pub(crate) fn get_file_name_from_url(url: &str) -> String {
    let last_param = url.split('/').last().unwrap_or("");
    let file_name = last_param.split('=').last().unwrap_or("");
    file_name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_get_data_source_valid() -> Result<(), Box<dyn Error + Send + Sync>> {
        let url = "https://www.progettosnaps.net/languages";
        let matching = "download";

        let result = get_data_source(url, matching);
        assert!(result.is_ok());
        let source_url = result.unwrap();
        assert!(source_url.contains(matching));
        assert!(source_url.ends_with("zip") || source_url.ends_with("7z"));

        Ok(())
    }

    #[test]
    fn test_get_data_source_no_match() -> Result<(), Box<dyn Error + Send + Sync>> {
        let url = "https://www.progettosnaps.net/languages";
        let matching = "nonexistentfile";

        let result = get_data_source(url, matching);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "No matching source found");

        Ok(())
    }

    #[test]
    fn test_get_file_name_basic() {
        let url = "https://example.com/downloads/file.zip";
        let expected = "file.zip";
        let result = get_file_name_from_url(url);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_file_name_with_query_params() {
        let url = "https://example.com/download?file=file.zip";
        let expected = "file.zip";
        let result = get_file_name_from_url(url);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_file_name_no_slash() {
        let url = "https://example.com/file.zip";
        let expected = "file.zip";
        let result = get_file_name_from_url(url);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_file_name_empty_string() {
        let url = "";
        let expected = "";
        let result = get_file_name_from_url(url);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_file_name_no_file_name() {
        let url = "https://example.com/downloads/";
        let expected = "";
        let result = get_file_name_from_url(url);
        assert_eq!(result, expected);
    }
}
