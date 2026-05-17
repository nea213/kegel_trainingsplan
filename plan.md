# Kegel-Trainingsplan Implementierungsplan

## Zweck

Dieses Dokument ist die fachliche und technische Referenz fuer die weitere Entwicklung des Projekts. Es haelt die aktuell getroffenen Entscheidungen fest und beschreibt, in welcher Reihenfolge die Features in diesem Repository umgesetzt werden sollen.

## Aktueller Stand im Repository

Bereits vorhanden:

- Dioxus 0.7 Fullstack-App
- SeaORM mit SQLite
- Benutzer-Registrierung und Login
- serverseitige Sessions mit Cookie
- geschuetzte Startseite und Basis-Navigation
- Theme-Speicherung pro Benutzer

Bereits vorhandene Kernstellen:

- `src/main.rs`
- `src/auth.rs`
- `src/server/auth.rs`
- `src/server/db.rs`
- `src/server/entities/user.rs`
- `src/server/entities/session.rs`

Wichtige Folgerung:

- Das bestehende offene Registrierungsmodell wird spaeter auf invite-basierte Registrierung umgebaut.
- Login, Logout, Session-Handling und geschuetzte Seiten bleiben die technische Basis.

## Fachmodell

Die fachliche Struktur wird so festgelegt:

- Ein `Verein` hat mehrere `Gruppen`.
- Eine `Gruppe` gehoert genau zu einem `Verein`.
- Eine `Gruppe` hat mehrere `Mannschaften`.
- Eine `Mannschaft` gehoert genau zu einem `Verein` und genau zu einer `Gruppe`.
- Ein Benutzer kann in mehreren Vereinen, Gruppen und Mannschaften aktiv sein.
- Ein Benutzer darf gleichzeitig `Trainer` und `Spieler` sein.

Beispiel:

- Verein: `KV Musterstadt`
- Gruppe: `Maenner`
- Mannschaften: `Maenner 1`, `Maenner 2`
- Gruppe: `Frauen`
- Mannschaft: `Frauen 1`

## Rollenmodell

Es gibt fuer den MVP genau drei Rollenarten:

1. `system_admin`
2. `trainer`
3. `player`

Die Rollen gelten auf unterschiedlichen Ebenen:

- `system_admin` ist global auf User-Ebene
- `trainer` ist auf Gruppen-Ebene
- `player` ist auf Mannschafts-Ebene

### Rechte

#### System-Admin

- ist global und initial keinem Verein zugeordnet
- kann Vereine anlegen und verwalten
- kann Gruppen und Mannschaften anlegen
- kann Trainer einer Gruppe zuweisen
- kann sich selbst einer Gruppe als Trainer zuweisen
- kann sich selbst einer Mannschaft als Spieler zuweisen
- kann alle Daten sehen und verwalten

#### Trainer

- kann nur in Gruppen arbeiten, in denen er Trainer ist
- kann nur fuer diese Gruppen Einladungen erzeugen
- kann nur Spieler oder Trainer in diese Gruppen einladen
- kann Trainings fuer diese Gruppen anlegen
- kann Trainings optional fuer eine konkrete Mannschaft innerhalb seiner Gruppe anlegen

#### Spieler

- sieht nur eigene Mannschaften
- sieht nur fuer ihn relevante Trainings
- hat keine Verwaltungsrechte

## Bootstrap-Entscheidung

Der erste Administrator wird nicht ueber die normale Registrierung erzeugt.

Festgelegt ist:

- Es gibt einen globalen `System-Admin`.
- Dieser wird per Bootstrap oder Seed-Prozess angelegt.
- Offene Registrierung wird langfristig deaktiviert.
- Neue Benutzer kommen grundsaetzlich ueber Einladungscode ins System.

### Empfehlung fuer die technische Umsetzung

Der Bootstrap sollte als kontrollierter Initialisierungsprozess umgesetzt werden, nicht durch manuelle SQL-Eingriffe.

Empfohlene Richtung:

- Seed-Command oder Initial-Bootstrap beim Start
- nur ausfuehrbar, wenn noch kein `system_admin` existiert

Beispielhafte Konfiguration:

- `BOOTSTRAP_ADMIN_USERNAME`
- `BOOTSTRAP_ADMIN_PASSWORD`

Alternative spaeter:

- eigener CLI-Command fuer Admin-Erzeugung

## Datenmodell

Die folgenden Tabellen sind fuer den MVP vorgesehen.

### 1. `users`

Bestehend, wird erweitert um:

- `is_system_admin BOOLEAN NOT NULL DEFAULT 0`

Bestehende Felder:

- `id`
- `username`
- `password_hash`
- `theme_mode`
- `created_at`
- `updated_at`

### 2. `clubs`

- `id INTEGER PRIMARY KEY`
- `name TEXT NOT NULL UNIQUE`
- `created_at INTEGER NOT NULL`
- `updated_at INTEGER NOT NULL`

### 3. `club_groups`

- `id INTEGER PRIMARY KEY`
- `club_id INTEGER NOT NULL`
- `name TEXT NOT NULL`
- `sort_order INTEGER NOT NULL DEFAULT 0`
- `created_at INTEGER NOT NULL`
- `updated_at INTEGER NOT NULL`

Constraints:

- unique auf `(club_id, name)`

### 4. `teams`

- `id INTEGER PRIMARY KEY`
- `club_id INTEGER NOT NULL`
- `group_id INTEGER NOT NULL`
- `name TEXT NOT NULL`
- `sort_order INTEGER NOT NULL DEFAULT 0`
- `created_at INTEGER NOT NULL`
- `updated_at INTEGER NOT NULL`

Constraints:

- unique auf `(group_id, name)`

### 5. `group_trainers`

- `id INTEGER PRIMARY KEY`
- `group_id INTEGER NOT NULL`
- `user_id INTEGER NOT NULL`
- `created_at INTEGER NOT NULL`

Constraints:

- unique auf `(group_id, user_id)`

### 6. `team_players`

- `id INTEGER PRIMARY KEY`
- `team_id INTEGER NOT NULL`
- `user_id INTEGER NOT NULL`
- `created_at INTEGER NOT NULL`

Constraints:

- unique auf `(team_id, user_id)`

Hinweis:

- Fuer den MVP werden `trainer` und `player` bewusst getrennt gespeichert.
- Falls spaeter noetig, kann das in eine generische Membership-Tabelle zusammengefuehrt werden.

### 7. `invitations`

- `id INTEGER PRIMARY KEY`
- `code_hash TEXT NOT NULL UNIQUE`
- `club_id INTEGER NOT NULL`
- `group_id INTEGER NOT NULL`
- `team_id INTEGER NULL`
- `target_role TEXT NOT NULL`
- `created_by_user_id INTEGER NOT NULL`
- `expires_at INTEGER NOT NULL`
- `used_at INTEGER NULL`
- `used_by_user_id INTEGER NULL`
- `revoked_at INTEGER NULL`
- `created_at INTEGER NOT NULL`

Regeln:

- `target_role` ist initial `trainer` oder `player`
- `team_id` ist optional
- `team_id` darf nur gesetzt sein, wenn die Mannschaft zur Gruppe gehoert
- Codes werden nur gehasht gespeichert
- Codes sind einmalig nutzbar

### 8. `training_sessions`

- `id INTEGER PRIMARY KEY`
- `club_id INTEGER NOT NULL`
- `group_id INTEGER NOT NULL`
- `team_id INTEGER NULL`
- `title TEXT NOT NULL`
- `description TEXT NOT NULL DEFAULT ''`
- `location TEXT NOT NULL DEFAULT ''`
- `start_at INTEGER NOT NULL`
- `end_at INTEGER NOT NULL`
- `status TEXT NOT NULL`
- `created_by_user_id INTEGER NOT NULL`
- `created_at INTEGER NOT NULL`
- `updated_at INTEGER NOT NULL`

Regeln:

- Training gehoert immer zu einer Gruppe
- `team_id` ist optional
- damit sind sowohl gruppenweite als auch mannschaftsspezifische Trainings moeglich
- `status` ist initial z. B. `planned`, `completed`, `cancelled`

## Beziehungen

- `clubs -> club_groups` ist 1:n
- `club_groups -> teams` ist 1:n
- `users -> group_trainers` ist 1:n
- `users -> team_players` ist 1:n
- `users -> invitations(created_by_user_id)` ist 1:n
- `users -> invitations(used_by_user_id)` ist optional 1:n
- `users -> training_sessions(created_by_user_id)` ist 1:n

## Einladungslogik

Der normale Registrierungsweg wird auf Einladungscodes umgebaut.

### Zielbild

1. Ein Trainer oder System-Admin erzeugt eine Einladung.
2. Die Einladung ist an eine Gruppe gebunden.
3. Optional ist die Einladung auch an eine konkrete Mannschaft gebunden.
4. Die Einladung enthaelt eine Zielrolle `trainer` oder `player`.
5. Der neue Benutzer registriert sich mit `Einladungscode + Username + Passwort`.
6. Der Code wird serverseitig geprueft.
7. Nach erfolgreicher Registrierung wird die passende Zuordnung angelegt.
8. Der Code wird danach als verbraucht markiert.

### Regeln

- Trainer duerfen nur Einladungen fuer Gruppen erzeugen, in denen sie Trainer sind.
- Spieler-Einladungen koennen optional direkt an eine Mannschaft gebunden sein.
- Trainer-Einladungen muessen mindestens an eine Gruppe gebunden sein.
- Abgelaufene oder widerrufene Codes sind ungueltig.
- Bereits verwendete Codes sind ungueltig.

## Trainingslogik

Trainings werden auf Gruppenebene gespeichert und koennen optional auf eine Mannschaft eingeschraenkt werden.

### Warum dieses Modell

- ein Training kann fuer alle `Maenner` gelten
- ein Training kann nur fuer `Maenner 1` gelten
- die Rechtepruefung bleibt an der Gruppe orientiert
- die Anzeige fuer Spieler bleibt einfach filterbar

### MVP-Felder fuer ein Training

- Titel
- Beschreibung
- Ort
- Startzeitpunkt
- Endzeitpunkt
- Status

## Zielstruktur im Code

Die folgenden Module sind mittelfristig im Projekt vorgesehen.

### Server

Neue Module unter `src/server/`:

- `src/server/clubs.rs`
- `src/server/groups.rs`
- `src/server/teams.rs`
- `src/server/invitations.rs`
- `src/server/training.rs`
- `src/server/permissions.rs`
- `src/server/bootstrap.rs`

Neue Entities unter `src/server/entities/`:

- `club.rs`
- `club_group.rs`
- `team.rs`
- `group_trainer.rs`
- `team_player.rs`
- `invitation.rs`
- `training_session.rs`

`src/server/mod.rs` wird entsprechend erweitert.

### Shared Server Functions

Neue API-Module im Stil von `src/auth.rs`:

- `src/clubs.rs`
- `src/groups.rs`
- `src/teams.rs`
- `src/invitations.rs`
- `src/training.rs`

Diese Dateien kapseln die Dioxus Server Functions fuer Frontend und Backend.

### Views

Neue Views unter `src/views/`:

- `dashboard.rs`
- `clubs.rs`
- `club_detail.rs`
- `group_detail.rs`
- `team_detail.rs`
- `invitations.rs`
- `training.rs`

### Components

Neue Komponenten unter `src/components/`:

- `dashboard/`
- `clubs/`
- `groups/`
- `teams/`
- `invitations/`
- `training/`

Hinweis:

- Die bestehende `RegisterPanel`-Logik wird spaeter zu einem Invite-Registrierungsformular umgebaut.

## Geplante Routen

Die genaue Benennung kann spaeter leicht angepasst werden. Aktuell geplant:

- `/`
- `/login`
- `/register`
- `/dashboard`
- `/clubs`
- `/clubs/:club_id`
- `/clubs/:club_id/groups/:group_id`
- `/clubs/:club_id/teams/:team_id`
- `/invitations`
- `/training`

## Berechtigungspruefung

Wichtige Regel:

- Berechtigungen werden serverseitig geprueft, nicht nur im UI versteckt.

Es wird ein zentrales Berechtigungsmodul empfohlen, damit Regeln nicht in mehreren Serverfunktionen dupliziert werden.

Beispielhafte Helper:

- `require_authenticated_user()`
- `require_system_admin()`
- `require_group_trainer(user_id, group_id)`
- `can_manage_team(user_id, team_id)`

## Implementierungsphasen

Die Umsetzung erfolgt bewusst inkrementell.

### Phase 1: Bootstrap und Rollenbasis

Ziel:

- globalen `system_admin` einfuehren
- Bootstrap-Mechanismus fuer ersten Admin einfuehren
- bestehende offene Registrierung fuer spaeter vorbereiten

Arbeitspakete:

1. `users` um `is_system_admin` erweitern
2. Bootstrap-Logik implementieren
3. Auth-Flow so erweitern, dass System-Admin korrekt erkannt wird
4. erste einfache Admin-Pruefung serverseitig einbauen

Abnahme:

- ein globaler Admin kann sicher erzeugt werden
- bestehender Login funktioniert weiter

### Phase 2: Vereinsstruktur

Ziel:

- Vereine, Gruppen und Mannschaften technisch einfuehren

Arbeitspakete:

1. Tabellen `clubs`, `club_groups`, `teams`
2. SeaORM-Entities anlegen
3. CRUD-Serverfunktionen anlegen
4. erste Verwaltungsseiten fuer den System-Admin bauen

Abnahme:

- Verein kann angelegt werden
- Gruppen koennen angelegt werden
- Mannschaften koennen innerhalb einer Gruppe angelegt werden

### Phase 3: Mitgliedschaften und Rechte

Ziel:

- Trainer- und Spielerzuordnungen umsetzen

Arbeitspakete:

1. Tabellen `group_trainers` und `team_players`
2. Serverfunktionen fuer Zuweisungen
3. serverseitige Rechtehelper erweitern
4. UI fuer Trainer- und Spielerzuweisung

Abnahme:

- Trainer kann einer Gruppe zugewiesen werden
- Spieler kann einer Mannschaft zugewiesen werden
- Rechte greifen serverseitig korrekt

### Phase 4: Einladungscodes

Ziel:

- Registrierung auf Codebasis umstellen

Arbeitspakete:

1. Tabelle `invitations`
2. Code-Erzeugung mit sicherem Zufallswert
3. Code nur gehasht speichern
4. Invite-Validierung bei Registrierung
5. bestehende `register_user`-Logik umbauen
6. Invite-Management fuer Trainer und Admin bauen

Abnahme:

- Trainer kann gueltige Einladungen erzeugen
- Registrierung mit gueltigem Code funktioniert
- Code wird nach Nutzung ungultig

### Phase 5: Dashboards und Navigation

Ziel:

- klare Navigation je Rolle

Arbeitspakete:

1. Dashboard-Ansichten fuer Admin, Trainer, Spieler
2. Navbar erweitern
3. zentrale Uebersichten fuer Vereine, Gruppen, Mannschaften und Einladungen

Abnahme:

- jeder Benutzer sieht nur relevante Bereiche

### Phase 6: Trainingsplanung

Ziel:

- eigentliche Trainingsplanung produktiv verfuegbar machen

Arbeitspakete:

1. Tabelle `training_sessions`
2. CRUD fuer Trainings
3. Listenansicht und Filter
4. Spieleransicht fuer relevante Trainings
5. Trainer duerfen nur in eigenen Gruppen planen

Abnahme:

- Trainings koennen angelegt, bearbeitet und angezeigt werden
- Spieler sehen nur relevante Termine

### Phase 7: Ausbau nach MVP

Erweiterungen fuer spaeter:

- Teilnahmezusage
- Anwesenheit
- Spieler-Notizen
- Saisonverwaltung
- Benachrichtigungen
- Exportfunktionen
- optional Vereins-Admin-Rolle

## Technische Leitlinien

### Datenbank

- Schema-Aenderungen werden zunaechst wie bisher in `src/server/db.rs` eingebaut
- bestehende Datenbankdatei bleibt `data/kegel-trainingsplan.sqlite`
- spaeter koennen echte Migrationen nachgezogen werden, falls noetig

### Auth und Sessions

- bestehende Session-Logik bleibt erhalten
- aktuelle `current_user()`-Antwort wird spaeter um Rolleninformationen erweitert
- Rechte werden nicht allein im Frontend ermittelt

### Dioxus Frontend

- geschuetzte Bereiche bleiben ueber serverseitige User-Ermittlung abgesichert
- Daten fuer geschuetzte Views bevorzugt mit `use_server_future` laden
- Formulare folgen dem bereits vorhandenen Stil aus `src/components/auth.rs`

## Offene Punkte fuer spaeter

Diese Punkte sind bewusst noch nicht festgelegt und koennen spaeter entschieden werden:

- braucht das System spaeter eine `club_admin`-Rolle?
- sollen Einladungscodes mehrfach nutzbar sein koennen?
- sollen Gruppen nur frei angelegt werden oder initial Standardgruppen erhalten?
- soll es spaeter Kalenderansichten geben?
- sollen Benachrichtigungen per E-Mail oder nur intern laufen?

## Nicht-Ziele fuer den ersten MVP

Diese Punkte gehoeren bewusst nicht in die erste Implementierungsrunde:

- E-Mail-Versand
- Push-Benachrichtigungen
- komplexes Rollenmodell ueber `system_admin`, `trainer`, `player` hinaus
- Saison- und Wettkampfmanagement
- Statistik- und Auswertungsmodule

## Erste konkrete Umsetzungsreihenfolge im Repository

Wenn direkt weiterentwickelt wird, sollte die Reihenfolge so sein:

1. `users` um `is_system_admin` erweitern
2. Bootstrap-Admin in `src/server/auth.rs` oder neuem `src/server/bootstrap.rs` vorbereiten
3. `clubs`, `club_groups`, `teams` in DB und Entities einfuehren
4. Admin-Oberflaeche fuer Vereinsstruktur bauen
5. `group_trainers` und `team_players` einfuehren
6. serverseitige Rechtehelper zentralisieren
7. Einladungs-Flow auf Codebasis implementieren
8. bestehende Registrierung auf Invite-Code umbauen
9. Trainingsmodul einfuehren

## Kurzfassung der finalen Entscheidungen

- Struktur: `Verein -> Gruppe -> Mannschaft`
- globaler `System-Admin`, initial ohne Vereinszuordnung
- `Trainer` auf Gruppenebene
- `Spieler` auf Mannschaftsebene
- Benutzer kann mehrere Rollen gleichzeitig haben
- Registrierung langfristig nur per Einladungscode
- Trainings gelten fuer eine Gruppe und optional fuer eine Mannschaft
