# Menu Bar Dropdowns
file = Bestand
view = Weergave
help = Help

# Menu Bar Actions
scan-directory = 📁 Map scannen
save-snapshot = 💾 Momentopname opslaan
load-snapshot = 📖 Momentopname laden

# Menu Bar Status
idle = Inactief

# View Menu Options
monospace-paths = Monospace paden
highlight-duplicates = ✨ Duplicaten markeren
deletion-confirmation = 🗑 Bevestiging voor verwijderen
trash-confirmation = ♻ Bevestiging voor prullenbak
time-format = 🕒 Tijdnotatie
language = 💬 Taal
layout-mode = Lay-outmodus:
classic-layout = Klassieke lay-out
windirstat-layout = WinDirStat lay-out
vis-mode-treemap = 📊 Treemap
vis-mode-plots = 📈 Grafieken
select-plot-label = Selecteer grafiek:
vis-mode-deduplicator = 👥 Duplicatenzoeker
search-filter-label = 🔍 Filter:

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ Linkerpaneel tonen (F9)
   *[false] ◀ Linkerpaneel verbergen (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ Rechterpaneel tonen (F11)
       *[false] ▶ Extensiepaneel tonen (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ Rechterpaneel verbergen (F11)
       *[false] ◀ Extensiepaneel verbergen (F11)
    }
}

collapse-all = ⏏ Alles inklappen
about = ℹ Over eDirStat

# Status Indicators
scanning-disk = Schijf scannen...
scan-complete = Scan voltooid
path-label = Pad: { $path }
worker-threads = ⚡ { $count } Worker-threads
worker-threads-hover = Het aantal parallelle, work-stealing CPU-kernen toegewezen voor het doorzoeken van de map.

# Stats Panel (Bottom)
directories-count = 📁 Mappen: { $count }
files-count = 📄 Bestanden: { $count }
total-size = 💾 Totale grootte: { $size }
elapsed-time = ⏱ Tijd: { $time }
scan-speed = ⚡ Snelheid: { $speed }/s

# Selection Info
selection-path = Selectie: { $path }
selection-items = Selection: { $count ->
    [one] 1 item
   *[other] { $count } items
}

# Plot Types
plot-size-distribution = 📊 Bestandsgrootteverdeling
plot-age-size = 🌌 Bestandsleeftijd vs. Bestandsgrootte
plot-dir-composition = 🍰 Mappencompositie
plot-extension-boxplot = 📦 Bestandsgrootte per extensie
plot-temporal-timeline = ⏱ Gekoppelde chronologische tijdlijnen
plot-deduplicator-waste = 👥 Duplicaatverspilling per extensie

# --- Deduplicator Strings ---
dedup-desc = Vind en verwijder veilig byte-voor-byte identieke bestanden met behulp van cryptografisch veilige BLAKE3-hashes.
dedup-how-it-works = ℹ Hoe het werkt
dedup-min-size = Minimale bestandsgrootte:
dedup-ignore-system = Systeembestanden negeren
dedup-ignore-hidden = Verborgen bestanden negeren
dedup-start-scan = ⚡ Duplicatenscan starten
dedup-scan-first = Scan eerst een map.
dedup-cancelled-msg = Scan is geannuleerd. Start een nieuwe scan om duplicaten te zoeken.
dedup-analyzing = Bestanden analyseren...
dedup-no-duplicates = Geen duplicaten gevonden. Probeer de minimale bestandsgrootte te verlagen of scan een andere map.
no-permission = Geen toestemming
hardlink-badge = Hardlink
dedup-select-items = 🎯 Elementen selecteren...
dedup-select-all-but-oldest = 🎯 Alles behalve oudste
dedup-select-all-but-newest = 🎯 Alles behalve nieuwste
dedup-select-all-but-shortest = 🎯 Alles behalve kortste pad
dedup-select-all-but-rootmost = 🎯 Alles behalve dichtst bij root
dedup-select-all-but-longest = 🎯 Alles behalve langste pad
dedup-pref-dir-pattern = Voorkeurspatroon map:
dedup-select-all-but-pref = 🎯 Alles behalve voorkeursmap
dedup-clear-selection = ❌ Selectie wissen
dedup-link-menu = 🔗 Koppelen... ({ $count } bestanden)
dedup-link-menu-disabled = 🔗 Koppelen... (0 bestanden)
dedup-link-hardlinks = 🔗 Vervang geselecteerde door hardlinks
dedup-link-softlinks = 🔗 Vervang geselecteerde door softlinks
dedup-remove-menu = 🗑 Verwijderen... ({ $count } bestanden, { $size })
dedup-remove-menu-disabled = 🗑 Verwijderen... (0 bestanden)
dedup-remove-trash = ♻ Geselecteerde naar prullenbak verplaatsen
dedup-remove-delete = 🗑 Geselecteerde permanent verwijderen
dedup-warning-title = ⚠ WAARSCHUWING DATA VERLIES
dedup-warning-desc = { $count ->
    [one] Alle versies van 1 bestand worden verwijderd
   *[other] Alle versies van { $count } bestanden worden verwijderd
}
dedup-warning-no-original = Geen origineel exemplaar blijft over:
dedup-warning-details = U heeft zowel het origineel als alle duplicaat-kopieën voor de hieronder vermelde bestanden geselecteerd. Het verwijderen ervan zal waarschijnlijk leiden tot permanent gegevensverlies:
dedup-cancel-hover = Klik om de scan te annuleren
dedup-current-label = Actueel
dedup-phase1-size = Fase 1/7: Bestanden groeperen op grootte...
dedup-phase1-filter = Fase 1/7: Uitsluitingen filteren voor duplicaat-kandidaten...
dedup-phase2-prefix = Fase 2/7: Hashing bestands-prefixes (eerste 4KB)...
dedup-phase3-midpoint = Fase 3/7: Hashing bestands-middenpunten...
dedup-phase4-suffix = Fase 4/7: Hashing bestands-suffixes...
dedup-phase5-multirange = Fase 5/7: Multi-range hashing grote bestanden...
dedup-phase6-full = Fase 6/7: Volledige BLAKE3-hashing van resterende kandidaten...
dedup-phase7-validation = Fase 7/7: Definitieve validatie van tijdstempels...
dedup-phase-finished = Voltooid in { $duration }! { $count } duplicaatgroepen gevonden. Potentieel terug te winnen ruimte: { $space }
dedup-scan-cancelled-with-error = Scan is geannuleerd: { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = Bestandsnaam
dedup-hdr-directory = Bovenliggende map
dedup-hdr-size = Grootte
dedup-hdr-reclaimable = Terug te winnen
dedup-hdr-created = Gemaakt
dedup-hdr-modified = Gewijzigd
dedup-copies-selected = ({ $count ->
    [one] 1 kopie geselecteerd
   *[other] { $count } kopieën geselecteerd
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ Details
explorer-deselect-hover = Selectie van elementen wissen
explorer-deselect-single-hover = Selectie van element wissen
explorer-selected-items-count = { $count ->
    [one] 1 Geselecteerd item
   *[other] { $count } Geselecteerde items
}
explorer-total-size = Totale grootte: { $size }
explorer-files = Bestanden: { $count }
explorer-directories = Mappen: { $count }
explorer-actions-title = Acties
explorer-actions-operations = Bewerkingen:
explorer-action-refresh-hover = Vernieuw alle geselecteerde map-subbomen
explorer-grid-type = Type:
explorer-grid-size = Grootte:
explorer-grid-bytes = Bytes:
explorer-grid-items = Items:
explorer-grid-files = Bestanden:
explorer-grid-subdirs = Submappen:
explorer-grid-user = Gebruiker:
explorer-grid-group = Groep:
explorer-grid-permissions = Machtigingen:
explorer-grid-path = Volledig pad:

# Explorer Type Names
type-symlink = Symbolische koppeling
type-directory = Map
type-file = Bestand

# Explorer Actions
explorer-action-copy-path = 📋 Pad kopiëren
explorer-action-open-manager = 🗁 Bestandsbeheer openen
explorer-action-refresh-subtree = 🔄 Subboom vernieuwen
explorer-action-move-trash = ♻ Naar prullenbak verplaatsen
explorer-action-delete-permanently = 🗑 Permanent verwijderen
explorer-action-refresh-directory = 🔄 Map vernieuwen

# Explorer Empty State
explorer-empty-state = Klik op 'Map scannen' om het schijfgebruik te verkennen.
placeholder-treemap = Gescande bestandssysteem wordt hier als treemap weergegeven.
placeholder-plots = Gescande bestandssysteem wordt hier grafisch weergegeven.

# --- Extensions Panel ---
extensions-header = 📂 Extensies
extensions-empty = Nog geen statistieken verzameld.
extensions-hover-files = Bestanden: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = Niveau omhoog
op-refresh-entire-scan = Volledige scan vernieuwen
op-refresh-directory = Map vernieuwen
op-open-file-manager = Openen in bestandsbeheer
op-open-terminal = Terminal hier openen
op-copy-path = Pad kopiëren
op-copy-name = Naam kopiëren
op-move-trash = Naar prullenbak verplaatsen
op-permanently-delete = Permanent verwijderen

# Toast Notifications
toast-already-root = Al op het hoogste niveau
toast-navigated-up = Niveau omhoog genavigeerd
toast-refreshing-scan = Volledige scan vernieuwen...
toast-refreshing-dir = Geselecteerde map(pen) vernieuwen...
toast-opened-manager = Geopend in bestandsbeheer: { $path }
toast-failed-open-manager = Openen in bestandsbeheer mislukt: { $error }
toast-opened-terminal = Terminal geopend op: { $path }
toast-failed-open-terminal = Terminal openen mislukt: { $error }
toast-copied-paths = { $count ->
    [one] 1 pad naar klembord gekopieerd
   *[other] { $count } paden naar klembord gekopieerd
}
toast-copied-names = { $count ->
    [one] 1 naam naar klembord gekopieerd
   *[other] { $count } namen naar klembord gekopieerd
}

# --- Modals ---
modal-remember-confirmation = Bevestiging onthouden voor alle toekomstige bestanden en mappen
modal-process-multiple = U staat op het punt om { $count } duplicaatbestanden/-items te verwerken:
modal-process-single = U staat op het punt het volgende pad te verwerken:
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ WAARSCHUWING PERMANENTE VERWIJDERING
modal-delete-header = ⚠ Waarschuwing permanente verwijdering!
modal-delete-info = Totale grootte: { $size }
modal-delete-warning = Dit is een recursieve verwijdering. Alle bestanden, mappen en submappen onder de geselecteerde pad(en) worden permanent verwijderd en kunnen niet worden hersteld (de prullenbak wordt omzeild).
modal-delete-checkbox = Ik begrijp dat bestanden permanent worden verwijderd en niet kunnen worden hersteld.
modal-delete-confirm = 🗑 Ja, permanent verwijderen

modal-trash-title = ♻ NAAR PRULLENBAK VERPLAATSEN
modal-trash-header = ♻ Naar prullenbak verplaatsen
modal-trash-info = Totale grootte: { $size }
modal-trash-warning = Dit verplaatst the geselecteerde pad(en) en hun inhoud naar uw systeemprullenbak, waar ze later kunnen worden hersteld of definitief verwijderd.
modal-trash-checkbox = Ik bevestig dat ik dit naar de prullenbak wil verplaatsen.
modal-trash-confirm = ♻ Ja, naar prullenbak verplaatsen

modal-delete-duplicates-title = ⚠ WAARSCHUWING PERMANENTE DUPLICATEN VERWIJDERING
modal-delete-duplicates-header = ⚠ Waarschuwing permanente verwijdering duplicaten!
modal-delete-duplicates-info = Totale terug te winnen ruimte: { $size }
modal-delete-duplicates-warning = Alle geselecteerde bestanden worden permanent verwijderd en kunnen niet worden hersteld (de prullenbak wordt omzeild).
modal-delete-duplicates-checkbox = Ik begrijp dat bestanden permanent worden verwijderd en niet kunnen worden hersteld.
modal-delete-duplicates-confirm = 🗑 Ja, geselecteerde permanent verwijderen

modal-trash-duplicates-title = ♻ DUPLICATEN NAAR PRULLENBAK VERPLAATSEN
modal-trash-duplicates-header = ♻ Duplikaten naar prullenbak verplaatsen
modal-trash-duplicates-info = Totale terug te winnen ruimte: { $size }
modal-trash-duplicates-warning = Alle geselecteerde bestanden worden naar de prullenbak verplaatst.
modal-trash-duplicates-checkbox = Ik bevestig dat ik deze bestanden naar de prullenbak wil verplaatsen.
modal-trash-duplicates-confirm = ♻ Ja, geselecteerde naar prullenbak verplaatsen

modal-hardlink-duplicates-title = 🔗 DUPLICATEN VERVANGEN DOOR HARDLINKS
modal-hardlink-duplicates-header = 🔗 Duplicaten vervangen door hardlinks
modal-hardlink-duplicates-info = Totaal aantal te verwerken bestanden: { $count }. Cumulatieve virtual grootte: { $size }
modal-hardlink-duplicates-warning = Dit verwijdert de geselecteerde duplicaatbestanden en vervangt ze door hardlinks op bestandssysteemniveau die naar het resterende originele bestand in elke groep verwijzen. Hierdoor blijven bestanden visueel behouden terwijl fysieke opslagruimte wordt vrijgemaakt.
modal-hardlink-duplicates-checkbox = Ik bevestig dat ik geselecteerde bestanden wil vervangen door hardlinks.
modal-hardlink-duplicates-confirm = 🔗 Ja, vervangen door hardlinks

modal-softlink-duplicates-title = 🔗 DUPLICATEN VERVANGEN DOOR SOFTLINKS
modal-softlink-duplicates-header = 🔗 Duplicaten vervangen door softlinks
modal-softlink-duplicates-info = Totaal aantal te verwerken bestanden: { $count }. Cumulatieve virtuele grootte: { $size }
modal-softlink-duplicates-warning = Dit verwijdert de geselecteerde duplicaatbestanden en vervangt ze door softlinks (symbolische koppelingen) op bestandssysteemniveau die naar het resterende originele bestand in elke groep verwijzen. Hierdoor blijven bestanden visueel behouden terwijl fysieke opslagruimte wordt vrijgemaakt.
modal-softlink-duplicates-checkbox = Ik bevestig dat ik geselecteerde bestanden wil vervangen door softlinks.
modal-softlink-duplicates-confirm = 🔗 Ja, vervangen door softlinks

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ Pad bestaat niet!
modal-path-not-exist-msg = Fout: Het pad dat u probeert te verwijderen bestaat niet op de schijf.
modal-close-btn = Sluiten
modal-details-label = Details: 
modal-cancel-btn = Annuleren

# Elevation Recommended Modal
modal-elevation-title = ⚠ Uitvoering met verhoogde rechten aanbevolen
modal-elevation-desc = eDirStat wordt standaard uitgevoerd met normale gebruikersrechten. Windows beperkt echter de toegang tot fysieke schijf-handles strikt tot administrator-accounts.
modal-elevation-mft-disabled = Windows NTFS MFT-stuurprogramma uitgeschakeld
modal-elevation-mft-desc = Zonder administratorrechten kan de directe MFT-scanner niet worden geïnitialiseerd. De bestandsanalyse maakt gebruik van het standaard alternatieve stuurprogramma, wat de scanprestaties met wel factor 20 vermindert.
modal-elevation-relaunch-prompt = Wilt u de toepassing nu opnieuw opstarten met administratorrechten?
modal-elevation-continue-std = Doorgaan als standaardgebruiker
modal-elevation-relaunch-btn = 🛡 Opnieuw starten als administrator

# About Modal
modal-about-title = ℹ Over eDirStat
modal-about-author = Door: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-desc1 = Een krachtige tool voor schijfgebruiksanalyse en deduplicatie geschreven in Rust.
modal-about-desc2 = Met parallelle work-stealing mapdoorzoeking, gecomprimeerde momentopnames met zero-parsing lay-out deserialisatie, en responsieve, interactieve treemaps.
modal-about-desc3 = De geïntegreerde duplicatenzoeker voert een meerfasige cryptografische hashing-pipeline uit om duplicaatgroepen veilig te isoleren, terug te winnen ruimte te berekenen en rekening te houden met hardlinks op systeemniveau.
modal-about-licenses-btn = Open Source licenties bekijken
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ Hoe deduplicatie werkt
modal-how-dedup-desc1 = In plaats van de bytes van elk bestand rechtstreeks te vergelijken (wat trage, paarsgewijze O(N²)-scans vereist), maakt dit systeem gebruik van een sterk geoptimaliseerde 7-fasen pipeline om identieke inhoud veilig en efficiënt te identificeren.
modal-how-dedup-pipeline-title = De 7-fasen pipeline:
modal-how-dedup-why-title = Waarom is dit voldoende?
modal-how-dedup-why-desc1 = Dit meerfasige filter zorgt ervoor dat alleen bestanden met identieke grootte, prefix, middenpunt, suffix en verspreide blokstichproeven volledig worden gelezen. Ten slotte biedt het vergelijken van een 256-bits BLAKE3 cryptografische hash een beveiligingsprofiel dat vergelijkbaar is met beveiligde overdrachtsprotocollen van industriekwaliteit, waardoor trage, paarsgewijze byte-voor-byte vergelijkingen overbodig zijn.

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. Grootte partitioneren
modal-how-dedup-step1-desc = Bestanden worden gegroepeerd op hun exacte grootte in bytes. Elk bestand met een unieke grootte wordt onmiddellijk uitgesloten, waardoor schijf-I/O volledig wordt vermeden.
modal-how-dedup-step2-title = 2. Prefix-hashing
modal-how-dedup-step2-desc = De eerste 4KB van de resterende kandidaten worden gehasht. Dit filtert snel bestanden met verschillende headers of metadataformaten uit.
modal-how-dedup-step3-title = 3. Middenpunt-hashing
modal-how-dedup-step3-desc = Een blok van 4KB uit het midden van de resterende bestanden wordt gehasht om interne structurele verschillen te detecteren.
modal-how-dedup-step4-title = 4. Suffix-hashing
modal-how-dedup-step4-desc = De laatste 4KB aan gegevens wordt gehasht. Dit is zeer effectief bij het identificeren van verschillen in de staart van de bestanden of metadata.
modal-how-dedup-step5-title = 5. Multi-range hashing
modal-how-dedup-step5-desc = Grote bestanden (meer dan 100MB) ondergaan periodieke blokbemonstering over hun gehele lengte om de consistentie van de inhoud te controleren zonder het hele bestand te lezen.
modal-how-dedup-step6-title = 6. Volledige BLAKE3-hashing
modal-how-dedup-step6-desc = Voor resterende kandidaten wordt een volledige BLAKE3 cryptografische hash berekend. Vanwege de hoge botsingsresistentie van een 256-bits ruimte duiden overeenkomende hashes op een astronomische onwaarschijnlijkheid dat de bestanden verschillen, wat een zeer betrouwbaar bewijs van identiteit levert zonder paarsgewijze vergelijkingen.
modal-how-dedup-step7-title = 7. Tijdstempelvalidatie
modal-how-dedup-step7-desc = Vlak voor het weergeven of uitvoeren van een deduplicatie-actie controleert de toepassing de tijdstempels van de bestanden op de schijf om te beschermen tegen wijzigingen die hebben plaatsgevonden sinds het maken van de momentopname.

# Open Source Licenses Modal
modal-licenses-title = 📜 Open Source licenties
modal-licenses-desc = De volgende externe bibliotheken en crates worden in deze toepassing gebruikt:


# Explorer Column Headers
explorer-hdr-name = Naam
explorer-hdr-percentage = Percentage
explorer-hdr-size = Grootte
explorer-hdr-items = Items
explorer-hdr-files = Bestanden
explorer-hdr-subdirs = Submappen
explorer-hdr-created = Gemaakt
explorer-hdr-modified = Gewijzigd

# Update Checker
update-checking = Controleren op updates...
update-available = Nieuwe versie { $version } beschikbaar!
update-up-to-date = U bent up-to-date
update-failed = Updatecontrole mislukt: { $error }
