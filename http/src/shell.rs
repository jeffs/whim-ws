#![allow(dead_code)]

use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct ShellID(pub u32);

#[derive(Debug, Serialize)]
pub struct CardID(pub u32);

#[derive(Debug, Serialize)]
pub enum Liveness {
    Static,
    Period(Duration),
}

// Metadata are normalized across card types
#[derive(Debug, Serialize)]
pub struct CardMeta {
    pub live: Liveness,
}

// Invocation of a shell command (ls, echo, etc.).
//
// Note that argv and environment variables may hold arbitrary values, not only
// strings.  OS-level environment variables may be heeded by the Whim server
// itself, but card environments are decoupled.
#[derive(Debug, Serialize)]
pub struct Command {
    pub name: String, // Client-side class or function name
    pub argv: Value,  // Argument Value
    pub env: HashMap<String, Value>,
    pub wd: PathBuf,
}

// Representation of a UI card for use in client/server communication, shell
// history, logging, etc.
//
// Note that the output is an arbitrary, strictly evaluated value, not a stream
// of bytes.  Live commands loaded in the client will make runtime
// calls to the server API to update the output in real time.
#[derive(Debug, Serialize)]
pub struct Card {
    pub id: CardID,
    pub command: Command,
    pub meta: CardMeta,
    pub output: Value,
}

#[derive(Debug, Serialize)]
pub struct Column {
    pub cards: Vec<CardID>,
}

#[derive(Debug, Serialize)]
pub struct Shell {
    pub id: ShellID,
    pub name: String,
    pub history: Vec<Card>,
    pub columns: Vec<Column>,
}
