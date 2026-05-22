use twips::scramble::{random_scramble_for_event, Event};

/// Genera una secuencia oficial de mezcla basada en el estado aleatorio uniforme
pub fn generate_wca_scramble(event_code: &str) -> Result<String, String> {
    // Mapeamos el string del cliente a la nomenclatura estricta del enum Event de twips
    let event = match event_code {
        "222" => Event::Cube2x2x2Speedsolving,
        "333" => Event::Cube3x3x3Speedsolving,
        "444" => Event::Cube4x4x4Speedsolving,
        _ => return Err(format!("Evento '{}' no soportado o inválido.", event_code)),
    };

    // random_scramble_for_event procesa el estado y devuelve la mezcla óptima
    match random_scramble_for_event(event) {
        Ok(scramble) => Ok(scramble.to_string()),
        Err(_) => Err("Error al calcular el estado aleatorio del cubo.".to_string()),
    }
}
