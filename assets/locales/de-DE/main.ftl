# Menu Bar Dropdowns
file = Datei
view = Ansicht
help = Hilfe

# Menu Bar Actions
scan-directory = 📁 Verzeichnis scannen
save-snapshot = 💾 Snapshot speichern
load-snapshot = 📖 Snapshot laden

# Menu Bar Status
idle = Bereit

# View Menu Options
monospace-paths = Monospace-Pfade
highlight-duplicates = ✨ Duplikate hervorheben
treemap-borders = 🔳 Treemap-Rahmen
deletion-confirmation = 🗑 Bestätigung vor Löschen
trash-confirmation = ♻ Bestätigung vor In-den-Papierkorb-Verschieben
time-format = 🕒 Zeitformat
language = 💬 Sprache
layout-mode = Layout-Modus:
classic-layout = Klassisches Layout
windirstat-layout = WinDirStat-Layout
vis-mode-treemap = 📊 Treemap
vis-mode-plots = 📈 Diagramme
select-plot-label = Diagramm auswählen:
vis-mode-deduplicator = 👥 Duplikatsuche
search-filter-label = 🔍 Filter:

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ Linkes Panel anzeigen (F9)
   *[false] ◀ Linkes Panel ausblenden (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ Rechtes Panel anzeigen (F11)
       *[false] ▶ Erweiterungspanel anzeigen (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ Rechtes Panel ausblenden (F11)
       *[false] ◀ Erweiterungspanel ausblenden (F11)
    }
}

collapse-all = ⏏ Alles einklappen
about = ℹ Über eDirStat

# Status Indicators
scanning-disk = Dateisystem wird gescannt...
scan-complete = Scan abgeschlossen
path-label = Pfad: { $path }
worker-threads = ⚡ { $count } Worker-Threads
worker-threads-hover = Die Anzahl der parallelen CPU-Kerne für das Durchsuchen des Verzeichnisses (Work-Stealing).

# Stats Panel (Bottom)
directories-count = 📁 Ordner: { $count }
files-count = 📄 Dateien: { $count }
total-size = 💾 Gesamtgröße: { $size }
elapsed-time = ⏱ Zeit: { $time }
scan-speed = ⚡ Geschwindigkeit: { $speed }/s

# Selection Info
selection-path = Auswahl: { $path }
selection-items = Auswahl: { $count ->
    [one] 1 Element
   *[other] { $count } Elemente
}

# Plot Types
plot-size-distribution = 📊 Dateigrößenverteilung
plot-age-size = 🌌 Dateialter vs. Dateigröße
plot-dir-composition = 🍰 Verzeichniszusammensetzung
plot-extension-boxplot = 📦 Dateigrößen nach Erweiterung
plot-temporal-timeline = ⏱ Verknüpfte zeitliche Verläufe
plot-deduplicator-waste = 👥 Duplikat-Platzverschwendung nach Erweiterung

# --- Deduplicator Strings ---
dedup-desc = Suchen und sicheres Entfernen von Byte-für-Byte identischen Dateien mithilfe kryptografisch sicherer BLAKE3-Hashes.
dedup-how-it-works = ℹ Funktionsweise
dedup-min-size = Mindestgröße:
dedup-ignore-system = Systemdateien ignorieren
dedup-ignore-hidden = Versteckte Dateien ignorieren
dedup-start-scan = ⚡ Duplikat-Scan starten
dedup-scan-first = Bitte scannen Sie zuerst ein Verzeichnis.
dedup-cancelled-msg = Scan wurde abgebrochen. Starten Sie einen neuen Scan, um Duplikate zu finden.
dedup-analyzing = Analysiere Dateien...
dedup-no-duplicates = Keine Duplikate gefunden. Verringern Sie die Mindestgröße oder scannen Sie einen anderen Ordner.
no-permission = Keine Berechtigung
hardlink-badge = Hardlink
dedup-select-items = 🎯 Elemente auswählen...
dedup-select-all-but-oldest = 🎯 Alle außer der ältesten
dedup-select-all-but-newest = 🎯 Alle außer der neuesten
dedup-select-all-but-shortest = 🎯 Alle außer dem kürzesten Pfad
dedup-select-all-but-rootmost = 🎯 Alle außer der obersten (wurzelnächsten)
dedup-select-all-but-longest = 🎯 Alle außer dem längsten Pfad
dedup-pref-dir-pattern = Bevorzugtes Verzeichnismuster:
dedup-select-all-but-pref = 🎯 Alle außer dem bevorzugten Verzeichnis
dedup-clear-selection = ❌ Auswahl aufheben
dedup-link-menu = 🔗 Verknüpfen... ({ $count } Dateien)
dedup-link-menu-disabled = 🔗 Verknüpfen... (0 Dateien)
dedup-link-hardlinks = 🔗 Ausgewählte durch Hardlinks ersetzen
dedup-link-softlinks = 🔗 Ausgewählte durch Softlinks ersetzen
dedup-remove-menu = 🗑 Löschen... ({ $count } Dateien, { $size })
dedup-remove-menu-disabled = 🗑 Löschen... (0 Dateien)
dedup-remove-trash = ♻ Ausgewählte in den Papierkorb verschieben
dedup-remove-delete = 🗑 Ausgewählte dauerhaft löschen
dedup-warning-title = ⚠ WARNUNG VOR DATENVERLUST
dedup-warning-desc = { $count ->
    [one] Alle Versionen von 1 Datei werden gelöscht
   *[other] Alle Versionen von { $count } Dateien werden gelöscht
}
dedup-warning-no-original = Keine Originalkopie bleibt übrig:
dedup-warning-details = Sie haben sowohl das Original als auch alle Duplikate der unten aufgeführten Dateien ausgewählt. Das Löschen führt wahrscheinlich zu dauerhaftem Datenverlust:
dedup-cancel-hover = Klicken zum Abbrechen des Scans
dedup-current-label = Aktuell
dedup-phase1-size = Phase 1/7: Dateien nach Größe gruppieren...
dedup-phase1-filter = Phase 1/7: Ausschlusskriterien filtern...
dedup-phase2-prefix = Phase 2/7: Hashing der Dateianfänge (erste 4KB)...
dedup-phase3-midpoint = Phase 3/7: Hashing der Dateimitten...
dedup-phase4-suffix = Phase 4/7: Hashing der Dateiendungen...
dedup-phase5-multirange = Phase 5/7: Mehrbereichs-Hashing großer Dateien...
dedup-phase6-full = Phase 6/7: Vollständiges BLAKE3-Hashing der verbleibenden Kandidaten...
dedup-phase7-validation = Phase 7/7: Abschließende Überprüfung der Zeitstempel...
dedup-phase-finished = Fertig in { $duration }! { $count } Duplikatgruppen gefunden. Potenzial freizugebender Speicherplatz: { $space }
dedup-scan-cancelled-with-error = Scan wurde abgebrochen: { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = Dateiname
dedup-hdr-directory = Elternverzeichnis
dedup-hdr-size = Größe
dedup-hdr-reclaimable = Einsparbar
dedup-hdr-created = Erstellt
dedup-hdr-modified = Geändert
dedup-copies-selected = ({ $count ->
    [one] 1 Kopie ausgewählt
   *[other] { $count } Kopien ausgewählt
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ Details
explorer-deselect-hover = Auswahl aufheben
explorer-deselect-single-hover = Auswahl aufheben
explorer-selected-items-count = { $count ->
    [one] 1 ausgewähltes Element
   *[other] { $count } ausgewählte Elemente
}
explorer-total-size = Gesamtgröße: { $size }
explorer-files = Dateien: { $count }
explorer-directories = Ordner: { $count }
explorer-actions-title = Aktionen
explorer-actions-operations = Operationen:
explorer-action-refresh-hover = Alle ausgewählten Unterverzeichnisse aktualisieren
explorer-grid-type = Typ:
explorer-grid-size = Größe:
explorer-grid-bytes = Bytes:
explorer-grid-items = Elemente:
explorer-grid-files = Dateien:
explorer-grid-subdirs = Unterordner:
explorer-grid-user = Benutzer:
explorer-grid-group = Gruppe:
explorer-grid-permissions = Berechtigungen:
explorer-grid-path = Vollständiger Pfad:

# Explorer Type Names
type-symlink = Symbolischer Link
type-directory = Ordner
type-file = Datei

# Explorer Actions
explorer-action-copy-path = 📋 Pfad kopieren
explorer-action-open-manager = 🗁 Dateimanager öffnen
explorer-action-refresh-subtree = 🔄 Unterbaum aktualisieren
explorer-action-move-trash = ♻ In den Papierkorb verschieben
explorer-action-delete-permanently = 🗑 Dauerhaft löschen
explorer-action-refresh-directory = 🔄 Ordner aktualisieren

# Explorer Empty State
explorer-empty-state = Klicken Sie auf 'Verzeichnis scannen', um die Speicherplatzbelegung zu analysieren.
placeholder-treemap = Das gescannte Dateisystem wird hier als Treemap visualisiert.
placeholder-plots = Das gescannte Dateisystem wird hier grafisch dargestellt.

# --- Extensions Panel ---
extensions-header = 📂 Erweiterungen
extensions-empty = Noch keine Statistiken erfasst.
extensions-hover-files = Dateien: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = Eine Ebene nach oben
op-refresh-entire-scan = Gesamten Scan aktualisieren
op-refresh-directory = Ordner aktualisieren
op-open-file-manager = Im Dateimanager öffnen
op-open-terminal = Terminal hier öffnen
op-copy-path = Pfad kopieren
op-copy-name = Name kopieren
op-move-trash = In den Papierkorb verschieben
op-permanently-delete = Dauerhaft löschen

# Toast Notifications
toast-already-root = Bereits auf der obersten Ebene
toast-navigated-up = Eine Ebene nach oben navigiert
toast-refreshing-scan = Gesamter Scan wird aktualisiert...
toast-refreshing-dir = Ausgewählte Ordner werden aktualisiert...
toast-opened-manager = Im Dateimanager geöffnet: { $path }
toast-failed-open-manager = Dateimanager konnte nicht geöffnet werden: { $error }
toast-opened-terminal = Terminal geöffnet bei: { $path }
toast-failed-open-terminal = Terminal konnte nicht geöffnet werden: { $error }
toast-copied-paths = { $count ->
    [one] 1 Pfad in die Zwischenablage kopiert
   *[other] { $count } Pfade in die Zwischenablage kopiert
}
toast-copied-names = { $count ->
    [one] 1 Name in die Zwischenablage kopiert
   *[other] { $count } Namen in die Zwischenablage kopiert
}

# --- Modals ---
modal-remember-confirmation = Entscheidung für alle zukünftigen Dateien und Ordner merken
modal-process-multiple = Sie sind im Begriff, { $count } doppelte Dateien/Elemente zu verarbeiten:
modal-process-single = Sie sind im Begriff, den folgenden Pfad zu verarbeiten:
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ WARNUNG: DAUERHAFTES LÖSCHEN
modal-delete-header = ⚠ Warnung: Dauerhaftes Löschen!
modal-delete-info = Gesamtgröße: { $size }
modal-delete-warning = Dies ist ein rekursiver Löschvorgang. Alle Dateien, Ordner und Unterverzeichnisse unter den ausgewählten Pfaden werden dauerhaft gelöscht und können nicht wiederhergestellt werden (der Papierkorb wird umgangen).
modal-delete-checkbox = Ich verstehe, dass Dateien dauerhaft gelöscht werden und nicht wiederhergestellt werden können.
modal-delete-confirm = 🗑 Ja, dauerhaft löschen

modal-trash-title = ♻ IN DEN PAPIERKORB VERSCHIEBEN
modal-trash-header = ♻ In den Papierkorb verschieben
modal-trash-info = Gesamtgröße: { $size }
modal-trash-warning = Dies verschiebt die ausgewählten Pfade und all ihre Inhalte in den Papierkorb Ihres Systems, von wo sie später wiederhergestellt oder dauerhaft gelöscht werden können.
modal-trash-checkbox = Ich bestätige, dass ich dies in den Papierkorb verschieben möchte.
modal-trash-confirm = ♻ Ja, in den Papierkorb verschieben

modal-delete-duplicates-title = ⚠ WARNUNG VOR DAUERHAFTER DUPLIKATLÖSCHUNG
modal-delete-duplicates-header = ⚠ Warnung vor dauerhafter Duplikatslöschung!
modal-delete-duplicates-info = Freizugebender Speicherplatz insgesamt: { $size }
modal-delete-duplicates-warning = Alle ausgewählten Dateien werden dauerhaft gelöscht und können nicht wiederhergestellt werden (der Papierkorb wird umgangen).
modal-delete-duplicates-checkbox = Ich verstehe, dass Dateien dauerhaft gelöscht werden und nicht wiederhergestellt werden können.
modal-delete-duplicates-confirm = 🗑 Ja, Ausgewählte dauerhaft löschen

modal-trash-duplicates-title = ♻ DUPLIKATE IN DEN PAPIERKORB VERSCHIEBEN
modal-trash-duplicates-header = ♻ Duplikate in den Papierkorb verschieben
modal-trash-duplicates-info = Freizugebender Speicherplatz insgesamt: { $size }
modal-trash-duplicates-warning = Alle ausgewählten Dateien werden in den Papierkorb verschoben.
modal-trash-duplicates-checkbox = Ich bestätige, dass ich diese Dateien in den Papierkorb verschieben möchte.
modal-trash-duplicates-confirm = ♻ Ja, Ausgewählte in den Papierkorb verschieben

modal-hardlink-duplicates-title = 🔗 DUPLIKATE DURCH HARDLINKS ERSETZEN
modal-hardlink-duplicates-header = 🔗 Duplikate durch Hardlinks ersetzen
modal-hardlink-duplicates-info = Zu verarbeitende Dateien insgesamt: { $count }. Kumulierte virtuelle Größe: { $size }
modal-hardlink-duplicates-warning = Dies löscht die ausgewählten Duplikate und ersetzt sie durch Hardlinks auf Dateisystemebene, die auf die verbleibende Originaldatei der jeweiligen Gruppe verweisen. Dadurch bleiben die Dateien visuell erhalten, während der physische Speicherplatz freigegeben wird.
modal-hardlink-duplicates-checkbox = Ich bestätige, dass ich die ausgewählten Dateien durch Hardlinks ersetzen möchte.
modal-hardlink-duplicates-confirm = 🔗 Ja, durch Hardlinks ersetzen

modal-softlink-duplicates-title = 🔗 DUPLIKATE DURCH SOFTLINKS ERSETZEN
modal-softlink-duplicates-header = 🔗 Duplikate durch Softlinks ersetzen
modal-softlink-duplicates-info = Zu verarbeitende Dateien insgesamt: { $count }. Kumulierte virtuelle Größe: { $size }
modal-softlink-duplicates-warning = Dies löscht die ausgewählten Duplikate und ersetzt sie durch Softlinks (symbolische Links) auf Dateisystemebene, die auf die verbleibende Originaldatei verweisen. Dadurch bleiben die Dateien visuell erhalten, während der physische Speicherplatz freigegeben wird.
modal-softlink-duplicates-checkbox = Ich bestätige, dass ich die ausgewählten Dateien durch Softlinks ersetzen möchte.
modal-softlink-duplicates-confirm = 🔗 Ja, durch Softlinks ersetzen

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ Pfad existiert nicht!
modal-path-not-exist-msg = Fehler: Der Pfad, den Sie löschen möchten, existiert nicht auf dem Datenträger.
modal-close-btn = Schließen
modal-details-label = Details: 
modal-cancel-btn = Abbrechen

# Elevation Recommended Modal
modal-elevation-title = ⚠ Administratorrechte empfohlen
modal-elevation-desc = eDirStat wird standardmäßig mit normalen Benutzerrechten ausgeführt. Windows schränkt jedoch den direkten Zugriff auf physische Datenträger-Handles streng auf Administrator-Konten ein.
modal-elevation-mft-disabled = NTFS-MFT-Treiber unter Windows deaktiviert
modal-elevation-mft-desc = Ohne Administratorrechte kann der direkte MFT-Scanner nicht initialisiert werden. Die Dateianalyse greift auf den standardmäßigen Verzeichnisdurchlauf zurück, was die Scan-Performance um das Bis zu 20-Fache verringert.
modal-elevation-relaunch-prompt = Möchten Sie die Anwendung jetzt mit Administratorrechten neu starten?
modal-elevation-continue-std = Als Standardbenutzer fortfahren
modal-elevation-relaunch-btn = 🛡 Als Administrator neu starten

# About Modal
modal-about-title = ℹ Über eDirStat
modal-about-author = Von: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-desc1 = Ein hochperformantes Tool zur Analyse von Speicherplatz und Deduplizierung, geschrieben in Rust.
modal-about-desc2 = Bietet parallelen Verzeichnisdurchlauf mit Work-Stealing, komprimierte Snapshots ohne Parsing-Aufwand beim Laden sowie interaktive Treemaps.
modal-about-desc3 = Die integrierte Deduplizierung führt eine mehrstufige Hashing-Pipeline aus, um Duplikate sicher zu isolieren, freizugebenden Speicherplatz zu ermitteln und bestehende Hardlinks zu berücksichtigen.
modal-about-licenses-btn = Open-Source-Lizenzen anzeigen
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ Funktionsweise der Deduplizierung
modal-how-dedup-desc1 = Anstatt die Bytes jeder Datei direkt miteinander vergleichen (was langsame, paarweise O(N²)-Scans erfordert), nutzt dieses System eine optimierte 7-stufige Pipeline zur sicheren und effizienten Identifizierung identischer Inhalte.
modal-how-dedup-pipeline-title = Die 7-stufige Pipeline:
modal-how-dedup-why-title = Warum reicht das aus?
modal-how-dedup-why-desc1 = Dieser mehrstufige Filter stellt sicher, dass nur Dateien mit absolut identischer Größe, identischem Anfang, identischem Ende und verteilten Blockstichproben vollständig gelesen werden. Schließlich bietet der Vergleich eines kryptografischen 256-Bit BLAKE3-Hashes ein Sicherheitsniveau auf Augenhöhe mit gängigen Datenübertragungsprotokollen, was paarweise Vergleiche überflüssig macht.

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. Aufteilung nach Größe
modal-how-dedup-step1-desc = Dateien werden nach ihrer genauen Bytegröße gruppiert. Dateien mit einzigartiger Größe werden sofort aussortiert, um Festplattenzugriffe komplett zu vermeiden.
modal-how-dedup-step2-title = 2. Präfix-Hashing
modal-how-dedup-step2-desc = Die ersten 4KB der verbleibenden Kandidaten werden gehasht. Dadurch werden Dateien mit unterschiedlichen Headern oder Metadatenstrukturen schnell aussortiert.
modal-how-dedup-step3-title = 3. Mittelpunkt-Hashing
modal-how-dedup-step3-desc = Ein 4KB-Block aus der Mitte der verbleibenden Dateien wird gehasht, um interne strukturelle Unterschiede aufzudecken.
modal-how-dedup-step4-title = 4. Suffix-Hashing
modal-how-dedup-step4-desc = Die letzten 4KB werden gehasht. Dies ist hocheffektiv für Unterschiede im Endbereich oder bei abschließenden Metadaten.
modal-how-dedup-step5-title = 5. Mehrbereichs-Hashing
modal-how-dedup-step5-desc = Große Dateien (über 100MB) werden in regelmäßigen Abständen stichprobenartig über ihre gesamte Länge gehasht, um die Konsistenz zu prüfen, ohne die Datei komplett lesen zu müssen.
modal-how-dedup-step6-title = 6. Vollständiges BLAKE3-Hashing
modal-how-dedup-step6-desc = Für die verbleibenden Kandidaten wird ein vollständiger BLAKE3-Hash berechnet. Wegen der extremen Kollisionsresistenz eines 256-Bit-Raumes bedeuten identische Hashes mit astronomischer Sicherheit identische Inhalte, was paarweise Byte-Vergleiche erübrigt.
modal-how-dedup-step7-title = 7. Zeitstempel-Validierung
modal-how-dedup-step7-desc = Unmittelbar vor der Anzeige oder Durchführung einer Deduplizierungsaktion prüft die Anwendung die Zeitstempel der Dateien, um sich gegen Änderungen abzusichern, die seit der Erstellung des Snapshots stattgefunden haben.

# Open Source Licenses Modal
modal-licenses-title = 📜 Open-Source-Lizenzen
modal-licenses-desc = Folgende Drittanbieter-Bibliotheken werden in dieser Anwendung verwendet:


# Explorer Column Headers
explorer-hdr-name = Name
explorer-hdr-percentage = Prozent
explorer-hdr-size = Größe
explorer-hdr-items = Elemente
explorer-hdr-files = Dateien
explorer-hdr-subdirs = Unterordner
explorer-hdr-created = Erstellt
explorer-hdr-modified = Geändert

# Update Checker
update-checking = Nach Updates suchen...
update-available = Neue Version { $version } verfügbar!
update-up-to-date = Sie sind auf dem neuesten Stand
update-failed = Update-Prüfung fehlgeschlagen: { $error }
