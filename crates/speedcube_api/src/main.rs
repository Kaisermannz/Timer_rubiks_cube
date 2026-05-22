// /speedcube_pwa/crates/speedcube_api/src/main.rs
// En /speedcube_pwa/crates/speedcube_api/src/main.rs (arriba)
use axum::{
    extract::{Path, State, Json},
    routing::{get, post, delete},
    Router,
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

use speedcube_domain::models::Penalty;
use speedcube_domain::wca_scramble::generate_wca_scramble;
use speedcube_domain::models::Solve;
use speedcube_domain::stats::{calculate_ao5, AverageResult}; // <-- Importamos la matemática
use speedcube_db::{establish_connection, initialize_db, insert_solve, get_recent_solves}; // <-- Importamos la consulta

#[tokio::main]
async fn main() {
    println!("--- Iniciando Servidor Speedcubing PWA ---");

    // 1. Preparar la Base de Datos
    let db_url = "sqlite://speedcube_data.db";
    let pool = establish_connection(db_url).await.expect("Error al conectar BD");
    initialize_db(&pool).await.expect("Error al inicializar tablas");

    // 2. Definir las rutas (Endpoints) y compartir la conexión SQLite (State)
    let app = Router::new()
            .route("/scramble/:event", get(get_scramble_handler))
            .route("/solve", post(submit_solve_handler))
            .route("/solve/:id", delete(delete_solve_handler))
            .route("/stats/:event/ao5", get(get_ao5_handler))
            .route("/history/:event", get(get_history_handler))
            .fallback_service(ServeDir::new("crates/speedcube_api/static"))
            .with_state(pool);

    // 3. Levantar el servidor en el puerto 3000
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("🚀 API escuchando en http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

// ==========================================
// CONTROLADORES (HANDLERS)
// ==========================================

/// GET /scramble/333
async fn get_scramble_handler(Path(event): Path<String>) -> Result<String, (StatusCode, String)> {
    match generate_wca_scramble(&event) {
        Ok(scramble) => Ok(scramble),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

/// POST /solve
async fn submit_solve_handler(
    State(pool): State<SqlitePool>, // Axum extrae la BD mágicamente del estado
    Json(solve): Json<Solve>,       // Axum parsea el JSON a tu struct Solve
) -> Result<impl IntoResponse, (StatusCode, String)> {

    match insert_solve(&pool, &solve).await {
        Ok(id) => Ok((StatusCode::CREATED, format!("Guardado exitoso. ID: {}", id))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// GET /stats/333/ao5
async fn get_ao5_handler(
    Path(event): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {

    // 1. Pedimos los últimos 5 tiempos a SQLite
    match get_recent_solves(&pool, &event, 5).await {
        Ok(solves) => {
            // 2. Pasamos los tiempos al cálculo del dominio
            let result = calculate_ao5(&solves);

            // 3. Formateamos la respuesta según las reglas de la WCA
            match result {
                            AverageResult::Ok(avg) => {
                                let seconds = avg as f64 / 1000.0; // Convertimos a segundos
                                Ok((StatusCode::OK, format!("Ao5: {:.2} s", seconds)))
                            },
                            AverageResult::Dnf => {
                                Ok((StatusCode::OK, "Ao5: DNF".to_string()))
                },
                AverageResult::NotEnoughSolves => {
                    Ok((StatusCode::BAD_REQUEST, "No hay suficientes tiempos (se requieren 5).".to_string()))
                }
            }
        },
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// GET /history/333
async fn get_history_handler(
    Path(event): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {

    match get_recent_solves(&pool, &event, 50).await {
        Ok(solves) => {
            let mut html = String::new();
            let total = solves.len();

            for (i, solve) in solves.iter().enumerate() {
                let time_sec = solve.time_ms as f64 / 1000.0;
                let solve_id = solve.id.unwrap_or(0); // Obtenemos el ID de la fila

                let (display_time, color_class, penalty_str) = match solve.penalty {
                    Penalty::None => (format!("{:.2}", time_sec), "text-zinc-200", "None"),
                    Penalty::PlusTwo => (
                        format!("{:.2} (+2)", time_sec + 2.0),
                        "text-yellow-500",
                        "PlusTwo",
                    ),
                    Penalty::Dnf => ("DNF".to_string(), "text-red-500", "DNF"),
                };

                let escaped_scramble = solve
                    .scramble
                    .replace('&', "&amp;")
                    .replace('"', "&quot;");

                html.push_str(&format!(
                    r#"<tr class="border-b border-zinc-800/50 hover:bg-zinc-800/30 transition-colors duration-150 cursor-pointer"
                            data-id="{}"
                            data-time="{}"
                            data-penalty="{}"
                            data-scramble="{}"
                            data-num="{}">
                        <td class="py-3 px-4 text-zinc-500 font-mono text-sm">#{}</td>
                        <td class="py-3 px-4 font-mono font-bold {}">{}</td>
                        <td class="py-3 px-4 text-xs font-mono text-zinc-600 truncate max-w-[180px]">{}</td>
                        <td class="py-3 px-4 text-right">
                            <button class="text-zinc-600 hover:text-red-400 font-mono text-xs transition-colors px-2 cursor-pointer"
                                    hx-delete="/solve/{}"
                                    hx-target="closest tr"
                                    hx-swap="outerHTML">
                                ✕
                            </button>
                        </td>
                    </tr>"#,
                    solve_id,
                    solve.time_ms,
                    penalty_str,
                    escaped_scramble,
                    total - i,
                    total - i,
                    color_class,
                    display_time,
                    solve.scramble,
                    solve_id
                ));
            }
            Ok(axum::response::Html(html))
        },
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// DELETE /solve/:id
async fn delete_solve_handler(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {

    match speedcube_db::delete_solve(&pool, id).await {
        Ok(_) => {
            // TRUCO DE MAGIA HTMX: Retornamos éxito (HTTP 200) con el header HX-Trigger.
            // Esto le avisa al navegador que se eliminó el tiempo, lo que obliga al
            // contenedor del Ao5 a volver a pedir sus estadísticas automáticamente.
            let headers = [("HX-Trigger", "solve-saved")];
            Ok((StatusCode::OK, headers, "")) // Cuerpo vacío porque el elemento desaparece del DOM
        },
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
