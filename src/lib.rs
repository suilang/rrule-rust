pub mod point_time;
pub mod rrule;
pub mod rrule_set;
mod constant;
use rrule::RRule;
use rrule_set::RRuleSet;

use wasm_bindgen::prelude::*;
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

    pub fn set_count(&mut self, count: u32) {
        self.rrule.count = count;
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

    pub fn set_dt_start(&mut self, str: &str) {
        self.rrule_set.set_dt_start(str)
    }

    pub fn set_count(&mut self, count: u32) {
        self.rrule_set.set_count(count);
    }

    pub fn set_until(&mut self, str: &str) {
        self.rrule_set.set_until(str);
    }

    pub fn between(&mut self, start: &str, end:&str){
        self.rrule_set.between(start, end);
    }

    pub fn all(&self) -> String {
        self.rrule_set
            .all()
            .iter()
            .map(|d| d.timestamp_millis().to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}
