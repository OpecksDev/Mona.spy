use super::subscription;
pub mod promotional_codes;

use super::persist;
use actix_web::error;
use parse_wiki_text::Node;
use serde::Serialize;
use serde_json::Value;
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
  get_cell_content(nodes).join("")
}

pub trait WikiResource:
  Sized + Serialize + serde::de::DeserializeOwned + std::fmt::Debug + Clone
{
  fn from(nodes: &Vec<Node>) -> Self;
  fn get_title() -> &'static str;
  fn difference(&self, other: &Self) -> Self;
  fn empty(&self) -> bool;
}

async fn get_wiki_resource<T: WikiResource>() -> Option<T> {
  persist::get::<T>().await
}

pub async fn update_wiki_resource<T: WikiResource>() -> Result<T> {
  let previous_resource = get_wiki_resource::<T>().await;

  let base_path = "https://genshin-impact.fandom.com/api.php";
  let query_string = [
    ("action", "query"),
    ("prop", "revisions"),
    ("titles", T::get_title()),
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

  let wiki_text_json = &serde_json::from_str::<Value>(res.as_str()).map_err(|_| WikiError)?
    ["query"]["pages"][0]["revisions"][0]["slots"]["main"]["content"];

  let wiki_text = match wiki_text_json {
    Value::String(string) => string
      .replace(r#"\n"#, "\n")
      .replace(r#"\""#, "\"")
      .replace(r#"\'"#, "\'")
      .replace(r#"\t"#, "\t"),
    _ => return Err(WikiError),
  };

  let result = create_configuration().parse(&wiki_text);
  let result: T = T::from(&result.nodes);
  persist::set(&result).await.map_err(|_| WikiError)?;

  wiki_resource_change_callback(previous_resource, &result).await;

  Ok(result)
}

async fn wiki_resource_change_callback<T: WikiResource>(previous: Option<T>, current: &T) {
  let difference = match previous {
    Some(previous) => current.difference(&previous),
    None => current.to_owned(),
  };

  if difference.empty() {
    return;
  }

  println!("Resource Updated, added {:?}", difference);
  match subscription::notify(&difference).await {
    Ok(_) => {}
    Err(err) => println!("{:?}", err),
  };
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
