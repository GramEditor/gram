use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::messages::buffer::Location;

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TaskContextForLocation {
    pub project_id: u64,
    pub location: Location,
    pub task_variables: HashMap<String, String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TaskContext {
    pub cwd: Option<String>,
    pub task_variables: HashMap<String, String>,
    pub project_env: HashMap<String, String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ShellWithArguments {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Shell {
    System(System),
    Program(String),
    WithArguments(ShellWithArguments),
}
impl Default for Shell {
    fn default() -> Self {
        Self::System(Default::default())
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct System {}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum RevealStrategy {
    #[default]
    RevealAlways,
    RevealNever,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum HideStrategy {
    #[default]
    HideAlways,
    HideNever,
    HideOnSuccess,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SpawnInTerminal {
    pub label: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDirectoryEnvironment {
    pub project_id: u64,
    pub shell: Shell,
    pub directory: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DirectoryEnvironment {
    pub environment: HashMap<String, String>,
}
