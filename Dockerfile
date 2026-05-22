# ========================================================
# ETAPA 1: Compilación Directa (A prueba de fallos)
# ========================================================
# Usamos la última versión oficial de Rust para evitar incompatibilidades
FROM rust:latest AS builder
WORKDIR /app

# Copiamos todo tu código fuente al contenedor
COPY . .

# Compilamos el binario directamente en modo release
RUN cargo build --release -p speedcube_api

# ========================================================
# ETAPA 2: Entorno de Producción (Ultra ligero)
# ========================================================
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Instalamos dependencias mínimas para la base de datos SQLite
RUN apt-get update && apt-get install -y libsqlite3-0 && rm -rf /var/lib/apt/lists/*

# Copiamos el ejecutable que acabamos de crear en la Etapa 1
COPY --from=builder /app/target/release/speedcube_api /app/speedcube_api

# Copiamos los archivos visuales (HTML, CSS, JS, íconos)
COPY --from=builder /app/crates/speedcube_api/static /app/crates/speedcube_api/static

EXPOSE 3000

CMD ["./speedcube_api"]
