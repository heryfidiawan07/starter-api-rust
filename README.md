# Starter API — Rust

REST API starter template menggunakan **Axum 0.7 + SQLx 0.8 + Tokio**, mendukung MySQL, PostgreSQL, dan SQLite.

## Fitur

- JWT Authentication (access + refresh token, disimpan di DB)
- Google & Facebook OAuth (raw HTTP via reqwest, tanpa SDK)
- Role & Permission (category → menu → action)
- `is_root` bypass semua permission
- Soft delete, UUID primary key
- Ganti password, lupa password (email link)
- Upload foto profil (max 2MB, JPEG/PNG/WebP)
- Multi-database: MySQL, PostgreSQL, SQLite
- Migration SQL otomatis saat startup

## Struktur Direktori

```
rust/
├── src/
│   ├── config.rs          # Konfigurasi dari .env
│   ├── errors.rs          # AppError enum → HTTP response
│   ├── response.rs        # Helper response (ok, created, paged)
│   ├── routes.rs          # Definisi semua route
│   ├── seeder.rs          # Seeder permission tree + root user
│   ├── main.rs            # Entry point
│   ├── models/            # Struct model + serializer
│   ├── services/          # Business logic (auth, user, role, permission)
│   ├── handlers/          # HTTP handler (auth, users, roles, permissions)
│   ├── middleware/        # AuthUser extractor (JWT Bearer)
│   └── utils/             # JWT, mail (lettre), upload
├── migrations/
│   └── 001_init.sql       # DDL tabel (dijalankan saat startup)
├── storage/photos/        # Foto yang di-upload
├── .env.example
├── Cargo.toml
├── Dockerfile
└── Makefile
```

## Persyaratan

- Rust 1.75+ (install via [rustup.rs](https://rustup.rs))
- Database: MySQL 8+ / PostgreSQL 14+ / SQLite

---

## Menjalankan Lokal (Development)

### 1. Install Rust

```bash
# Install rustup (semua platform)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart shell atau
source $HOME/.cargo/env
```

### 2. Setup environment

```bash
cp .env.example .env
# Edit .env sesuai konfigurasi lokal Anda
```

### 3. Jalankan server

```bash
# Development (build + run)
cargo run

# Dengan hot reload (install cargo-watch dulu)
cargo install cargo-watch
cargo watch -x run

# Atau via Makefile
make run
```

Server berjalan di `http://localhost:8000`

> Build pertama kali membutuhkan waktu 1-3 menit untuk mengkompilasi semua dependensi.

### Environment Variables

| Variable | Default | Keterangan |
|---|---|---|
| `APP_PORT` | `8000` | Port server |
| `APP_URL` | `http://localhost:8000` | Base URL |
| `DB_DRIVER` | `mysql` | `mysql` / `postgres` / `sqlite` |
| `DB_HOST` | `127.0.0.1` | Host database |
| `DB_PORT` | `3306` | Port database |
| `DB_USER` | `root` | Username database |
| `DB_PASS` | _(kosong)_ | Password database |
| `DB_NAME` | `starter_api` | Nama database |
| `JWT_SECRET` | `secret` | Secret key JWT |
| `JWT_ACCESS_EXPIRE` | `15` | Expire access token (menit) |
| `JWT_REFRESH_EXPIRE` | `10080` | Expire refresh token (menit) |
| `EMAIL_VERIFICATION_REQUIRED` | `false` | Wajib verifikasi email |
| `MAIL_HOST` | _(kosong)_ | SMTP host |
| `MAIL_PORT` | `587` | SMTP port |
| `MAIL_USER` | _(kosong)_ | SMTP username |
| `MAIL_PASS` | _(kosong)_ | SMTP password |
| `MAIL_FROM` | `no-reply@example.com` | Alamat pengirim |
| `GOOGLE_CLIENT_ID` | _(kosong)_ | Google OAuth client ID |
| `FACEBOOK_CLIENT_ID` | _(kosong)_ | Facebook OAuth client ID |
| `STORAGE_PATH` | `./storage/photos` | Direktori foto |

### Akun Default (setelah seeder)

| Field | Value |
|---|---|
| Email | `root@example.com` |
| Password | `password` |

---

## Deploy ke VPS (Production)

### 1. Build binary di lokal / CI

```bash
# Build release binary (Linux target dari mesin lokal)
cargo build --release
# Binary: target/release/starter-api

# Cross-compile ke Linux dari Windows/Mac (gunakan cross)
cargo install cross
cross build --release --target x86_64-unknown-linux-musl
```

### 2. Kirim ke VPS

```bash
scp target/release/starter-api user@your-vps:/opt/starter-api-rust/
scp -r migrations/ user@your-vps:/opt/starter-api-rust/
scp .env.production user@your-vps:/opt/starter-api-rust/.env

ssh user@your-vps
chmod +x /opt/starter-api-rust/starter-api
mkdir -p /opt/starter-api-rust/storage/photos
```

### 3. Buat systemd service

```bash
sudo nano /etc/systemd/system/starter-api-rust.service
```

```ini
[Unit]
Description=Starter API Rust Axum
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/starter-api-rust
ExecStart=/opt/starter-api-rust/starter-api
EnvironmentFile=/opt/starter-api-rust/.env
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable starter-api-rust
sudo systemctl start starter-api-rust
sudo systemctl status starter-api-rust
```

### 4. Nginx reverse proxy

```nginx
server {
    listen 80;
    server_name api.example.com;

    client_max_body_size 10M;

    location / {
        proxy_pass http://127.0.0.1:8000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }

    location /storage/photos/ {
        alias /opt/starter-api-rust/storage/photos/;
        expires 30d;
    }
}
```

```bash
sudo nginx -t && sudo systemctl reload nginx
sudo certbot --nginx -d api.example.com
```

---

## Deploy dengan Docker

### 1. Build image

```bash
docker build -t starter-api-rust .
```

> Build image Docker pertama kali membutuhkan waktu lebih lama karena mengkompilasi Rust dari source. Hasil image akhir sangat kecil (~15MB) karena menggunakan Alpine.

### 2. Jalankan container

```bash
docker run -d \
  --name starter-api-rust \
  -p 8000:8000 \
  --env-file .env \
  -v $(pwd)/storage:/app/storage \
  --restart unless-stopped \
  starter-api-rust
```

### 3. Menggunakan Docker Compose

```yaml
version: "3.9"

services:
  api:
    build: .
    container_name: starter-api-rust
    ports:
      - "8000:8000"
    env_file: .env
    volumes:
      - ./storage:/app/storage
    depends_on:
      db:
        condition: service_healthy
    restart: unless-stopped

  db:
    image: mysql:8.0
    container_name: starter-api-db
    environment:
      MYSQL_ROOT_PASSWORD: secret
      MYSQL_DATABASE: starter_api
    volumes:
      - db_data:/var/lib/mysql
    healthcheck:
      test: ["CMD", "mysqladmin", "ping", "-h", "localhost"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

volumes:
  db_data:
```

```bash
# Set DB_HOST=db di .env
docker compose up -d
docker compose logs -f api
```

---

## API Endpoints

| Method | Endpoint | Auth | Keterangan |
|---|---|---|---|
| POST | `/api/v1/auth/register` | — | Register |
| POST | `/api/v1/auth/login` | — | Login |
| POST | `/api/v1/auth/logout` | ✓ | Logout |
| POST | `/api/v1/auth/refresh` | — | Refresh token |
| POST | `/api/v1/auth/revoke` | — | Revoke token |
| POST | `/api/v1/auth/forgot-password` | — | Kirim link reset |
| POST | `/api/v1/auth/reset-password` | — | Reset password |
| GET | `/api/v1/auth/verify-email?token=` | — | Verifikasi email |
| POST | `/api/v1/auth/change-password` | ✓ | Ganti password |
| GET | `/api/v1/auth/me` | ✓ | Profil sendiri |
| PUT | `/api/v1/profile` | ✓ | Update profil |
| POST | `/api/v1/profile/photo` | ✓ | Upload foto |
| POST | `/api/v1/auth/oauth/google` | — | Login Google |
| POST | `/api/v1/auth/oauth/facebook` | — | Login Facebook |
| GET | `/api/v1/users` | ✓ `user:index` | Daftar user |
| POST | `/api/v1/users` | ✓ `user:create` | Buat user |
| GET | `/api/v1/users/:id` | ✓ `user:show` | Detail user |
| PUT | `/api/v1/users/:id` | ✓ `user:edit` | Update user |
| DELETE | `/api/v1/users/:id` | ✓ `user:delete` | Hapus user |
| POST | `/api/v1/users/:id/photo` | ✓ `user:edit` | Upload foto user |
| GET | `/api/v1/roles` | ✓ `role:index` | Daftar role |
| POST | `/api/v1/roles` | ✓ `role:create` | Buat role |
| GET | `/api/v1/roles/:id` | ✓ `role:show` | Detail role |
| PUT | `/api/v1/roles/:id` | ✓ `role:edit` | Update role |
| DELETE | `/api/v1/roles/:id` | ✓ `role:delete` | Hapus role |
| GET | `/api/v1/permissions` | ✓ `permission:index` | Daftar permission |
| GET | `/api/v1/permissions/tree` | ✓ `permission:index` | Tree permission |
| GET | `/api/v1/permissions/by-role/:id` | ✓ `permission:index` | Permission by role |

## Format Response

```json
{
  "success": true,
  "message": "Data retrieved",
  "data": {},
  "meta": {
    "page": 1,
    "per_page": 10,
    "total": 100,
    "total_page": 10
  }
}
```

## Performa

Rust + Axum memberikan performa sangat tinggi dengan memory footprint yang rendah. Binary yang dihasilkan tidak memerlukan runtime dan dapat langsung dijalankan di server tanpa instalasi tambahan.
