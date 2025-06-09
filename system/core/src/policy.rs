use crate::dense_bitfield;
use crate::policy;
use crate::tagged_dense_bitfield;
use crate::db;

// Define system capabilities that can be granted to components or processes.
dense_bitfield!(
    Capability u16
    READ_FILESYSTEM = 0x01,
    WRITE_LOG = 0x02,
    SPAWN_TASK = 0x04,
    NETWORK_ACCESS = 0x08,
);
impl Capability {
    pub fn new() -> Self {
        Self(0)
    }
}

tagged_dense_bitfield!(
    Action u16
    ID = 0xf0,
    START_TASK = 0x01,
    ACCESS_DEVICE = 0x02,
    WRITE_TO = 0x04,
);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PolicyRuleHandle(pub u16);
#[derive(Default, Debug)]
pub struct PolicyRule {
    pub subject: db::ObjectHandle,
    pub allowed: Action,
    pub capabilities: Capability,
}

/// Policy engine that holds all rules and evaluates them. (zero sized)
pub struct PolicyEngine;

impl PolicyEngine {
    pub fn new() -> Self {
        Self {}
    }
    pub fn add_rule(db: &mut db::Database, rule: PolicyRule) -> PolicyRuleHandle {
        for i in 1..db.policy_rule.len() {
            let r = db.policy_rule.get_mut(i).unwrap();
            if r.subject == db::ObjectHandle::default() {
                *r = rule;
                return PolicyRuleHandle(i as u16);
            }
        }
        db.policy_rule.push(rule);
        PolicyRuleHandle((db.policy_rule.len() - 1) as u16)
    }
    pub fn remove_rule(db: &mut db::Database, handle: PolicyRuleHandle) {
        if let Some(r) = db.policy_rule.get_mut(handle.0 as usize) {
            *r = PolicyRule::default();
        }
    }
    pub fn check_action(db: &db::Database, subject: db::ObjectHandle, what: Action) -> bool {
        for i in 1..db.policy_rule.len() {
            let r = &db.policy_rule[i];
            if r.subject == subject {
                return r.allowed.contains(what);
            }
        }
        false
    }
    pub fn check_capability(db: &db::Database, subject: db::ObjectHandle, what: Capability) -> bool {
        for i in 1..db.policy_rule.len() {
            let r = &db.policy_rule[i];
            if r.subject == subject {
                return r.capabilities.contains(what);
            }
        }
        false
    }
    pub fn for_each_policy_rule<F: FnMut(&policy::PolicyRule)>(db: &db::Database, mut f: F) {
        for i in 1..db.policy_rule.len() {
            let r = &db.policy_rule[i];
            if r.subject != db::ObjectHandle::default() {
                f(r);
            }
        }
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}
