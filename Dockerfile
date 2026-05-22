# ========================================================
# ETAPA 1: Planificador (Usamos la imagen oficial que ya trae cargo-chef)
# ========================================================
FROM lukemathwalker/cargo-chef:latest-rust-1.78 AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ========================================================
# ETAPA 2: Caché de dependencias
# ========================================================
FROM lukemathwalker/cargo-chef:latest-rust-1.78 AS cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Esto compila y cachea las dependencias de tu proyecto
RUN cargo chef cook --release --recipe-path recipe.json

# ========================================================
# ETAPA 3: Compilación del binario
# ========================================================
FROM rust:1.78 AS builder
WORKDIR /app
COPY . .
# Copiamos la caché de la etapa anterior
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
# Compilamos tu proyecto en modo ultra optimizado
RUN cargo build --release -p speedcube_api

# ========================================================
# ETAPA 4: Entorno de Producción (Ultra ligero)
# ========================================================
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Instalamos dependencias mínimas para SQLite
RUN apt-get update && apt-get install -y libsqlite3-0 && rm -rf /var/lib/apt/lists/*

# Copiamos el ejecutable desde el builder
COPY --from=builder /app/target/release/speedcube_api /app/speedcube_api

# Copiamos los archivos estáticos
COPY --from=builder /app/crates/speedcube_api/static /app/crates/speedcube_api/static

EXPOSE 3000

CMD ["./speedcube_api"]
