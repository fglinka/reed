use clap::crate_version;
use model::LibraryEntryMeta;
use quick_error::quick_error;
use reqwest::blocking::ClientBuilder;
use reqwest::StatusCode;

quick_error! {
    #[derive(Debug)]
    pub enum FetchBibError {
        /// Indicates that no match was found online for a query term
        NoMatch {
            display(self_) -> ("No data was found for the article.")
        }
        NetworkError(err: reqwest::Error) {
            display(self_) -> ("Network error: {}", err)
            from()
        }
        RequestFailed(status: StatusCode) {
            display(self_) -> ("Request failed: {}", status)
        }
    }
}

/// A struct used to deserialize a reply received by the crossref api endpoint
#[derive(Debug, Clone, Deserialize)]
struct CrossrefResponse {
    status: String,
    message: Option<CrossrefMessage>,
}

///A struct used to deserialize the message part of a crossref reply
#[derive(Debug, Clone, Deserialize)]
struct CrossrefMessage {
    title: Vec<String>,
    #[serde(rename = "published-print")]
    published_print: CrossrefDate,
    #[serde(rename = "type")]
    doctype: String,
}

/// A struct used to deserialize author names obtained from Crossref
#[derive(Debug, Clone, Deserialize)]
struct CrossrefAuthor {
    #[serde(rename = "given")]
    given_name: String,
    #[serde(rename = "family")]
    family_name: String,
}

/// A struct used to deserialize dates obtained from crossref replies
#[derive(Debug, Clone, Deserialize)]
struct CrossrefDate {
    date_parts: Vec<Vec<i32>>,
}

fn fetch_doi_metadata(doi: &str) -> Result<LibraryEntryMeta, FetchBibError> {
    let client = ClientBuilder::new()
        // Introduce ourselves to the crossref API as described in https://github.com/CrossRef/rest-api-doc
        .user_agent(format!(
            "reed/{} (https://github.com/fglinka/reed; mailto:devglinka@posteo.eu) using reqwest",
            crate_version!()
        ))
        .build()?;
    let url = format!("https://api.crossref.org/works/{}", doi);
    let response = client.get(&url).send()?;
    if !response.status().is_success() {
        return Err(FetchBibError::RequestFailed(response.status()));
    }
    let json: CrossrefResponse = response.json()?;
}
