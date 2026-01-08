# Используем официальный образ Rust
FROM rust:1.83 as builder

# Устанавливаем рабочую директорию внутри контейнера
WORKDIR /usr/src/app

# Устанавливаем зависимости
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Копируем файлы проекта
COPY Cargo.toml ./
COPY src ./src

# Собираем проект
RUN cargo build --release

# Финальный образ на основе Debian
FROM debian:bookworm-slim

# Устанавливаем зависимости для работы бинарника
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Копируем бинарник из этапа сборки
COPY --from=builder /usr/src/app/target/release/telegram_bot /usr/local/bin/telegram_bot

# Запускаем бота
CMD ["/usr/local/bin/telegram_bot"]