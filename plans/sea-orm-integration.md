# SeaORM-Integration Plan

## Context
- Das Projekt ist aktuell ein kleines Dioxus-0.7.1-Template mit `router` und `fullstack`.
- Bisher gibt es nur Demo-UI (`Home`, `Blog`, `Navbar`) und eine Demo-Serverfunktion in `src/components/echo.rs`.
- Es gibt noch keine Datenbank-Anbindung, keine Models/Entities, keine Migrations und kein serverseitiges DB-Bootstrapping.
- Im Repository sind aktuell auch keine `.env`-Dateien, keine vorhandene DB-Konfiguration und kein Migrations-Verzeichnis vorhanden.
- Ziel ist, SeaORM sauber in die bestehende Dioxus-Fullstack-Struktur einzubinden.
- Vom Nutzer bestätigt: `SQLite` als erstes Backend, direkt mit `Auth`- und `User`-System, Zielplattformen `Web` und `Mobile`.

## Approach
- SeaORM serverseitig integrieren, damit DB-Zugriffe nicht im Client landen.
- SQLite als primäre persistente Datenbank verwenden.
- Auf der bestehenden Dioxus-Fullstack-Struktur aufbauen und Serverfunktionen für Auth-Operationen nutzen.
- Als ersten fachlichen Umfang ein minimales Auth-System einplanen: `users`, Passwort-Hashing, Login/Logout, Current-User-Abfrage und eine einfache geschützte Ansicht bzw. Auth-Status im UI.
- Migrationen sind technisch nicht zwingend erforderlich, aber für Schema-Versionierung, reproduzierbares Setup und spätere Änderungen stark zu empfehlen. Ohne Migrationen müsste das Schema per Startup-SQL oder manuell erzeugt werden.

## Files to modify
- `Cargo.toml`
- `src/main.rs`
- `src/components/echo.rs` (eher als Referenz; Auth/DB-Logik sollte wahrscheinlich in neue Module ausgelagert werden)
- `src/views/home.rs`
- `src/views/navbar.rs`
- vermutlich neue Module für DB-Setup, Entities/Models, Auth-Serverfunktionen und Auth-UI-Zustand
- optional separates Migrations-Crate/-Verzeichnis, falls `sea-orm-migration` aufgenommen wird

## Reuse
- Dioxus-Fullstack-Pattern aus `src/components/echo.rs`
- Routing/App-Struktur aus `src/main.rs`
- bestehende Modulaufteilung in `src/components` und `src/views`
- aktuell existiert noch keine wiederverwendbare DB-/Config-Infrastruktur im Projekt, die SeaORM direkt aufnehmen könnte

## Steps
- [ ] Offene Architekturentscheidungen für Web + Mobile klären (gemeinsamer Server vs. lokales Mobile-SQLite, Session-Modell)
- [ ] SeaORM-, SQLite- und Auth-nahe Abhängigkeiten für den Server festlegen
- [ ] Struktur für Connection-Management und Initialisierung der SQLite-Datenbank definieren
- [ ] User-Entity und zugehöriges Schema planen (`id`, Login-Identifier, Passwort-Hash, Zeitstempel, optional Rollen/Felder)
- [ ] Login/Logout/Current-User-Flows als Dioxus-Serverfunktionen einplanen
- [ ] UI-Anpassungen für Login-Status, Login-Form und geschützte Ansicht festlegen
- [ ] Entscheidung zu Migrationen dokumentieren; falls ja, initiale Migration für `users` vorsehen, andernfalls Startup-Schema-Initialisierung definieren
- [ ] Verifikation für Web und Mobile-Client gegen denselben Auth-Backend-Flow festlegen

## Verification
- `cargo check`
- Web-App lokal starten (`dx serve` bzw. Server-Run)
- SQLite-Datei/DB-Initialisierung prüfen
- Test eines kompletten Auth-Flows: User anlegen/seeden, Login, Current-User abrufen, geschützte UI sehen, Logout
- Mobile-Client gegen denselben Backend-Flow prüfen, sobald die Kommunikationsstrategie feststeht

## Offene Fragen
- Soll `Mobile` gegen denselben Server authentifizieren wie `Web`, oder möchtest du auf Mobile eine lokale SQLite-Datenbank auf dem Gerät?
- Sollen Benutzer sich selbst registrieren können oder gibt es zunächst nur Login für vorab angelegte Benutzer?
- Reicht ein einfaches Auth-Modell ohne Rollen/Rechte, oder brauchst du direkt z. B. `admin`/`user`?
- Sollen Migrationen empfohlen und eingeplant werden, oder bevorzugst du bewusst ein schlankes Setup ohne separates Migrations-Tool?
