use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufReader},
    path::Path,
};

use ariadne::{Report, ReportKind};
use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::{
    lang_errors::{LangError, LangMessage},
    spans::Span,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ElementRules {
    #[serde(default = "truev")]
    pub allow_generic_end: bool,
    #[serde(default = "falsev", alias = "parse_content_raw")]
    pub parse_raw: bool,
    #[serde(default = "truev", alias = "allow_xml_construction")]
    pub allow_xml: bool,
}
#[inline(always)]
const fn truev() -> bool {
    true
}

#[inline(always)]
const fn falsev() -> bool {
    false
}
#[derive(Debug, Serialize, Deserialize, From)]
pub struct ElementSchema(pub HashMap<String, ElementRules>);

impl ElementSchema {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn get_rule(&self, rule: impl AsRef<str>) -> Option<&ElementRules> {
        return self.0.get(rule.as_ref());
    }
    pub fn has_element(&self, name: impl AsRef<str>) -> bool {
        self.0.contains_key(name.as_ref())
    }
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, LangError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let schema: Self = serde_json::from_reader(reader)?;
        Ok(schema)
    }
}
