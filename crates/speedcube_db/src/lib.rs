// /speedcube_pwa/crates/speedcube_db/src/lib.rs
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, SqlitePool, Row}; // <-- Fíjate que aquí agregamos 'Row'
use std::str::FromStr;
use speedcube_domain::models::{Solve, Penalty};

/// Crea y configura un pool de conexiones optimizado para SQLite
pub async fn establish_connection(db_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_url)?
        .create_if_missing(true)
        .pragma("journal_mode", "WAL")
        .pragma("synchronous", "NORMAL")
        .pragma("foreign_keys", "ON");

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

/// Crea la tabla de tiempos si no existe
pub async fn initialize_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS solves (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_code TEXT NOT NULL,
            time_ms INTEGER NOT NULL,
            penalty TEXT NOT NULL DEFAULT 'NONE',
            scramble TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Inserta un nuevo solve en la base de datos y retorna su ID generado
pub async fn insert_solve(pool: &SqlitePool, solve: &Solve) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO solves (event_code, time_ms, penalty, scramble)
        VALUES (?1, ?2, ?3, ?4)
        "#
    )
    .bind(&solve.event_code)
    .bind(solve.time_ms)
    .bind(solve.penalty.as_str())
    .bind(&solve.scramble)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}


/// Obtiene los últimos N tiempos para un evento específico, ordenados del más reciente al más antiguo.
pub async fn get_recent_solves(
    pool: &SqlitePool,
    event_code: &str,
    limit: u32
) -> Result<Vec<Solve>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, event_code, time_ms, penalty, scramble FROM solves WHERE event_code = ?1 ORDER BY created_at DESC LIMIT ?2"
    )
    .bind(event_code)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut solves = Vec::new();

    // Mapeamos manualmente las filas de la base de datos a nuestras estructuras de Rust
    for row in rows {
        let penalty_str: String = row.get("penalty");
        let penalty = match penalty_str.as_str() {
            "PLUS_TWO" => Penalty::PlusTwo,
            "DNF" => Penalty::Dnf,
            _ => Penalty::None,
        };

        solves.push(Solve {
            id: Some(row.get("id")),
            event_code: row.get("event_code"),
            time_ms: row.get("time_ms"),
            penalty,
            scramble: row.get("scramble"),
        });
    }

    Ok(solves)
}

pub async fn delete_solve(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM solves WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
