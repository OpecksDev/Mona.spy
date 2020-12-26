use super::{get_cell_content_as_string, WikiResource};
use parse_wiki_text::Node;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PromotionalCodes {
  codes: Vec<PromotionalCode>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
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

impl WikiResource for PromotionalCodes {
  fn empty(&self) -> bool {
    self.codes.is_empty()
  }

  fn difference(&self, other: &Self) -> Self {
    let mut difference: Vec<PromotionalCode> = Vec::new();

    for code in &self.codes {
      if !other.codes.contains(&code) {
        difference.push(code.to_owned())
      }
    }
    PromotionalCodes { codes: difference }
  }

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

    PromotionalCodes { codes: vec![] }
  }

  fn get_title() -> &'static str {
    "Promotional_Codes"
  }
}
