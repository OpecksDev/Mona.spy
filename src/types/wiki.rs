extern crate parse_wiki_text;

use actix_web::error;
use async_trait::async_trait;
use parse_wiki_text::Node;
use serde::{Deserialize, Serialize};
use std::fmt;

type Result<T> = std::result::Result<T, WikiError>;

#[derive(Debug)]
pub struct WikiError;

impl error::ResponseError for WikiError {}

impl fmt::Display for WikiError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Ocurred an Error during Wiki fetching")
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromotionalCodes {
  codes: Vec<PromotionalCode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromotionalCode {
  code: Option<String>,
  server: Option<String>,
  reward: Option<String>,
  discovered: Option<String>,
  expires: Option<String>,
}

impl PromotionalCode {
  fn new() -> PromotionalCode {
    PromotionalCode {
      code: None,
      server: None,
      reward: None,
      discovered: None,
      expires: None,
    }
  }
}

fn get_cell_content<'a>(nodes: &'a Vec<Node>) -> Vec<&'a str> {
  let mut content: Vec<&str> = Vec::new();
  for node in nodes {
    match node {
      Node::Text { value, .. } => {
        content.push(value);
      }
      Node::Link { text, .. } => {
        content.append(&mut get_cell_content(text));
      }
      _ => {}
    };
  }

  content
}

fn get_cell_content_as_string(nodes: &Vec<Node>) -> String {
  get_cell_content(nodes).join(" ")
}

impl WikiResource for PromotionalCodes {
  fn from(nodes: &Vec<Node>) -> Self {
    let mut after_available = false;

    for node in nodes {
      match node {
        Node::Text { value, .. } => {
          if value.contains("== Available ==") {
            after_available = true;
          }
          if value.contains("== Expired ==") {
            after_available = false;
          }
        }
        Node::Table { rows, .. } => {
          if !after_available {
            continue;
          }

          let mut it = rows.iter();

          let headers: Vec<String> = it
            .next()
            .unwrap()
            .cells
            .iter()
            .map(|x| get_cell_content_as_string(&x.content))
            .collect();

          let codes = it
            .map(|row| {
              let mut code = PromotionalCode::new();

              for (idx, cell) in row.cells.iter().enumerate() {
                let value = get_cell_content_as_string(&cell.content);

                match headers[idx].as_str() {
                  "Code" => code.code = Some(value),
                  "Server" => code.server = Some(value),
                  "Reward" => code.reward = Some(value),
                  "Discovered" => code.discovered = Some(value),
                  "Expires" => code.expires = Some(value),
                  _ => {}
                }
              }
              code
            })
            .collect::<Vec<_>>();
          return PromotionalCodes { codes };
        }
        _ => {}
      }
    }

    return PromotionalCodes { codes: vec![] };
  }

  fn get_title() -> &'static str {
    "Promotional_Codes"
  }
}

#[async_trait]
pub trait WikiResource: Sized {
  fn from(nodes: &Vec<Node>) -> Self;
  fn get_title() -> &'static str;

  async fn get_wiki_resource() -> Result<Self> {
    let base_path = "https://genshin-impact.fandom.com/api.php";
    let query_string = [
      ("action", "query"),
      ("prop", "revisions"),
      ("titles", Self::get_title()),
      ("rvslots", "*"),
      ("rvprop", "content"),
      ("formatversion", "2"),
      ("format", "json"),
    ];

    let client = reqwest::Client::new();
    let res = client
      .get(base_path)
      .query(&query_string)
      .send()
      .await
      .map_err(|_| WikiError)?
      .text()
      .await
      .map_err(|_| WikiError)?;

    let wiki_text = json::parse(res.as_str()).map_err(|_| WikiError)?["query"]["pages"][0]
      ["revisions"][0]["slots"]["main"]["content"]
      .dump()
      .replace(r#"\n"#, "\n")
      .replace(r#"\""#, "\"")
      .replace(r#"\'"#, "\'")
      .replace(r#"\t"#, "\t");

    let result = create_configuration().parse(&wiki_text);

    Ok(Self::from(&result.nodes))
  }
}

pub fn create_configuration() -> ::parse_wiki_text::Configuration {
  ::parse_wiki_text::Configuration::new(&::parse_wiki_text::ConfigurationSource {
    category_namespaces: &["category"],
    extension_tags: &[
      "activityfeed",
      "bloglist",
      "categorytree",
      "ce",
      "charinsert",
      "chem",
      "choose",
      "comments",
      "coordinates",
      "discussions",
      "display_line",
      "display_map",
      "display_point",
      "display_points",
      "distance",
      "dpl",
      "dynamicpagelist",
      "fb:follow",
      "fb:like",
      "fb:like-box",
      "fb:page",
      "fb:share-button",
      "finddestination",
      "forum",
      "gallery",
      "geocode",
      "geodistance",
      "helper",
      "imagemap",
      "imap",
      "indicator",
      "infobox",
      "inputbox",
      "jwplayer",
      "loggedin",
      "loggedout",
      "mainpage-endcolumn",
      "mainpage-leftcolumn-start",
      "mainpage-rightcolumn-start",
      "maplib",
      "mapsdoc",
      "math",
      "metadesc",
      "mp3",
      "nowiki",
      "pageby",
      "pagetools",
      "poem",
      "poll",
      "pre",
      "randomimage",
      "ref",
      "references",
      "rhtml",
      "rss",
      "section",
      "source",
      "spotify",
      "staff",
      "syntaxhighlight",
      "tabber",
      "templatedata",
      "timeline",
      "twitter",
      "verbatim",
      "vote",
      "widget",
      "youtube",
    ],
    file_namespaces: &["file", "image"],
    link_trail: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
    magic_words: &[
      "APPROVEDREVS",
      "EXPECTUNUSEDCATEGORY",
      "FORCECATEGORYGALLERY",
      "FORCETOC",
      "HIDDENCAT",
      "HIDEFROMDRILLDOWN",
      "INDEX",
      "NEWSECTIONLINK",
      "NOCACHE",
      "NOCATEGORYEXHIBITION",
      "NOCATEGORYGALLERY",
      "NOCC",
      "NOCONTENTCONVERT",
      "NODEFAULTLINKS",
      "NOEDITSECTION",
      "NOFACTBOX",
      "NOGALLERY",
      "NOINDEX",
      "NONEWSECTIONLINK",
      "NOSHAREDHELP",
      "NOTC",
      "NOTITLE",
      "NOTITLECONVERT",
      "NOTOC",
      "NOWYSIWYG",
      "SHOWFACTBOX",
      "SHOWINDRILLDOWN",
      "STATICREDIRECT",
      "TOC",
    ],
    protocols: &[
      "//",
      "bitcoin:",
      "ftp://",
      "ftps://",
      "geo:",
      "git://",
      "gopher://",
      "http://",
      "https://",
      "irc://",
      "ircs://",
      "magnet:",
      "mailto:",
      "mms://",
      "news:",
      "nntp://",
      "redis://",
      "sftp://",
      "sip:",
      "sips:",
      "sms:",
      "ssh://",
      "svn://",
      "tel:",
      "telnet://",
      "urn:",
      "worldwind://",
      "xmpp:",
    ],
    redirect_magic_words: &["REDIRECT"],
  })
}
