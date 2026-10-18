# Menu Bar Dropdowns
file = Arquivo
view = Exibir
help = Ajuda

# Menu Bar Actions
new-scan = 📁 Nova análise
save-snapshot = 💾 Salvar snapshot
load-snapshot = 📖 Carregar snapshot

# Menu Bar Status
idle = Ocioso

# View Menu Options
monospace-paths = 🅰 Caminhos monoespaçados
highlight-duplicates = ✨ Destacar duplicados
treemap-borders = 🔳 Bordas do mapa de árvore
treemap-style =  Estilo do mapa de árvore
treemap-style-vertical = Gradiente vertical
treemap-style-offset-vertical = Gradiente vertical deslocado
treemap-style-diagonal = Gradiente diagonal
treemap-style-cushion = Sombreamento almofadado
deletion-confirmation = 🗑 Confirmação de exclusão
trash-confirmation = ♻ Confirmação da lixeira
time-format = 🕒 Formato de hora
language = 💬 Idioma
layout-mode = Modo de layout:
classic-layout = Layout clássico
windirstat-layout = Layout WinDirStat
vis-mode-treemap = 📊 Mapa de árvore
vis-mode-plots = 📈 Gráficos
select-plot-label = Selecionar gráfico:
vis-mode-deduplicator = 👥 Localizador de duplicados
search-filter-label = 🔍 Filtrar:

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ Mostrar painel esquerdo (F9)
   *[false] ◀ Ocultar painel esquerdo (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ Mostrar painel direito (F11)
       *[false] ▶ Mostrar painel de extensões (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ Ocultar painel direito (F11)
       *[false] ◀ Ocultar painel de extensões (F11)
    }
}

collapse-all = ⏏ Recolher tudo
about = ℹ Sobre
web-not-available = Recurso não disponível na versão web

# Status Indicators
scanning-disk = Analisando disco...
scan-complete = Análise concluída
scan-cancelled = Análise cancelada
path-label = Caminho: { $path }
worker-threads = ⚡ { $count } threads de trabalho
worker-threads-hover = O número de núcleos de CPU paralelos com roubo de trabalho (work-stealing) alocados para a varredura de diretórios.

# Stats Panel (Bottom)
directories-count = 📁 Diretórios: { $count }
files-count = 📄 Arquivos: { $count }
total-size = 💾 Tamanho total: { $size }
elapsed-time = ⏱ Tempo: { $time }
scan-speed = ⚡ Velocidade: { $speed }/s

# Selection Info
selection-path = Seleção: { $path }
selection-items = Seleção: { $count ->
    [one] 1 item
   *[other] { $count } itens
}

# Plot Types
plot-size-distribution = 📊 Distribuição do tamanho dos arquivos
plot-age-size = 🌌 Idade do arquivo vs. tamanho do arquivo
plot-dir-composition = 🍰 Composição do diretório
plot-extension-boxplot = 📦 Tamanho dos arquivos por extensão
plot-temporal-timeline = ⏱ Linhas do tempo associadas
plot-deduplicator-waste = 👥 Espaço desperdiçado com duplicados por extensão

# --- Deduplicator Strings ---
dedup-desc = Encontre e remova com segurança arquivos idênticos byte a byte usando hashes criptográficos BLAKE3 seguros.
dedup-how-it-works = ℹ Como funciona
dedup-min-size = Tamanho mín. do arquivo:
dedup-ignore-system = Ignorar arquivos do sistema
dedup-ignore-hidden = Ignorar arquivos ocultos
dedup-start-scan = ⚡ Iniciar análise de duplicados
dedup-scan-first = Analise um diretório primeiro.
dedup-cancelled-msg = A análise foi cancelada. Inicie uma nova análise para encontrar duplicados.
dedup-analyzing = Analisando arquivos...
dedup-no-duplicates = Nenhum grupo de duplicados encontrado. Tente reduzir o tamanho mínimo do arquivo ou analisar outra pasta.
no-permission = Sem permissão
hardlink-badge = Link físico
dedup-select-items = 🎯 Selecionar itens...
dedup-select-all-but-oldest = 🎯 Todos exceto o mais antigo
dedup-select-all-but-newest = 🎯 Todos exceto o mais recente
dedup-select-all-but-shortest = 🎯 Todos exceto o caminho mais curto
dedup-select-all-but-rootmost = 🎯 Todos exceto o mais próximo da raiz
dedup-select-all-but-longest = 🎯 Todos exceto o caminho mais longo
dedup-pref-dir-pattern = Padrão de diretório preferido:
dedup-select-all-but-pref = 🎯 Todos exceto o diretório preferido
dedup-clear-selection = ❌ Limpar seleção
dedup-link-menu = 🔗 Vincular... ({ $count } arquivos)
dedup-link-menu-disabled = 🔗 Vincular... (0 arquivos)
dedup-link-hardlinks = 🔗 Substituir selecionados por links físicos
dedup-link-softlinks = 🔗 Substituir selecionados por links simbólicos
dedup-remove-menu = 🗑 Remover... ({ $count } arquivos, { $size })
dedup-remove-menu-disabled = 🗑 Remover... (0 arquivos)
dedup-remove-trash = ♻ Mover selecionados para a lixeira
dedup-remove-delete = 🗑 Excluir permanentemente os arquivos selecionados
dedup-warning-title = ⚠ AVISO DE PERDA DE DADOS
dedup-warning-desc = { $count ->
    [one] Excluindo todas as versões de 1 arquivo
   *[other] Excluindo todas as versões de { $count } arquivos
}
dedup-warning-no-original = Nenhuma cópia original restará:
dedup-warning-details = Você selecionou tanto o arquivo original quanto todas as cópias duplicadas dos arquivos listados abaixo. Excluí-los provavelmente resultará em perda permanente de dados:
dedup-cancel-hover = Clique para cancelar a análise
scan-cancel-hover = Clique para cancelar a análise
dedup-current-label = Atual
dedup-phase1-size = Fase 1/7: Agrupando arquivos por tamanho...
dedup-phase1-filter = Fase 1/7: Filtrando exclusões nos candidatos a duplicados...
dedup-phase2-prefix = Fase 2/7: Calculando hash dos prefixos dos arquivos (primeiros 4KB)...
dedup-phase3-midpoint = Fase 3/7: Calculando hash dos pontos médios dos arquivos...
dedup-phase4-suffix = Fase 4/7: Calculando hash dos sufixos dos arquivos...
dedup-phase5-multirange = Fase 5/7: Hash de múltiplos intervalos para arquivos grandes...
dedup-phase6-full = Fase 6/7: Hash BLAKE3 completo dos candidatos restantes...
dedup-phase7-validation = Fase 7/7: Validação final dos registros de data/hora...
dedup-phase-finished = Concluído em { $duration }! { $count } grupos de duplicados encontrados. Espaço potencialmente recuperável: { $space }
dedup-scan-cancelled-with-error = A análise foi cancelada: { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = Nome do arquivo
dedup-hdr-directory = Diretório pai
dedup-hdr-size = Tamanho
dedup-hdr-reclaimable = Recuperável
dedup-hdr-created = Criado
dedup-hdr-modified = Modificado
dedup-copies-selected = ({ $count ->
    [one] 1 cópia selecionada
   *[other] { $count } cópias selecionadas
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ Detalhes
explorer-deselect-hover = Desmarcar itens
explorer-deselect-single-hover = Desmarcar item
explorer-selected-items-count = { $count ->
    [one] 1 item selecionado
   *[other] { $count } itens selecionados
}
explorer-total-size = Tamanho total: { $size }
explorer-files = Arquivos: { $count }
explorer-directories = Diretórios: { $count }
explorer-actions-title = Ações
explorer-actions-operations = Operações:
explorer-action-refresh-hover = Atualizar todas as subárvores de diretórios selecionadas
explorer-grid-type = Tipo:
explorer-grid-size = Tamanho:
explorer-grid-bytes = Bytes:
explorer-grid-items = Itens:
explorer-grid-files = Arquivos:
explorer-grid-subdirs = Subdiretórios:
explorer-grid-user = Usuário:
explorer-grid-group = Grupo:
explorer-grid-permissions = Permissões:
explorer-grid-path = Caminho completo:

# Explorer Type Names
type-symlink = Link simbólico
type-directory = Diretório
type-file = Arquivo

# Explorer Actions
explorer-action-copy-path = 📋 Copiar caminho
explorer-action-open-file = 📄 Abrir arquivo
explorer-action-open-manager = 🗁 Abrir gerenciador
explorer-action-refresh-subtree = 🔄 Atualizar subárvore
explorer-action-move-trash = ♻ Mover para a lixeira
explorer-action-delete-permanently = 🗑 Excluir permanentemente
explorer-action-refresh-directory = 🔄 Atualizar diretório

# Explorer Empty State
explorer-empty-state = Clique em 'Nova análise' para explorar o uso do disco.
choose-an-option = Escolha uma opção
web-viewer = Visualizador Web
load-demo = 👁 Carregar snapshot de demonstração
placeholder-treemap = O sistema de arquivos analisado será exibido aqui como um mapa de árvore.
placeholder-plots = O sistema de arquivos analisado será representado em gráficos aqui.

# Treemap Zoom & Navigation
zoom-up = ⏶ Subir
zoom-reset = ❌ Redefinir
zoom-to-dir = 🔍 Focar no mapa de árvore
zoom-up-level = ⏶ Subir um nível
zoom-empty-dir = O diretório está vazio

# --- Extensions Panel ---
extensions-header = 📂 Extensões
extensions-empty = Nenhuma estatística coletada ainda.
extensions-hover-files = Arquivos: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = Subir um nível
op-zoom-treemap = Focar no mapa de árvore
op-refresh-entire-scan = Atualizar análise completa
op-refresh-directory = Atualizar diretório
op-open-file = Abrir arquivo
op-open-file-manager = Abrir no gerenciador de arquivos
op-open-terminal = Abrir terminal aqui
op-copy-path = Copiar caminho
op-copy-name = Copiar nome
op-move-trash = Mover para a lixeira
op-permanently-delete = Excluir permanentemente

# Toast Notifications
toast-already-root = Você já está no nível raiz
toast-navigated-up = Subiu um nível
toast-zoomed-treemap = Mapa de árvore focado no diretório
toast-refreshing-scan = Atualizando a análise completa...
toast-refreshing-dir = Atualizando o(s) diretório(s) selecionado(s)...
toast-opened-file = Aberto: { $path }
toast-failed-open-file = Falha ao abrir o arquivo: { $error }
toast-opened-manager = Aberto no gerenciador de arquivos: { $path }
toast-failed-open-manager = Falha ao abrir no gerenciador de arquivos: { $error }
toast-opened-terminal = Terminal aberto em: { $path }
toast-failed-open-terminal = Falha ao abrir o terminal: { $error }
toast-copied-paths = { $count ->
    [one] 1 caminho copiado para a área de transferência
   *[other] { $count } caminhos copiados para a área de transferência
}
toast-copied-names = { $count ->
    [one] 1 nome copiado para a área de transferência
   *[other] { $count } nomes copiados para a área de transferência
}

# --- Modals ---
modal-remember-confirmation = Lembrar a confirmação para todos os arquivos e diretórios futuros
modal-process-multiple = Você está prestes a processar { $count } arquivos/itens duplicados:
modal-process-single = Você está prestes a processar o seguinte caminho:
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ AVISO DE EXCLUSÃO PERMANENTE
modal-delete-header = ⚠ Aviso de exclusão permanente!
modal-delete-info = Tamanho total: { $size }
modal-delete-warning = Esta é uma exclusão recursiva. Todos os arquivos, pastas e subdiretórios sob o(s) caminho(s) selecionado(s) serão excluídos permanentemente e não poderão ser recuperados (sem passar pela lixeira).
modal-delete-checkbox = Entendo que os arquivos serão excluídos permanentemente e não poderão ser recuperados.
modal-delete-confirm = 🗑 Sim, excluir permanentemente

modal-trash-title = ♻ MOVER PARA A LIXEIRA
modal-trash-header = ♻ Mover para a lixeira
modal-trash-info = Tamanho total: { $size }
modal-trash-warning = Isso moverá o(s) caminho(s) selecionado(s) e todo o seu conteúdo para a lixeira do sistema, de onde poderão ser restaurados ou excluídos permanentemente mais tarde.
modal-trash-checkbox = Confirmo que desejo mover isto para a lixeira.
modal-trash-confirm = ♻ Sim, mover para a lixeira

modal-delete-duplicates-title = ⚠ AVISO DE DEDUPLICAÇÃO PERMANENTE
modal-delete-duplicates-header = ⚠ Aviso de exclusão permanente de duplicados!
modal-delete-duplicates-info = Espaço total a recuperar: { $size }
modal-delete-duplicates-warning = Todos os arquivos selecionados serão excluídos permanentemente e não poderão ser recuperados (sem passar pela lixeira).
modal-delete-duplicates-checkbox = Entendo que os arquivos serão excluídos permanentemente e não poderão ser recuperados.
modal-delete-duplicates-confirm = 🗑 Sim, excluir permanentemente os arquivos selecionados

modal-trash-duplicates-title = ♻ MOVER DUPLICADOS PARA A LIXEIRA
modal-trash-duplicates-header = ♻ Mover duplicados para a lixeira
modal-trash-duplicates-info = Espaço total a recuperar: { $size }
modal-trash-duplicates-warning = Todos os arquivos selecionados serão movidos para a lixeira.
modal-trash-duplicates-checkbox = Confirmo que desejo mover estes arquivos para a lixeira.
modal-trash-duplicates-confirm = ♻ Sim, mover os arquivos selecionados para a lixeira

modal-hardlink-duplicates-title = 🔗 SUBSTITUIR DUPLICADOS POR LINKS FÍSICOS
modal-hardlink-duplicates-header = 🔗 Substituir duplicados por links físicos
modal-hardlink-duplicates-info = Total de arquivos a processar: { $count }. Tamanho virtual acumulado: { $size }
modal-hardlink-duplicates-warning = Isso excluirá os arquivos duplicados selecionados e os substituirá por links físicos no sistema de arquivos, apontando para o arquivo original restante em cada grupo. Os arquivos continuam visíveis, enquanto o espaço físico real de armazenamento é liberado.
modal-hardlink-duplicates-checkbox = Confirmo que desejo substituir os arquivos selecionados por links físicos.
modal-hardlink-duplicates-confirm = 🔗 Sim, substituir por links físicos

modal-softlink-duplicates-title = 🔗 SUBSTITUIR DUPLICADOS POR LINKS SIMBÓLICOS
modal-softlink-duplicates-header = 🔗 Substituir duplicados por links simbólicos
modal-softlink-duplicates-info = Total de arquivos a processar: { $count }. Tamanho virtual acumulado: { $size }
modal-softlink-duplicates-warning = Isso excluirá os arquivos duplicados selecionados e os substituirá por links simbólicos (softlinks) no sistema de arquivos, apontando para o arquivo original restante em cada grupo. Os arquivos continuam visíveis, enquanto o espaço físico real de armazenamento é liberado.
modal-softlink-duplicates-checkbox = Confirmo que desejo substituir os arquivos selecionados por links simbólicos.
modal-softlink-duplicates-confirm = 🔗 Sim, substituir por links simbólicos

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ O caminho não existe!
modal-path-not-exist-msg = Erro: o caminho que você está tentando excluir não existe no disco.
modal-close-btn = Fechar
modal-details-label = Detalhes:
modal-cancel-btn = Cancelar

# Elevation Recommended Modal
modal-elevation-title = ⚠ Privilégios recomendados
modal-elevation-desc = Por padrão, o eDirStat é executado com privilégios normais de usuário. No entanto, o Windows restringe rigorosamente o acesso direto ao disco físico a contas de administrador.
modal-elevation-mft-disabled = Driver NTFS MFT do Windows desativado
modal-elevation-mft-desc = O leitor direto de MFT não pode ser inicializado. A análise de arquivos usará o driver alternativo de varredura padrão, reduzindo o desempenho da análise em até 20x.
modal-elevation-relaunch-prompt = Deseja reiniciar o aplicativo com privilégios de administrador agora?
modal-elevation-continue-std = Continuar como usuário padrão
modal-elevation-relaunch-btn = 🛡 Reiniciar como administrador

# About Modal
modal-about-title = ℹ Sobre o eDirStat
modal-about-author = Por: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-license-btn = 📜 Licença (MIT)
modal-about-desc1 = Uma ferramenta de alto desempenho para análise de espaço em disco e deduplicação, escrita em Rust.
modal-about-desc2 = Inclui varredura paralela de diretórios com roubo de trabalho, snapshots compactados com desserialização de esquema sem análise (zero-parse) e mapas de árvore interativos e fluidos.
modal-about-desc3 = O leitor direto de MFT lê o volume diretamente sem percorrer os diretórios, e o deduplicador integrado executa hashes criptográficos BLAKE3 para calcular o espaço duplicado.
modal-about-licenses-btn = Ver licenças de código aberto
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ Como funciona a deduplicação
modal-how-dedup-desc1 = Em vez de comparar diretamente os bytes de cada arquivo (o que exigiria lentas comparações O(N²) aos pares), este sistema usa um pipeline otimizado de 7 etapas para identificar conteúdo idêntico com segurança e eficiência.
modal-how-dedup-pipeline-title = O pipeline de 7 etapas:
modal-how-dedup-why-title = Por que isso é suficiente?
modal-how-dedup-why-desc1 = Este filtro de várias etapas garante que apenas arquivos com tamanho, prefixo, ponto médio, sufixo e amostras de blocos distribuídos idênticos sejam lidos por completo. Por fim, a comparação de um hash criptográfico BLAKE3 de 256 bits oferece um nível de segurança comparável aos protocolos de transferência segura do setor, eliminando a necessidade de lentas comparações byte a byte aos pares.

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. Particionamento por tamanho
modal-how-dedup-step1-desc = Os arquivos são agrupados pelo tamanho exato em bytes. Qualquer arquivo com tamanho único é descartado imediatamente, evitando totalmente operações de E/S no disco.
modal-how-dedup-step2-title = 2. Hash do prefixo
modal-how-dedup-step2-desc = É calculado o hash dos primeiros 4KB dos candidatos restantes. Isso filtra rapidamente arquivos com cabeçalhos ou formatos de metadados diferentes.
modal-how-dedup-step3-title = 3. Hash do ponto médio
modal-how-dedup-step3-desc = É calculado o hash de um bloco de 4KB do centro dos arquivos restantes, detectando diferenças estruturais internas.
modal-how-dedup-step4-title = 4. Hash do sufixo
modal-how-dedup-step4-desc = É calculado o hash dos últimos 4KB de dados. Isso é muito eficaz para identificar diferenças no conteúdo final ou nos metadados.
modal-how-dedup-step5-title = 5. Hash de múltiplos intervalos
modal-how-dedup-step5-desc = Arquivos grandes (acima de 100MB) passam por amostragem periódica de blocos ao longo de todo o seu comprimento para verificar a consistência do conteúdo sem ler o arquivo inteiro.
modal-how-dedup-step6-title = 6. Hash BLAKE3 completo
modal-how-dedup-step6-desc = Para os candidatos restantes, é calculado um hash criptográfico BLAKE3 completo. Graças à altíssima resistência a colisões de um espaço de 256 bits, hashes correspondentes indicam uma probabilidade astronomicamente baixa de os arquivos serem diferentes, oferecendo uma prova de identidade muito confiável sem necessidade de comparações aos pares.
modal-how-dedup-step7-title = 7. Validação de data/hora
modal-how-dedup-step7-desc = Imediatamente antes de exibir ou executar qualquer ação de deduplicação, o aplicativo valida os registros de data/hora dos arquivos no disco para se proteger de alterações ocorridas desde a geração do snapshot.

# Open Source Licenses Modal
modal-licenses-title = 📜 Licenças de código aberto
modal-licenses-tab-app = eDirStat (MIT)
modal-licenses-tab-deps = Bibliotecas de terceiros
modal-licenses-app-desc = O eDirStat é um software de código aberto distribuído sob a Licença MIT:
modal-licenses-desc = As seguintes bibliotecas e crates de terceiros são usadas neste aplicativo:
modal-licenses-copy-btn = 📋 Copiar licença
modal-licenses-copy-all-btn = 📋 Copiar licenças

# Processing Modal
modal-processing-title = ⏳ Processando...
modal-processing-deletion = Excluindo arquivos e diretórios...
modal-processing-trash = Movendo arquivos e diretórios para a lixeira...
modal-processing-hardlink = Substituindo duplicados por links físicos (hardlinks)...
modal-processing-softlink = Substituindo duplicados por links simbólicos (softlinks)...

# Explorer Column Headers
explorer-hdr-name = Nome
explorer-hdr-percentage = Porcentagem
explorer-hdr-size = Tamanho
explorer-hdr-items = Itens
explorer-hdr-files = Arquivos
explorer-hdr-subdirs = Subdir.
explorer-hdr-created = Criado
explorer-hdr-modified = Modificado

# Update Checker
update-checking = Verificando atualizações...
update-available = Nova versão { $version } disponível!
update-up-to-date = Você está com a versão mais recente
update-failed = Falha ao verificar atualizações: { $error }

# Themes
theme = 🎨 Tema
theme-dark = Escuro
theme-high-contrast = Alto contraste
theme-light = Claro
theme-system = Sistema

# New Scan Options Modal
modal-scan-options-title = Opções da nova análise
modal-scan-options-header = Iniciar nova análise
modal-scan-options-path-label = Caminho da pasta a analisar:
modal-scan-options-paste-tooltip = Colar da área de transferência
modal-scan-options-browse-tooltip = Procurar pasta...
modal-scan-options-scan-btn = Analisar
modal-scan-options-cancel-btn = Cancelar
modal-scan-options-same-filesystem = Limitar a análise ao mesmo sistema de arquivos/volume
modal-scan-options-drives-header = 💽 Unidades de armazenamento e volumes
modal-scan-options-refresh-tooltip = Atualizar unidades de armazenamento
modal-scan-options-root-system = Sistema raiz
modal-scan-options-selected-badge = ✅ Selecionado
modal-scan-options-free-of = { $free } livres de { $total }
modal-scan-options-subtitle = Selecione um volume de armazenamento, um local de acesso rápido ou um diretório personalizado para analisar.
modal-scan-options-quick-access = 📍 Atalhos de acesso rápido
modal-scan-options-path-hint = /caminho/para/analisar
modal-scan-options-hint = ℹ Selecione uma unidade acima ou digite o caminho de um diretório.
modal-scan-options-sandbox-auth = 🔒 Acesso ao sandbox necessário — clique em Analisar para conceder acesso
modal-scan-options-valid-dir = ✅ Diretório válido — pronto para analisar
modal-scan-options-points-to-file = ⚠ O caminho aponta para um arquivo — selecione uma pasta.
modal-scan-options-dir-not-exist = ⚠ O diretório não existe no sistema de arquivos.
quick-loc-home = 🏠 Pasta pessoal
quick-loc-documents = 📄 Documentos
quick-loc-downloads = 📥 Downloads
quick-loc-desktop = 🖥 Área de Trabalho
quick-loc-pictures = 🖼 Imagens
search-use-regex = Usar expressão regular (Regex)
search-match-case = Diferenciar maiúsculas de minúsculas
dedup-pref-dir-hint = ex.: /home/user/Archive

file-menu-close = Fechar análise
file-menu-quit = Sair

badge-dataless-cloud = Arquivo na nuvem / sem dados locais
badge-symlink = Link simbólico
badge-special-file = Arquivo especial (Pipe / Socket / Dispositivo)
badge-permission-denied = Acesso negado

# Docker Disk Usage
vis-mode-docker = 📦 Docker
docker-desc = Uso de disco da instalação local do Docker, lido diretamente do diretório de dados no disco — sem conexão com o daemon.
docker-no-install = Nenhuma instalação do Docker detectada nesta máquina.
docker-env-roots-title = Diretórios de dados
docker-env-vmdisks-title = Discos de máquina virtual
docker-env-socket = Socket do daemon:
docker-env-roots-header = Raízes de dados ({ $count })
docker-env-vmdisks-header = Discos de máquinas virtuais ({ $count })
docker-kind-overlay2 = overlay2
docker-kind-containerd = containerd (armazenamento de imagens)
docker-kind-unknown = layout desconhecido
docker-kind-overlay = overlay
docker-kind-aufs = aufs
docker-kind-btrfs = btrfs
docker-kind-zfs = zfs
docker-kind-devicemapper = devicemapper
docker-kind-vfs = vfs
docker-kind-windowsfilter = windowsfilter
docker-scope-system = sistema
docker-scope-rootless = rootless
docker-readable = ✅ legível
docker-not-readable = ❌ ilegível
docker-vm-size = { $apparent } aparentes, { $allocated } alocados no disco
docker-vm-note = 💽 O Docker Desktop armazena todos os dados dentro do disco da VM acima; o conteúdo não pode ser atribuído a partir do host.
docker-needs-elevated = ⚠ Um diretório de dados do Docker foi encontrado, mas não pode ser lido. Execute o eDirStat com privilégios elevados (sudo ou administrador) para analisá-lo.
docker-unsupported-layout = Esta raiz de dados do Docker usa um layout de armazenamento que o eDirStat ainda não consegue analisar (por enquanto, apenas overlay2).
docker-analyze = ⚡ Analisar
docker-analyzing = Analisando o diretório de dados do Docker...
docker-error = Falha ao inventariar o Docker: { $error }
docker-lbl-images = Dados de imagens
docker-lbl-shared = Camadas compartilhadas
docker-lbl-unique = Dados exclusivos de imagens
docker-lbl-containers = RW dos contêineres
docker-lbl-volumes = Volumes
docker-lbl-build-cache = Cache de build
docker-lbl-logs = Logs dos contêineres
docker-lbl-reclaimable = Recuperável
docker-section-images = Imagens
docker-section-containers = Contêineres
docker-section-volumes = Volumes
docker-hdr-id = ID
docker-hdr-shared = Compartilhado
docker-hdr-exclusive = Exclusivo
docker-hdr-layers = Camadas
docker-hdr-rw = Tamanho RW
docker-hdr-log = Tamanho do log
docker-no-entries = Nenhuma entrada.
docker-warnings-title = ⚠ Avisos
docker-warn-large-log = O contêiner '{ $container }' tem um arquivo de log grande ({ $size }).
docker-warn-unref-layer = A camada { $cache_id } não é referenciada por nenhuma imagem ({ $size } recuperáveis).
docker-warn-root-unreadable = O diretório de dados { $path } não pode ser lido (são necessários privilégios elevados).
docker-warn-metadata = Problema de metadados: { $detail }
docker-warn-containerd-partial = O armazenamento de imagens do containerd está em uso: imagens e camadas não podem ser atribuídas a partir do disco (suporte à API do Docker planejado); este inventário cobre apenas contêineres, logs, volumes e cache de build.
docker-source-disk = análise do disco
docker-source-partial = análise do disco (parcial)
docker-source-api = API do Docker
docker-collected-at = Coletado em { $time }
docker-refresh = 🔄 Atualizar
docker-no-snapshot-data = Este snapshot não contém dados do Docker.
docker-sandbox-unavailable = A integração ao vivo com o Docker não está disponível na versão do eDirStat da App Store, mas snapshots coletados em outras máquinas ainda podem ser analisados aqui.
docker-delete = Excluir
docker-delete-title = Excluir imagem do Docker
docker-delete-confirm = Excluir a imagem do Docker { $name } ({ $size }) através do daemon do Docker?
docker-delete-warning = Isso remove a tag e exclui a imagem através do daemon do Docker. A operação não pode ser desfeita; camadas ainda usadas por outras imagens são mantidas.
docker-delete-force = Forçar remoção
docker-delete-force-hover = Remover a imagem mesmo que contêineres a estejam usando (opção force do daemon).
docker-deleting = Excluindo a imagem...
docker-delete-done = Imagem { $name } excluída: { $untagged } tag(s) removida(s), { $deleted } camada(s) excluída(s).
docker-delete-failed = Não foi possível excluir a imagem: { $error }
docker-delete-container = Excluir contêiner
docker-delete-volume = Excluir volume
docker-delete-container-confirm = Excluir o contêiner { $name } (camada gravável { $size }, log { $extra }) através do daemon do Docker?
docker-delete-volume-confirm = Excluir o volume { $name } ({ $size }) através do daemon do Docker?
docker-delete-container-warning = Isso remove o contêiner permanentemente através do daemon do Docker. A operação não pode ser desfeita.
docker-delete-volume-warning = Isso remove o volume e seus dados permanentemente através do daemon do Docker. A operação não pode ser desfeita.
docker-delete-container-force-hover = Remover o contêiner mesmo que esteja em execução (opção force do daemon).
docker-delete-volume-force-hover = Remover o volume mesmo que contêineres o estejam usando (opção force do daemon).
docker-deleting-container = Excluindo o contêiner...
docker-deleting-volume = Excluindo o volume...
docker-delete-container-done = Contêiner { $name } excluído.
docker-delete-volume-done = Volume { $name } excluído.
docker-delete-container-failed = Não foi possível excluir o contêiner: { $error }
docker-delete-volume-failed = Não foi possível excluir o volume: { $error }
docker-hdr-created = Criado
docker-hdr-refs = Refs
docker-delete-title-multi = Excluir imagens do Docker
docker-delete-container-multi = Excluir contêineres
docker-delete-volume-multi = Excluir volumes
docker-delete-confirm-multi = Excluir { $count } imagens ({ $size } no total) através do daemon do Docker?
docker-delete-container-confirm-multi = Excluir { $count } contêineres (camada gravável { $size }, log { $extra } no total) através do daemon do Docker?
docker-delete-volume-confirm-multi = Excluir { $count } volumes ({ $size } no total) através do daemon do Docker?
docker-delete-more = ... e mais { $count }
docker-delete-done-multi = { $count } imagens excluídas: { $untagged } tag(s) removida(s), { $deleted } camada(s) excluída(s).
docker-delete-container-done-multi = { $count } contêineres excluídos.
docker-delete-volume-done-multi = { $count } volumes excluídos.
docker-delete-partial-multi = { $succeeded } de { $count } excluídos, { $failed } falharam. Primeiro erro: { $error }
badge-docker = Diretório de dados do Docker
badge-docker-vm = Disco virtual do Docker
badge-docker-area = Armazenamento { $area } do Docker

# Cleanup
vis-mode-cleanup = 🗑 Limpeza
cleanup-desc = Diretórios recuperáveis conhecidos encontrados nesta análise: artefatos de build e caches que você pode mover para a lixeira, além de caches do sistema que devem ser limpos pelas próprias ferramentas.
cleanup-summary = { $count } locais podem liberar { $size }
cleanup-regenerable-total = { $size } recuperáveis diretamente
cleanup-tool-total = { $size } devem ser limpos por ferramentas do sistema
cleanup-tier-regenerable = Regenerável
cleanup-tier-tool = Ferramenta do sistema
cleanup-empty = Nenhum alvo de limpeza conhecido nesta análise.
cleanup-analyzing = Procurando alvos de limpeza na análise...
cleanup-run-command = Execute em um terminal:
cleanup-copy = Copiar
cleanup-copied = Comando copiado para a área de transferência.
cleanup-age-days = { $days } dias
cleanup-age-fresh = Recente
cleanup-toast = 🗑 { $size } de espaço recuperável encontrado — abra a aba Limpeza para revisar.
cleanup-hdr-tier = Nível
cleanup-hdr-title = O quê
cleanup-hdr-path = Local
cleanup-hdr-age = Idade
cleanup-hdr-action = Ação
