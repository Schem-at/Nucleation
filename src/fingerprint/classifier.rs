//! Rule-based functional-equivalence classifier. Maps an (already
//! rotation-canonicalized) [`BlockState`] to an optional [`Token`]; `None`
//! means "ignored" (air / decoration). Rulesets are data — loadable from RON.

use std::collections::BTreeSet;

use serde::Deserialize;
use smol_str::SmolStr;

use crate::block_state::BlockState;

pub type Token = SmolStr;

#[derive(Clone, Debug, Deserialize)]
pub enum Match {
    Names(BTreeSet<String>),
    Glob(String), // only leading/trailing `*` supported, e.g. "*_slab"
    Category(BpCategory),
    Prop(String, String), // property key == value
    All(Vec<Match>),
    Any(Vec<Match>),
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum BpCategory {
    Solid,
    Transparent,
}

impl Match {
    pub fn and(self, other: Match) -> Match {
        Match::All(vec![self, other])
    }

    pub fn matches(&self, b: &BlockState) -> bool {
        match self {
            Match::Names(set) => set.contains(b.get_name()),
            Match::Glob(p) => glob_match(p, b.get_name()),
            Match::Category(cat) => match_category(*cat, b.get_name()),
            Match::Prop(k, v) => b.get_property(k).map(|x| x.as_str()) == Some(v.as_str()),
            Match::All(ms) => ms.iter().all(|m| m.matches(b)),
            Match::Any(ms) => ms.iter().any(|m| m.matches(b)),
        }
    }
}

fn glob_match(pattern: &str, name: &str) -> bool {
    if let Some(suf) = pattern.strip_prefix('*') {
        name.ends_with(suf)
    } else if let Some(pre) = pattern.strip_suffix('*') {
        name.starts_with(pre)
    } else {
        pattern == name
    }
}

fn match_category(cat: BpCategory, name: &str) -> bool {
    match blockpedia::get_block(name) {
        Some(facts) => match cat {
            BpCategory::Transparent => facts.transparent,
            BpCategory::Solid => !facts.transparent,
        },
        None => false,
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Rule {
    pub matcher: Match,
    pub class: Token,
    pub keep_props: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Classifier {
    pub name: String,
    pub rules: Vec<Rule>,
}

#[derive(Debug)]
pub struct ClassifierError(pub String);

impl Classifier {
    /// First matching rule wins. `None` = ignored.
    pub fn tokenize(&self, b: &BlockState) -> Option<Token> {
        let rule = self.rules.iter().find(|r| r.matcher.matches(b))?;
        if rule.keep_props.is_empty() {
            return Some(rule.class.clone());
        }
        let mut token = rule.class.to_string();
        for key in &rule.keep_props {
            if let Some(v) = b.get_property(key) {
                token.push('|');
                token.push_str(key);
                token.push('=');
                token.push_str(v);
            }
        }
        Some(Token::from(token))
    }

    pub fn from_config(src: &str) -> Result<Self, ClassifierError> {
        ron::from_str(src).map_err(|e| ClassifierError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_state::BlockState;

    fn stone() -> BlockState {
        BlockState::new("minecraft:stone")
    }
    fn top_slab() -> BlockState {
        BlockState::new("minecraft:oak_slab").with_properties(vec![("type".into(), "top".into())])
    }

    #[test]
    fn first_match_wins_and_ignores_unmatched() {
        let c = Classifier {
            name: "t".into(),
            rules: vec![
                Rule {
                    matcher: Match::Glob("*_slab".into())
                        .and(Match::Prop("type".into(), "top".into())),
                    class: "transparent".into(),
                    keep_props: vec![],
                },
                Rule {
                    matcher: Match::Names(["minecraft:stone".into()].into_iter().collect()),
                    class: "solid".into(),
                    keep_props: vec![],
                },
            ],
        };
        assert_eq!(c.tokenize(&top_slab()).as_deref(), Some("transparent"));
        assert_eq!(c.tokenize(&stone()).as_deref(), Some("solid"));
        assert_eq!(c.tokenize(&BlockState::new("minecraft:air")), None);
    }

    #[test]
    fn keep_props_are_appended_to_token() {
        let c = Classifier {
            name: "t".into(),
            rules: vec![Rule {
                matcher: Match::Names(["minecraft:repeater".into()].into_iter().collect()),
                class: "repeater".into(),
                keep_props: vec!["facing".into()],
            }],
        };
        let r = BlockState::new("minecraft:repeater")
            .with_properties(vec![("facing".into(), "east".into())]);
        assert_eq!(c.tokenize(&r).as_deref(), Some("repeater|facing=east"));
    }

    #[test]
    fn loads_from_ron() {
        let src = r#"(name:"t", rules:[
            (matcher: Glob("*_slab"), class:"slab", keep_props:["type"]),
            (matcher: Names(["minecraft:stone"]), class:"solid", keep_props:[]),
        ])"#;
        let c = Classifier::from_config(src).expect("parse");
        assert_eq!(c.rules.len(), 2);
        assert_eq!(
            c.tokenize(&BlockState::new("minecraft:stone")).as_deref(),
            Some("solid")
        );
    }
}
