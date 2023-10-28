default:
  @just --choose

bench:
  hyperfine -i 'seq -w 0 99 | parallel cargo run --release --bin client post'

bench-via-go:
  cd client-go; hyperfine -i 'seq -w 0 99 | parallel go run . post'

client:
  cargo run --release --bin client

server:
  cargo run --release --bin server
