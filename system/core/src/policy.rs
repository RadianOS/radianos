use core::str;

use crate::db;
use crate::dense_bitfield;
use crate::policy;
use crate::tagged_dense_bitfield;

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

#[derive(Default, Clone, Copy, Debug)]
pub struct PasswordHash {
    data: [u64; 8], //512-bit, AES-512?
}

#[derive(Default, Debug)]
pub struct User {
    name: [u8; 16],
    pass_hash: PasswordHash,
}
impl User {
    pub fn get_name(&self) -> &str {
        let len = self
            .name
            .iter()
            .enumerate()
            .find(|&(_, c)| *c == 0)
            .map(|(i, _)| i)
            .unwrap_or(self.name.len());
        unsafe { str::from_raw_parts(self.name.as_ptr(), len) }
    }
}

#[derive(Default, Debug)]
pub struct Group {
    name: [u8; 16],
}
impl Group {
    pub fn get_name(&self) -> &str {
        let len = self
            .name
            .iter()
            .enumerate()
            .find(|&(_, c)| *c == 0)
            .map(|(i, _)| i)
            .unwrap_or(self.name.len());
        unsafe { str::from_raw_parts(self.name.as_ptr(), len) }
    }
}

/// Policy engine that holds all rules and evaluates them. (zero sized)
pub struct Manager;
impl Manager {
    pub fn init(db: &mut db::Database) {
        Self::new_user(db, "admin");
        Self::new_group(db, "admin");
    }
    pub fn new_user(db: &mut db::Database, name: &str) -> db::ObjectHandle {
        let mut obj = User::default();
        for (i, b) in name.bytes().enumerate() {
            obj.name[i] = b;
        }
        db.users.push(obj);
        db::ObjectHandle::new::<{db::ObjectHandle::USER}>((db.users.len() - 1) as u16)
    }
    pub fn new_group(db: &mut db::Database, name: &str) -> db::ObjectHandle {
        let mut obj = Group::default();
        for (i, b) in name.bytes().enumerate() {
            obj.name[i] = b;
        }
        db.groups.push(obj);
        db::ObjectHandle::new::<{db::ObjectHandle::GROUP}>((db.groups.len() - 1) as u16)
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
    pub fn check_capability(
        db: &db::Database,
        subject: db::ObjectHandle,
        what: Capability,
    ) -> bool {
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
    pub fn get_user<'a>(db: &'a db::Database, id: db::ObjectHandle) -> &'a User {
        db.users.get(id.get_id() as usize).unwrap()
    }
    pub fn get_user_mut<'a>(db: &'a mut db::Database, id: db::ObjectHandle) -> &'a mut User {
        db.users.get_mut(id.get_id() as usize).unwrap()
    }
    pub fn for_each_user<F: FnMut(&User)>(db: &db::Database, mut f: F) {
        for i in 0..db.users.len() {
            f(&db.users[i]);
        }
    }
    pub fn for_each_group<F: FnMut(&Group)>(db: &db::Database, mut f: F) {
        for i in 0..db.groups.len() {
            f(&db.groups[i]);
        }
    }
}
