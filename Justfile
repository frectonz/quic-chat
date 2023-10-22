default:
  @just --choose

client:
  cargo run --bin client

server:
  cargo run --bin server
