# Poseidon

Poseidon egy teljes web-rendszer bemutató projekt: Rust alapú REST backend, SQLite adatbázis, SvelteKit frontend, valós idejű WebSocket események és dokumentált AI használati napló tartozik hozzá.

## Előfeltételek

- Rust toolchain és `cargo`
- Node.js 20 vagy újabb
- `npm`

## Gyors indítás

1. Indítsd el a backendet:

   ```bash
   cd backend
   cargo run
   ```

2. Indítsd el a webalkalmazást egy másik terminálban:

   ```bash
   cd frontend
   npm install
   npm run dev
   ```

3. Nyisd meg a böngészőben a Vite által kiírt címet.

## Asztali futtatás

Ha a Tauri csomagolt verzióját szeretnéd futtatni:

```bash
cd frontend
npm run desktop:dev
```

Több kliens futtatása:

```bash
cd frontend
npm run desktop:dev:client1
```

```bash
cd frontend
npm run desktop:dev:client2
```

## Alapértelmezett környezet

A backend alapból a következő értékeket használja:

- `HOST=127.0.0.1`
- `PORT=8080`
- `DATABASE_URL=sqlite://poseidon.db?mode=rwc`
- `JWT_SECRET=dev-secret-change-me`
- `ACCESS_TOKEN_MINUTES=30`
- `REFRESH_TOKEN_DAYS=14`

Ezeket a `backend/.env` fájlban vagy a futtatási környezetben lehet felülírni.

## Demo belépők

A seedelt adatbázis tartalmaz demó felhasználókat:

- `admin` / `admin123`
- `alice` / `demo1234`
- `bob` / `demo1234`

## Hasznos végpontok

- Backend health check: `GET /health`
- OpenAPI / Swagger UI: `http://127.0.0.1:8080/api/docs`
- REST API gyökér: `http://127.0.0.1:8080/api/v1`

## Megjegyzés a demó adatokról

A backend induláskor lefuttatja az SQLx migrációkat, így az adatbázis sémája és a demó rekordok automatikusan rendelkezésre állnak egy friss adatbázis esetén. Ha teljesen tiszta indulást szeretnél, töröld a meglévő `backend/poseidon.db` fájlt, majd indítsd újra a backendet.