/**
 * Centralized translations for WhytCard
 * Single source of truth for all UI text in the extension
 */

export interface LocaleStrings {
  // Extension messages
  "welcome.message": string;
  "welcome.openSidebar": string;
  "welcome.later": string;
  "error.noWorkspace": string;
  "error.noModel": string;
  "error.apiError": string;
  "error.cancelled": string;
  "generation.success": string;

  // Setup page
  "setup.title": string;
  "setup.description": string;
  "setup.context7Label": string;
  "setup.context7Desc": string;
  "setup.context7Placeholder": string;
  "setup.tavilyLabel": string;
  "setup.tavilyDesc": string;
  "setup.tavilyPlaceholder": string;
  "setup.modelLabel": string;
  "setup.modelDesc": string;
  "setup.getApiKey": string;
  "setup.optional": string;
  "setup.startChat": string;
  "setup.skip": string;
  "setup.githubConnected": string;
  "setup.githubConnect": string;
  "setup.modelsAvailable": string;

  // Chat page
  "chat.title": string;
  "chat.welcome": string;
  "chat.placeholder": string;
  "chat.send": string;
  "chat.settings": string;
  "chat.newChat": string;
  "chat.generate": string;
  "chat.openFile": string;
  "chat.filesCreated": string;
  "chat.noApiWarning": string;
  "chat.configureNow": string;

  // Thinking bubble
  "thinking.title": string;
  "thinking.analyzing": string;
  "thinking.searchingContext7": string;
  "thinking.searchingTavily": string;
  "thinking.reasoning": string;
  "thinking.noResults": string;
  "thinking.foundDocs": string;
  "thinking.collapse": string;
  "thinking.expand": string;

  // Project detection
  "project.detected": string;
  "project.framework": string;
  "project.language": string;
  "project.styling": string;
  "project.database": string;
  "project.testing": string;
  "project.packageManager": string;

  // Natural responses
  "response.greeting": string;
  "response.askProject": string;
  "response.understood": string;
  "response.suggestion": string;
  "response.question": string;
  "response.ready": string;
  "response.generating": string;
  "response.done": string;
  "response.error": string;

  // Confirmations
  "confirm.generate": string;
  "confirm.overwrite": string;
  "confirm.yes": string;
  "confirm.no": string;
  "confirm.later": string;

  // Templates
  "templates.title": string;
  "templates.new": string;
  "templates.save": string;
  "templates.use": string;
  "templates.delete": string;
  "templates.export": string;
  "templates.import": string;
  "templates.saveConversation": string;
  "templates.noTemplates": string;
  "templates.saved": string;

  // MCP Servers
  "mcp.filesystem.name": string;
  "mcp.filesystem.desc": string;
  "mcp.tavily.name": string;
  "mcp.tavily.desc": string;
  "mcp.sequential.name": string;
  "mcp.sequential.desc": string;
  "mcp.context7.name": string;
  "mcp.context7.desc": string;
  "mcp.memory.name": string;
  "mcp.memory.desc": string;
  "mcp.puppeteer.name": string;
  "mcp.puppeteer.desc": string;
  "mcp.github.name": string;
  "mcp.github.desc": string;
  "mcp.sqlite.name": string;
  "mcp.sqlite.desc": string;
}

export type LocaleKey = keyof LocaleStrings;
export type SupportedLanguage = "en" | "fr" | "es" | "de" | "it" | "pt" | "nl";

const en: LocaleStrings = {
  // Extension messages
  "welcome.message":
    "Welcome to WhytCard! I'll help you set up your workspace for the best experience.",
  "welcome.openSidebar": "Open WhytCard",
  "welcome.later": "Later",
  "error.noWorkspace": "Please open a workspace folder first.",
  "error.noModel": "No language model available. Make sure GitHub Copilot is installed.",
  "error.apiError": "Something went wrong with the API call. Let me try again.",
  "error.cancelled": "Operation cancelled.",
  "generation.success": "Done! I created {count} files for you.",

  // Setup page
  "setup.title": "Let's set up WhytCard",
  "setup.description":
    "Add your API keys to unlock the full experience. I'll use them to search official documentation and give you better answers.",
  "setup.context7Label": "Context7 API Key",
  "setup.context7Desc": "Lets me search official docs for React, Vue, Next.js, and more",
  "setup.context7Placeholder": "Your Context7 key",
  "setup.tavilyLabel": "Tavily API Key",
  "setup.tavilyDesc": "Lets me search the web for up-to-date information",
  "setup.tavilyPlaceholder": "Your Tavily key",
  "setup.modelLabel": "Preferred AI Model",
  "setup.modelDesc": "Which Copilot model should I use?",
  "setup.getApiKey": "Get a key",
  "setup.optional": "optional",
  "setup.startChat": "Let's go!",
  "setup.skip": "Skip for now",
  "setup.githubConnected": "Connected to GitHub",
  "setup.githubConnect": "Connect to GitHub",
  "setup.modelsAvailable": "{count} models available",

  // Chat page
  "chat.title": "WhytCard",
  "chat.welcome": "Hey! I'm here to help you configure your workspace. Tell me about your project!",
  "chat.placeholder": "Tell me about your project...",
  "chat.send": "Send",
  "chat.settings": "Settings",
  "chat.newChat": "New chat",
  "chat.generate": "Generate files",
  "chat.openFile": "Open",
  "chat.filesCreated": "files created",
  "chat.noApiWarning": "No API keys configured. My answers might be less accurate.",
  "chat.configureNow": "Configure now",

  // Thinking bubble
  "thinking.title": "Let me think about this...",
  "thinking.analyzing": "Understanding your request...",
  "thinking.searchingContext7": "Searching {tech} documentation...",
  "thinking.searchingTavily": "Looking up latest information...",
  "thinking.reasoning": "Putting it all together...",
  "thinking.noResults": "No relevant docs found, using my knowledge",
  "thinking.foundDocs": "Found helpful documentation",
  "thinking.collapse": "Hide details",
  "thinking.expand": "Show my reasoning",

  // Project detection
  "project.detected": "Here's what I found:",
  "project.framework": "Framework",
  "project.language": "Language",
  "project.styling": "Styling",
  "project.database": "Database",
  "project.testing": "Testing",
  "project.packageManager": "Package manager",

  // Natural responses
  "response.greeting": "Hey!",
  "response.askProject": "What kind of project are you working on?",
  "response.understood": "Got it!",
  "response.suggestion": "Here's what I'd suggest:",
  "response.question": "Quick question:",
  "response.ready": "I'm ready to generate your config files. Want me to go ahead?",
  "response.generating": "Creating your files...",
  "response.done": "All done!",
  "response.error": "Oops, something went wrong:",

  // Confirmations
  "confirm.generate": "Ready to create the files?",
  "confirm.overwrite": "This will overwrite existing files. Continue?",
  "confirm.yes": "Yes, go ahead",
  "confirm.no": "No, wait",
  "confirm.later": "Maybe later",

  // Templates
  "templates.title": "Conversation Templates",
  "templates.new": "New Template",
  "templates.save": "Save as Template",
  "templates.use": "Use Template",
  "templates.delete": "Delete Template",
  "templates.export": "Export Templates",
  "templates.import": "Import Templates",
  "templates.saveConversation": "Save this conversation as a template",
  "templates.noTemplates": "No templates yet. Create one from a conversation!",
  "templates.saved": "Template saved!",

  // MCP Servers
  "mcp.filesystem.name": "Codebase Access",
  "mcp.filesystem.desc": "Allows reading files from your workspace (Essential).",
  "mcp.tavily.name": "Tavily Web Search & Crawl",
  "mcp.tavily.desc": "Search and crawl the web for up-to-date information.",
  "mcp.sequential.name": "Sequential Thinking",
  "mcp.sequential.desc": "Advanced problem-solving with dynamic thought processes.",
  "mcp.context7.name": "Context7 Docs",
  "mcp.context7.desc": "Get up-to-date documentation for libraries and frameworks.",
  "mcp.memory.name": "Knowledge Graph (Memory)",
  "mcp.memory.desc": "Persistent memory and knowledge graph for your project.",
  "mcp.puppeteer.name": "Browser Automation",
  "mcp.puppeteer.desc": "Control a headless browser for testing and scraping.",
  "mcp.github.name": "GitHub",
  "mcp.github.desc": "Interact with GitHub repositories, issues, and PRs.",
  "mcp.sqlite.name": "SQLite",
  "mcp.sqlite.desc": "Query SQLite databases in your workspace.",
};

const fr: LocaleStrings = {
  // Extension messages
  "welcome.message":
    "Bienvenue dans WhytCard ! Je vais t'aider a configurer ton workspace pour une experience optimale.",
  "welcome.openSidebar": "Ouvrir WhytCard",
  "welcome.later": "Plus tard",
  "error.noWorkspace": "Ouvre d'abord un dossier workspace.",
  "error.noModel": "Pas de modèle disponible. Vérifie que GitHub Copilot est installé.",
  "error.apiError": "Problème avec l'API. Je réessaie.",
  "error.cancelled": "Opération annulée.",
  "generation.success": "C'est fait ! J'ai créé {count} fichiers.",

  // Setup page
  "setup.title": "Configuration de WhytCard",
  "setup.description":
    "Ajoute tes clés API pour débloquer toutes les fonctionnalités. Je les utilise pour chercher dans la documentation officielle et te donner de meilleures réponses.",
  "setup.context7Label": "Clé API Context7",
  "setup.context7Desc": "Me permet de chercher dans les docs officielles (React, Vue, Next.js...)",
  "setup.context7Placeholder": "Ta clé Context7",
  "setup.tavilyLabel": "Clé API Tavily",
  "setup.tavilyDesc": "Me permet de chercher sur le web les infos les plus récentes",
  "setup.tavilyPlaceholder": "Ta clé Tavily",
  "setup.modelLabel": "Modèle IA préféré",
  "setup.modelDesc": "Quel modèle Copilot je dois utiliser ?",
  "setup.getApiKey": "Obtenir une clé",
  "setup.optional": "optionnel",
  "setup.startChat": "C'est parti !",
  "setup.skip": "Passer pour l'instant",
  "setup.githubConnected": "Connecté à GitHub",
  "setup.githubConnect": "Se connecter à GitHub",
  "setup.modelsAvailable": "{count} modèles disponibles",

  // Chat page
  "chat.title": "WhytCard",
  "chat.welcome":
    "Salut ! Je suis la pour t'aider a configurer ton workspace. Parle-moi de ton projet !",
  "chat.placeholder": "Parle-moi de ton projet...",
  "chat.send": "Envoyer",
  "chat.settings": "Paramètres",
  "chat.newChat": "Nouveau chat",
  "chat.generate": "Générer les fichiers",
  "chat.openFile": "Ouvrir",
  "chat.filesCreated": "fichiers créés",
  "chat.noApiWarning": "Pas de clés API configurées. Mes réponses peuvent être moins précises.",
  "chat.configureNow": "Configurer",

  // Thinking bubble
  "thinking.title": "Je réfléchis...",
  "thinking.analyzing": "Je comprends ta demande...",
  "thinking.searchingContext7": "Je cherche dans la doc {tech}...",
  "thinking.searchingTavily": "Je cherche les dernières infos...",
  "thinking.reasoning": "Je rassemble tout ça...",
  "thinking.noResults": "Pas de doc trouvée, j'utilise mes connaissances",
  "thinking.foundDocs": "J'ai trouvé de la doc utile",
  "thinking.collapse": "Masquer les détails",
  "thinking.expand": "Voir mon raisonnement",

  // Project detection
  "project.detected": "Voilà ce que j'ai trouvé :",
  "project.framework": "Framework",
  "project.language": "Langage",
  "project.styling": "Style CSS",
  "project.database": "Base de données",
  "project.testing": "Tests",
  "project.packageManager": "Gestionnaire de paquets",

  // Natural responses
  "response.greeting": "Salut !",
  "response.askProject": "C'est quoi ton projet ?",
  "response.understood": "Compris !",
  "response.suggestion": "Voilà ce que je te propose :",
  "response.question": "Petite question :",
  "response.ready": "Je suis prêt à générer tes fichiers de config. Je lance ?",
  "response.generating": "Je crée tes fichiers...",
  "response.done": "C'est fait !",
  "response.error": "Oups, y'a eu un souci :",

  // Confirmations
  "confirm.generate": "Je génère les fichiers ?",
  "confirm.overwrite": "Ça va écraser les fichiers existants. On continue ?",
  "confirm.yes": "Oui, vas-y",
  "confirm.no": "Non, attends",
  "confirm.later": "Plus tard",

  // Templates
  "templates.title": "Templates de conversation",
  "templates.new": "Nouveau template",
  "templates.save": "Sauvegarder comme template",
  "templates.use": "Utiliser ce template",
  "templates.delete": "Supprimer le template",
  "templates.export": "Exporter les templates",
  "templates.import": "Importer des templates",
  "templates.saveConversation": "Sauvegarder cette conversation comme template",
  "templates.noTemplates": "Pas encore de templates. Crée-en un à partir d'une conversation !",
  "templates.saved": "Template sauvegardé !",

  // MCP Servers
  "mcp.filesystem.name": "Accès au Codebase",
  "mcp.filesystem.desc": "Permet de lire les fichiers de votre espace de travail (Essentiel).",
  "mcp.tavily.name": "Recherche & Crawl Web Tavily",
  "mcp.tavily.desc": "Recherche et explore le web pour des informations à jour.",
  "mcp.sequential.name": "Pensée Séquentielle",
  "mcp.sequential.desc": "Résolution de problèmes avancée avec processus de pensée dynamique.",
  "mcp.context7.name": "Documentation Context7",
  "mcp.context7.desc": "Obtenez la documentation à jour pour les bibliothèques et frameworks.",
  "mcp.memory.name": "Graphe de Connaissances (Mémoire)",
  "mcp.memory.desc": "Mémoire persistante et graphe de connaissances pour votre projet.",
  "mcp.puppeteer.name": "Automatisation Navigateur",
  "mcp.puppeteer.desc": "Contrôlez un navigateur sans tête pour les tests et le scraping.",
  "mcp.github.name": "GitHub",
  "mcp.github.desc": "Interagissez avec les dépôts, tickets et PRs GitHub.",
  "mcp.sqlite.name": "SQLite",
  "mcp.sqlite.desc": "Interrogez les bases de données SQLite dans votre espace de travail.",
};

const es: LocaleStrings = {
  // Extension messages
  "welcome.message":
    "Bienvenido a WhytCard! Te ayudare a configurar tu workspace para la mejor experiencia.",
  "welcome.openSidebar": "Abrir WhytCard",
  "welcome.later": "Más tarde",
  "error.noWorkspace": "Primero abre una carpeta de workspace.",
  "error.noModel": "No hay modelo disponible. Asegúrate de que GitHub Copilot está instalado.",
  "error.apiError": "Hubo un problema con la API. Voy a intentar de nuevo.",
  "error.cancelled": "Operación cancelada.",
  "generation.success": "¡Listo! Creé {count} archivos.",

  // Setup page
  "setup.title": "Configuremos WhytCard",
  "setup.description":
    "Agrega tus claves API para desbloquear todas las funciones. Las uso para buscar en la documentación oficial y darte mejores respuestas.",
  "setup.context7Label": "Clave API Context7",
  "setup.context7Desc": "Me permite buscar en docs oficiales (React, Vue, Next.js...)",
  "setup.context7Placeholder": "Tu clave Context7",
  "setup.tavilyLabel": "Clave API Tavily",
  "setup.tavilyDesc": "Me permite buscar información actualizada en la web",
  "setup.tavilyPlaceholder": "Tu clave Tavily",
  "setup.modelLabel": "Modelo IA preferido",
  "setup.modelDesc": "¿Qué modelo de Copilot uso?",
  "setup.getApiKey": "Obtener clave",
  "setup.optional": "opcional",
  "setup.startChat": "¡Vamos!",
  "setup.skip": "Saltar por ahora",
  "setup.githubConnected": "Conectado a GitHub",
  "setup.githubConnect": "Conectar a GitHub",
  "setup.modelsAvailable": "{count} modelos disponibles",

  // Chat page
  "chat.title": "WhytCard",
  "chat.welcome":
    "Hola! Estoy aqui para ayudarte a configurar tu workspace. Cuentame de tu proyecto!",
  "chat.placeholder": "Cuéntame de tu proyecto...",
  "chat.send": "Enviar",
  "chat.settings": "Ajustes",
  "chat.newChat": "Nuevo chat",
  "chat.generate": "Generar archivos",
  "chat.openFile": "Abrir",
  "chat.filesCreated": "archivos creados",
  "chat.noApiWarning": "Sin claves API configuradas. Mis respuestas pueden ser menos precisas.",
  "chat.configureNow": "Configurar",

  // Thinking bubble
  "thinking.title": "Déjame pensar...",
  "thinking.analyzing": "Entendiendo tu solicitud...",
  "thinking.searchingContext7": "Buscando en la documentación de {tech}...",
  "thinking.searchingTavily": "Buscando información actualizada...",
  "thinking.reasoning": "Juntando todo...",
  "thinking.noResults": "No encontré docs, uso mis conocimientos",
  "thinking.foundDocs": "Encontré documentación útil",
  "thinking.collapse": "Ocultar detalles",
  "thinking.expand": "Ver mi razonamiento",

  // Project detection
  "project.detected": "Esto es lo que encontré:",
  "project.framework": "Framework",
  "project.language": "Lenguaje",
  "project.styling": "Estilos",
  "project.database": "Base de datos",
  "project.testing": "Tests",
  "project.packageManager": "Gestor de paquetes",

  // Natural responses
  "response.greeting": "¡Hola!",
  "response.askProject": "¿En qué proyecto estás trabajando?",
  "response.understood": "¡Entendido!",
  "response.suggestion": "Esto es lo que sugiero:",
  "response.question": "Una pregunta rápida:",
  "response.ready": "Listo para generar tus archivos. ¿Procedo?",
  "response.generating": "Creando tus archivos...",
  "response.done": "¡Listo!",
  "response.error": "Ups, algo salió mal:",

  // Confirmations
  "confirm.generate": "¿Genero los archivos?",
  "confirm.overwrite": "Esto sobrescribirá archivos existentes. ¿Continúo?",
  "confirm.yes": "Sí, adelante",
  "confirm.no": "No, espera",
  "confirm.later": "Después",

  // Templates
  "templates.title": "Plantillas de conversación",
  "templates.new": "Nueva plantilla",
  "templates.save": "Guardar como plantilla",
  "templates.use": "Usar plantilla",
  "templates.delete": "Eliminar plantilla",
  "templates.export": "Exportar plantillas",
  "templates.import": "Importar plantillas",
  "templates.saveConversation": "Guardar esta conversación como plantilla",
  "templates.noTemplates": "Aún no hay plantillas. ¡Crea una desde una conversación!",
  "templates.saved": "¡Plantilla guardada!",

  // MCP Servers
  "mcp.filesystem.name": "Acceso al Código",
  "mcp.filesystem.desc": "Permite leer archivos de tu espacio de trabajo (Esencial).",
  "mcp.tavily.name": "Búsqueda y Rastreo Web Tavily",
  "mcp.tavily.desc": "Busca y rastrea la web para obtener información actualizada.",
  "mcp.sequential.name": "Pensamiento Secuencial",
  "mcp.sequential.desc": "Resolución de problemas avanzada con procesos de pensamiento dinámicos.",
  "mcp.context7.name": "Documentación Context7",
  "mcp.context7.desc": "Obtén documentación actualizada para bibliotecas y frameworks.",
  "mcp.memory.name": "Gráfico de Conocimiento (Memoria)",
  "mcp.memory.desc": "Memoria persistente y gráfico de conocimiento para tu proyecto.",
  "mcp.puppeteer.name": "Automatización de Navegador",
  "mcp.puppeteer.desc": "Controla un navegador headless para pruebas y scraping.",
  "mcp.github.name": "GitHub",
  "mcp.github.desc": "Interactúa con repositorios, issues y PRs de GitHub.",
  "mcp.sqlite.name": "SQLite",
  "mcp.sqlite.desc": "Consulta bases de datos SQLite en tu espacio de trabajo.",
};

const de: LocaleStrings = {
  // Extension messages
  "welcome.message":
    "Willkommen bei WhytCard! Ich helfe dir, deinen Workspace fur das beste Erlebnis einzurichten.",
  "welcome.openSidebar": "WhytCard öffnen",
  "welcome.later": "Später",
  "error.noWorkspace": "Bitte öffne zuerst einen Workspace-Ordner.",
  "error.noModel": "Kein Modell verfügbar. Stelle sicher, dass GitHub Copilot installiert ist.",
  "error.apiError": "API-Problem. Ich versuche es nochmal.",
  "error.cancelled": "Vorgang abgebrochen.",
  "generation.success": "Fertig! Ich habe {count} Dateien erstellt.",

  // Setup page
  "setup.title": "WhytCard einrichten",
  "setup.description":
    "Füge deine API-Schlüssel hinzu, um alle Funktionen freizuschalten. Ich nutze sie, um in der offiziellen Dokumentation zu suchen und dir bessere Antworten zu geben.",
  "setup.context7Label": "Context7 API-Schlüssel",
  "setup.context7Desc": "Ermöglicht mir, in offiziellen Docs zu suchen (React, Vue, Next.js...)",
  "setup.context7Placeholder": "Dein Context7-Schlüssel",
  "setup.tavilyLabel": "Tavily API-Schlüssel",
  "setup.tavilyDesc": "Ermöglicht mir, aktuelle Informationen im Web zu suchen",
  "setup.tavilyPlaceholder": "Dein Tavily-Schlüssel",
  "setup.modelLabel": "Bevorzugtes KI-Modell",
  "setup.modelDesc": "Welches Copilot-Modell soll ich verwenden?",
  "setup.getApiKey": "Schlüssel holen",
  "setup.optional": "optional",
  "setup.startChat": "Los geht's!",
  "setup.skip": "Erstmal überspringen",
  "setup.githubConnected": "Mit GitHub verbunden",
  "setup.githubConnect": "Mit GitHub verbinden",
  "setup.modelsAvailable": "{count} Modelle verfugbar",

  // Chat page
  "chat.title": "WhytCard",
  "chat.welcome":
    "Hey! Ich bin hier, um dir bei der Workspace-Konfiguration zu helfen. Erzähl mir von deinem Projekt!",
  "chat.placeholder": "Erzähl mir von deinem Projekt...",
  "chat.send": "Senden",
  "chat.settings": "Einstellungen",
  "chat.newChat": "Neuer Chat",
  "chat.generate": "Dateien generieren",
  "chat.openFile": "Öffnen",
  "chat.filesCreated": "Dateien erstellt",
  "chat.noApiWarning":
    "Keine API-Schlüssel konfiguriert. Meine Antworten könnten weniger genau sein.",
  "chat.configureNow": "Jetzt konfigurieren",

  // Thinking bubble
  "thinking.title": "Lass mich nachdenken...",
  "thinking.analyzing": "Ich verstehe deine Anfrage...",
  "thinking.searchingContext7": "Suche in der {tech}-Dokumentation...",
  "thinking.searchingTavily": "Suche nach aktuellen Informationen...",
  "thinking.reasoning": "Ich füge alles zusammen...",
  "thinking.noResults": "Keine Docs gefunden, nutze mein Wissen",
  "thinking.foundDocs": "Hilfreiche Dokumentation gefunden",
  "thinking.collapse": "Details ausblenden",
  "thinking.expand": "Meine Überlegung zeigen",

  // Project detection
  "project.detected": "Das habe ich gefunden:",
  "project.framework": "Framework",
  "project.language": "Sprache",
  "project.styling": "Styling",
  "project.database": "Datenbank",
  "project.testing": "Tests",
  "project.packageManager": "Paketmanager",

  // Natural responses
  "response.greeting": "Hey!",
  "response.askProject": "An welchem Projekt arbeitest du?",
  "response.understood": "Verstanden!",
  "response.suggestion": "Das würde ich vorschlagen:",
  "response.question": "Kurze Frage:",
  "response.ready": "Ich bin bereit, deine Konfigurationsdateien zu erstellen. Soll ich loslegen?",
  "response.generating": "Erstelle deine Dateien...",
  "response.done": "Fertig!",
  "response.error": "Ups, etwas ist schiefgelaufen:",

  // Confirmations
  "confirm.generate": "Soll ich die Dateien erstellen?",
  "confirm.overwrite": "Das überschreibt vorhandene Dateien. Weitermachen?",
  "confirm.yes": "Ja, mach weiter",
  "confirm.no": "Nein, warte",
  "confirm.later": "Später",

  // Templates
  "templates.title": "Gesprächsvorlagen",
  "templates.new": "Neue Vorlage",
  "templates.save": "Als Vorlage speichern",
  "templates.use": "Vorlage verwenden",
  "templates.delete": "Vorlage löschen",
  "templates.export": "Vorlagen exportieren",
  "templates.import": "Vorlagen importieren",
  "templates.saveConversation": "Dieses Gespräch als Vorlage speichern",
  "templates.noTemplates": "Noch keine Vorlagen. Erstelle eine aus einem Gespräch!",
  "templates.saved": "Vorlage gespeichert!",

  // MCP Servers
  "mcp.filesystem.name": "Codebase-Zugriff",
  "mcp.filesystem.desc": "Ermöglicht das Lesen von Dateien in Ihrem Arbeitsbereich (Wesentlich).",
  "mcp.tavily.name": "Tavily Websuche & Crawl",
  "mcp.tavily.desc": "Sucht und crawlt das Web nach aktuellen Informationen.",
  "mcp.sequential.name": "Sequentielles Denken",
  "mcp.sequential.desc": "Fortgeschrittene Problemlösung mit dynamischen Denkprozessen.",
  "mcp.context7.name": "Context7 Dokumentation",
  "mcp.context7.desc": "Erhalten Sie aktuelle Dokumentation für Bibliotheken und Frameworks.",
  "mcp.memory.name": "Wissensgraph (Gedächtnis)",
  "mcp.memory.desc": "Persistenter Speicher und Wissensgraph für Ihr Projekt.",
  "mcp.puppeteer.name": "Browser-Automatisierung",
  "mcp.puppeteer.desc": "Steuern Sie einen Headless-Browser für Tests und Scraping.",
  "mcp.github.name": "GitHub",
  "mcp.github.desc": "Interagieren Sie mit GitHub-Repositories, Issues und PRs.",
  "mcp.sqlite.name": "SQLite",
  "mcp.sqlite.desc": "Fragen Sie SQLite-Datenbanken in Ihrem Arbeitsbereich ab.",
};

const it: LocaleStrings = {
  // Extension messages
  "welcome.message":
    "Benvenuto in WhytCard! Ti aiutero a configurare il tuo workspace per la migliore esperienza.",
  "welcome.openSidebar": "Apri WhytCard",
  "welcome.later": "Più tardi",
  "error.noWorkspace": "Apri prima una cartella workspace.",
  "error.noModel": "Nessun modello disponibile. Assicurati che GitHub Copilot sia installato.",
  "error.apiError": "Qualcosa non ha funzionato con l'API. Riprovo.",
  "error.cancelled": "Operazione annullata.",
  "generation.success": "Fatto! Ho creato {count} file.",

  // Setup page
  "setup.title": "Configuriamo WhytCard",
  "setup.description":
    "Aggiungi le tue chiavi API per sbloccare tutte le funzionalità. Le uso per cercare nella documentazione ufficiale e darti risposte migliori.",
  "setup.context7Label": "Chiave API Context7",
  "setup.context7Desc": "Mi permette di cercare nei docs ufficiali (React, Vue, Next.js...)",
  "setup.context7Placeholder": "La tua chiave Context7",
  "setup.tavilyLabel": "Chiave API Tavily",
  "setup.tavilyDesc": "Mi permette di cercare informazioni aggiornate sul web",
  "setup.tavilyPlaceholder": "La tua chiave Tavily",
  "setup.modelLabel": "Modello IA preferito",
  "setup.modelDesc": "Quale modello Copilot devo usare?",
  "setup.getApiKey": "Ottieni chiave",
  "setup.optional": "opzionale",
  "setup.startChat": "Iniziamo!",
  "setup.skip": "Salta per ora",
  "setup.githubConnected": "Connesso a GitHub",
  "setup.githubConnect": "Connetti a GitHub",
  "setup.modelsAvailable": "{count} modelli disponibili",

  // Chat page
  "chat.title": "WhytCard",
  "chat.welcome":
    "Ciao! Sono qui per aiutarti a configurare il tuo workspace. Parlami del tuo progetto!",
  "chat.placeholder": "Parlami del tuo progetto...",
  "chat.send": "Invia",
  "chat.settings": "Impostazioni",
  "chat.newChat": "Nuova chat",
  "chat.generate": "Genera file",
  "chat.openFile": "Apri",
  "chat.filesCreated": "file creati",
  "chat.noApiWarning":
    "Nessuna chiave API configurata. Le mie risposte potrebbero essere meno accurate.",
  "chat.configureNow": "Configura ora",

  // Thinking bubble
  "thinking.title": "Fammi pensare...",
  "thinking.analyzing": "Capisco la tua richiesta...",
  "thinking.searchingContext7": "Cerco nella documentazione {tech}...",
  "thinking.searchingTavily": "Cerco informazioni aggiornate...",
  "thinking.reasoning": "Metto tutto insieme...",
  "thinking.noResults": "Nessun doc trovato, uso le mie conoscenze",
  "thinking.foundDocs": "Trovata documentazione utile",
  "thinking.collapse": "Nascondi dettagli",
  "thinking.expand": "Mostra il mio ragionamento",

  // Project detection
  "project.detected": "Ecco cosa ho trovato:",
  "project.framework": "Framework",
  "project.language": "Linguaggio",
  "project.styling": "Stile",
  "project.database": "Database",
  "project.testing": "Test",
  "project.packageManager": "Gestore pacchetti",

  // Natural responses
  "response.greeting": "Ciao!",
  "response.askProject": "A che progetto stai lavorando?",
  "response.understood": "Capito!",
  "response.suggestion": "Ecco cosa suggerirei:",
  "response.question": "Una domanda veloce:",
  "response.ready": "Sono pronto a generare i tuoi file di config. Procedo?",
  "response.generating": "Creo i tuoi file...",
  "response.done": "Fatto!",
  "response.error": "Ops, qualcosa è andato storto:",

  // Confirmations
  "confirm.generate": "Genero i file?",
  "confirm.overwrite": "Questo sovrascriverà i file esistenti. Continuo?",
  "confirm.yes": "Sì, procedi",
  "confirm.no": "No, aspetta",
  "confirm.later": "Più tardi",

  // Templates
  "templates.title": "Template conversazione",
  "templates.new": "Nuovo template",
  "templates.save": "Salva come template",
  "templates.use": "Usa template",
  "templates.delete": "Elimina template",
  "templates.export": "Esporta template",
  "templates.import": "Importa template",
  "templates.saveConversation": "Salva questa conversazione come template",
  "templates.noTemplates": "Nessun template ancora. Creane uno da una conversazione!",
  "templates.saved": "Template salvato!",

  // MCP Servers
  "mcp.filesystem.name": "Accesso Codebase",
  "mcp.filesystem.desc": "Permette di leggere i file nel tuo workspace (Essenziale).",
  "mcp.tavily.name": "Ricerca & Crawl Web Tavily",
  "mcp.tavily.desc": "Cerca e naviga il web per informazioni aggiornate.",
  "mcp.sequential.name": "Pensiero Sequenziale",
  "mcp.sequential.desc": "Problem solving avanzato con processi di pensiero dinamici.",
  "mcp.context7.name": "Documentazione Context7",
  "mcp.context7.desc": "Ottieni documentazione aggiornata per librerie e framework.",
  "mcp.memory.name": "Grafo della Conoscenza (Memoria)",
  "mcp.memory.desc": "Memoria persistente e grafo della conoscenza per il tuo progetto.",
  "mcp.puppeteer.name": "Automazione Browser",
  "mcp.puppeteer.desc": "Controlla un browser headless per test e scraping.",
  "mcp.github.name": "GitHub",
  "mcp.github.desc": "Interagisci con repository, issue e PR di GitHub.",
  "mcp.sqlite.name": "SQLite",
  "mcp.sqlite.desc": "Interroga database SQLite nel tuo workspace.",
};

const pt: LocaleStrings = {
  // Extension messages
  "welcome.message":
    "Bem-vindo ao WhytCard! Vou te ajudar a configurar seu workspace para a melhor experiência.",
  "welcome.openSidebar": "Abrir WhytCard",
  "welcome.later": "Mais tarde",
  "error.noWorkspace": "Por favor, abra uma pasta de workspace primeiro.",
  "error.noModel":
    "Nenhum modelo disponível. Certifique-se de que o GitHub Copilot está instalado.",
  "error.apiError": "Algo deu errado com a API. Vou tentar novamente.",
  "error.cancelled": "Operação cancelada.",
  "generation.success": "Pronto! Criei {count} arquivos.",

  // Setup page
  "setup.title": "Vamos configurar o WhytCard",
  "setup.description":
    "Adicione suas chaves de API para desbloquear todas as funcionalidades. Uso-as para buscar na documentação oficial e te dar melhores respostas.",
  "setup.context7Label": "Chave API Context7",
  "setup.context7Desc": "Me permite buscar nos docs oficiais (React, Vue, Next.js...)",
  "setup.context7Placeholder": "Sua chave Context7",
  "setup.tavilyLabel": "Chave API Tavily",
  "setup.tavilyDesc": "Me permite buscar informações atualizadas na web",
  "setup.tavilyPlaceholder": "Sua chave Tavily",
  "setup.modelLabel": "Modelo de IA preferido",
  "setup.modelDesc": "Qual modelo Copilot devo usar?",
  "setup.getApiKey": "Obter chave",
  "setup.optional": "opcional",
  "setup.startChat": "Vamos lá!",
  "setup.skip": "Pular por agora",
  "setup.githubConnected": "Conectado ao GitHub",
  "setup.githubConnect": "Conectar ao GitHub",
  "setup.modelsAvailable": "{count} modelos disponíveis",

  // Chat page
  "chat.title": "WhytCard",
  "chat.welcome":
    "Olá! Estou aqui para te ajudar a configurar seu workspace. Me conte sobre seu projeto!",
  "chat.placeholder": "Me conte sobre seu projeto...",
  "chat.send": "Enviar",
  "chat.settings": "Configurações",
  "chat.newChat": "Nova conversa",
  "chat.generate": "Gerar arquivos",
  "chat.openFile": "Abrir",
  "chat.filesCreated": "arquivos criados",
  "chat.noApiWarning": "Nenhuma chave API configurada. Minhas respostas podem ser menos precisas.",
  "chat.configureNow": "Configurar agora",

  // Thinking bubble
  "thinking.title": "Deixa eu pensar...",
  "thinking.analyzing": "Entendendo sua solicitação...",
  "thinking.searchingContext7": "Buscando na documentação {tech}...",
  "thinking.searchingTavily": "Buscando informações atualizadas...",
  "thinking.reasoning": "Juntando tudo...",
  "thinking.noResults": "Nenhum doc encontrado, usando meu conhecimento",
  "thinking.foundDocs": "Encontrei documentação útil",
  "thinking.collapse": "Ocultar detalhes",
  "thinking.expand": "Mostrar meu raciocínio",

  // Project detection
  "project.detected": "Aqui está o que encontrei:",
  "project.framework": "Framework",
  "project.language": "Linguagem",
  "project.styling": "Estilos",
  "project.database": "Banco de dados",
  "project.testing": "Testes",
  "project.packageManager": "Gerenciador de pacotes",

  // Natural responses
  "response.greeting": "Olá!",
  "response.askProject": "Em que projeto você está trabalhando?",
  "response.understood": "Entendido!",
  "response.suggestion": "Aqui está o que eu sugiro:",
  "response.question": "Uma pergunta rápida:",
  "response.ready": "Pronto para gerar seus arquivos. Prossigo?",
  "response.generating": "Criando seus arquivos...",
  "response.done": "Pronto!",
  "response.error": "Ops, algo deu errado:",

  // Confirmations
  "confirm.generate": "Gero os arquivos?",
  "confirm.overwrite": "Isso vai sobrescrever arquivos existentes. Continuo?",
  "confirm.yes": "Sim, pode continuar",
  "confirm.no": "Não, espera",
  "confirm.later": "Depois",

  // Templates
  "templates.title": "Templates de conversa",
  "templates.new": "Novo template",
  "templates.save": "Salvar como template",
  "templates.use": "Usar template",
  "templates.delete": "Excluir template",
  "templates.export": "Exportar templates",
  "templates.import": "Importar templates",
  "templates.saveConversation": "Salvar esta conversa como template",
  "templates.noTemplates": "Nenhum template ainda. Crie um a partir de uma conversa!",
  "templates.saved": "Template salvo!",

  // MCP Servers
  "mcp.filesystem.name": "Acesso ao Código",
  "mcp.filesystem.desc": "Permite ler arquivos do seu workspace (Essencial).",
  "mcp.tavily.name": "Busca & Crawl Web Tavily",
  "mcp.tavily.desc": "Busca e navega a web por informações atualizadas.",
  "mcp.sequential.name": "Pensamento Sequencial",
  "mcp.sequential.desc": "Resolução de problemas avançada com processos de pensamento dinâmicos.",
  "mcp.context7.name": "Documentação Context7",
  "mcp.context7.desc": "Obtenha documentação atualizada para bibliotecas e frameworks.",
  "mcp.memory.name": "Grafo de Conhecimento (Memória)",
  "mcp.memory.desc": "Memória persistente e grafo de conhecimento para seu projeto.",
  "mcp.puppeteer.name": "Automação de Navegador",
  "mcp.puppeteer.desc": "Controle um navegador headless para testes e scraping.",
  "mcp.github.name": "GitHub",
  "mcp.github.desc": "Interaja com repositórios, issues e PRs do GitHub.",
  "mcp.sqlite.name": "SQLite",
  "mcp.sqlite.desc": "Consulte bancos de dados SQLite no seu workspace.",
};

const nl: LocaleStrings = {
  // Extension messages
  "welcome.message":
    "Welkom bij WhytCard! Ik help je je workspace in te stellen voor de beste ervaring.",
  "welcome.openSidebar": "WhytCard openen",
  "welcome.later": "Later",
  "error.noWorkspace": "Open eerst een workspace-map.",
  "error.noModel": "Geen model beschikbaar. Zorg dat GitHub Copilot is geïnstalleerd.",
  "error.apiError": "Er ging iets mis met de API. Ik probeer het opnieuw.",
  "error.cancelled": "Operatie geannuleerd.",
  "generation.success": "Klaar! Ik heb {count} bestanden aangemaakt.",

  // Setup page
  "setup.title": "WhytCard instellen",
  "setup.description":
    "Voeg je API-sleutels toe om alle functies te ontgrendelen. Ik gebruik ze om in officiële documentatie te zoeken en betere antwoorden te geven.",
  "setup.context7Label": "Context7 API-sleutel",
  "setup.context7Desc": "Laat me zoeken in officiële docs (React, Vue, Next.js...)",
  "setup.context7Placeholder": "Je Context7-sleutel",
  "setup.tavilyLabel": "Tavily API-sleutel",
  "setup.tavilyDesc": "Laat me actuele informatie op het web zoeken",
  "setup.tavilyPlaceholder": "Je Tavily-sleutel",
  "setup.modelLabel": "Voorkeurs AI-model",
  "setup.modelDesc": "Welk Copilot-model moet ik gebruiken?",
  "setup.getApiKey": "Sleutel ophalen",
  "setup.optional": "optioneel",
  "setup.startChat": "Laten we beginnen!",
  "setup.skip": "Nu overslaan",
  "setup.githubConnected": "Verbonden met GitHub",
  "setup.githubConnect": "Verbinden met GitHub",
  "setup.modelsAvailable": "{count} modellen beschikbaar",

  // Chat page
  "chat.title": "WhytCard",
  "chat.welcome":
    "Hoi! Ik ben hier om je te helpen met je workspace-configuratie. Vertel me over je project!",
  "chat.placeholder": "Vertel me over je project...",
  "chat.send": "Verzenden",
  "chat.settings": "Instellingen",
  "chat.newChat": "Nieuwe chat",
  "chat.generate": "Bestanden genereren",
  "chat.openFile": "Openen",
  "chat.filesCreated": "bestanden aangemaakt",
  "chat.noApiWarning":
    "Geen API-sleutels geconfigureerd. Mijn antwoorden kunnen minder nauwkeurig zijn.",
  "chat.configureNow": "Nu configureren",

  // Thinking bubble
  "thinking.title": "Even nadenken...",
  "thinking.analyzing": "Je verzoek begrijpen...",
  "thinking.searchingContext7": "Zoeken in {tech}-documentatie...",
  "thinking.searchingTavily": "Zoeken naar actuele informatie...",
  "thinking.reasoning": "Alles samenbrengen...",
  "thinking.noResults": "Geen docs gevonden, gebruik mijn kennis",
  "thinking.foundDocs": "Nuttige documentatie gevonden",
  "thinking.collapse": "Details verbergen",
  "thinking.expand": "Mijn redenering tonen",

  // Project detection
  "project.detected": "Dit heb ik gevonden:",
  "project.framework": "Framework",
  "project.language": "Taal",
  "project.styling": "Styling",
  "project.database": "Database",
  "project.testing": "Testen",
  "project.packageManager": "Pakketbeheerder",

  // Natural responses
  "response.greeting": "Hoi!",
  "response.askProject": "Aan welk project werk je?",
  "response.understood": "Begrepen!",
  "response.suggestion": "Dit zou ik voorstellen:",
  "response.question": "Even een vraag:",
  "response.ready": "Ik ben klaar om je configuratiebestanden te genereren. Zal ik doorgaan?",
  "response.generating": "Je bestanden aanmaken...",
  "response.done": "Klaar!",
  "response.error": "Oeps, er ging iets mis:",

  // Confirmations
  "confirm.generate": "Zal ik de bestanden aanmaken?",
  "confirm.overwrite": "Dit overschrijft bestaande bestanden. Doorgaan?",
  "confirm.yes": "Ja, ga door",
  "confirm.no": "Nee, wacht",
  "confirm.later": "Later",

  // Templates
  "templates.title": "Gesprekssjablonen",
  "templates.new": "Nieuw sjabloon",
  "templates.save": "Opslaan als sjabloon",
  "templates.use": "Sjabloon gebruiken",
  "templates.delete": "Sjabloon verwijderen",
  "templates.export": "Sjablonen exporteren",
  "templates.import": "Sjablonen importeren",
  "templates.saveConversation": "Dit gesprek als sjabloon opslaan",
  "templates.noTemplates": "Nog geen sjablonen. Maak er een van een gesprek!",
  "templates.saved": "Sjabloon opgeslagen!",

  // MCP Servers
  "mcp.filesystem.name": "Codebase-toegang",
  "mcp.filesystem.desc": "Maakt het mogelijk om bestanden in je workspace te lezen (Essentieel).",
  "mcp.tavily.name": "Tavily Webzoeken & Crawl",
  "mcp.tavily.desc": "Zoekt en crawlt het web voor actuele informatie.",
  "mcp.sequential.name": "Sequentieel Denken",
  "mcp.sequential.desc": "Geavanceerde probleemoplossing met dynamische denkprocessen.",
  "mcp.context7.name": "Context7 Documentatie",
  "mcp.context7.desc": "Krijg actuele documentatie voor bibliotheken en frameworks.",
  "mcp.memory.name": "Kennisgraaf (Geheugen)",
  "mcp.memory.desc": "Persistent geheugen en kennisgraaf voor je project.",
  "mcp.puppeteer.name": "Browser-automatisering",
  "mcp.puppeteer.desc": "Bestuur een headless browser voor testen en scraping.",
  "mcp.github.name": "GitHub",
  "mcp.github.desc": "Interacteer met GitHub-repositories, issues en PRs.",
  "mcp.sqlite.name": "SQLite",
  "mcp.sqlite.desc": "Query SQLite-databases in je workspace.",
};

export const locales: Record<SupportedLanguage, LocaleStrings> = {
  en,
  fr,
  es,
  de,
  it,
  pt,
  nl,
};

export const supportedLanguages: {
  code: SupportedLanguage;
  name: string;
  flag: string;
}[] = [
  { code: "en", name: "English", flag: "(EN)" },
  { code: "fr", name: "Français", flag: "(FR)" },
  { code: "es", name: "Español", flag: "(ES)" },
  { code: "de", name: "Deutsch", flag: "(DE)" },
  { code: "it", name: "Italiano", flag: "(IT)" },
  { code: "pt", name: "Português", flag: "(PT)" },
  { code: "nl", name: "Nederlands", flag: "(NL)" },
];
