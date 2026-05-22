# ========================================================
# ETAPA 1: Planificador (cargo-chef)
# ========================================================
FROM rust:1.75-slim as planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ========================================================
# ETAPA 2: Caché de dependencias
# ========================================================
FROM rust:1.75-slim as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
# Esto compila y cachea las dependencias. Solo se repite si cambias el Cargo.toml
RUN cargo chef cook --release --recipe-path recipe.json

# ========================================================
# ETAPA 3: Compilación del binario
# ========================================================
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
# Copiamos la caché de la etapa anterior
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
# Compilamos el proyecto en modo ultra optimizado
RUN cargo build --release -p speedcube_api

# ========================================================
# ETAPA 4: Entorno de Producción (Ultra ligero)
# ========================================================
FROM debian:bookworm-slim as runtime
WORKDIR /app

# Instalamos dependencias mínimas para SQLite
RUN apt-get update && apt-get install -y libsqlite3-0 && rm -rf /var/lib/apt/lists/*

# Copiamos el ejecutable desde el builder
COPY --from=builder /app/target/release/speedcube_api /app/speedcube_api

# Copiamos los archivos estáticos (HTML, CSS, JS, manifest, iconos)
COPY --from=builder /app/crates/speedcube_api/static /app/crates/speedcube_api/static

# Exponemos el puerto de Axum
EXPOSE 3000

# Comando de arranque
CMD ["./speedcube_api"]
