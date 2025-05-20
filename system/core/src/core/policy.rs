#[derive(Debug, PartialEq)]
pub enum Action {
    StartTask,
    AccessDevice(String),
    WriteTo(String),
    SendMessageTo(String),
    ReceiveMessage,
}

#[derive(Debug)]
pub struct PolicyRule {
    pub subject: String,
    pub action: Action,
    pub allow: bool,
}

pub struct PolicyEngine {
    rules: Vec<PolicyRule>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self { rules: vec![] }
    }

    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
    }

    pub fn check(&self, subject: &str, action: &Action) -> bool {
        for rule in &self.rules {
            if &rule.subject == subject && &rule.action == action {
                return rule.allow;
            }
        }
        false
    }
}
