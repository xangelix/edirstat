# Menu Bar Dropdowns
file = Ficheiro
view = Ver
help = Ajuda

# Menu Bar Actions
new-scan = 📁 Nova análise
save-snapshot = 💾 Guardar captura instantânea
load-snapshot = 📖 Carregar captura instantânea

# Menu Bar Status
idle = Inativo

# View Menu Options
monospace-paths = 🅰 Caminhos monoespaçados
highlight-duplicates = ✨ Destacar duplicados
treemap-borders = 🔳 Bordas do mapa de árvore
treemap-style =  Estilo do mapa de árvore
treemap-style-vertical = Gradiente vertical
treemap-style-offset-vertical = Gradiente vertical deslocado
treemap-style-diagonal = Gradiente diagonal
treemap-style-cushion = Sombreamento almofada
deletion-confirmation = 🗑 Confirmação de eliminação
trash-confirmation = ♻ Confirmação de lixo
time-format = 🕒 Formato da hora
language = 💬 Idioma
layout-mode = Modo de esquema:
classic-layout = Esquema clássico
windirstat-layout = Esquema WinDirStat
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

collapse-all = ⏏ Contraer tudo
about = ℹ Acerca de
web-not-available = Funcionalidade não disponível na versão web

# Status Indicators
scanning-disk = A analisar disco...
scan-complete = Análise concluída
scan-cancelled = Análise cancelada
path-label = Caminho: { $path }
worker-threads = ⚡ { $count } Threads de trabalho
worker-threads-hover = O número de núcleos de CPU paralelos com roubo de trabalho (work-stealing) alocados para o percurso do diretório.

# Stats Panel (Bottom)
directories-count = 📁 Diretórios: { $count }
files-count = 📄 Ficheiros: { $count }
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
plot-size-distribution = 📊 Distribuição do tamanho do ficheiro
plot-age-size = 🌌 Idade do ficheiro vs. Tamanho do ficheiro
plot-dir-composition = 🍰 Composição do diretório
plot-extension-boxplot = 📦 Tamanho do ficheiro por extensão
plot-temporal-timeline = ⏱ Linhas temporais associadas
plot-deduplicator-waste = 👥 Espaço duplicado desperdiçado por extensão

# --- Deduplicator Strings ---
dedup-desc = Encontre e remova com segurança ficheiros idênticos byte a byte utilizando hashes criptográficos BLAKE3 seguros.
dedup-how-it-works = ℹ Como funciona
dedup-min-size = Tamanho mín. do ficheiro:
dedup-ignore-system = Ignorar ficheiros do sistema
dedup-ignore-hidden = Ignorar ficheiros ocultos
dedup-start-scan = ⚡ Iniciar análise de duplicados
dedup-scan-first = Analise primeiro um diretório.
dedup-cancelled-msg = A análise foi cancelada. Inicie uma nova análise para encontrar duplicados.
dedup-analyzing = A analisar ficheiros...
dedup-no-duplicates = Não foram encontrados grupos de duplicados. Tente reduzir o tamanho mínimo do ficheiro ou analisar uma pasta diferente.
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
dedup-link-menu = 🔗 Associar... ({ $count } ficheiros)
dedup-link-menu-disabled = 🔗 Associar... (0 ficheiros)
dedup-link-hardlinks = 🔗 Substituir selecionados por links físicos
dedup-link-softlinks = 🔗 Substituir selecionados por links simbólicos
dedup-remove-menu = 🗑 Remover... ({ $count } ficheiros, { $size })
dedup-remove-menu-disabled = 🗑 Remover... (0 ficheiros)
dedup-remove-trash = ♻ Mover selecionados para o lixo
dedup-remove-delete = 🗑 Eliminar permanentemente os ficheiros selecionados
dedup-warning-title = ⚠ AVISO DE PERDA DE DADOS
dedup-warning-desc = { $count ->
    [one] A eliminar todas as versões de 1 ficheiro
   *[other] A eliminar todas as versões de { $count } ficheiros
}
dedup-warning-no-original = Não restará nenhuma cópia original:
dedup-warning-details = Selecionou tanto o ficheiro original como todas as cópias duplicadas dos ficheiros listados abaixo. A eliminação dos mesmos resultará provavelmente em perda permanente de dados:
dedup-cancel-hover = Clique para cancelar a análise
scan-cancel-hover = Clique para cancelar a análise
dedup-current-label = Atual
dedup-phase1-size = Fase 1/7: A agrupar ficheiros por tamanho...
dedup-phase1-filter = Fase 1/7: A filtrar exclusões nos candidatos duplicados...
dedup-phase2-prefix = Fase 2/7: Hashing de prefixos de ficheiro (primeiros 4KB)...
dedup-phase3-midpoint = Fase 3/7: Hashing de pontos médios de ficheiro...
dedup-phase4-suffix = Fase 4/7: Hashing de sufixos de ficheiro...
dedup-phase5-multirange = Fase 5/7: Hashing multi-intervalo para ficheiros grandes...
dedup-phase6-full = Fase 6/7: Hashing BLAKE3 completo dos candidatos restantes...
dedup-phase7-validation = Fase 7/7: Validação final dos carimbos de data/hora...
dedup-phase-finished = Concluído em { $duration }! Encontrados { $count } grupos de duplicados. Potencial espaço recuperável: { $space }
dedup-scan-cancelled-with-error = A análise foi cancelada: { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = Nome do ficheiro
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
explorer-files = Ficheiros: { $count }
explorer-directories = Diretórios: { $count }
explorer-actions-title = Ações
explorer-actions-operations = Operações:
explorer-action-refresh-hover = Atualizar todas as subárvores de diretórios selecionadas
explorer-grid-type = Tipo:
explorer-grid-size = Tamanho:
explorer-grid-bytes = Bytes:
explorer-grid-items = Itens:
explorer-grid-files = Ficheiros:
explorer-grid-subdirs = Subdiretórios:
explorer-grid-user = Utilizador:
explorer-grid-group = Grupo:
explorer-grid-permissions = Permissões:
explorer-grid-path = Caminho completo:

# Explorer Type Names
type-symlink = Link simbólico
type-directory = Diretório
type-file = Ficheiro

# Explorer Actions
explorer-action-copy-path = 📋 Copiar caminho
explorer-action-open-manager = 🗁 Abrir gestor
explorer-action-refresh-subtree = 🔄 Atualizar subárvore
explorer-action-move-trash = ♻ Mover para o lixo
explorer-action-delete-permanently = 🗑 Eliminar permanentemente
explorer-action-refresh-directory = 🔄 Atualizar diretório

# Explorer Empty State
explorer-empty-state = Clique em 'Nova análise' para explorar a utilização do disco.
choose-an-option = Escolha uma opção
web-viewer = Visualizador Web
load-demo = 👁 Carregar snapshot de demonstração
placeholder-treemap = O sistema de ficheiros analisado será visualizado como um mapa de árvore aqui.
placeholder-plots = O sistema de ficheiros analisado será traçado aqui.

# --- Extensions Panel ---
extensions-header = 📂 Extensões
extensions-empty = Nenhuma estatística recolhida ainda.
extensions-hover-files = Ficheiros: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = Subir um nível
op-refresh-entire-scan = Atualizar análise completa
op-refresh-directory = Atualizar diretório
op-open-file-manager = Abrir no gestor de ficheiros
op-open-terminal = Abrir terminal aqui
op-copy-path = Copiar caminho
op-copy-name = Copiar nome
op-move-trash = Mover para o lixo
op-permanently-delete = Eliminar permanentemente

# Toast Notifications
toast-already-root = Já se encontra no nível raiz
toast-navigated-up = Navegou um nível acima
toast-refreshing-scan = A atualizar toda a análise...
toast-refreshing-dir = A atualizar o(s) diretório(s) selecionado(s)...
toast-opened-manager = Aberto no gestor de ficheiros: { $path }
toast-failed-open-manager = Falha ao abrir no gestor de ficheiros: { $error }
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
modal-remember-confirmation = Lembrar a confirmação para todos os futuros ficheiros e diretórios
modal-process-multiple = Está prestes a processar { $count } ficheiros/itens duplicados:
modal-process-single = Está prestes a processar o seguinte caminho:
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ AVISO DE ELIMINAÇÃO PERMANENTE
modal-delete-header = ⚠ Aviso de eliminação permanente!
modal-delete-info = Tamanho total: { $size }
modal-delete-warning = Esta é uma eliminação recursiva. Todos os ficheiros, pastas e subdiretórios sob o(s) caminho(s) selecionado(s) serão permanentemente eliminados e não poderão ser recuperados (ignorando a reciclagem/lixo).
modal-delete-checkbox = Compreendo que os ficheiros serão eliminados permanentemente e não poderão ser recuperados.
modal-delete-confirm = 🗑 Sim, eliminar permanentemente

modal-trash-title = ♻ MOVER PARA O LIXO
modal-trash-header = ♻ Mover para o lixo
modal-trash-info = Tamanho total: { $size }
modal-trash-warning = Isto moverá o(s) caminho(s) selecionado(s) e todos os seus conteúdos para a reciclagem/lixo do sistema, de onde poderão ser recuperados ou eliminados permanentemente mais tarde.
modal-trash-checkbox = Confirmo que pretendo mover isto para o lixo.
modal-trash-confirm = ♻ Sim, mover para o lixo

modal-delete-duplicates-title = ⚠ AVISO DE DEDUPLICAÇÃO PERMANENTE
modal-delete-duplicates-header = ⚠ Aviso de eliminação permanente de duplicados!
modal-delete-duplicates-info = Total de espaço a recuperar: { $size }
modal-delete-duplicates-warning = Todos os ficheiros selecionados serão permanentemente eliminados e não poderão ser recuperados (ignorando a reciclagem/lixo).
modal-delete-duplicates-checkbox = Compreendo que os ficheiros serão eliminados permanentemente e não poderão ser recuperados.
modal-delete-duplicates-confirm = 🗑 Sim, eliminar ficheiros selecionados permanentemente

modal-trash-duplicates-title = ♻ MOVER DUPLICADOS PARA O LIXO
modal-trash-duplicates-header = ♻ Mover duplicados para o lixo
modal-trash-duplicates-info = Total de espaço a recuperar: { $size }
modal-trash-duplicates-warning = Todos os ficheiros selecionados serão movidos para a reciclagem/lixo.
modal-trash-duplicates-checkbox = Confirmo que pretendo mover estes ficheiros para o lixo.
modal-trash-duplicates-confirm = ♻ Sim, mover ficheiros selecionados para o lixo

modal-hardlink-duplicates-title = 🔗 SUBSTITUIR DUPLICADOS POR LINKS FÍSICOS
modal-hardlink-duplicates-header = 🔗 Substituir duplicados por links físicos
modal-hardlink-duplicates-info = Total de ficheiros a processar: { $count }. Tamanho virtual acumulado: { $size }
modal-hardlink-duplicates-warning = Isto eliminará os ficheiros duplicados selecionados e substitui-los-á por links físicos no sistema de ficheiros que apontam para o ficheiro original restante em cada grupo. Isto preserva os ficheiros visualmente, enquanto liberta espaço de armazenamento físico real.
modal-hardlink-duplicates-checkbox = Confirmo que pretendo substituir os ficheiros selecionados por links físicos.
modal-hardlink-duplicates-confirm = 🔗 Sim, substituir por links físicos

modal-softlink-duplicates-title = 🔗 SUBSTITUIR DUPLICADOS POR LINKS SIMBÓLICOS
modal-softlink-duplicates-header = 🔗 Substituir duplicados por links simbólicos
modal-softlink-duplicates-info = Total de ficheiros a processar: { $count }. Tamanho virtual acumulado: { $size }
modal-softlink-duplicates-warning = Isto eliminará os ficheiros duplicados selecionados e substitui-los-á por links simbólicos (softlinks) no sistema de ficheiros que apontam para o ficheiro original restante em cada grupo. Isto preserva os ficheiros visualmente, enquanto liberta espaço de armazenamento físico real.
modal-softlink-duplicates-checkbox = Confirmo que pretendo substituir os ficheiros selecionados por links simbólicos.
modal-softlink-duplicates-confirm = 🔗 Sim, substituir por links simbólicos

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ O caminho não existe!
modal-path-not-exist-msg = Erro: O caminho que está a tentar eliminar não existe no disco.
modal-close-btn = Fechar
modal-details-label = Detalhes: 
modal-cancel-btn = Cancelar

# Elevation Recommended Modal
modal-elevation-title = ⚠ Privilégios recomendados
modal-elevation-desc = O eDirStat é executado com privilégios normais de utilizador por omissão. No entanto, o Windows restringe estritamente o acesso ao identificador físico de disco a contas de administrador.
modal-elevation-mft-disabled = Controlador NTFS MFT do Windows desativado
modal-elevation-mft-desc = digitalizador direto de MFT não pode ser inicializado. A análise de ficheiros utilizará o controlador alternativo de percurso padrão, reduzindo o desempenho da análise até 20x.
modal-elevation-relaunch-prompt = Pretende reiniciar a aplicação com privilégios de administrador agora?
modal-elevation-continue-std = Continuar como utilizador padrão
modal-elevation-relaunch-btn = 🛡 Reiniciar como administrador

# About Modal
modal-about-title = ℹ Acerca do eDirStat
modal-about-author = Por: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-desc1 = Uma ferramenta de análise de espaço em disco e deduplicação de alto desempenho escrita em Rust.
modal-about-desc2 = Inclui percurso paralelo de diretórios por roubo de trabalho, capturas comprimidas sem análise para a desserialização do esquema, e mapas de árvore interativos e fluidos.
modal-about-desc3 = O digitalizador direto de MFT digitaliza o volume directamente sem ler o directório, e o deduplicador integrado corre hashes criptográficos BLAKE3 para calcular o espaço duplicado.
modal-about-licenses-btn = Ver licenças open source
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ Como funciona a deduplicação
modal-how-dedup-desc1 = Em vez de comparar diretamente os bytes de cada ficheiro (o que exigiria análises lentas O(N²) em pares), este sistema utiliza um pipeline otimizado de 7 etapas para identificar conteúdo idêntico de forma segura e eficiente.
modal-how-dedup-pipeline-title = O pipeline de 7 etapas:
modal-how-dedup-why-title = Por que razão é suficiente?
modal-how-dedup-why-desc1 = Este filtro multi-etapas garante que apenas ficheiros com tamanho, prefixo, ponto médio, sufixo e amostras de blocos distribuídos idênticos sejam lidos na totalidade. Por fim, a comparação de um hash criptográfico BLAKE3 de 256 bits oferece um perfil de segurança ao nível dos protocolos de transferência seguros da indústria, eliminando a necessidade de análises lentas byte a byte em pares.

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. Partição por tamanho
modal-how-dedup-step1-desc = Os ficheiros são agrupados pelo seu tamanho exato em bytes. Qualquer ficheiro com tamanho único é descartado de imediato, evitando totalmente acessos de E/S ao disco.
modal-how-dedup-step2-title = 2. Hashing de prefixo
modal-how-dedup-step2-desc = Os primeiros 4KB dos candidatos restantes são geocodificados (hashed). Isto filtra rapidamente ficheiros com diferentes cabeçalhos ou formatos de metadados.
modal-how-dedup-step3-title = 3. Hashing de ponto médio
modal-how-dedup-step3-desc = Um bloco de 4KB do centro dos ficheiros restantes é geocodificado, detetando diferenças estruturais internas.
modal-how-dedup-step4-title = 4. Hashing de sufixo
modal-how-dedup-step4-desc = Os últimos 4KB de dados são geocodificados. Isto é muito eficaz na identificação de diferenças no conteúdo final ou metadatos.
modal-how-dedup-step5-title = 5. Hashing de multi-intervalo
modal-how-dedup-step5-desc = Os ficheiros grandes (com mais de 100MB) são submetidos a amostragem periódica de blocos ao longo de todo o seu comprimento para verificar a consistência do conteúdo sem ler o ficheiro na totalidade.
modal-how-dedup-step6-title = 6. Hashing BLAKE3 completo
modal-how-dedup-step6-desc = Para os candidatos restantes, é calculado um hash criptográfico BLAKE3 completo. Devido à elevada resistência a colisões de um espaço de 256 bits, hashes correspondentes indicam uma improbabilidade astronómica de os ficheiros diferirem, fornecendo uma prova de identidade muito fiável sem necessidade de comparações em pares.
modal-how-dedup-step7-title = 7. Validação de marca temporal
modal-how-dedup-step7-desc = Imediatamente antes de mostrar ou executar qualquer ação de deduplicação, a aplicação valida as marcas temporais dos ficheiros no disco para se proteger de alterações ocorridas desde a geração da captura instantânea.

# Open Source Licenses Modal
modal-licenses-title = 📜 Licenças Open Source
modal-licenses-desc = As seguintes bibliotecas e crates de terceiros são utilizadas nesta aplicação:

# Processing Modal
modal-processing-title = ⏳ Processando...
modal-processing-deletion = Eliminando ficheiros e diretórios...
modal-processing-trash = A mover ficheiros e diretórios para a reciclagem...
modal-processing-hardlink = A substituir duplicados por atalhos rígidos (hardlinks)...
modal-processing-softlink = A substituir duplicados por atalhos simbólicos (softlinks)...

# Explorer Column Headers
explorer-hdr-name = Nome
explorer-hdr-percentage = Percentagem
explorer-hdr-size = Tamanho
explorer-hdr-items = Itens
explorer-hdr-files = Ficheiros
explorer-hdr-subdirs = Subdir.
explorer-hdr-created = Criado
explorer-hdr-modified = Modificado

# Update Checker
update-checking = A procurar atualizações...
update-available = Nova versão { $version } disponível!
update-up-to-date = Já se encontra atualizado
update-failed = Falha ao procurar updates: { $error }

# Themes
theme = 🎨 Tema
theme-dark = Escuro
theme-high-contrast = Alto Contraste
theme-light = Claro
theme-system = Sistema

# New Scan Options Modal
modal-scan-options-title = Opções de nova análise
modal-scan-options-header = Iniciar nova análise
modal-scan-options-path-label = Caminho da pasta a analisar:
modal-scan-options-paste-tooltip = Colar da área de transferência
modal-scan-options-browse-tooltip = Procurar pasta...
modal-scan-options-scan-btn = Analisar
modal-scan-options-cancel-btn = Cancelar
modal-scan-options-same-filesystem = Limitar a análise ao mesmo sistema de ficheiros/volume
modal-scan-options-drives-header = 💽 Unidades de armazenamento e volumes
modal-scan-options-refresh-tooltip = Atualizar unidades de armazenamento
modal-scan-options-root-system = Sistema raiz
modal-scan-options-selected-badge = ✅ Selecionado
modal-scan-options-free-of = { $free } livres de { $total }
