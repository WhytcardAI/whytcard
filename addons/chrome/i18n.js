// WhytCard Chrome Extension - Internationalization
// Supported languages: en, fr, es, de, it, pt

const translations = {
  en: {
    // Header
    appTitle: "WhytCard",
    captureScreenshot: "Capture screenshot",
    refreshConnection: "Refresh connection",
    checking: "Checking...",
    connected: "Connected",
    disconnected: "Disconnected",

    // Project bar
    project: "Project",
    selectProject: "Select project",
    refreshProjects: "Refresh projects",

    // Session bar
    session: "Session",
    selectSession: "Select session",
    selectProjectFirst: "Select project first",
    newSession: "New session",
    newSessionName: "Enter session name:",
    sessionCreated: "Session created",
    selectSessionFirst: "Please select a session first",

    // Context panel
    pageContext: "Page Context",
    includeInChat: "Include in chat",
    currentPage: "Current Page",

    // Quick actions
    summarize: "Summarize",
    explain: "Explain",
    keyPoints: "Key Points",
    translate: "Translate",
    savePage: "Save Page",

    // Empty state
    startConversation: "Start a conversation",
    emptyStateDesc:
      "Select a project and session to start chatting. Your conversations will sync with the desktop app!",

    // Input area
    attachPage: "Page",
    attachScreenshot: "Screenshot",
    attachFile: "File",
    inputPlaceholder: "Ask WhytCard...",

    // Messages
    noPageContext: "No page context available. Navigate to a page first.",
    savingPage: "Saving page to WhytCard...",
    pageSaved: "Page saved to WhytCard successfully.",
    failedToSave: "Failed to save:",
    failedToExtract: "Failed to extract page content",
    screenshotFailed: "Screenshot failed:",
    error: "Error:",
    failedToSend: "Failed to send message:",

    // Prompts
    promptSummarize: "Please summarize the content of this page:",
    promptExplain: "Please explain the main concepts on this page:",
    promptTranslate: "Please translate the main content of this page to",
    promptKeyPoints: "What are the key points or takeaways from this page:",

    // Language selector
    language: "Language",

    // Popup specific
    openChat: "Open Chat",
    settings: "Settings",
    showFloatingBtn: "Show floating button",
    shortcut: "Shortcut",
    connectHub: "Connect to Hub",

    // Library
    library: "Library",
    localLibrary: "Local Library",
    highlights: "Highlights",
    clips: "Clips",
    notes: "Notes",
    pages: "Pages",
    syncSelected: "Sync Selected",
    indexForAi: "Index for AI",
    noHighlights: "No highlights saved",
    noClips: "No clips saved",
    noNotes: "No notes saved",
    noPages: "No pages captured",
    local: "local",
    synced: "synced",
    indexed: "indexed",
    pending: "pending",
    failed: "failed",
    confirmSync: "Confirm Sync",
    syncItemsQuestion: "Sync {count} items to WhytCard Hub?",
    syncDescription: "This will upload the content to your local Hub for storage.",
    confirmRagIndex: "Index for AI",
    ragItemsQuestion: "Index {count} items for AI context?",
    ragDescription: "This will allow the AI to use this content when answering your questions.",
    ragRequireSync: "Note: Items must be synced first.",
    syncSuccess: "Synced {count} items to Hub",
    syncFailed: "{count} items failed to sync",
    indexSuccess: "Indexed {count} items for AI",
    indexFailed: "{count} items failed to index",
    selectProjectFirst: "Please select a project first",
    itemsMustBeSynced: "Selected items must be synced to Hub first",
    pageSavedToLibrary: "Page saved to local library! Open Library to sync to Hub.",
    cancel: "Cancel",
    sync: "Sync",
    index: "Index",

    // Library item actions
    syncToHub: "Sync to Hub",
    addToRag: "Add to AI context",
    removeFromRag: "Remove from AI context",
    delete: "Delete",

    // More menu
    capturePage: "Capture page",
    screenshot: "Screenshot",
    graph: "Graph",
    docs: "Docs",
  },

  fr: {
    appTitle: "WhytCard",
    captureScreenshot: "Capturer l'ecran",
    refreshConnection: "Rafraichir la connexion",
    checking: "Verification...",
    connected: "Connecte",
    disconnected: "Deconnecte",

    project: "Projet",
    selectProject: "Selectionner un projet",
    refreshProjects: "Rafraichir les projets",

    session: "Session",
    selectSession: "Selectionner une session",
    selectProjectFirst: "Selectionner un projet d'abord",
    newSession: "Nouvelle session",
    newSessionName: "Nom de la session:",
    sessionCreated: "Session creee",
    selectSessionFirst: "Veuillez d'abord selectionner une session",

    pageContext: "Contexte de la page",
    includeInChat: "Inclure dans le chat",
    currentPage: "Page actuelle",

    summarize: "Resumer",
    explain: "Expliquer",
    keyPoints: "Points cles",
    translate: "Traduire",
    savePage: "Sauvegarder",

    startConversation: "Demarrer une conversation",
    emptyStateDesc:
      "Selectionnez un projet et une session pour commencer a discuter. Vos conversations seront synchronisees avec l'application de bureau!",

    attachPage: "Page",
    attachScreenshot: "Capture",
    attachFile: "Fichier",
    inputPlaceholder: "Demandez a WhytCard...",

    noPageContext: "Aucun contexte de page disponible. Naviguez d'abord vers une page.",
    savingPage: "Sauvegarde de la page dans WhytCard...",
    pageSaved: "Page sauvegardee dans WhytCard avec succes.",
    failedToSave: "Echec de la sauvegarde:",
    failedToExtract: "Echec de l'extraction du contenu de la page",
    screenshotFailed: "Echec de la capture d'ecran:",
    error: "Erreur:",
    failedToSend: "Echec de l'envoi du message:",

    promptSummarize: "Veuillez resumer le contenu de cette page:",
    promptExplain: "Veuillez expliquer les concepts principaux de cette page:",
    promptTranslate: "Veuillez traduire le contenu principal de cette page en",
    promptKeyPoints: "Quels sont les points cles ou les enseignements de cette page:",

    language: "Langue",

    openChat: "Ouvrir le chat",
    settings: "Parametres",
    showFloatingBtn: "Afficher le bouton flottant",
    shortcut: "Raccourci",
    connectHub: "Connexion au Hub",

    // Library
    library: "Bibliotheque",
    localLibrary: "Bibliotheque locale",
    highlights: "Surlignages",
    clips: "Extraits",
    notes: "Notes",
    pages: "Pages",
    syncSelected: "Synchroniser",
    indexForAi: "Indexer pour l'IA",
    noHighlights: "Aucun surlignage",
    noClips: "Aucun extrait",
    noNotes: "Aucune note",
    noPages: "Aucune page capturee",
    local: "local",
    synced: "synchronise",
    indexed: "indexe",
    pending: "en attente",
    failed: "echec",
    confirmSync: "Confirmer la synchronisation",
    syncItemsQuestion: "Synchroniser {count} elements vers le Hub?",
    syncDescription: "Cela va telecharger le contenu vers votre Hub local.",
    confirmRagIndex: "Indexer pour l'IA",
    ragItemsQuestion: "Indexer {count} elements pour le contexte IA?",
    ragDescription: "Cela permettra a l'IA d'utiliser ce contenu pour repondre a vos questions.",
    ragRequireSync: "Note: Les elements doivent etre synchronises d'abord.",
    syncSuccess: "{count} elements synchronises",
    syncFailed: "{count} elements en echec",
    indexSuccess: "{count} elements indexes pour l'IA",
    indexFailed: "{count} elements en echec d'indexation",
    selectProjectFirst: "Veuillez d'abord selectionner un projet",
    itemsMustBeSynced: "Les elements doivent d'abord etre synchronises",
    pageSavedToLibrary: "Page sauvegardee! Ouvrez la bibliotheque pour synchroniser.",
    cancel: "Annuler",
    sync: "Synchroniser",
    index: "Indexer",

    // Library item actions
    syncToHub: "Synchroniser vers le Hub",
    addToRag: "Ajouter au contexte IA",
    removeFromRag: "Retirer du contexte IA",
    delete: "Supprimer",

    // More menu
    capturePage: "Capturer la page",
    screenshot: "Capture d'ecran",
    graph: "Graphe",
    docs: "Documents",
  },

  es: {
    appTitle: "WhytCard",
    captureScreenshot: "Capturar pantalla",
    refreshConnection: "Actualizar conexion",
    checking: "Verificando...",
    connected: "Conectado",
    disconnected: "Desconectado",

    project: "Proyecto",
    selectProject: "Seleccionar proyecto",
    refreshProjects: "Actualizar proyectos",

    session: "Sesion",
    selectSession: "Seleccionar sesion",
    selectProjectFirst: "Seleccionar proyecto primero",
    newSession: "Nueva sesion",
    newSessionName: "Nombre de la sesion:",
    sessionCreated: "Sesion creada",
    selectSessionFirst: "Por favor seleccione primero una sesion",

    pageContext: "Contexto de pagina",
    includeInChat: "Incluir en chat",
    currentPage: "Pagina actual",

    summarize: "Resumir",
    explain: "Explicar",
    keyPoints: "Puntos clave",
    translate: "Traducir",
    savePage: "Guardar",

    startConversation: "Iniciar conversacion",
    emptyStateDesc:
      "Seleccione un proyecto y sesion para comenzar a chatear. Sus conversaciones se sincronizaran con la aplicacion de escritorio!",

    attachPage: "Pagina",
    attachScreenshot: "Captura",
    attachFile: "Archivo",
    inputPlaceholder: "Pregunta a WhytCard...",

    noPageContext: "No hay contexto de pagina disponible. Navega primero a una pagina.",
    savingPage: "Guardando pagina en WhytCard...",
    pageSaved: "Pagina guardada en WhytCard exitosamente.",
    failedToSave: "Error al guardar:",
    failedToExtract: "Error al extraer el contenido de la pagina",
    screenshotFailed: "Error en la captura de pantalla:",
    error: "Error:",
    failedToSend: "Error al enviar mensaje:",

    promptSummarize: "Por favor, resume el contenido de esta pagina:",
    promptExplain: "Por favor, explica los conceptos principales de esta pagina:",
    promptTranslate: "Por favor, traduce el contenido principal de esta pagina a",
    promptKeyPoints: "Cuales son los puntos clave de esta pagina:",

    language: "Idioma",

    openChat: "Abrir chat",
    settings: "Configuracion",
    showFloatingBtn: "Mostrar boton flotante",
    shortcut: "Atajo",
    connectHub: "Conectar al Hub",

    // Library
    library: "Biblioteca",
    localLibrary: "Biblioteca local",
    highlights: "Resaltados",
    clips: "Recortes",
    notes: "Notas",
    pages: "Paginas",
    syncSelected: "Sincronizar",
    indexForAi: "Indexar para IA",
    noHighlights: "Sin resaltados",
    noClips: "Sin recortes",
    noNotes: "Sin notas",
    noPages: "Sin paginas capturadas",
    local: "local",
    synced: "sincronizado",
    indexed: "indexado",
    pending: "pendiente",
    failed: "fallido",
    confirmSync: "Confirmar sincronizacion",
    syncItemsQuestion: "Sincronizar {count} elementos al Hub?",
    syncDescription: "Esto subira el contenido a tu Hub local.",
    confirmRagIndex: "Indexar para IA",
    ragItemsQuestion: "Indexar {count} elementos para contexto IA?",
    ragDescription: "Esto permitira a la IA usar este contenido para responder.",
    ragRequireSync: "Nota: Los elementos deben sincronizarse primero.",
    syncSuccess: "{count} elementos sincronizados",
    syncFailed: "{count} elementos fallaron",
    indexSuccess: "{count} elementos indexados",
    indexFailed: "{count} elementos fallaron al indexar",
    selectProjectFirst: "Seleccione primero un proyecto",
    itemsMustBeSynced: "Los elementos deben sincronizarse primero",
    pageSavedToLibrary: "Pagina guardada! Abra la biblioteca para sincronizar.",
    cancel: "Cancelar",
    sync: "Sincronizar",
    index: "Indexar",

    // Library item actions
    syncToHub: "Sincronizar al Hub",
    addToRag: "Agregar al contexto IA",
    removeFromRag: "Quitar del contexto IA",
    delete: "Eliminar",

    // More menu
    capturePage: "Capturar pagina",
    screenshot: "Captura de pantalla",
    graph: "Grafico",
    docs: "Documentos",
  },

  de: {
    appTitle: "WhytCard",
    captureScreenshot: "Screenshot aufnehmen",
    refreshConnection: "Verbindung aktualisieren",
    checking: "Pruefe...",
    connected: "Verbunden",
    disconnected: "Getrennt",

    project: "Projekt",
    selectProject: "Projekt auswahlen",
    refreshProjects: "Projekte aktualisieren",

    session: "Sitzung",
    selectSession: "Sitzung auswahlen",
    selectProjectFirst: "Zuerst Projekt auswahlen",
    newSession: "Neue Sitzung",
    newSessionName: "Sitzungsname:",
    sessionCreated: "Sitzung erstellt",
    selectSessionFirst: "Bitte wahlen Sie zuerst eine Sitzung",

    pageContext: "Seitenkontext",
    includeInChat: "Im Chat einschliessen",
    currentPage: "Aktuelle Seite",

    summarize: "Zusammenfassen",
    explain: "Erklaeren",
    keyPoints: "Kernpunkte",
    translate: "Uebersetzen",
    savePage: "Speichern",

    startConversation: "Gespraech starten",
    emptyStateDesc:
      "Wahlen Sie ein Projekt und eine Sitzung aus um zu chatten. Ihre Gesprache werden mit der Desktop-App synchronisiert!",

    attachPage: "Seite",
    attachScreenshot: "Screenshot",
    attachFile: "Datei",
    inputPlaceholder: "Fragen Sie WhytCard...",

    noPageContext: "Kein Seitenkontext verfuegbar. Navigieren Sie zuerst zu einer Seite.",
    savingPage: "Seite wird in WhytCard gespeichert...",
    pageSaved: "Seite erfolgreich in WhytCard gespeichert.",
    failedToSave: "Speichern fehlgeschlagen:",
    failedToExtract: "Seiteninhalt konnte nicht extrahiert werden",
    screenshotFailed: "Screenshot fehlgeschlagen:",
    error: "Fehler:",
    failedToSend: "Nachricht konnte nicht gesendet werden:",

    promptSummarize: "Bitte fassen Sie den Inhalt dieser Seite zusammen:",
    promptExplain: "Bitte erklaeren Sie die Hauptkonzepte auf dieser Seite:",
    promptTranslate: "Bitte uebersetzen Sie den Hauptinhalt dieser Seite in",
    promptKeyPoints: "Was sind die wichtigsten Punkte dieser Seite:",

    language: "Sprache",

    openChat: "Chat offnen",
    settings: "Einstellungen",
    showFloatingBtn: "Schwebende Taste anzeigen",
    shortcut: "Tastenkombination",
    connectHub: "Mit Hub verbinden",

    // Library
    library: "Bibliothek",
    localLibrary: "Lokale Bibliothek",
    highlights: "Markierungen",
    clips: "Ausschnitte",
    notes: "Notizen",
    pages: "Seiten",
    syncSelected: "Synchronisieren",
    indexForAi: "Fur KI indexieren",
    noHighlights: "Keine Markierungen",
    noClips: "Keine Ausschnitte",
    noNotes: "Keine Notizen",
    noPages: "Keine Seiten erfasst",
    local: "lokal",
    synced: "synchronisiert",
    indexed: "indexiert",
    pending: "ausstehend",
    failed: "fehlgeschlagen",
    confirmSync: "Sync bestatigen",
    syncItemsQuestion: "{count} Elemente zum Hub synchronisieren?",
    syncDescription: "Der Inhalt wird in Ihren lokalen Hub hochgeladen.",
    confirmRagIndex: "Fur KI indexieren",
    ragItemsQuestion: "{count} Elemente fur KI-Kontext indexieren?",
    ragDescription: "Die KI kann diesen Inhalt fur Antworten verwenden.",
    ragRequireSync: "Hinweis: Elemente mussen zuerst synchronisiert werden.",
    syncSuccess: "{count} Elemente synchronisiert",
    syncFailed: "{count} Elemente fehlgeschlagen",
    indexSuccess: "{count} Elemente indexiert",
    indexFailed: "{count} Elemente Indexierung fehlgeschlagen",
    selectProjectFirst: "Bitte wahlen Sie zuerst ein Projekt",
    itemsMustBeSynced: "Elemente mussen zuerst synchronisiert werden",
    pageSavedToLibrary: "Seite gespeichert! Offnen Sie die Bibliothek zum Synchronisieren.",
    cancel: "Abbrechen",
    sync: "Synchronisieren",
    index: "Indexieren",

    // Library item actions
    syncToHub: "Mit Hub synchronisieren",
    addToRag: "Zum KI-Kontext hinzufugen",
    removeFromRag: "Aus KI-Kontext entfernen",
    delete: "Loschen",

    // More menu
    capturePage: "Seite erfassen",
    screenshot: "Screenshot",
    graph: "Graph",
    docs: "Dokumente",
  },

  it: {
    appTitle: "WhytCard",
    captureScreenshot: "Cattura schermo",
    refreshConnection: "Aggiorna connessione",
    checking: "Verifica...",
    connected: "Connesso",
    disconnected: "Disconnesso",

    project: "Progetto",
    selectProject: "Seleziona progetto",
    refreshProjects: "Aggiorna progetti",

    session: "Sessione",
    selectSession: "Seleziona sessione",
    selectProjectFirst: "Prima seleziona un progetto",
    newSession: "Nuova sessione",
    newSessionName: "Nome della sessione:",
    sessionCreated: "Sessione creata",
    selectSessionFirst: "Per favore seleziona prima una sessione",

    pageContext: "Contesto pagina",
    includeInChat: "Includi nella chat",
    currentPage: "Pagina corrente",

    summarize: "Riassumi",
    explain: "Spiega",
    keyPoints: "Punti chiave",
    translate: "Traduci",
    savePage: "Salva",

    startConversation: "Inizia una conversazione",
    emptyStateDesc:
      "Seleziona un progetto e una sessione per iniziare a chattare. Le tue conversazioni saranno sincronizzate con l'app desktop!",

    attachPage: "Pagina",
    attachScreenshot: "Screenshot",
    attachFile: "File",
    inputPlaceholder: "Chiedi a WhytCard...",

    noPageContext: "Nessun contesto di pagina disponibile. Naviga prima verso una pagina.",
    savingPage: "Salvataggio pagina in WhytCard...",
    pageSaved: "Pagina salvata con successo in WhytCard.",
    failedToSave: "Salvataggio fallito:",
    failedToExtract: "Impossibile estrarre il contenuto della pagina",
    screenshotFailed: "Screenshot fallito:",
    error: "Errore:",
    failedToSend: "Invio messaggio fallito:",

    promptSummarize: "Per favore riassumi il contenuto di questa pagina:",
    promptExplain: "Per favore spiega i concetti principali di questa pagina:",
    promptTranslate: "Per favore traduci il contenuto principale di questa pagina in",
    promptKeyPoints: "Quali sono i punti chiave di questa pagina:",

    language: "Lingua",

    openChat: "Apri chat",
    settings: "Impostazioni",
    showFloatingBtn: "Mostra pulsante mobile",
    shortcut: "Scorciatoia",
    connectHub: "Connetti all'Hub",

    // Library
    library: "Libreria",
    localLibrary: "Libreria locale",
    highlights: "Evidenziazioni",
    clips: "Ritagli",
    notes: "Note",
    pages: "Pagine",
    syncSelected: "Sincronizza",
    indexForAi: "Indicizza per IA",
    noHighlights: "Nessuna evidenziazione",
    noClips: "Nessun ritaglio",
    noNotes: "Nessuna nota",
    noPages: "Nessuna pagina catturata",
    local: "locale",
    synced: "sincronizzato",
    indexed: "indicizzato",
    pending: "in attesa",
    failed: "fallito",
    confirmSync: "Conferma sincronizzazione",
    syncItemsQuestion: "Sincronizzare {count} elementi all'Hub?",
    syncDescription: "Il contenuto verra caricato nel tuo Hub locale.",
    confirmRagIndex: "Indicizza per IA",
    ragItemsQuestion: "Indicizzare {count} elementi per contesto IA?",
    ragDescription: "L'IA potra usare questo contenuto per rispondere.",
    ragRequireSync: "Nota: Gli elementi devono essere sincronizzati prima.",
    syncSuccess: "{count} elementi sincronizzati",
    syncFailed: "{count} elementi falliti",
    indexSuccess: "{count} elementi indicizzati",
    indexFailed: "{count} elementi falliti nell'indicizzazione",
    selectProjectFirst: "Seleziona prima un progetto",
    itemsMustBeSynced: "Gli elementi devono essere sincronizzati prima",
    pageSavedToLibrary: "Pagina salvata! Apri la libreria per sincronizzare.",
    cancel: "Annulla",
    sync: "Sincronizza",
    index: "Indicizza",

    // Library item actions
    syncToHub: "Sincronizza all'Hub",
    addToRag: "Aggiungi al contesto IA",
    removeFromRag: "Rimuovi dal contesto IA",
    delete: "Elimina",

    // More menu
    capturePage: "Cattura pagina",
    screenshot: "Screenshot",
    graph: "Grafico",
    docs: "Documenti",
  },

  pt: {
    appTitle: "WhytCard",
    captureScreenshot: "Capturar tela",
    refreshConnection: "Atualizar conexao",
    checking: "Verificando...",
    connected: "Conectado",
    disconnected: "Desconectado",

    project: "Projeto",
    selectProject: "Selecionar projeto",
    refreshProjects: "Atualizar projetos",

    session: "Sessao",
    selectSession: "Selecionar sessao",
    selectProjectFirst: "Selecione um projeto primeiro",
    newSession: "Nova sessao",
    newSessionName: "Nome da sessao:",
    sessionCreated: "Sessao criada",
    selectSessionFirst: "Por favor selecione primeiro uma sessao",

    pageContext: "Contexto da pagina",
    includeInChat: "Incluir no chat",
    currentPage: "Pagina atual",

    summarize: "Resumir",
    explain: "Explicar",
    keyPoints: "Pontos-chave",
    translate: "Traduzir",
    savePage: "Salvar",

    startConversation: "Iniciar conversa",
    emptyStateDesc:
      "Selecione um projeto e sessao para comecar a conversar. Suas conversas serao sincronizadas com o aplicativo de desktop!",

    attachPage: "Pagina",
    attachScreenshot: "Captura",
    attachFile: "Arquivo",
    inputPlaceholder: "Pergunte ao WhytCard...",

    noPageContext: "Nenhum contexto de pagina disponivel. Navegue primeiro para uma pagina.",
    savingPage: "Salvando pagina no WhytCard...",
    pageSaved: "Pagina salva no WhytCard com sucesso.",
    failedToSave: "Falha ao salvar:",
    failedToExtract: "Falha ao extrair o conteudo da pagina",
    screenshotFailed: "Falha na captura de tela:",
    error: "Erro:",
    failedToSend: "Falha ao enviar mensagem:",

    promptSummarize: "Por favor, resuma o conteudo desta pagina:",
    promptExplain: "Por favor, explique os principais conceitos desta pagina:",
    promptTranslate: "Por favor, traduza o conteudo principal desta pagina para",
    promptKeyPoints: "Quais sao os pontos-chave desta pagina:",

    language: "Idioma",

    openChat: "Abrir chat",
    settings: "Configuracoes",
    showFloatingBtn: "Mostrar botao flutuante",
    shortcut: "Atalho",
    connectHub: "Conectar ao Hub",

    // Library
    library: "Biblioteca",
    localLibrary: "Biblioteca local",
    highlights: "Destaques",
    clips: "Recortes",
    notes: "Notas",
    pages: "Paginas",
    syncSelected: "Sincronizar",
    indexForAi: "Indexar para IA",
    noHighlights: "Sem destaques",
    noClips: "Sem recortes",
    noNotes: "Sem notas",
    noPages: "Sem paginas capturadas",
    local: "local",
    synced: "sincronizado",
    indexed: "indexado",
    pending: "pendente",
    failed: "falhou",
    confirmSync: "Confirmar sincronizacao",
    syncItemsQuestion: "Sincronizar {count} itens ao Hub?",
    syncDescription: "O conteudo sera enviado ao seu Hub local.",
    confirmRagIndex: "Indexar para IA",
    ragItemsQuestion: "Indexar {count} itens para contexto IA?",
    ragDescription: "A IA podera usar este conteudo para responder.",
    ragRequireSync: "Nota: Itens devem ser sincronizados primeiro.",
    syncSuccess: "{count} itens sincronizados",
    syncFailed: "{count} itens falharam",
    indexSuccess: "{count} itens indexados",
    indexFailed: "{count} itens falharam ao indexar",
    selectProjectFirst: "Selecione primeiro um projeto",
    itemsMustBeSynced: "Itens devem ser sincronizados primeiro",
    pageSavedToLibrary: "Pagina salva! Abra a biblioteca para sincronizar.",
    cancel: "Cancelar",
    sync: "Sincronizar",
    index: "Indexar",

    // Library item actions
    syncToHub: "Sincronizar ao Hub",
    addToRag: "Adicionar ao contexto IA",
    removeFromRag: "Remover do contexto IA",
    delete: "Excluir",

    // More menu
    capturePage: "Capturar pagina",
    screenshot: "Captura de tela",
    graph: "Grafico",
    docs: "Documentos",
  },

  nl: {
    appTitle: "WhytCard",
    captureScreenshot: "Screenshot maken",
    refreshConnection: "Verbinding vernieuwen",
    checking: "Controleren...",
    connected: "Verbonden",
    disconnected: "Niet verbonden",

    project: "Project",
    selectProject: "Selecteer project",
    refreshProjects: "Projecten vernieuwen",

    session: "Sessie",
    selectSession: "Selecteer sessie",
    selectProjectFirst: "Selecteer eerst een project",
    newSession: "Nieuwe sessie",
    newSessionName: "Sessienaam:",
    sessionCreated: "Sessie aangemaakt",
    selectSessionFirst: "Selecteer eerst een sessie",

    pageContext: "Pagina-context",
    includeInChat: "Opnemen in chat",
    currentPage: "Huidige pagina",

    summarize: "Samenvatten",
    explain: "Uitleggen",
    keyPoints: "Kernpunten",
    translate: "Vertalen",
    savePage: "Opslaan",

    startConversation: "Start een gesprek",
    emptyStateDesc:
      "Selecteer een project en sessie om te chatten. Je gesprekken worden gesynchroniseerd met de desktop-app!",

    attachPage: "Pagina",
    attachScreenshot: "Screenshot",
    attachFile: "Bestand",
    inputPlaceholder: "Vraag WhytCard...",

    noPageContext: "Geen pagina-context beschikbaar. Navigeer eerst naar een pagina.",
    savingPage: "Pagina opslaan in WhytCard...",
    pageSaved: "Pagina succesvol opgeslagen in WhytCard.",
    failedToSave: "Opslaan mislukt:",
    failedToExtract: "Kon pagina-inhoud niet extraheren",
    screenshotFailed: "Screenshot mislukt:",
    error: "Fout:",
    failedToSend: "Verzenden mislukt:",

    promptSummarize: "Vat de inhoud van deze pagina samen:",
    promptExplain: "Leg de hoofdconcepten van deze pagina uit:",
    promptTranslate: "Vertaal de hoofdinhoud van deze pagina naar",
    promptKeyPoints: "Wat zijn de kernpunten van deze pagina:",

    language: "Taal",

    openChat: "Chat openen",
    settings: "Instellingen",
    showFloatingBtn: "Zwevende knop tonen",
    shortcut: "Sneltoets",
    connectHub: "Verbinden met Hub",

    // Library
    library: "Bibliotheek",
    localLibrary: "Lokale bibliotheek",
    highlights: "Markeringen",
    clips: "Knipsels",
    notes: "Notities",
    pages: "Paginas",
    syncSelected: "Synchroniseren",
    indexForAi: "Indexeren voor AI",
    noHighlights: "Geen markeringen",
    noClips: "Geen knipsels",
    noNotes: "Geen notities",
    noPages: "Geen paginas vastgelegd",
    local: "lokaal",
    synced: "gesynchroniseerd",
    indexed: "geindexeerd",
    pending: "in behandeling",
    failed: "mislukt",
    confirmSync: "Synchronisatie bevestigen",
    syncItemsQuestion: "{count} items synchroniseren naar Hub?",
    syncDescription: "De inhoud wordt geupload naar uw lokale Hub.",
    confirmRagIndex: "Indexeren voor AI",
    ragItemsQuestion: "{count} items indexeren voor AI-context?",
    ragDescription: "De AI kan deze inhoud gebruiken om te antwoorden.",
    ragRequireSync: "Let op: Items moeten eerst gesynchroniseerd worden.",
    syncSuccess: "{count} items gesynchroniseerd",
    syncFailed: "{count} items mislukt",
    indexSuccess: "{count} items geindexeerd",
    indexFailed: "{count} items indexering mislukt",
    selectProjectFirst: "Selecteer eerst een project",
    itemsMustBeSynced: "Items moeten eerst gesynchroniseerd worden",
    pageSavedToLibrary: "Pagina opgeslagen! Open bibliotheek om te synchroniseren.",
    cancel: "Annuleren",
    sync: "Synchroniseren",
    index: "Indexeren",

    // Library item actions
    syncToHub: "Synchroniseren naar Hub",
    addToRag: "Toevoegen aan AI-context",
    removeFromRag: "Verwijderen uit AI-context",
    delete: "Verwijderen",

    // More menu
    capturePage: "Pagina vastleggen",
    screenshot: "Screenshot",
    graph: "Grafiek",
    docs: "Documenten",
  },
};

// Available languages with native names
const availableLanguages = {
  en: "English",
  fr: "Francais",
  es: "Espanol",
  de: "Deutsch",
  it: "Italiano",
  pt: "Portugues",
  nl: "Nederlands",
};

// Current language
let currentLanguage = "en";

// Get translation
function t(key) {
  const lang = translations[currentLanguage] || translations.en;
  return lang[key] || translations.en[key] || key;
}

// Set language
async function setLanguage(lang) {
  if (translations[lang]) {
    currentLanguage = lang;
    await chrome.storage.local.set({ language: lang });
    updateUI();
  }
}

// Load saved language
async function loadLanguage() {
  const result = await chrome.storage.local.get(["language"]);
  if (result.language && translations[result.language]) {
    currentLanguage = result.language;
  } else {
    // Try to detect from browser
    const browserLang = navigator.language.split("-")[0];
    if (translations[browserLang]) {
      currentLanguage = browserLang;
    }
  }
}

// Update all UI elements with translations
function updateUI() {
  // Update all elements with data-i18n attribute
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const key = el.getAttribute("data-i18n");
    if (key && t(key) !== key) {
      el.textContent = t(key);
    }
  });

  // Update all elements with data-i18n-title attribute
  document.querySelectorAll("[data-i18n-title]").forEach((el) => {
    const key = el.getAttribute("data-i18n-title");
    if (key && t(key) !== key) {
      el.setAttribute("title", t(key));
    }
  });

  // Update all elements with data-i18n-placeholder attribute
  document.querySelectorAll("[data-i18n-placeholder]").forEach((el) => {
    const key = el.getAttribute("data-i18n-placeholder");
    if (key && t(key) !== key) {
      el.setAttribute("placeholder", t(key));
    }
  });

  // Update language selector value
  const langSelect = document.getElementById("languageSelect");
  if (langSelect) langSelect.value = currentLanguage;

  // Legacy support: Update specific elements that may not use data-i18n
  // Header - keep the image, update text
  const headerTitle = document.querySelector(".header-title");
  if (headerTitle) {
    const img = headerTitle.querySelector("img");
    if (img) {
      headerTitle.innerHTML = "";
      headerTitle.appendChild(img);
      headerTitle.appendChild(document.createTextNode(" " + t("appTitle")));
    }
  }

  document.getElementById("screenshotBtn")?.setAttribute("title", t("captureScreenshot"));
  document.getElementById("refreshBtn")?.setAttribute("title", t("refreshConnection"));

  // Input placeholder
  const inputField = document.getElementById("inputField");
  if (inputField) inputField.placeholder = t("inputPlaceholder");

  // Quick actions with SVG icons
  document.querySelectorAll(".quick-action").forEach((btn) => {
    const action = btn.dataset.action;
    const svg = btn.querySelector("svg")?.outerHTML || "";
    switch (action) {
      case "summarize":
        btn.innerHTML = svg + " " + t("summarize");
        break;
      case "explain":
        btn.innerHTML = svg + " " + t("explain");
        break;
      case "key-points":
        btn.innerHTML = svg + " " + t("keyPoints");
        break;
      case "translate":
        btn.innerHTML = svg + " " + t("translate");
        break;
      case "save-page":
        btn.innerHTML = svg + " " + t("savePage");
        break;
    }
  });

  // Input tools with SVG icons
  document.querySelectorAll(".input-tool-btn").forEach((btn) => {
    const svg = btn.querySelector("svg")?.outerHTML || "";
    if (btn.id === "attachPageBtn") {
      btn.innerHTML = svg + " " + t("attachPage");
    } else if (btn.id === "attachScreenshotBtn") {
      btn.innerHTML = svg + " " + t("attachScreenshot");
    } else if (btn.id === "attachFileBtn") {
      btn.innerHTML = svg + " " + t("attachFile");
    }
  });
}

// Get target language name for translation prompt
function getTargetLanguageName() {
  const langNames = {
    en: "English",
    fr: "French",
    es: "Spanish",
    de: "German",
    it: "Italian",
    pt: "Portuguese",
    nl: "Dutch",
  };
  return langNames[currentLanguage] || "English";
}
