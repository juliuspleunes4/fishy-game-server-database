docker build -t ghcr.io/flawake/fishy-game-backend-rust:latest .
docker run --env-file backend/.env -it ghcr.io/flawake/fishy-game-backend-rust:latest
