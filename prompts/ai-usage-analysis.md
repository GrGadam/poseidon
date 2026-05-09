# AI Használati Elemzés

## Összefoglaló

A Poseidon projekt fejlesztése során az AI-t szisztematikus és fázisok között felépített módon használtam. Az egyik legfontosabb tanulság az volt, hogy az AI leghatékonyabban **egy jól strukturált kezdeti specifikáción** keresztül működik. A projekt teljes tervével kezdtem, amelyből részletezett implementációs fázisok, architektúra döntések és ellenőrzési pontok származtak. Az AI ezután ezen terv alapján generálta a scaffoldot, a moduláris kódszerkezetet, a migrációkat és a frontend komponenseket.

A projekt valós alkalmazási példát mutat a **Teamspeak/Discord-szerű csevegő alkalmazás** kontextusában: REST API + WebSocket realtime + role-based access control + desktop-first UI.

## Projekt összefoglaló

- **Végtermék**: Teljes web-rendszer (backend + frontend), amely felhasználók közti valós idejű kommunikációt, szerver alapú csatornákat, barátlistákat és DM-eket támogat.
- **Techstack**: Rust (backend), SvelteKit + Tauri (frontend), SQLite (adatbázis).
- **Felhasználás**: Desktop-first, multiplatform Tauri alkalmazás.

## Mely fázisokban használtam AI-t

1. **Specifikáció és tervezés** (fázis 0):
   - Az AI-t egy kitűnő, részletes specifikációs prompton keresztül arra használtam, hogy teljes projekttervet szaruljak: architektura, adatmodell, CRUD követelmények, WebSocket realtime integráció, role-based access control.
   - A terv felépítésbe fázisok, függőségek és verifikációs pontok voltak.

2. **Projekt bootstrapping** (fázis 1):
   - Frontend scaffold (SvelteKit, DaisyUI, Tauri) és backend template (Rust, Axum, SQLx, migrációk) előállítása.
   - Non-interactive módok és pipe-olt input az interaktív npm/cargo promptokhoz.

3. **Backend architektúra** (fázis 2):
   - Moduláris kódelrendezés (routes, services, models), SQLite migrációk, JWT auth, REST CRUD endpoints, OpenAPI dokumentáció, WebSocket handler.

4. **Frontend komponensek** (fázis 3):
   - Desktop shell (Teamspeak-szerű layout), auth UI, Friends/Servers fülök, DM chat, szerver kezelés, valós idejű frissítések.

5. **Dokumentáció** (fázis 4):
   - README (futtatás, demó adatok), technológiai stack indoklása, megvalósított követelmények listázása.
   - AI-használati elemzés és prompt export dokumentációja.

## Jól működő promptok

1. **Kezdeti specifikációs prompt (Poseidon projekt létrehozása)**
   ```
   Készíts egy teamspeak3 és discord szerű csevegő alkalmazást. Ablakméret Teamspeak szerű legyen.
   
   Frontend: Tauri + Svelte + Typescript + DaisyUI
   Backend: Rust + Axum + SQLite
   
   [Követelmények, adatmodell, felhasználói felület spec...]
   ```
   - Működött, mert: Konkrét specifikáció, megadott techstack, pontosan definiált UI-követelmények, funkcionalitás list.
   - Eredmény: Teljes projekt terv, architecture discovery, implementációs roadmap.

2. „Start implementation"
   - Működött, mert: Egyértelmű, konkrét utasítás az execution kezdetére.
   - Eredmény: Scaffold generálás, dependencies telepítés, alap modulok.

3. „Szigorúan nézve, mi az amiket nem teljesít a project jelenlegi állapotában?"
   - Működött, mert: Konkrét ellenőrző szemléletet adott, és a hiányzó részekre fókuszált.

4. „A projektet fel kell tölteni a GitHub Classroom-ban létrehozott repository-ba. Ezenkívül README.md fájlt is készíteni kell."
   - Működött, mert: Egyértelművé tette a kimeneti formát és a dokumentációs igényt.

5. „A jelenlegi projekt alapján készítsd el a technológiai stack leírását és a megvalósított követelmények listáját."
   - Működött, mert: Megadta a célfájlt és a tartalmi fókuszt.

## Kevésbé jól működő promptok

- „Készítsd el.”
  - Túl rövid és túl sokértelmű volt, nem jelölte meg a célfájlt, a formátumot vagy a tartalmi hatókört.

- „Írd meg a dokumentációt.”
  - Ez sem volt elég pontos, mert nem különítette el a setup leírást, a stack indoklását és a követelménylistát.

- „Legyen kész a projekt.”
  - Ez fejlesztési szempontból nem használható jól, mert nem ad mérhető, ellenőrizhető részcélokat.- Interaktív telepítési parancsok (npm create svelte, npx tauri init stb.)
  - Működési elv: Ezek a parancsok interaktív promptokat adnak, amit az agent automatikus eszközei nem tudnak könnyen kezelni.
  - Megoldás: Non-interactive flagek használata (--no-install, --ci) vagy pipe-olt jóváhagyás (echo 'y' | ...).
  - Tanulság: Az automatizált setupban szabad utat az non-interactive módok számára.
## Tanulság

### Optimális prompt szerkezet

Az export alapján a legjobban azok a promptok működtek, amelyek tartalmaztak:

- **Konkrét célt/terméket**: "Készíts egy teamspeak3 és discord szerű csevegő alkalmazást."
- **Megadott technológiai stacket**: Konkrét nyelvek, keretrendszerek és könyvtárak.
- **Részletes követelménylista**: UI mockup, adatmodell, CRUD műveletek, realtime követelmények.
- **Szűk és mérhető hatókört**: Fázisos terv függőségekkel.

### Az AI-használat leghatékonyabb mintázata

1. **Specifikáció → Terv → Implementáció**: Az AI a terv szintjén dolgozik legjobban. Egy jól strukturált specifikáció → automatikus terv (fázisok, deps) → implementációs scaffolding.

2. **Nem-interaktív módok preferálása**: Az npm/cargo/tauri setup parancsok interaktív promptjai blokkolhatják az automatizált toolokat. Megoldás: non-interactive flagek (`--ci`, `--no-install`, `-y` pipe) vagy explicit jóváhagyás.

3. **Moduláris kód: AI-friendly**: Amikor az AI moduláris szerkezetekkel dolgozik (routes, services, models), könnyebben generál méretarányos és karbantartható kódot.

### Tanulság a GitHub Copilot Chat kontextusban

- Az AI-t fázisok között használtam, nem az egész projekthez egyszerre.
- Az "Start implementation" után az AI sikeresen orchestrated complex setup tasks (Tauri init, Cargo add, database migrations).
- Az export-ből kiderül: az AI jól kezeli a **deklaratív terveket**, de az **interaktív setupot** nehéznek találja.

### Prompt írási ajánlások

**Jó**: "Készíts egy Rust + Axum backend-et a következő specifikáció szerint: [detailed spec]"
**Rossz**: "Csináld meg"

**Jó**: "Frissítsd az ai-usage-analysis.md fájlt a prompts_export.txt alapján."
**Rossz**: "Dokumentáció"

### Felhasználás heurisztikái

| Szituáció | Ajánlott | Eredmény |
|-----------|---------|---------|
| Projekt tervezés | Részletes spec + kérdések | Teljes fázis-terv |
| Scaffold | Non-interactive CLI, konkrét flags | Gyors bootstrap |
| Kódgeneráció | Moduláris, konkrét fájlok | Jó struktúra |
| Dokumentáció | Konkrét tartalom, célfájl | Egységes, beadásra kész |
| Reparáció | Konkrét hiba + kontextus | Pontos fix |

## Az export-ből nyert konkrét promptok

Az alábbi promptok a `prompts/prompts_export.txt`-ben találhatók és tényleges munkafolyamatot mutatnak:

### Alapító prompt (fázis 0 terv)
```
Készíts egy teamspeak3 és discord szerű csevegő alkalmazást. 
Ablakméret Teamspeak szerű legyen.

Frontend: Tauri + Svelte + Typescript + DaisyUI
Backend: Rust + Axum + SQLite

I. Követelmények:
1. Minden kommunikáció realtime kell történjen (websocket)...
[12 db konkrét követelmény]

II. Az app leírása:
[15 db UI/UX követelmény]
```
**Hatás**: Az AI teljes architektúra tervet hozott létre, kockázat-azonosítást, és fázisokra bontott implementációs sablont.

### Orchestration prompt
```
Start implementation
```
**Hatás**: Az AI párhuzamos subagent-eket indított backend/frontend discoveryhez, utána szekvenciális kódgenerálásra váltott.

## Exportból tanultak

1. **Terv-primeiro hozzáállás működik**: Az AI az első specifikációból automatikusan memória-fájlt, task-listát és risk-listát generált.

2. **Subagent parallelization**: Összetett taskok (frontend arch + backend arch) párhuzamosan futnak, majd az eredmények integrálódnak.

3. **Interaktivitás csapda**: Az npm/tauri setup-ok interaktív prompt-blokkjainak kezelésére az agentnek explicit nem-interaktív módok szükségesek.

4. **Realtime fejlesztés**: A projekt már funkcionális (auth, CRUD, WebSocket, UI) a exportban dokumentált prompts után pár órával.