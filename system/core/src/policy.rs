use crate::tagged_dense_bitfield;
use crate::dense_soa_generic;
use crate::db;

tagged_dense_bitfield!(
    Action u16
    ID = 0xf0,
    START_TASK = 0x01,
    ACCESS_DEVICE = 0x02,
    WRITE_TO = 0x04,
);

pub struct PolicyRule {
    pub subject: db::Handle,
    pub allowed: Action
}

/// Policy engine that holds all rules and evaluates them. (zero sized)
pub struct PolicyEngine;
impl PolicyEngine {
    pub fn new() -> Self {
        Self{}
    }
    pub fn add_rule(db: &mut db::Database, rule: PolicyRule) {
        db.policy_rule.push(rule);
    }
    pub fn check(db: &db::Database, subject: db::Handle, what: Action) -> bool {
        for i in 0..db.policy_rule.len() {
            let r = db.policy_rule.get(i).unwrap();
            if r.subject == subject {
                return r.allowed.contains(what);
            }
        }
        false
    }
}
