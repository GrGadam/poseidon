# Poseidon Projekt Dokumentáció

## Cél
Ez a projekt egy teljes web-rendszert mutat be: adatbázist, REST API-t, webes kliensfelületet, valamint a megvalósítást kísérő műszaki dokumentációt.

## Választott technológiai stack
- Backend: Rust, Axum, SQLx, SQLite, JWT, Argon2, WebSocket.
- Frontend: SvelteKit, TypeScript, Svelte állapotkezelés.
- Desktop csomagolás: Tauri.
- API dokumentáció: OpenAPI / Swagger UI.

## Miért ezek a megoldások
- A Rust backend biztonságos és jól illeszkedik a hitelesítéshez, session-kezeléshez és az adatbázis-hozzáféréshez.
- Az Axum egyszerűen kezeli a REST útvonalakat és a middleware jellegű összekötést.
- Az SQLx közvetlen, típusos adatbázis-hozzáférést ad SQLite-on.
- A SvelteKit gyorsan építhető, interaktív kliensoldalt ad a CRUD műveletekhez.
- A Tauri lehetővé teszi, hogy a webalkalmazás asztali csomagként is futtatható legyen.

## Megvalósított funkcionális követelmények
- Regisztráció, bejelentkezés, token-alapú session-kezelés, kijelentkezés és tokenfrissítés.
- Autentikált CRUD műveletek a felhasználók, barátok, DM-ek, szerverek, csatornák és üzenetek kezeléséhez.
- REST API a kliens és a szerver közti kommunikációhoz.
- Websocket alapú valós idejű eseményküldés.
- GUI a műveletek végrehajtásához és az eredmények megjelenítéséhez.

## Megvalósított nem funkcionális követelmények
- OpenAPI dokumentáció a backendhez.
- Külön backend és frontend réteg.
- Titkosított jelszótárolás Argon2-val.
- Session persistálás frissítő tokenekkel.
- Több entitás közötti kapcsolatkezelés az adatbázisban.

## Adatmodell
A jelenlegi séma több mint öt entitást kezel, például:
- users
- refresh_tokens
- friend_requests
- friendships
- dm_threads
- dm_messages
- servers
- server_members
- server_invites
- server_user_invites
- channels
- channel_messages

## Demo adatok
A seedelt adatbázis három demó felhasználót és több kapcsolódó mintarekordot tartalmaz. A belépéshez használható példák:
- admin / admin123
- alice / demo1234
- bob / demo1234

A demó adatok között szerepelnek baráti kapcsolatok, pending barátkérés, publikus és privát szerverek, csatornák, csatornaüzenetek, DM szál és szervermeghívások is.