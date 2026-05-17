# kegel_trainingsplan

Dioxus 0.7 Fullstack-App mit SeaORM, SQLite und einem kleinen Auth-/User-System.

## Enthalten
- `SeaORM` mit `SQLite`
- automatische Schema-Erstellung beim ersten Start
- Benutzer-Registrierung
- Login / Logout
- serverseitige Sessions
- Cookie-basierte Session-Wiederherstellung für Web und Mobile mit `dioxus-cookie`
- optionaler Bootstrap fuer einen globalen System-Admin per Umgebungsvariablen

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

## Bootstrap-System-Admin

Beim ersten Start kann optional automatisch ein globaler System-Admin angelegt werden.

Dafuer beide Variablen setzen:

```sh
BOOTSTRAP_ADMIN_USERNAME=admin
BOOTSTRAP_ADMIN_PASSWORD=ein-sicheres-passwort
```

Regeln:

- der Bootstrap laeuft nur, wenn noch kein System-Admin existiert
- sind beide Variablen nicht gesetzt, passiert nichts
- ist nur eine der beiden Variablen gesetzt, bricht die Initialisierung mit einer Fehlermeldung ab
- existiert bereits ein Benutzer mit dem konfigurierten Benutzernamen, wird kein Admin erzeugt und die Initialisierung bricht mit einer Fehlermeldung ab

## Auth-Cookie
Für lokale HTTP-Entwicklung ist das Cookie standardmäßig **nicht** `secure`, damit Login unter `localhost` funktioniert.

Für Deployment hinter HTTPS solltest du setzen:
```sh
AUTH_COOKIE_SECURE=true
```
