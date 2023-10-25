default:
  @just --choose

bench:
  hyperfine -i 'seq -w 0 99 | parallel cargo run --bin client post'

client:
  cargo run --bin client

server:
  cargo run --bin server
