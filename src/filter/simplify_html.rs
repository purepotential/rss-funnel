use dom_smoothie::{DomSmoothie, DomSmoothieOptions, Config};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::Result;
use crate::{feed::Feed, ConfigError};

use super::{FeedFilter, FeedFilterConfig, FilterContext};

#[derive(
  JsonSchema, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash,
)]
/// Konfiguracja filtru upraszczającego HTML.
pub struct SimplifyHtmlConfig {
  /// Czy zachować style CSS
  #[serde(default = "default_preserve_styles")]
  pub preserve_styles: bool,
  
  /// Czy zachować skrypty JavaScript
  #[serde(default = "default_preserve_scripts")]
  pub preserve_scripts: bool,
  
  /// Czy zachować komentarze HTML
  #[serde(default = "default_preserve_comments")]
  pub preserve_comments: bool,
  
  /// Czy zachować atrybuty HTML
  #[serde(default = "default_preserve_attributes")]
  pub preserve_attributes: bool,
  
  /// Czy zachować formatowanie tekstu
  #[serde(default = "default_preserve_formatting")]
  pub preserve_formatting: bool,

  /// Minimalny score wymagany do uznania dokumentu za czytelny
  #[serde(default = "default_readable_min_score")]
  pub readable_min_score: f32,
}

fn default_preserve_styles() -> bool { false }
fn default_preserve_scripts() -> bool { false }
fn default_preserve_comments() -> bool { false }
fn default_preserve_attributes() -> bool { true }
fn default_preserve_formatting() -> bool { true }
fn default_readable_min_score() -> f32 { 20.0 }

pub struct SimplifyHtmlFilter {
  config: SimplifyHtmlConfig,
}

#[async_trait::async_trait]
impl FeedFilterConfig for SimplifyHtmlConfig {
  type Filter = SimplifyHtmlFilter;

  async fn build(self) -> Result<Self::Filter, ConfigError> {
    Ok(SimplifyHtmlFilter { config: self })
  }
}

#[async_trait::async_trait]
impl FeedFilter for SimplifyHtmlFilter {
  async fn run(
    &self,
    _ctx: &mut FilterContext,
    mut feed: Feed,
  ) -> Result<Feed> {
    let mut posts = feed.take_posts();
    let options = DomSmoothieOptions {
      preserve_styles: self.config.preserve_styles,
      preserve_scripts: self.config.preserve_scripts,
      preserve_comments: self.config.preserve_comments,
      preserve_attributes: self.config.preserve_attributes,
      preserve_formatting: self.config.preserve_formatting,
    };

    let config = Config {
      keep_classes: false,
      classes_to_preserve: vec![],
      max_elements_to_parse: 0,
      disable_json_ld: true,
      n_top_candidates: 5,
      char_threshold: 500,
      readable_min_score: self.config.readable_min_score,
      readable_min_content_length: 140,
      candidate_select_mode: dom_smoothie::CandidateSelectMode::DomSmoothie,
      text_mode: dom_smoothie::TextMode::Formatted,
    };

    for post in &mut posts {
      let link = post.link().unwrap_or("").to_string();
      post.modify_bodies(|body| {
        if let Some(simplified) = simplify(body, &link, &options, &config) {
          *body = simplified;
        }
      });
    }

    feed.set_posts(posts);
    Ok(feed)
  }

  fn cache_granularity(&self) -> super::CacheGranularity {
    super::CacheGranularity::FeedAndPost
  }
}

pub(super) fn simplify(text: &str, url: &str, options: &DomSmoothieOptions, config: &Config) -> Option<String> {
  let url = Url::parse(url).ok()?;
  let mut smoothie = DomSmoothie::new_with_config(options, config);
  smoothie.process(text).ok()
}
