# UI-Refactor-Plan

Stand: 2026-05-21

Dieser Plan beschreibt einen vollstaendigen UI-Refactor fuer die App mit dem Ziel, die Anwendung moderner, ruhiger und fuer Web sowie App-Nutzung besser geeignet zu machen.

Der Fokus liegt auf:

- klarer Informationsarchitektur
- konsistenter Nutzerfuehrung
- moderner, aber nicht ueberladener Oberflaeche
- guter Nutzbarkeit auf Desktop, Tablet und Smartphone
- Wiederverwendung vorhandener Dioxus-UI-Bausteine statt unnötigem Neuaufbau

## Produktziel

Die App soll sich anfuehlen wie ein modernes Organisationswerkzeug fuer Vereine und Trainer.

Sie soll:

- auf dem Desktop effizient und informationsreich sein
- auf dem Smartphone fokussiert und fingerfreundlich bleiben
- als Web-App und als installierte App dieselbe Informationslogik behalten
- Rollen klar fuehren: System-Admin, Trainer, Spieler

## Zielbild

### Was die UI nach dem Refactor leisten soll

- Jede Seite hat eine klare Hauptaufgabe.
- Es gibt sichtbare Hierarchie zwischen Uebersicht, Arbeitsschritt und Detailaktion.
- Formulare sind nur sichtbar, wenn sie gerade gebraucht werden.
- Wichtige Aktionen sind sofort auffindbar, Nebenaktionen stoeren nicht.
- Die App bleibt auch mit mehr Vereinen, Gruppen, Mannschaften und Trainings uebersichtlich.

### Gewuenschter Eindruck

- modern
- ruhig
- vertrauenswuerdig
- mobil gut benutzbar
- eher produktiv als verspielt

## Designprinzipien

### 1. Ein Screen, ein Fokus

- Jede Seite bekommt eine primaere Aufgabe.
- Sekundaere Aufgaben wandern in aufklappbare Bereiche, Sheets, Dialoge oder Untersektionen.

### 2. Progressive Offenlegung

- Erst Ueberblick, dann Details, dann Formulare.
- Nutzer sollen nicht sofort alle Eingaben und Listen gleichzeitig sehen.

### 3. Mobile-first Denkweise

- Inhalte muessen auf schmalen Screens ohne horizontales Chaos funktionieren.
- Aktionen muessen mit dem Daumen erreichbar und als Touch-Ziele gross genug sein.

### 4. Wiederkehrende Muster statt Einzelloesungen

- Gleiche Arten von Inhalten bekommen gleiche Struktur.
- Beispiel: Listen, Statusboxen, Aktionsleisten, Bereichsheader, Leerezustaende.

### 5. Weniger Techniksprache in der UI

- Keine technischen Eingabefelder wie manuelle IDs, wenn die App den Kontext schon kennt.
- Nutzer werden durch Auswahl und Kontext gefuehrt.

## Aktuelle Hauptprobleme

- `src/views/club_detail.rs` ist funktional stark, aber visuell und strukturell ueberladen.
- `src/views/group_detail.rs` mischt Auswahl, Zuweisung und Planung zu dicht.
- `src/views/navbar.rs` wird mit dynamischen Gruppenlinks unruhig und skaliert schlecht.
- `src/views/home.rs` ist nur ein Redirect und bietet keine echte Startlogik.
- `src/views/clubs.rs` ist funktional okay, aber noch zu formularzentriert.
- Auth-Seiten sind klarer als die Admin-Seiten, aber gestalterisch noch nicht auf demselben Systemniveau.
- `assets/styling/main.css` und `assets/styling/navbar.css` liefern Basisstruktur, aber noch keine starke Hierarchie fuer komplexe Screens.

## Zielstruktur der App

## Globale Navigation

Die Hauptnavigation soll stabil, kurz und rollenfest sein.

### Ziel fuer Desktop

- Logo oder App-Name links
- 2-4 primaere Navigationspunkte in der Mitte oder daneben
- Nutzerbereich rechts

### Ziel fuer Mobile/App

- Top-Bar mit Titel und Nutzeraktionen
- darunter entweder:
  - kompakte Segment-Navigation pro Screen
  - oder spaeter eine Bottom-Navigation fuer die wichtigsten Bereiche

### Empfohlene Hauptnavigation

- `Dashboard`
- `Vereine` nur fuer System-Admins
- keine dynamischen Gruppenlinks im globalen Header

Gruppen, Teams und Trainings gehoeren in den Seitenkontext, nicht in die globale Hauptnavigation.

## Seitenarchitektur

## 1. `Home`

Datei: `src/views/home.rs`

### Zielbild

- `Home` sollte nicht nur ein technischer Redirect sein.
- Entweder:
  - echte Startseite fuer nicht eingeloggte Nutzer
  - oder saubere Router-Weiterleitung ohne sichtbaren Zwischenzustand

### Empfehlung

- Kurzfristig: redirectfrei und unauffaellig halten
- Mittelfristig: echte Landing- oder Einstiegseite fuer Web

### Moegliche Inhalte einer echten Startseite

- kurzer App-Nutzen
- Einstieg `Anmelden`
- Einstieg `Mit Einladung registrieren`
- Hinweis fuer Vereine und Trainer

## 2. `Dashboard`

Datei: `src/views/dashboard.rs`

### Zielbild

Das Dashboard wird zur zentralen Arbeitsstartseite.

Es soll nicht alles zeigen, sondern den Nutzer in seine naechsten Aufgaben fuehren.

### Zielaufbau

- Begruessungsbereich mit Rollenhinweis
- Bereich `Heute / Naechste Schritte`
- Bereich `Meine Gruppen` oder `Meine Mannschaften`
- Bereich `Kommende Trainings`
- bei Admins zusaetzlich `Vereine verwalten`

### Verbesserungen

- Trainings und Gruppen nicht nur listen, sondern als handlungsorientierte Karten zeigen.
- Wichtige Zustände hervorheben:
  - offene Zuweisungen
  - keine Mannschaft zugeordnet
  - keine Trainings geplant
- Dashboard als Ersatz fuer ueberladene globale Navigation staerken.

### Mobile-Ziel

- Karten untereinander
- kurze Aktionsbuttons
- keine breite horizontale Navigation im Contentbereich

## 3. `Clubs`

Datei: `src/views/clubs.rs`

### Zielbild

- Die Seite ist eine ruhige Vereinsuebersicht mit schneller Suche, klarer Liste und sekundärer Erstellaktion.

### Zielaufbau

- Seitenkopf mit Titel und kurzer Beschreibung
- Aktionsbutton `Verein anlegen`
- Vereinsliste als scanbare Cards oder Item-Liste
- optional spaeter Suchfeld

### Verbesserungen

- Formular `Verein anlegen` nicht permanent offen anzeigen.
- Stattdessen Button plus Dialog oder Sheet.
- Bestehende Vereine visueller priorisieren als das Erstellen.

### Mobile-Ziel

- Liste als gut tappbare Eintraege
- neuer Verein in einem Sheet statt in dauerhaft offenem Formular

## 4. `ClubDetail`

Datei: `src/views/club_detail.rs`

### Zielbild

`ClubDetail` wird von einer langen Verwaltungsseite zu einer klar gegliederten Steuerzentrale fuer einen Verein.

### Neue Informationsarchitektur

- Kopfbereich mit:
  - Vereinsname
  - kurze Einordnung
  - primaere Aktionen
- darunter Tabs oder segmentierte Bereiche:
  - `Uebersicht`
  - `Gruppen`
  - `Einladungen`
  - spaeter optional `Mitglieder`

### Bereich `Uebersicht`

- Kennzahlen:
  - Anzahl Gruppen
  - Anzahl Trainer
  - Anzahl Mannschaften
  - Anzahl aktiver Einladungen
- Letzte oder wichtige Aktionen
- Schnellaktionen:
  - `Gruppe anlegen`
  - `Spieler-Code erstellen`

### Bereich `Gruppen`

- Gruppenliste als Accordion oder kompakte Cards
- Jede Gruppe zeigt zuerst nur:
  - Name
  - Anzahl Trainer
  - Anzahl Mannschaften
  - wichtige offene Aktionen
- Details erst nach Oeffnen

### Innerhalb einer Gruppe

Pro Gruppe zwei direkt sichtbare Arbeitsbereiche:

- `Trainer`
- `Mannschaften`

Einladungen gehoeren nicht in den Hauptfluss, sondern in eine bewusst zurueckhaltende Aktionsflaeche wie `Mehr Aktionen`.

### Bereich `Einladungen`

- Vereinsweite Einladungen von gruppenbezogenen Einladungen sauber trennen.
- Neue Codes nur bei Bedarf ueber eine eigene Aktionsflaeche oder ein Aktionsmenue erzeugen.
- Aktive Codes als kurze Liste mit:
  - Rolle
  - Gueltigkeit
  - Copy-Aktion
  - Widerrufen

### Bereich `Mannschaften`

- Mannschaften zuerst kompakt zeigen.
- Spielerlisten nur im aufgeklappten Zustand zeigen.
- Spieler hinzufuegen nicht dauerhaft als offenes Formular, sondern per Button oder Inline-Expansion.

### Erwartete UI-Muster

- `Tabs` fuer die Hauptbereiche
- `Accordion` oder `Collapsible` fuer Gruppen
- `Sheet` oder `Dialog` fuer Anlegen und Zuweisen
- `AlertDialog` fuer riskante Aktionen

### Mobile-Ziel

- Tabs als scrollbare Segmente oder gestapelte Sektionen
- Detailbearbeitung in Sheets statt in breiten Inline-Formen
- klare vertikale Reihenfolge statt verschachtelter Karten in Karten in Karten

## 5. `GroupDetail`

Datei: `src/views/group_detail.rs`

### Zielbild

Die Seite wird zu einer gefuehrten Arbeitsseite fuer Trainer.

### Neue Struktur

- Kopfbereich mit Gruppe und Verein
- darunter ein klarer Ablauf:
  - `1. Mannschaft waehlen`
  - `2. Spieler organisieren`
  - `3. Training planen`
  - `4. Kommende Trainings`

### Konkrete UI-Aenderungen

- Ausgewaehlte Mannschaft als aktive Kontextkarte anzeigen.
- Keine manuelle `team_id` mehr.
- Stattdessen Zielgruppe direkt aus der UI bestimmen:
  - ganze Gruppe
  - aktuell ausgewaehlte Mannschaft
- Spieler ohne Mannschaft als fokussierte Zuweisungsliste mit aktivem Zielteam.
- Trainingserstellung als kompakte Card oder Sheet mit guter Reihenfolge:
  - Titel
  - Zielgruppe
  - Datum und Zeit
  - Ort
  - Beschreibung

### Kommende Trainings

- Liste kompakter machen
- Status-Badges einfuehren
- wichtige Metadaten konsistent darstellen

### Mobile-Ziel

- Mannschaftsauswahl horizontal als Chips oder vertikale Tapp-Liste
- Trainingserstellung in Sheet oder eigener Untersektion
- Zuweisungsliste grossflaechig tappbar

## 6. `Login` und `Register`

Dateien:

- `src/views/login.rs`
- `src/views/register.rs`

### Zielbild

- Beide Seiten sollen dieselbe visuelle Sprache wie die App verwenden.
- Auth darf moderner aussehen als heute, aber nicht wie eine fremde Landingpage.

### Verbesserungen

- klarer zentrierter Auth-Container
- kurze Nutzenbeschreibung
- bessere visuelle Trennung von Formular und Hilfetext
- Einladungskontext in `Register` deutlich sichtbarer machen
- gute mobile Abstaende und klare Formularhierarchie

### Mobile-Ziel

- maximal einspaltig
- starke Lesbarkeit
- grosse Eingabefelder und Buttons

## Designsystem und visuelle Sprache

## Gewuenschte Stilrichtung

Nicht generisch-glossy, sondern modern-produktiv.

Empfohlene Richtung:

- helle, ruhige Flaechen
- starke Typografie-Hierarchie
- klare Linien und gute Abstaende
- zurueckhaltende Farbakzente fuer Aktionen und Status
- eher produktive Vereinssoftware als Marketing-Seite

## Layoutprinzipien

- maximaler Content-Container fuer Desktop
- klare vertikale Rhythmik
- wiederkehrende Seitenkoepfe
- konsistente Kartenabstaende
- definierte Aktionsbereiche oben oder unten in Sektionen

## Typografie

- Titel klar groesser und mit mehr Gewicht
- Beschreibungstexte ruhiger und kuerzer
- Metadaten visuell sekundär
- keine gleichstarke Darstellung fuer Titel, Metatext und Aktionshinweis

## Komponentenstrategie

Vorhandene UI-Komponenten koennen genutzt werden und sollten das Rueckgrat des Refactors bilden.

Bereits verfuegbare Bausteine, die besonders relevant sind:

- `tabs`
- `accordion`
- `collapsible`
- `sheet`
- `dialog`
- `alert_dialog`
- `select`
- `combobox`
- `badge`
- `toast`
- `skeleton`
- `sidebar`

## Empfohlene Nutzung der Komponenten

### `Tabs`

- fuer Hauptbereiche innerhalb einer Seite
- z. B. `ClubDetail`

### `Accordion` oder `Collapsible`

- fuer Gruppen und Mannschaften mit viel Detailtiefe

### `Sheet`

- fuer mobilefreundliche Anlege- und Bearbeitungsflows
- z. B. `Verein anlegen`, `Gruppe anlegen`, `Training planen`

### `Dialog`

- fuer kurze fokussierte Desktop-Dialoge

### `AlertDialog`

- fuer `Entfernen`, `Widerrufen`, spaeter `Loeschen`

### `Select` oder `Combobox`

- fuer Mannschaftsauswahl und kuenftig auch Nutzerzuweisungen
- Benutzername-Freitext sollte mittelfristig ersetzt werden

### `Badge`

- fuer Status, Rollen und Zuordnungsarten

### `Toast`

- fuer kurze Erfolgsmeldungen statt ueberall fest eingebauter Statusboxen

### `Skeleton`

- fuer ruhige Ladezustaende bei Listen und Detailseiten

## Responsives Verhalten

## Desktop

- mehrspaltige Layouts nur dort, wo sie wirklichen Mehrwert geben
- Verwaltungslisten koennen dichter sein
- Formulare duerfen inline erscheinen, wenn genug Platz da ist

## Tablet

- Desktop-Struktur vereinfachen
- aufklappbare Bereiche bevorzugen
- keine zu breiten Werkzeugleisten

## Smartphone und installierte App

- einspaltige Hauptstruktur
- Aktionen vorzugsweise in Sheets oder klare Primärbuttons
- Header kompakt halten
- wenige globale Navigationspunkte
- keine langen Inline-Formbereiche zwischen Listen

## Web-und-App-Kompatibilitaet

Der Refactor sollte nicht zwei getrennte UIs bauen, sondern ein gemeinsames System mit unterschiedlichen Praesentationsmustern.

### Gemeinsame Regeln

- gleiche Screen-Struktur
- gleiche Benennungen
- gleiche Aktionshierarchie
- gleiche States fuer leer, laden, erfolg, fehler

### Unterschiedliche Darstellung je Kontext

- Web darf breitere Listen und Split-Layouts nutzen
- App sollte staerker mit Sheets, Segmenten und vertikaler Fuehrung arbeiten

## Zustaende, Feedback und Accessibility

## Leere Zustaende

Jede wichtige Liste braucht einen guten Leerezustand.

Beispiele:

- keine Vereine
- keine Gruppen
- keine Mannschaften
- keine kommenden Trainings
- keine offenen Einladungen

Leerezustaende sollen immer mindestens eine naechste sinnvolle Aktion anbieten.

## Ladezustaende

- statt nur Text besser `Skeleton` oder strukturierter Platzhalter
- Ladezustand soll dem finalen Layout aehneln

## Erfolg und Fehler

- kurze Erfolgsmeldungen als `Toast`
- inline Fehler dort, wo der Fehler entstanden ist
- globale Fehler nur bei grossen Ladeproblemen

## Accessibility

- ausreichender Kontrast
- eindeutige Fokuszustände
- grosse Touch-Ziele
- semantisch korrekte Ueberschriftenhierarchie
- Dialoge und Sheets mit gutem Tastaturverhalten

## Inhalte und Sprache

Alle UI-Texte bleiben auf Deutsch.

## Sprachprinzipien

- kurz
- klar
- aufgabenorientiert
- keine unnoetige Fachsprache

### Beispiele fuer bessere Sprache

- statt `Spieler per Benutzername` eher `Spieler zuweisen`
- statt `Mannschaft optional` eher `Fuer wen ist dieses Training?`
- statt `Aktuell ausgewählt` eher `Aktive Mannschaft`

## Konkrete Refactor-Phasen

## Phase 5: Visuelles Feintuning

Ziel: modernes, kohärentes Endbild.

Umfang:

- Farben und Oberflaechenfeeling
- Typografie-Hierarchie
- Abstaende und Dichte
- Status-Badges und Mikrodetails

## Empfohlene Implementierungsreihenfolge

1. `src/views/navbar.rs`
2. `src/views/group_detail.rs`
3. `src/views/club_detail.rs`
4. `src/views/dashboard.rs`
5. `src/views/clubs.rs`
6. `src/views/login.rs` und `src/views/register.rs`
7. `assets/styling/main.css` und `assets/styling/navbar.css`

## Minimaler erster Meilenstein

Wenn der Refactor in kleinen Schritten starten soll, ist dieser Umfang der beste Einstieg:

1. Navbar auf feste Hauptnavigation reduzieren
2. `GroupDetail` in klaren Workflow umbauen
3. `ClubDetail` mit Tabs oder Accordions entlasten

Damit entsteht schon ein deutlich modernerer und ruhigerer Eindruck, ohne sofort alle Screens gleichzeitig neu zu bauen.

## Zweiter Meilenstein

4. Formulare in Dialoge oder Sheets verschieben
5. Toasts und Confirm-Dialoge einfuehren
6. Lade- und Leerezustaende vereinheitlichen

## Dritter Meilenstein

7. Auth-Seiten modernisieren
8. Dashboard als echte Steuerzentrale ausbauen
9. visuelles System in CSS nachziehen

## Offene Folgeideen

- globale Suche fuer Vereine, Gruppen und Nutzer
- Filter fuer grosse Vereinsstrukturen
- eigene Uebersichtsseite fuer Einladungen
- eigene Uebersichtsseite fuer offene Zuweisungen
- Trainingskalender oder Wochenansicht
- mobile Bottom-Navigation fuer spaetere App-Version
