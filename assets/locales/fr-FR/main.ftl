# Menu Bar Dropdowns
file = Fichier
view = Affichage
help = Aide

# Menu Bar Actions
scan-directory = 📁 Analiser le répertoire
save-snapshot = 💾 Enregistrer l'instantané
load-snapshot = 📖 Charger l'instantané

# Menu Bar Status
idle = Inactif

# View Menu Options
monospace-paths = Polices à espacement fixe
highlight-duplicates = ✨ Mettre en évidence les doublons
treemap-borders = 🔳 Bordures de la treemap
deletion-confirmation = 🗑 Confirmation de suppression
trash-confirmation = ♻ Confirmation de mise à la corbeille
time-format = 🕒 Format de l'heure
language = 💬 Langue
layout-mode = Mode de disposition:
classic-layout = Disposition classique
windirstat-layout = Disposition WinDirStat
vis-mode-treemap = 📊 Treemap
vis-mode-plots = 📈 Graphiques
select-plot-label = Sélectionner le graphique:
vis-mode-deduplicator = 👥 Recherche de doublons
search-filter-label = 🔍 Filtrar:

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ Afficher le panneau gauche (F9)
   *[false] ◀ Masquer le panneau gauche (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ Afficher le panneau droit (F11)
       *[false] ▶ Afficher le panneau d'extensions (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ Masquer le panneau droit (F11)
       *[false] ◀ Masquer le panneau d'extensions (F11)
    }
}

collapse-all = ⏏ Tout réduire
about = ℹ À propos

# Status Indicators
scanning-disk = Analyse du disque...
scan-complete = Analyse terminée
path-label = Chemin: { $path }
worker-threads = ⚡ { $count } Threads de travail
worker-threads-hover = Le nombre de cœurs de processeur parallèles de vol de travail (work-stealing) alloués à l'exploration des répertoires.

# Stats Panel (Bottom)
directories-count = 📁 Dossiers: { $count }
files-count = 📄 Fichiers: { $count }
total-size = 💾 Taille totale: { $size }
elapsed-time = ⏱ Temps: { $time }
scan-speed = ⚡ Vitesse: { $speed }/s

# Selection Info
selection-path = Sélection: { $path }
selection-items = Sélection: { $count ->
    [one] 1 élément
   *[other] { $count } éléments
}

# Plot Types
plot-size-distribution = 📊 Distribution de la taille des fichiers
plot-age-size = 🌌 Âge vs Taille du fichier
plot-dir-composition = 🍰 Composition du répertoire
plot-extension-boxplot = 📦 Tailles de fichiers par extension
plot-temporal-timeline = ⏱ Chronologies temporelles liées
plot-deduplicator-waste = 👥 Espace doublon gaspillé par extension

# --- Deduplicator Strings ---
dedup-desc = Recherchez et supprimez en toute sécurité des fichiers identiques octet par octet à l'aide de hachages BLAKE3 cryptographiquement sécurisés.
dedup-how-it-works = ℹ Comment ça marche
dedup-min-size = Taille min. du fichier:
dedup-ignore-system = Ignorer les fichiers système
dedup-ignore-hidden = Ignorer les fichiers cachés
dedup-start-scan = ⚡ Démarrer l'analyse des doublons
dedup-scan-first = Veuillez analyser un répertoire d'abord.
dedup-cancelled-msg = L'analyse a été annulée. Démarrez une nouvelle analyse pour trouver des doublons.
dedup-analyzing = Analyse des fichiers...
dedup-no-duplicates = Aucun doublon trouvé. Essayez de réduire la taille minimale ou d'analyser un autre dossier.
no-permission = Pas d'autorisation
hardlink-badge = Lien physique
dedup-select-items = 🎯 Sélectionner les éléments...
dedup-select-all-but-oldest = 🎯 Tous sauf le plus ancien
dedup-select-all-but-newest = 🎯 Tous sauf le plus récent
dedup-select-all-but-shortest = 🎯 Tous sauf le chemin le plus court
dedup-select-all-but-rootmost = 🎯 Tous sauf le plus proche de la racine
dedup-select-all-but-longest = 🎯 Tous sauf le chemin le plus long
dedup-pref-dir-pattern = Motif de répertoire préféré:
dedup-select-all-but-pref = 🎯 Tous sauf le répertoire préféré
dedup-clear-selection = ❌ Effacer la sélection
dedup-link-menu = 🔗 Lier... ({ $count } fichiers)
dedup-link-menu-disabled = 🔗 Lier... (0 fichier)
dedup-link-hardlinks = 🔗 Remplacer les éléments sélectionnés par des liens physiques (hardlinks)
dedup-link-softlinks = 🔗 Remplacer les éléments sélectionnés par des liens symboliques (softlinks)
dedup-remove-menu = 🗑 Supprimer... ({ $count } fichiers, { $size })
dedup-remove-menu-disabled = 🗑 Supprimer... (0 fichier)
dedup-remove-trash = ♻ Déplacer les éléments sélectionnés vers la corbeille
dedup-remove-delete = 🗑 Supprimer définitivement les éléments sélectionnés
dedup-warning-title = ⚠ DANGER DE PERTE DE DONNÉES
dedup-warning-desc = { $count ->
    [one] Suppression de toutes les versions d'un fichier
   *[other] Suppression de toutes les versions de { $count } fichiers
}
dedup-warning-no-original = Aucune copie originale ne subsistera:
dedup-warning-details = Vous avez coché l'original ainsi que toutes les copies de doublons pour les fichiers énumérés ci-dessous. Les supprimer entraînera probablement une perte définitive de données:
dedup-cancel-hover = Cliquer pour annuler l'analyse
dedup-current-label = Actuel
dedup-phase1-size = Phase 1/7: Regroupement de tous les fichiers par taille...
dedup-phase1-filter = Phase 1/7: Filtrage des exclusions sur les candidats doublons...
dedup-phase2-prefix = Phase 2/7: Hachage des préfixes de fichiers (premiers 4 Ko)...
dedup-phase3-midpoint = Phase 3/7: Hachage des points médians des fichiers...
dedup-phase4-suffix = Phase 4/7: Hachage des suffixes de fichiers...
dedup-phase5-multirange = Phase 5/7: Hachage multi-gamme des fichiers volumineux...
dedup-phase6-full = Phase 6/7: Hachage BLAKE3 complet des candidats restants...
dedup-phase7-validation = Phase 7/7: Validation finale des horodatages...
dedup-phase-finished = Terminé en { $duration }! { $count } groupes de doublons trouvés. Espace potentiellement récupérable: { $space }
dedup-scan-cancelled-with-error = L'analyse a été annulée : { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = Nom du fichier
dedup-hdr-directory = Répertoire parent
dedup-hdr-size = Taille
dedup-hdr-reclaimable = Récupérable
dedup-hdr-created = Créé
dedup-hdr-modified = Modifié
dedup-copies-selected = ({ $count ->
    [one] 1 copie sélectionnée
   *[other] { $count } copies sélectionnées
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ Détails
explorer-deselect-hover = Désélectionner les éléments
explorer-deselect-single-hover = Désélectionner l'élément
explorer-selected-items-count = { $count ->
    [one] 1 élément sélectionné
   *[other] { $count } éléments sélectionnés
}
explorer-total-size = Taille totale: { $size }
explorer-files = Fichiers: { $count }
explorer-directories = Répertoires: { $count }
explorer-actions-title = Actions
explorer-actions-operations = Opérations:
explorer-action-refresh-hover = Rafraîchir tous les sous-arbres des répertoires sélectionnés
explorer-grid-type = Type:
explorer-grid-size = Taille:
explorer-grid-bytes = Octets:
explorer-grid-items = Éléments:
explorer-grid-files = Fichiers:
explorer-grid-subdirs = Sous-répertoires:
explorer-grid-user = Utilisateur:
explorer-grid-group = Groupe:
explorer-grid-permissions = Autorisations:
explorer-grid-path = Chemin complet:

# Explorer Type Names
type-symlink = Lien symbolique
type-directory = Répertoire
type-file = Fichier

# Explorer Actions
explorer-action-copy-path = 📋 Copier le chemin
explorer-action-open-manager = 🗁 Ouvrir le gestionnaire
explorer-action-refresh-subtree = 🔄 Rafraîchir le sous-arbre
explorer-action-move-trash = ♻ Déplacer vers la corbeille
explorer-action-delete-permanently = 🗑 Supprimer définitivement
explorer-action-refresh-directory = 🔄 Rafraîchir le répertoire

# Explorer Empty State
explorer-empty-state = Cliquez sur « Analyser le répertoire » pour explorer l'utilisation du disque.
placeholder-treemap = Le système de fichiers analysé sera visualisé sous forme de treemap ici.
placeholder-plots = Le système de fichiers analysé sera tracé ici.

# --- Extensions Panel ---
extensions-header = 📂 Extensions
extensions-empty = Aucune statistique collectée pour le moment.
extensions-hover-files = Fichiers: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = Monter d'un niveau
op-refresh-entire-scan = Actualiser toute l'analyse
op-refresh-directory = Actualiser le répertoire
op-open-file-manager = Ouvrir dans le gestionnaire de fichiers
op-open-terminal = Ouvrir un terminal ici
op-copy-path = Copier le chemin
op-copy-name = Copier le nom
op-move-trash = Déplacer vers la corbeille
op-permanently-delete = Supprimer définitivement

# Toast Notifications
toast-already-root = Déjà au niveau racine
toast-navigated-up = Navigation d'un niveau vers le haut
toast-refreshing-scan = Actualisation de toute l'analyse...
toast-refreshing-dir = Actualisation du ou des répertoires sélectionnés...
toast-opened-manager = Ouvert dans le gestionnaire de fichiers: { $path }
toast-failed-open-manager = Échec de l'ouverture du gestionnaire de fichiers: { $error }
toast-opened-terminal = Terminal ouvert à l'emplacement: { $path }
toast-failed-open-terminal = Échec de l'ouverture du terminal: { $error }
toast-copied-paths = { $count ->
    [one] 1 chemin copié dans le presse-papiers
   *[other] { $count } chemins copiés dans le presse-papiers
}
toast-copied-names = { $count ->
    [one] 1 nom copié dans le presse-papiers
   *[other] { $count } noms copiés dans le presse-papiers
}

# --- Modals ---
modal-remember-confirmation = Se souvenir de la confirmation pour tous les futurs fichiers et répertoires
modal-process-multiple = Vous êtes sur le point de traiter { $count } fichiers/éléments doublons :
modal-process-single = Vous êtes sur le point de traiter le chemin suivant :
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ AVERTISSEMENT DE SUPPRESSION DÉFINITIVE
modal-delete-header = ⚠ Avertissement de suppression définitive!
modal-delete-info = Taille totale: { $size }
modal-delete-warning = Il s'agit d'une suppression récursive. Tous les fichiers, dossiers et sous-répertoires situés sous le(s) chemin(s) sélectionné(s) seront définitivement supprimés et ne pourront pas être récupérés (en contournant la corbeille).
modal-delete-checkbox = Je comprends que les fichiers seront définitivement supprimés et ne pourront pas être récupérés.
modal-delete-confirm = 🗑 Oui, supprimer définitivement

modal-trash-title = ♻ DÉPLACER VERS LA CORBEILLE
modal-trash-header = ♻ Déplacer vers la corbeille
modal-trash-info = Taille totale: { $size }
modal-trash-warning = Cela déplacera le(s) chemin(s) sélectionné(s) et tout leur contenu vers la corbeille de votre système, d'où ils pourront être récupérés ou définitivement supprimés plus tard.
modal-trash-checkbox = Je confirme vouloir déplacer cet élément vers la corbeille.
modal-trash-confirm = ♻ Oui, déplacer vers la corbeille

modal-delete-duplicates-title = ⚠ AVERTISSEMENT DE DÉDUPLICATION DÉFINITIVE
modal-delete-duplicates-header = ⚠ Avertissement de suppression définitive des doublons!
modal-delete-duplicates-info = Espace total à récupérer: { $size }
modal-delete-duplicates-warning = Tous les fichiers sélectionnés seront définitivement supprimés et ne pourront pas être récupérés (en contournant la corbeille).
modal-delete-duplicates-checkbox = Je comprends que les fichiers seront définitivement supprimés et ne pourront pas être récupérés.
modal-delete-duplicates-confirm = 🗑 Oui, supprimer définitivement les éléments sélectionnés

modal-trash-duplicates-title = ♻ DÉPLACER LES DOUBLONS VERS LA CORBEILLE
modal-trash-duplicates-header = ♻ Déplacer les doublons vers la corbeille
modal-trash-duplicates-info = Espace total à récupérer: { $size }
modal-trash-duplicates-warning = Tous les fichiers sélectionnés seront déplacés vers la corbeille.
modal-trash-duplicates-checkbox = Je confirme vouloir déplacer ces fichiers vers la corbeille.
modal-trash-duplicates-confirm = ♻ Oui, déplacer les éléments sélectionnés vers la corbeille

modal-hardlink-duplicates-title = 🔗 REMPLACER LES DOUBLONS PAR DES LIENS PHYSIQUES
modal-hardlink-duplicates-header = 🔗 Remplacer les doublons par des liens physiques
modal-hardlink-duplicates-info = Total des fichiers à traiter: { $count }. Taille virtuelle cumulée: { $size }
modal-hardlink-duplicates-warning = Cela supprimera les fichiers doublons sélectionnés et les remplacera par des liens physiques (hardlinks) au niveau du système de fichiers pointant vers le fichier original restant de chaque groupe. Cela permet de conserver visuellement les fichiers tout en libérant de l'espace de stockage physique réel.
modal-hardlink-duplicates-checkbox = Je confirme vouloir remplacer les fichiers sélectionnés par des liens physiques.
modal-hardlink-duplicates-confirm = 🔗 Oui, remplacer par des liens physiques

modal-softlink-duplicates-title = 🔗 REMPLACER LES DOUBLONS PAR DES LIENS SYMBOLIQUES
modal-softlink-duplicates-header = 🔗 Remplacer les doublons par des liens symboliques
modal-softlink-duplicates-info = Total des fichiers à traiter: { $count }. Taille virtuelle cumulée: { $size }
modal-softlink-duplicates-warning = Cela supprimera les fichiers doublons sélectionnés et les remplacera par des liens symboliques (softlinks) au niveau du système de fichiers pointant vers le fichier original restant de chaque groupe. Cela permet de conserver visuellement les fichiers tout en libérant de l'espace de stockage physique réel.
modal-softlink-duplicates-checkbox = Je confirme vouloir remplacer les fichiers sélectionnés par des liens symboliques.
modal-softlink-duplicates-confirm = 🔗 Oui, remplacer par des liens symboliques

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ Le chemin n'existe pas!
modal-path-not-exist-msg = Erreur: Le chemin que vous tentez de supprimer n'existe pas sur le disque.
modal-close-btn = Fermer
modal-details-label = Détails: 
modal-cancel-btn = Annuler

# Elevation Recommended Modal
modal-elevation-title = ⚠ Élévation recommandée
modal-elevation-desc = Par défaut, eDirStat s'exécute avec les privilèges d'un utilisateur standard. Cependant, Windows limite strictement l'accès brut aux disques physiques aux comptes administrateurs.
modal-elevation-mft-disabled = Pilote NTFS MFT de Windows désactivé
modal-elevation-mft-desc = Sans privilèges d'administrateur, le scanner direct MFT ne peut pas s'initialiser. L'analyse des fichiers utilisera le scanner standard alternatif, ce qui réduit les performances d'analyse jusqu'à 20 fois.
modal-elevation-relaunch-prompt = Souhaitez-vous relancer l'application avec les privilèges d'administrateur maintenant?
modal-elevation-continue-std = Continuer comme utilisateur standard
modal-elevation-relaunch-btn = 🛡 Relancer en tant qu'administrateur

# About Modal
modal-about-title = ℹ À propos de eDirStat
modal-about-author = Par: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-desc1 = Un outil d'analyse d'espace disque et de déduplication haute performance écrit en Rust.
modal-about-desc2 = Offre une exploration parallèle des répertoires par vol de travail, des instantanés compressés sans analyse pour la désérialisation de la disposition, et des treemaps interactives et réactives.
modal-about-desc3 = Le déduplicateur intégré exécute un pipeline de hachage cryptographique multi-étapes pour isoler en toute sécurité les groupes de doublons, calculer l'espace récupérable et respecter les liens physiques du système.
modal-about-licenses-btn = Voir les licences open source
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ Fonctionnement de la déduplication
modal-how-dedup-desc1 = Plutôt que de comparer directement les octets de chaque fichier (ce qui nécessite des analyses lentes O(N²) par paires), ce système utilise un pipeline optimisé en 7 étapes pour identifier le contenu identique de manière sûre et efficace.
modal-how-dedup-pipeline-title = Le pipeline en 7 étapes:
modal-how-dedup-why-title = Pourquoi cela est-il suffisant?
modal-how-dedup-why-desc1 = Ce filtre multi-étapes garantit que seuls les fichiers présentant une taille, un préfixe, un point médian, un suffixe et des échantillons de blocs distribués identiques soient entièrement lus. Enfin, la comparaison d'un hachage cryptographique BLAKE3 de 256 bits offre un profil de sécurité comparable aux protocoles de transfert sécurisés de l'industrie, éliminant ainsi le besoin de comparaisons lentes octet par octet par paires.

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. Partitionnement par taille
modal-how-dedup-step1-desc = Les fichiers sont regroupés par leur taille exacte en octets. Tout fichier doté d'une taille unique est immédiatement rejeté, ce qui évite complètement les E/S disque.
modal-how-dedup-step2-title = 2. Hachage du préfixe
modal-how-dedup-step2-desc = Les premiers 4 Ko des candidats restants sont hachés. Cela permet de filtrer rapidement les fichiers ayant des en-têtes ou des formats de métadonnées différents.
modal-how-dedup-step3-title = 3. Hachage du point médian
modal-how-dedup-step3-desc = Un bloc de 4 Ko situé au centre des fichiers restants est haché, ce qui permet de détecter les différences structurelles internes.
modal-how-dedup-step4-title = 4. Hachage du suffixe
modal-how-dedup-step4-desc = Les 4 derniers Ko de données sont hachés. Cette méthode est très efficace pour identifier les différences dans le contenu final ou les métadonnées.
modal-how-dedup-step5-title = 5. Hachage multi-gamme
modal-how-dedup-step5-desc = Les fichiers volumineux (plus de 100 Mo) font l'objet d'un échantillonnage périodique de blocs sur toute leur longueur afin de vérifier la cohérence du contenu sans lire l'intégralité du fichier.
modal-how-dedup-step6-title = 6. Hachage BLAKE3 complet
modal-how-dedup-step6-desc = Pour les candidats restants, un hachage cryptographique BLAKE3 complet est calculé. En raison de la grande résistance aux collisions d'un espace de 256 bits, des hachages correspondants indiquent une improbabilité astronomique que les fichiers diffèrent, fournissant ainsi une preuve d'identité très fiable sans nécessiter de comparaisons par paires.
modal-how-dedup-step7-title = 7. Validation de l'horodatage
modal-how-dedup-step7-desc = Juste avant d'afficher ou d'exécuter toute action de déduplication, l'application vérifie les horodatages des fichiers sur le disque pour se prémunir contre les modifications survenues depuis la génération de l'instantané.

# Open Source Licenses Modal
modal-licenses-title = 📜 Licences Open Source
modal-licenses-desc = Les bibliothèques et crates tierces suivantes sont utilisées dans cette application:

# Processing Modal
modal-processing-title = ⏳ Traitement en cours...
modal-processing-deletion = Suppression des fichiers et répertoires...
modal-processing-trash = Déplacement des fichiers et répertoires vers la corbeille...
modal-processing-hardlink = Remplacement des doublons par des liens physiques...
modal-processing-softlink = Remplacement des doublons par des liens symboliques...

# Explorer Column Headers
explorer-hdr-name = Nom
explorer-hdr-percentage = Pourcentage
explorer-hdr-size = Taille
explorer-hdr-items = Éléments
explorer-hdr-files = Fichiers
explorer-hdr-subdirs = Sous-rép.
explorer-hdr-created = Créé
explorer-hdr-modified = Modifié

# Update Checker
update-checking = Recherche de mises à jour...
update-available = Nouvelle version { $version } disponible !
update-up-to-date = Vous êtes à jour
update-failed = Échec de la vérification de mise à jour : { $error }

# Themes
theme = 🎨 Thème
theme-dark = Sombre
theme-high-contrast = Contraste élevé
theme-light = Clair
theme-system = Système
