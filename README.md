# kegel_trainingsplan

Dioxus 0.7 Fullstack-App mit SeaORM, SQLite und einem kleinen Auth-/User-System.

## Enthalten
- `SeaORM` mit `SQLite`
- automatische Schema-Erstellung beim ersten Start
- Benutzer-Registrierung
- Login / Logout
- serverseitige Sessions
- Cookie-basierte Session-Wiederherstellung für Web und Mobile mit `dioxus-cookie`

## Lokale Entwicklung
```sh
dx serve
```

Alternativ direkt mit Cargo:
```sh
cargo run --no-default-features --features web,server
```

## Datenbank
Standardmäßig wird die SQLite-Datei hier erzeugt:
- `data/kegel-trainingsplan.sqlite`

Optional kannst du eine eigene DB setzen:
```sh
DATABASE_URL=sqlite://data/my-app.sqlite?mode=rwc
```

## Auth-Cookie
Für lokale HTTP-Entwicklung ist das Cookie standardmäßig **nicht** `secure`, damit Login unter `localhost` funktioniert.

Für Deployment hinter HTTPS solltest du setzen:
```sh
AUTH_COOKIE_SECURE=true
```
