# Poseidon Projekt Dokumentáció

## Cél

Ez a projekt egy teljes web-rendszert valósít meg: adatbázist, REST API-t, webes kliensfelületet, valós idejű kommunikációt, valamint külön dokumentációt a technológiai döntésekről és az AI-használatról.

## Választott technológiai stack

### Backend

- Rust 2024
- Axum REST API-hoz és WebSockethez
- SQLx SQLite adatbázis-hozzáféréshez és migrációkhoz
- JWT access tokenekhez
- Argon2 jelszóhash-eléshez
- Tokio aszinkron futtatókörnyezethez
- Tower HTTP CORS kezeléshez
- Utoipa és Swagger UI API dokumentációhoz

### Frontend

- SvelteKit a webalkalmazás felületéhez
- Svelte 5 runes-alapú állapotkezelés
- TypeScript a típusbiztonságért
- Tailwind CSS 4 és daisyUI a felhasználói felülethez
- WebSocket kliens a valós idejű eseményekhez
- Tauri a desktop csomagoláshoz

## Miért ezek a megoldások

- A Rust biztonságos és jól illeszkedik hitelesített, adatbázis-központú szolgáltatásokhoz.
- Az Axum egyszerűen kezeli a REST útvonalakat, a middleware jellegű logikát és a WebSocket kapcsolatot.
- Az SQLx típusos lekérdezést, migrációkezelést és SQLite támogatást ad külön ORM réteg nélkül.
- Az Argon2 megfelel a jelszavak biztonságos tárolásának.
- A JWT + refresh token megoldás stabil session-kezelést biztosít a kliens oldalon.
- A SvelteKit gyors, interaktív és könnyen karbantartható felületet ad a CRUD műveletekhez.
- A Tauri lehetővé teszi, hogy ugyanaz a webes UI asztali alkalmazásként is futtatható legyen.

## Megvalósított funkcionális követelmények

- Felhasználói regisztráció.
- Bejelentkezés username vagy email alapján.
- Access token és refresh token alapú session-kezelés.
- Kijelentkezés.
- Saját profil lekérdezése és módosítása.
- Jelszócsere.
- Saját avatar feltöltése és avatar lekérdezése.
- Barátkérés küldése.
- Függőben lévő barátkérések listázása, elfogadása és elutasítása.
- Barátlista lekérdezése és barát törlése.
- DM szálak listázása és létrehozása.
- DM üzenetek listázása, küldése, szerkesztése és törlése.
- Szerver létrehozása, listázása, módosítása és törlése.
- Nyilvános szerverek listázása, keresése és csatlakozása.
- Meghívó kód alapú csatlakozás.
- Függőben lévő szervermeghívók listázása, elfogadása és elutasítása.
- Privát szerverre felhasználó meghívása.
- Szerverből való kilépés.
- Szervertag szerepkörének módosítása és eltávolítása.
- Csatornák létrehozása, listázása, módosítása és törlése.
- Szerver avatar feltöltése és lekérése.
- Csatornaüzenetek küldése, listázása, szerkesztése és törlése.
- Valós idejű WebSocket események kezelése.
- A fenti funkciók GUI-n keresztüli végrehajtása a webalkalmazásban.

## Megvalósított nem funkcionális követelmények

- Külön backend és frontend réteg valósult meg.
- A backend OpenAPI / Swagger UI dokumentációt szolgáltat.
- Az adatbázisséma migrációkkal kezelt.
- A rendszer seedelt demó adatokat tartalmaz.
- A jelszavak hash-elve kerülnek tárolásra.
- A session információ kliensoldalon is perzisztálható, refresh token alapú újratöltéssel.
- A backend CORS beállítással támogatja a külön frontend hostról érkező kéréseket.
- A rendszer több entitás közti kapcsolatokat kezel, nem lapos CRUD mintát.
- A projekt desktop csomagként is futtatható.

## Adatmodell

A séma legalább öt, egymással kapcsolatban álló entitást kezel. A főbb táblák:

- `users`
- `refresh_tokens`
- `friend_requests`
- `friendships`
- `dm_threads`
- `dm_messages`
- `servers`
- `server_members`
- `server_invites`
- `server_user_invites`
- `channels`
- `channel_messages`

Kapcsolati példák:

- Egy felhasználó több szerver tagja lehet.
- Egy szerverhez több csatorna tartozhat.
- Egy csatornához több üzenet tartozhat.
- Két felhasználó között DM szál és DM üzenetek jöhetnek létre.
- A szervermeghívók és barátkérések külön kapcsolatkezelést biztosítanak.

## Demo adatok

A seedelt adatbázis több demó felhasználót és kapcsolódó mintaadatot tartalmaz. A belépéshez használható példák:

- `admin` / `admin123`
- `alice` / `demo1234`
- `bob` / `demo1234`

A demó adatok között szerepelnek baráti kapcsolatok, függőben lévő barátkérés, publikus és privát szerverek, csatornák, csatornaüzenetek, DM szálak és szervermeghívók is.

## Kézbesítési megfelelés

A projekt megfelel a teljes web-rendszer követelményének, mert tartalmaz:

- adatbázist,
- REST API-t autentikációval és session-kezeléssel,
- webalkalmazást GUI-val,
- szoftveres dokumentációt,
- AI-használati elemzést.