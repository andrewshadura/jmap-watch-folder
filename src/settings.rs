use jmap_client::email::query::Filter;
use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Serialize, Deserialize, Debug)]
pub struct WatchFilter {
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default = "default_seen_keyword")]
    pub seen_keyword: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: Option<String>,
    pub has_attachment: Option<bool>,
}

fn default_seen_keyword() -> String {
    "$jmap-watch-folder-seen".into()
}

impl Default for WatchFilter {
    fn default() -> Self {
        Self {
            keywords: vec![],
            seen_keyword: default_seen_keyword(),
            from: None,
            to: None,
            cc: None,
            bcc: None,
            subject: None,
            has_attachment: None,
        }
    }
}

impl WatchFilter {
    /// Parse the filter definition into a vector of `Filter`s
    pub fn as_filters(&self) -> Vec<Filter> {
        let mut filters = vec![];
        if !self.keywords.is_empty() {
            filters.extend(
                self.keywords
                    .iter()
                    .map(|keyword| Filter::has_keyword(keyword.clone())),
            );
        }

        if !self.seen_keyword.is_empty() {
            filters.push(Filter::not_keyword(&self.seen_keyword))
        }

        if let Some(from) = self.from.as_deref() {
            if !from.is_empty() {
                filters.push(Filter::from(from));
            }
        }

        if let Some(to) = self.to.as_deref() {
            if !to.is_empty() {
                filters.push(Filter::to(to));
            }
        }

        if let Some(cc) = self.cc.as_deref() {
            if !cc.is_empty() {
                filters.push(Filter::cc(cc));
            }
        }

        if let Some(bcc) = self.bcc.as_deref() {
            if !bcc.is_empty() {
                filters.push(Filter::bcc(bcc));
            }
        }

        if let Some(subject) = self.subject.as_deref() {
            if !subject.is_empty() {
                filters.push(Filter::subject(subject));
            }
        }

        if let Some(has_attachment) = self.has_attachment {
            filters.push(Filter::has_attachment(has_attachment));
        }

        filters
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub username: Option<String>,
    pub server: Option<String>,
    pub mailbox: Option<String>,
    pub watch: Vec<WatchFilter>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            username: None,
            server: None,
            mailbox: None,
            watch: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let kw_filter = WatchFilter {
            keywords: vec!["foo".into(), "bar".into()],
            ..Default::default()
        };
        assert_eq!(
            format!("{:?}", kw_filter.as_filters()),
            r#"[HasKeyword { value: "foo" }, HasKeyword { value: "bar" }, NotKeyword { value: "$jmap-watch-folder-seen" }]"#
        );
    }

    #[test]
    fn test_from() {
        let from_filter = WatchFilter {
            from: Some("foo".into()),
            seen_keyword: "".into(),
            ..Default::default()
        };
        assert_eq!(
            format!("{:?}", from_filter.as_filters()),
            r#"[From { value: "foo" }]"#
        );
    }

    #[test]
    fn test_to() {
        let to_filter = WatchFilter {
            to: Some("foo".into()),
            seen_keyword: "".into(),
            ..Default::default()
        };
        assert_eq!(
            format!("{:?}", to_filter.as_filters()),
            r#"[To { value: "foo" }]"#
        );
    }

    #[test]
    fn test_cc() {
        let cc_filter = WatchFilter {
            cc: Some("foo".into()),
            seen_keyword: "".into(),
            ..Default::default()
        };
        assert_eq!(
            format!("{:?}", cc_filter.as_filters()),
            r#"[Cc { value: "foo" }]"#
        );
    }

    #[test]
    fn test_bcc() {
        let bcc_filter = WatchFilter {
            bcc: Some("foo".into()),
            seen_keyword: "".into(),
            ..Default::default()
        };
        assert_eq!(
            format!("{:?}", bcc_filter.as_filters()),
            r#"[Bcc { value: "foo" }]"#
        );
    }

    #[test]
    fn test_subject() {
        let subject_filter = WatchFilter {
            subject: Some("foo".into()),
            seen_keyword: "".into(),
            ..Default::default()
        };
        assert_eq!(
            format!("{:?}", subject_filter.as_filters()),
            r#"[Subject { value: "foo" }]"#
        );
    }
}
