use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::messages::buffer::Location;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TaskContextForLocation {
    pub project_id: u64,
    pub location: Location,
    pub task_variables: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TaskContext {
    pub cwd: Option<String>,
    pub task_variables: HashMap<String, String>,
    pub project_env: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct System {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum RevealStrategy {
    RevealAlways,
    RevealNever,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum HideStrategy {
    HideAlways,
    HideNever,
    HideOnSuccess,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SpawnInTerminal {
    pub label: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetDirectoryEnvironment {
    pub project_id: u64,
    pub shell: Shell,
    pub directory: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DirectoryEnvironment {
    pub environment: HashMap<String, String>,
}
