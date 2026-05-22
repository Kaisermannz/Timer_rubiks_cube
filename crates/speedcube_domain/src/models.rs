// /speedcube_pwa/crates/speedcube_domain/src/models.rs
use serde::{Deserialize, Serialize}; // <-- Importamos Serde

#[derive(Debug, Clone, Serialize, Deserialize)] // <-- Añadimos aquí
pub enum Penalty {
    None,
    PlusTwo,
    Dnf,
}

impl Penalty {
    pub fn as_str(&self) -> &'static str {
        match self {
            Penalty::None => "NONE",
            Penalty::PlusTwo => "PLUS_TWO",
            Penalty::Dnf => "DNF",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)] // <-- Y añadimos aquí
pub struct Solve {
    pub id: Option<i64>,
    pub event_code: String,
    pub time_ms: i32,
    pub penalty: Penalty,
    pub scramble: String,
}
