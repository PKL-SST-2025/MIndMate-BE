# MindMate Backend

Backend untuk aplikasi MindMate, dibuat dengan Rust dan Axum menggunakan SQLite sebagai database.

## Branch: feature/auth
Fitur autentikasi (register dan login) dengan JWT.

### Prasyarat
1. Instal Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Instal Diesel CLI: `cargo install diesel_cli --no-default-features --features sqlite`
3. Instal SQLite:
   ```bash
   # Windows: Unduh dari https://sqlite.org/download.html


Buat file database: mkdir data && touch data/mindmate.db

Cara Menjalankan

Buat file .env:DATABASE_URL=sqlite://data/mindmate.db
JWT_SECRET=rahasia_jwt_anda_ubah_ini


Jalankan migrasi: diesel migration run
Jalankan server: cargo run

Endpoint

POST /auth/register: Mendaftar pengguna baru.curl -X POST http://127.0.0.1:8080/auth/register -H "Content-Type: application/json" -d '{"username":"testuser","email":"test@example.com","password":"password123"}'


POST /auth/login: Login dan mendapatkan token JWT.curl -X POST http://127.0.0.1:8080/auth/login -H "Content-Type: application/json" -d '{"email":"test@example.com","password":"password123"}'



Struktur Direktori
MindMate-BE/
├── src/
│   ├── api/
│   │   ├── auth_handler.rs
│   ├── service/
│   │   ├── auth_service.rs
│   ├── models/
│   │   ├── auth.rs
│   │   ├── user.rs
│   ├── db/
│   │   ├── pool.rs
│   │   ├── user_query.rs
│   ├── path/
│   │   ├── auth_path.rs
│   ├── errors/
│   │   ├── app_error.rs
│   ├── utils/
│   │   ├── jwt.rs
│   ├── config/
│   │   ├── app_config.rs
│   ├── main.rs
│   ├── lib.rs
│   ├── schema.rs
├── migrations/
├── data/
│   ├── mindmate.db
├── .env
├── .gitignore
├── Cargo.toml
├── README.md


