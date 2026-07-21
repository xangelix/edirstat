# Menu Bar Dropdowns
file = File
view = Visualizza
help = Aiuto

# Menu Bar Actions
new-scan = 📁 Nuova analisi
save-snapshot = 💾 Salva istantanea
load-snapshot = 📖 Carica istantanea

# Menu Bar Status
idle = Inattivo

# View Menu Options
monospace-paths = 🅰 Percorsi a spaziatura fissa
highlight-duplicates = ✨ Evidenzia duplicati
treemap-borders = 🔳 Bordi della treemap
treemap-style =  Stile della treemap
treemap-style-vertical = Gradiente verticale
treemap-style-offset-vertical = Gradiente verticale sfalsato
treemap-style-diagonal = Gradiente diagonale
treemap-style-cushion = Ombreggiatura cushion
deletion-confirmation = 🗑 Conferma eliminazione
trash-confirmation = ♻ Conferma spostamento nel cestino
time-format = 🕒 Formato orario
language = 💬 Lingua
layout-mode = Modalità layout:
classic-layout = Layout classico
windirstat-layout = Layout WinDirStat
vis-mode-treemap = 📊 Treemap
vis-mode-plots = 📈 Grafici
select-plot-label = Seleziona grafico:
vis-mode-deduplicator = 👥 Ricerca duplicati
search-filter-label = 🔍 Filtra:

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ Mostra pannello sinistro (F9)
   *[false] ◀ Nascondi pannello sinistro (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ Mostra pannello destro (F11)
       *[false] ▶ Mostra pannello estensioni (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ Nascondi pannello destro (F11)
       *[false] ◀ Nascondi pannello estensioni (F11)
    }
}

collapse-all = ⏏ Comprimi tutto
about = ℹ Informazioni su eDirStat

# Status Indicators
scanning-disk = Analisi del disco...
scan-complete = Analisi completata
scan-cancelled = Analisi annullata
path-label = Percorso: { $path }
worker-threads = ⚡ { $count } Thread di lavoro
worker-threads-hover = Il numero di core CPU paralleli con work-stealing allocati per l'esplorazione delle cartelle.

# Stats Panel (Bottom)
directories-count = 📁 Cartelle: { $count }
files-count = 📄 File: { $count }
total-size = 💾 Dimensione totale: { $size }
elapsed-time = ⏱ Tempo: { $time }
scan-speed = ⚡ Velocità: { $speed }/s

# Selection Info
selection-path = Selezione: { $path }
selection-items = Selezione: { $count ->
    [one] 1 elemento
   *[other] { $count } elementi
}

# Plot Types
plot-size-distribution = 📊 Distribuzione dimensioni file
plot-age-size = 🌌 Età file vs Dimensione file
plot-dir-composition = 🍰 Composizione cartelle
plot-extension-boxplot = 📦 Dimensioni file per estensione
plot-temporal-timeline = ⏱ Cronologie temporali collegate
plot-deduplicator-waste = 👥 Spazio duplicati sprecato per estensione

# --- Deduplicator Strings ---
dedup-desc = Trova e rimuovi in sicurezza i file identici byte per byte utilizzando hash BLAKE3 crittograficamente sicuri.
dedup-how-it-works = ℹ Come funziona
dedup-min-size = Dimensione min file:
dedup-ignore-system = Ignora file di sistema
dedup-ignore-hidden = Ignora file nascosti
dedup-start-scan = ⚡ Avvia ricerca duplicati
dedup-scan-first = Analizza prima una cartella.
dedup-cancelled-msg = L'analisi è stata annullata. Avvia una nuova analisi per trovare i duplicati.
dedup-analyzing = Analisi dei file...
dedup-no-duplicates = Nessun duplicato trovato. Prova a ridurre la dimensione minima del file o analizza un'altra cartella.
no-permission = Nessun permesso
hardlink-badge = Hardlink
dedup-select-items = 🎯 Seleziona elementi...
dedup-select-all-but-oldest = 🎯 Tutti tranne il più vecchio
dedup-select-all-but-newest = 🎯 Tutti tranne il più recente
dedup-select-all-but-shortest = 🎯 Tutti tranne il percorso più corto
dedup-select-all-but-rootmost = 🎯 Tutti tranne il più vicino alla radice
dedup-select-all-but-longest = 🎯 Tutti tranne il percorso più lungo
dedup-pref-dir-pattern = Modello cartella preferito:
dedup-select-all-but-pref = 🎯 Tutti tranne la cartella preferita
dedup-clear-selection = ❌ Cancella selezione
dedup-link-menu = 🔗 Collega... ({ $count } file)
dedup-link-menu-disabled = 🔗 Collega... (0 file)
dedup-link-hardlinks = 🔗 Sostituisci selezionati con Hardlink
dedup-link-softlinks = 🔗 Sostituisci selezionati con Softlink (collegamenti simbolici)
dedup-remove-menu = 🗑 Rimuovi... ({ $count } file, { $size })
dedup-remove-menu-disabled = 🗑 Rimuovi... (0 file)
dedup-remove-trash = ♻ Sposta selezionati nel cestino
dedup-remove-delete = 🗑 Elimina selezionati permanentemente
dedup-warning-title = ⚠ AVVISO PERDITA DATI
dedup-warning-desc = { $count ->
    [one] Eliminazione di tutte le versioni di 1 file
   *[other] Eliminazione di tutte le versioni di { $count } file
}
dedup-warning-no-original = Nessuna copia originale rimarrà:
dedup-warning-details = Hai selezionato sia l'originale sia tutte le copie duplicate per i file elencati di seguito. La loro eliminazione comporterà probabilmente una perdita permanente di dati:
dedup-cancel-hover = Clicca per annullare l'analisi
scan-cancel-hover = Clicca per annullare l'analisi
dedup-current-label = Corrente
dedup-phase1-size = Fase 1/7: Raggruppamento dei file per dimensione...
dedup-phase1-filter = Fase 1/7: Filtro delle esclusioni sui candidati duplicati...
dedup-phase2-prefix = Fase 2/7: Hashing dei prefissi dei file (primi 4KB)...
dedup-phase3-midpoint = Fase 3/7: Hashing dei punti medi dei file...
dedup-phase4-suffix = Fase 4/7: Hashing dei suffissi dei file...
dedup-phase5-multirange = Fase 5/7: Hashing multi-intervallo per file grandi...
dedup-phase6-full = Fase 6/7: Hashing BLAKE3 completo dei candidati rimanenti...
dedup-phase7-validation = Fase 7/7: Validazione finale della marca temporale...
dedup-phase-finished = Finito in { $duration }! Trovati { $count } gruppi duplicati. Spazio potenziale recuperabile: { $space }
dedup-scan-cancelled-with-error = Analisi annullata: { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = Nome file
dedup-hdr-directory = Cartella principale
dedup-hdr-size = Dimensione
dedup-hdr-reclaimable = Recuperabile
dedup-hdr-created = Creato
dedup-hdr-modified = Modificato
dedup-copies-selected = ({ $count ->
    [one] 1 copia selezionata
   *[other] { $count } copie selezionate
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ Dettagli
explorer-deselect-hover = Deseleziona elementi
explorer-deselect-single-hover = Deseleziona elemento
explorer-selected-items-count = { $count ->
    [one] 1 elemento selezionato
   *[other] { $count } elementi selezionati
}
explorer-total-size = Dimensione totale: { $size }
explorer-files = File: { $count }
explorer-directories = Cartelle: { $count }
explorer-actions-title = Azioni
explorer-actions-operations = Operazioni:
explorer-action-refresh-hover = Aggiorna tutte le sotto-cartelle selezionate
explorer-grid-type = Tipo:
explorer-grid-size = Dimensione:
explorer-grid-bytes = Byte:
explorer-grid-items = Elementi:
explorer-grid-files = File:
explorer-grid-subdirs = Sotto-cartelle:
explorer-grid-user = Utente:
explorer-grid-group = Gruppo:
explorer-grid-permissions = Permessi:
explorer-grid-path = Percorso completo:

# Explorer Type Names
type-symlink = Collegamento simbolico
type-directory = Cartella
type-file = File

# Explorer Actions
explorer-action-copy-path = 📋 Copia percorso
explorer-action-open-manager = 🗁 Apri gestore file
explorer-action-refresh-subtree = 🔄 Aggiorna sotto-albero
explorer-action-move-trash = ♻ Sposta nel cestino
explorer-action-delete-permanently = 🗑 Elimina permanentemente
explorer-action-refresh-directory = 🔄 Aggiorna cartella

# Explorer Empty State
explorer-empty-state = Clicca su 'Nuova analisi' per esplorare l'uso del disco.
choose-an-option = Scegli un'opzione
web-viewer = Visualizzatore Web
load-demo = 👁 Carica snapshot di demo
placeholder-treemap = Il file system scansionato verrà visualizzato qui come mappa ad albero.
placeholder-plots = Il file system analizzato verrà rappresentato graficamente qui.

# --- Extensions Panel ---
extensions-header = 📂 Estensioni
extensions-empty = Nessuna statistica ancora raccolta.
extensions-hover-files = File: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = Sali di un livello
op-refresh-entire-scan = Aggiorna analisi completa
op-refresh-directory = Aggiorna cartella
op-open-file-manager = Apri nel gestore file
op-open-terminal = Apri terminale qui
op-copy-path = Copia percorso
op-copy-name = Copia nome
op-move-trash = Sposta nel cestino
op-permanently-delete = Elimina permanentemente

# Toast Notifications
toast-already-root = Sei già al livello principale
toast-navigated-up = Sali di un livello navigato
toast-refreshing-scan = Aggiornamento dell'analisi in corso...
toast-refreshing-dir = Aggiornamento delle cartelle selezionate...
toast-opened-manager = Aperto nel gestore file: { $path }
toast-failed-open-manager = Impossibile aprire nel gestore file: { $error }
toast-opened-terminal = Terminale aperto in: { $path }
toast-failed-open-terminal = Impossibile aprire il terminale: { $error }
toast-copied-paths = { $count ->
    [one] 1 percorso copiato negli appunti
   *[other] { $count } percorsi copiati negli appunti
}
toast-copied-names = { $count ->
    [one] 1 nome copiato negli appunti
   *[other] { $count } nomi copiati negli appunti
}

# --- Modals ---
modal-remember-confirmation = Ricorda la conferma per tutti i file e le cartelle future
modal-process-multiple = Stai per elaborare { $count } file/elementi duplicati:
modal-process-single = Stai per elaborare il seguente percorso:
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ AVVISO ELIMINAZIONE PERMANENTE
modal-delete-header = ⚠ Avviso di eliminazione permanente!
modal-delete-info = Dimensione totale: { $size }
modal-delete-warning = Questa è un'eliminazione ricorsiva. Tutti i file, le cartelle e le sotto-cartelle sotto i percorsi selezionati verranno eliminati permanentemente e non potranno essere recuperati (saltando il cestino).
modal-delete-checkbox = Capisco che i file verranno eliminati permanentemente e non potranno essere recuperati.
modal-delete-confirm = 🗑 Sì, elimina permanentemente

modal-trash-title = ♻ SPOSTA NEL CESTINO
modal-trash-header = ♻ Sposta nel cestino
modal-trash-info = Dimensione totale: { $size }
modal-trash-warning = Questo sposterà i percorsi selezionati e tutti i loro contenuti nel cestino del sistema, da dove potranno essere recuperati o eliminati permanentemente in seguito.
modal-trash-checkbox = Confermo che desidero spostare questo elemento nel cestino.
modal-trash-confirm = ♻ Sì, sposta nel cestino

modal-delete-duplicates-title = ⚠ AVVISO DEDUPLICAZIONE PERMANENTE
modal-delete-duplicates-header = ⚠ Avviso di eliminazione permanente duplicati!
modal-delete-duplicates-info = Spazio totale da recuperare: { $size }
modal-delete-duplicates-warning = Tutti i file selezionati verranno eliminati permanentemente e non potranno essere recuperati (saltando il cestino).
modal-delete-duplicates-checkbox = Capisco che i file verranno eliminati permanentemente e non potranno essere recuperati.
modal-delete-duplicates-confirm = 🗑 Sì, elimina selezionati permanentemente

modal-trash-duplicates-title = ♻ SPOSTA DUPLICATI NEL CESTINO
modal-trash-duplicates-header = ♻ Sposta duplicati nel cestino
modal-trash-duplicates-info = Spazio totale da recuperare: { $size }
modal-trash-duplicates-warning = Tutti i file selezionati verranno spostati nel cestino del sistema.
modal-trash-duplicates-checkbox = Confermo che desidero spostare questi file nel cestino.
modal-trash-duplicates-confirm = ♻ Sì, sposta selezionati nel cestino

modal-hardlink-duplicates-title = 🔗 SOSTITUISCI DUPLICATI CON HARDLINK
modal-hardlink-duplicates-header = 🔗 Sostituisci duplicati con Hardlink
modal-hardlink-duplicates-info = Totale file da elaborare: { $count }. Dimensione virtuale cumulata: { $size }
modal-hardlink-duplicates-warning = Questo eliminerà i file duplicati selezionati e li sostituirà con hardlink a livello di file system que puntano al file originale rimanente in ciascun gruppo. Ciò conserva visivamente i file liberando spazio di archiviazione fisico reale.
modal-hardlink-duplicates-checkbox = Confermo que desidero sostituire i file selezionati con hardlink.
modal-hardlink-duplicates-confirm = 🔗 Sì, sostituisci con Hardlink

modal-softlink-duplicates-title = 🔗 SOSTITUISCI DUPLICATI CON COLLEGAMENTI SIMBOLICI
modal-softlink-duplicates-header = 🔗 Sostituisci duplicati con collegamenti simbolici
modal-softlink-duplicates-info = Totale file da elaborare: { $count }. Dimensione virtuale cumulata: { $size }
modal-softlink-duplicates-warning = Questo eliminerà i file duplicati selezionati e li sostituirà con collegamenti simbolici (softlink) a livello di file system che puntano al file originale rimanente in ciascun gruppo. Ciò conserva visivamente i file liberando spazio di archiviazione fisico reale.
modal-softlink-duplicates-checkbox = Confermo che desidero sostituire i file selezionati con collegamenti simbolici.
modal-softlink-duplicates-confirm = 🔗 Sì, sostituisci con collegamenti simbolici

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ Il percorso non esiste!
modal-path-not-exist-msg = Errore: Il percorso che stai tentando di eliminare non esiste sul disco.
modal-close-btn = Chiudi
modal-details-label = Dettagli: 
modal-cancel-btn = Annulla

# Elevation Recommended Modal
modal-elevation-title = ⚠ Elevazione consigliata
modal-elevation-desc = eDirStat viene eseguito con privilegi utente standard per impostazione predefinita. Tuttavia, Windows limita strettamente l'accesso diretto all'handle fisico del disco agli account amministratore.
modal-elevation-mft-disabled = Driver NTFS MFT di Windows disabilitato
modal-elevation-mft-desc = Senza privilegi di amministratore, lo scanner MFT diretto non può essere inizializzato. L'analisi dei file utilizzerà il driver di attraversamento standard alternativo, riducendo le prestazioni dell'analisi fino a 20 volte.
modal-elevation-relaunch-prompt = Vuoi riavviare l'applicazione con privilegi di amministratore adesso?
modal-elevation-continue-std = Continua come utente standard
modal-elevation-relaunch-btn = 🛡 Riavvia come amministratore

# About Modal
modal-about-title = ℹ Informazioni su eDirStat
modal-about-author = Di: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-desc1 = Uno strumento ad alte prestazioni per l'analisi dello spazio su disco e la deduplicazione scritto in Rust.
modal-about-desc2 = Offre attraversamento parallelo delle cartelle tramite work-stealing, istantanee compresse senza analisi della disposizione per la deserializzazione, e treemap interattive e reattive.
modal-about-desc3 = Il deduplicatore integrato esegue una pipeline di hashing crittografico multifase per isolare in sicurezza i gruppi duplicati, calcolare lo spazio recuperabile e rispettare gli hardlink a livello di sistema.
modal-about-licenses-btn = Visualizza licenze open source
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ Come funziona la deduplicazione
modal-how-dedup-desc1 = Invece di confrontare direttamente i byte di ogni file (il che richiederebbe scansioni lente O(N²) in coppia), questo sistema utilizza una pipeline ottimizzata a 7 fasi per identificar i contenuti identici in modo sicuro ed efficiente.
modal-how-dedup-pipeline-title = La pipeline a 7 fasi:
modal-how-dedup-why-title = Perché questo è sufficiente?
modal-how-dedup-why-desc1 = Questo filtro multifase garantisce che vengano letti interamente solo i file con dimensione, prefisso, punto medio, suffisso e campioni di blocchi distribuiti identici. Infine, il confronto di un hash crittografico BLAKE3 a 256 bit offre un profilo di sicurezza paragonabile ai protocolli di trasferimento sicuri del settore, eliminando la necessità di lenti confronti byte per byte in coppia.

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. Partizionamento per dimensione
modal-how-dedup-step1-desc = I file sono raggruppati per la loro dimensione esatta in byte. Qualsiasi file con una dimensione unica viene scartato immediatamente, saltando completamente l'I/O del disco.
modal-how-dedup-step2-title = 2. Hashing del prefisso
modal-how-dedup-step2-desc = Vengono calcolati gli hash dei primi 4KB dei candidati rimanenti. Questo permette di filtrare rapidamente i file con intestazioni o formati di metadati differenti.
modal-how-dedup-step3-title = 3. Hashing del punto medio
modal-how-dedup-step3-desc = Viene calcolato l'hash di un blocco di 4KB al centro dei file rimanenti, rilevando differenze strutturali interne.
modal-how-dedup-step4-title = 4. Hashing del suffisso
modal-how-dedup-step4-desc = Vengono calcolati gli hash degli ultimi 4KB di dati. Questo è altamente efficace nell'identificare differenze nelle parti finali o nei metadati.
modal-how-dedup-step5-title = 5. Hashing multi-intervallo
modal-how-dedup-step5-desc = I file di grandi dimensioni (oltre 100MB) vengono sottoposti a campionamento periodico di blocchi su tutta la loro lunghezza per verificare la coerenza del contenuto senza leggere l'intero file.
modal-how-dedup-step6-title = 6. Hashing BLAKE3 completo
modal-how-dedup-step6-desc = Per i candidati rimanenti, viene calcolato un hash crittografico BLAKE3 completo. A causa dell'elevata resistenza alle collisioni di uno spazio a 256 bit, hash corrispondenti indicano un'astronomica improbabilità che i file differiscano, fornendo una prova di identità altamente affidabile senza richiedere confronti in coppia.
modal-how-dedup-step7-title = 7. Validazione della marca temporale
modal-how-dedup-step7-desc = Appena prima di visualizzare o eseguire qualsiasi azione di deduplicazione, l'applicazione verifica le marche temporali dei file sul disco per proteggersi da modifiche avvenute dopo la generazione dell'istantanea.

# Open Source Licenses Modal
modal-licenses-title = 📜 Licenze Open Source
modal-licenses-desc = Le seguenti librerie e crate di terze parti sono utilizzate in questa applicazione:

# Processing Modal
modal-processing-title = ⏳ Elaborazione...
modal-processing-deletion = Eliminazione di file e directory...
modal-processing-trash = Spostamento di file e directory nel cestino...
modal-processing-hardlink = Sostituzione dei duplicati con hardlink...
modal-processing-softlink = Sostituzione dei duplicati con softlink...

# Explorer Column Headers
explorer-hdr-name = Nome
explorer-hdr-percentage = Percentuale
explorer-hdr-size = Dimensione
explorer-hdr-items = Elementi
explorer-hdr-files = File
explorer-hdr-subdirs = Sotto-cart.
explorer-hdr-created = Creato
explorer-hdr-modified = Modificato

# Update Checker
update-checking = Verifica aggiornamenti in corso...
update-available = Nuova versione { $version } disponibile!
update-up-to-date = L'applicazione è aggiornata
update-failed = Verifica aggiornamenti fallita: { $error }

# Themes
theme = 🎨 Tema
theme-dark = Scuro
theme-high-contrast = Contrasto elevato
theme-light = Chiaro
theme-system = Sistema

# New Scan Options Modal
modal-scan-options-title = Opzioni nuova analisi
modal-scan-options-header = Avvia una nuova analisi
modal-scan-options-path-label = Percorso della cartella da analizzare:
modal-scan-options-paste-tooltip = Incolla dagli appunti
modal-scan-options-browse-tooltip = Sfoglia cartella...
modal-scan-options-scan-btn = Analizza
modal-scan-options-cancel-btn = Annulla
modal-scan-options-same-filesystem = Limita l'analisi allo stesso filesystem/volume
