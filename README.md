# Шаблон для сервера на rust

## Настройка

Включение pre-commit `pre-commit install`

Локальный запуск `pre-commit run --verbose --all-files`

Для корректной сборки sqlx в gitlab необходимо выполнить `cargo sqlx prepare`

## Запуск

Для запуска с tracing `RUST_LOG=info cargo run`