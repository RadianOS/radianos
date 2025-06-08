// // Define actions that subjects might want to perform.
// #[derive(Debug)]
// pub enum Action {
//     StartTask,
//     AccessDevice(String), // Accessing a named device
//     WriteTo(String),      // Writing to a specific target
// }

// // Defines a rule mapping subject to allowed/denied action.
// #[derive(Debug)]
// pub struct PolicyRule {
//     subject: String,  // Actor or process name
//     action: Action,   // Attempted action
//     allow: bool,      // Whether the action is permitted
// }

// // Policy engine that holds all rules and evaluates them.
// pub struct PolicyEngine {
//     rules: Vec<PolicyRule>,
// }

use crate::tagged_dense_bitfield;

tagged_dense_bitfield!(
    Action u16
    ID = 0xf0,
    START_TASK = 0x01,
    ACCESS_DEVICE = 0x02,
    WRITE_TO = 0x04,
);


