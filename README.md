# Condorcet

Scrutin de Condorcet randomisé, sans compte (façon Doodle). Rust + Axum + SQLite + Tailwind.

## Dev

```sh
tailwindcss -i static/input.css -o static/app.css --minify   # à relancer après modif des classes
cargo test
PORT=3999 cargo run
```

## Déploiement

```sh
kamal deploy   # build multi-arch croisé (cargo-zigbuild), image joel1di1/condorcet
```

nginx (certbot) sur l'EC2 proxifie `condorcet.joel1di1.com` vers kamal-proxy (127.0.0.1:3001).
La base SQLite est dans le volume Docker `condorcet_data`.
