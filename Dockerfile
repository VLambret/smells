FROM ubuntu:latest

RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    pkg-config \
    libssl-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /app

# Copier les fichiers de configuration Rust (Cargo.toml et Cargo.lock) séparément
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch --locked

# Copier le reste du code source
COPY src ./src

# Compiler le projet
RUN cargo build --release

# Définir la commande par défaut pour exécuter les tests
CMD ["cargo", "test"]