
pub mod parse_date_string;
pub mod rrule;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::rrule::{RRule, RRuleSet};
#[wasm_bindgen]
pub struct JsRRule {
    rrule: RRule,
}

#[wasm_bindgen]
impl JsRRule {
    #[wasm_bindgen(constructor)]
    pub fn new(s: &str) -> JsRRule {
        JsRRule {
            rrule: RRule::from_str(s),
        }
    }
}

#[wasm_bindgen]
pub struct JsRRuleSet {
    rrule_set: RRuleSet,
}
#[wasm_bindgen]
impl JsRRuleSet {
    #[wasm_bindgen(constructor)]
    pub fn new(s: &str) -> JsRRuleSet {
        JsRRuleSet {
            rrule_set: RRuleSet::from_str(s).unwrap(),
        }
    }
    #[wasm_bindgen]
    pub fn add_rrule(&mut self, rrule: &str) {
        self.rrule_set.add_rrule(rrule)
    }

    pub fn tz(&mut self, tz: &str) {
        self.rrule_set.tz(tz);
    }

    pub fn all(&self, limit: u32) -> String {
        self.rrule_set
            .all(limit)
            .iter()
            .map(|d| d.timestamp_millis().to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}
