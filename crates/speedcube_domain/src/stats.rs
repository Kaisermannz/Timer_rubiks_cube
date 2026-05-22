// /speedcube_pwa/crates/speedcube_domain/src/stats.rs

use crate::models::{Solve, Penalty};
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum AverageResult {
    Ok(i32), // Retorna el promedio en milisegundos
    Dnf,
    NotEnoughSolves,
}

/// Calcula el Average of 5 (Ao5) de un slice de resoluciones.
pub fn calculate_ao5(solves: &[Solve]) -> AverageResult {
    // El Ao5 estricto necesita exactamente 5 tiempos
    if solves.len() < 5 {
        return AverageResult::NotEnoughSolves;
    }

    // Tomamos solo los últimos 5 por si nos pasan un historial más largo
    let recent_solves = &solves[solves.len() - 5..];

    let mut times = Vec::with_capacity(5);
    let mut dnf_count = 0;

    // 1. Procesar tiempos y penalizaciones
    for solve in recent_solves {
        match solve.penalty {
            Penalty::Dnf => {
                dnf_count += 1;
                times.push(i32::MAX); // DNF es matemáticamente el peor tiempo posible
            }
            Penalty::PlusTwo => {
                times.push(solve.time_ms + 2000);
            }
            Penalty::None => {
                times.push(solve.time_ms);
            }
        }
    }

    // 2. Regla WCA: 2 o más DNF arruinan el promedio completo
    if dnf_count >= 2 {
        return AverageResult::Dnf;
    }

    // 3. Ordenamos de menor a mayor (sort_unstable es más rápido en Rust para primitivos)
    times.sort_unstable();

    // 4. Descartamos el mejor (índice 0) y el peor (índice 4). Sumamos los 3 del medio.
    let valid_times = &times[1..4];
    let sum: i32 = valid_times.iter().sum();

    // 5. Retornamos el promedio truncado
    AverageResult::Ok(sum / 3)
}

// Añade esto al final de /speedcube_pwa/crates/speedcube_domain/src/stats.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Penalty;

    // Helper para generar solves ficticios rápidamente
    fn mock_solve(time_ms: i32, penalty: Penalty) -> Solve {
        Solve {
            id: None,
            event_code: "333".to_string(),
            time_ms,
            penalty,
            scramble: "R U R'".to_string(),
        }
    }

    #[test]
    fn test_ao5_normal() {
        let solves = vec![
            mock_solve(12000, Penalty::None), // Mejor (se descarta)
            mock_solve(14000, Penalty::None), // Medio
            mock_solve(15000, Penalty::None), // Medio
            mock_solve(13000, Penalty::None), // Medio
            mock_solve(20000, Penalty::None), // Peor (se descarta)
        ];
        // Medios: 14000 + 15000 + 13000 = 42000 / 3 = 14000
        assert_eq!(calculate_ao5(&solves), AverageResult::Ok(14000));
    }

    #[test]
    fn test_ao5_with_one_dnf() {
        let solves = vec![
            mock_solve(12000, Penalty::None), // Mejor (se descarta)
            mock_solve(14000, Penalty::None),
            mock_solve(15000, Penalty::None),
            mock_solve(13000, Penalty::None),
            mock_solve(0, Penalty::Dnf),      // Peor por ser DNF (se descarta)
        ];
        assert_eq!(calculate_ao5(&solves), AverageResult::Ok(14000));
    }

    #[test]
    fn test_ao5_with_two_dnf() {
        let solves = vec![
            mock_solve(14000, Penalty::None),
            mock_solve(15000, Penalty::None),
            mock_solve(13000, Penalty::None),
            mock_solve(0, Penalty::Dnf),
            mock_solve(0, Penalty::Dnf),
        ];
        assert_eq!(calculate_ao5(&solves), AverageResult::Dnf);
    }
}
