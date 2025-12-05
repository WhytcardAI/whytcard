// WhytCard Side Panel - Compact Tech Interface
// Handles modals, graph view, chat with Hub sync, and local library management

let hubUrl = "http://localhost:3000";
let isConnected = false;
let currentContext = null;
let currentProjectId = null;
let currentSessionId = null;
let projects = [];
let sessions = [];
let documents = [];
let chatHistory = [];

// Library state
let currentLibraryTab = "highlights";
let libraryItems = {
  highlights: [],
  clips: [],
  notes: [],
  pages: [],
};

// ==================== DOM ELEMENTS ====================
const statusDot = document.getElementById("statusDot");
const projectBtn = document.getElementById("projectBtn");
const projectName = document.getElementById("projectName");
const sessionBtn = document.getElementById("sessionBtn");
const sessionName = document.getElementById("sessionName");
const langBtn = document.getElementById("langBtn");
const currentLangSpan = document.getElementById("currentLang");
const chatMessages = document.getElementById("chatMessages");
const emptyChat = document.getElementById("emptyChat");
const inputField = document.getElementById("inputField");
const sendBtn = document.getElementById("sendBtn");
const contextBar = document.getElementById("contextBar");
const contextPageTitle = document.getElementById("contextPageTitle");

// Views
const chatView = document.getElementById("chatView");
const graphView = document.getElementById("graphView");
const docsView = document.getElementById("docsView");
const graphCanvas = document.getElementById("graphCanvas");

// Modals
const projectModal = document.getElementById("projectModal");
const sessionModal = document.getElementById("sessionModal");
const langModal = document.getElementById("langModal");
const projectList = document.getElementById("projectList");
const sessionList = document.getElementById("sessionList");

// Stats
const graphCount = document.getElementById("graphCount");
const docsCount = document.getElementById("docsCount");
const statProjects = document.getElementById("statProjects");
const statSessions = document.getElementById("statSessions");
const statDocs = document.getElementById("statDocs");

// ==================== INITIALIZATION ====================
async function init() {
  await loadLanguage();
  updateLangDisplay();

  const state = await chrome.storage.local.get([
    "activeHubUrl",
    "isHubConnected",
    "currentProjectId",
    "currentSessionId",
  ]);

  if (state.activeHubUrl) hubUrl = state.activeHubUrl;
  if (state.currentProjectId) currentProjectId = state.currentProjectId;
  if (state.currentSessionId) currentSessionId = state.currentSessionId;

  updateConnectionStatus(state.isHubConnected);
  await updatePageContext();
  await loadProjects();

  if (currentProjectId) {
    const project = projects.find((p) => p.id === currentProjectId);
    if (project) projectName.textContent = truncate(project.name || "Project", 12);
    await loadProjectSessions(currentProjectId);

    if (currentSessionId) {
      const session = sessions.find((s) => s.id === currentSessionId);
      if (session) sessionName.textContent = truncate(session.name || "Session", 12);
      await loadSessionMessages(currentSessionId);
    }
  }

  setupEventListeners();
  checkConnection();
  await loadLibraryData();
}

// ==================== EVENT LISTENERS ====================
function setupEventListeners() {
  // Main view tabs (Chat and Library)
  document.querySelectorAll(".tool-chip[data-view]").forEach((chip) => {
    chip.addEventListener("click", () => {
      const view = chip.dataset.view;
      switchView(view);
      // Update tab selection
      document
        .querySelectorAll(".tool-chip[data-view]")
        .forEach((c) => c.classList.remove("selected"));
      chip.classList.add("selected");
    });
  });

  // More menu toggle
  const moreBtn = document.getElementById("moreBtn");
  const moreMenu = document.getElementById("moreMenu");
  moreBtn?.addEventListener("click", (e) => {
    e.stopPropagation();
    moreMenu?.classList.toggle("hidden");
  });

  // Close more menu when clicking outside
  document.addEventListener("click", () => {
    moreMenu?.classList.add("hidden");
  });

  // More menu items
  document.getElementById("capturePageBtn")?.addEventListener("click", () => {
    moreMenu?.classList.add("hidden");
    saveCurrentPage();
  });
  document.getElementById("screenshotBtn")?.addEventListener("click", () => {
    moreMenu?.classList.add("hidden");
    captureScreenshot();
  });
  document.getElementById("graphMenuBtn")?.addEventListener("click", () => {
    moreMenu?.classList.add("hidden");
    switchView("graph");
  });
  document.getElementById("docsMenuBtn")?.addEventListener("click", () => {
    moreMenu?.classList.add("hidden");
    switchView("docs");
  });

  // Header buttons -> open modals
  projectBtn.addEventListener("click", () => openModal("projectModal"));
  sessionBtn.addEventListener("click", () => {
    if (!currentProjectId) {
      alert("Select a project first");
      return;
    }
    openModal("sessionModal");
  });
  langBtn.addEventListener("click", () => openModal("langModal"));

  // Input
  inputField.addEventListener("input", updateSendButton);
  inputField.addEventListener("keydown", (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  });
  sendBtn.addEventListener("click", sendMessage);

  // Modal overlays - close on background click
  document.querySelectorAll(".modal-overlay").forEach((overlay) => {
    overlay.addEventListener("click", (e) => {
      if (e.target === overlay) closeModal(overlay.id);
    });
  });

  // Language items
  document.querySelectorAll("#langList .modal-item").forEach((item) => {
    item.addEventListener("click", () => selectLanguage(item.dataset.lang));
  });

  // Modal close buttons
  document.getElementById("clearContextBtn")?.addEventListener("click", clearContext);
  document
    .getElementById("closeProjectModal")
    ?.addEventListener("click", () => closeModal("projectModal"));
  document
    .getElementById("cancelProjectModal")
    ?.addEventListener("click", () => closeModal("projectModal"));
  document
    .getElementById("closeSessionModal")
    ?.addEventListener("click", () => closeModal("sessionModal"));
  document
    .getElementById("closeLangModal")
    ?.addEventListener("click", () => closeModal("langModal"));

  // New session
  document.getElementById("newSessionModalBtn")?.addEventListener("click", createNewSession);

  // Graph refresh
  document.getElementById("refreshGraphBtn")?.addEventListener("click", renderGraph);

  // Add doc
  document.getElementById("addDocBtn")?.addEventListener("click", () => {
    document.getElementById("fileInput")?.click();
  });
  document.getElementById("fileInput")?.addEventListener("change", handleFileUpload);

  // Library tabs (inline view)
  document.querySelectorAll(".library-tab").forEach((tab) => {
    tab.addEventListener("click", () => {
      currentLibraryTab = tab.dataset.libraryTab;
      document.querySelectorAll(".library-tab").forEach((t) => t.classList.remove("active"));
      tab.classList.add("active");
      renderLibraryContent();
    });
  });

  // Chrome messages (including SSE events from background)
  chrome.runtime.onMessage.addListener((request) => {
    switch (request.type) {
      case "PREFILL_CHAT":
        inputField.value = request.text;
        updateSendButton();
        break;

      // ==================== SSE EVENT HANDLERS ====================
      case "SSE_CONNECTED":
        console.log("SSE: Connected to Hub");
        updateSSEIndicator(true);
        break;

      case "SSE_DISCONNECTED":
        console.log("SSE: Disconnected from Hub", request.error);
        updateSSEIndicator(false);
        break;

      case "CHAT_CHUNK":
        handleSSEChatChunk(request);
        break;

      case "CHAT_COMPLETE":
        handleSSEChatComplete(request);
        break;

      case "CHAT_STARTED":
        handleSSEChatStarted(request);
        break;

      case "CONTENT_INGESTED":
        handleSSEContentIngested(request);
        break;

      case "CLIENT_EVENT":
        console.log("SSE: Client event", request.eventType, request.clientType);
        break;
    }
  });
}

// ==================== SSE EVENT HANDLERS ====================

let sseStreamingMessageId = null;
let sseStreamingContent = "";

function updateSSEIndicator(connected) {
  // Update status dot to show SSE connection
  if (statusDot) {
    statusDot.title = connected ? "Connected (SSE active)" : "Connected";
  }
}

function handleSSEChatChunk(data) {
  // Only process if we're in the same session
  if (data.sessionId && data.sessionId !== currentSessionId) {
    return;
  }

  // Create or update streaming message element
  if (sseStreamingMessageId !== data.messageId) {
    sseStreamingMessageId = data.messageId;
    sseStreamingContent = "";

    // Create new message element
    emptyChat.style.display = "none";
    const el = document.createElement("div");
    el.className = "msg assistant";
    el.id = `sse-msg-${data.messageId || "stream"}`;
    el.innerHTML = `<div class="msg-content"></div>`;
    chatMessages.appendChild(el);
  }

  // Append chunk
  sseStreamingContent += data.chunk || "";
  const msgEl = document.getElementById(`sse-msg-${sseStreamingMessageId || "stream"}`);
  if (msgEl) {
    const contentEl = msgEl.querySelector(".msg-content");
    if (contentEl) {
      contentEl.innerHTML = escapeHtml(sseStreamingContent);
    }
  }
  chatMessages.scrollTop = chatMessages.scrollHeight;
}

function handleSSEChatComplete(data) {
  if (data.sessionId && data.sessionId !== currentSessionId) {
    return;
  }

  const finalContent = data.content || sseStreamingContent;

  // Update or create final message
  const msgEl = document.getElementById(`sse-msg-${sseStreamingMessageId || "stream"}`);
  if (msgEl) {
    const contentEl = msgEl.querySelector(".msg-content");
    if (contentEl) {
      contentEl.innerHTML = escapeHtml(finalContent);
    }
    // Add timestamp
    const timeEl = document.createElement("div");
    timeEl.className = "msg-time";
    timeEl.textContent = new Date().toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
    });
    msgEl.appendChild(timeEl);
    msgEl.id = ""; // Remove streaming ID
  }

  // Add to chat history
  chatHistory.push({
    role: "assistant",
    content: finalContent,
    time: new Date().toISOString(),
  });

  // Reset streaming state
  sseStreamingMessageId = null;
  sseStreamingContent = "";

  // Remove typing indicator if present
  document.getElementById("typing")?.remove();

  chatMessages.scrollTop = chatMessages.scrollHeight;
}

function handleSSEChatStarted(data) {
  if (data.sessionId && data.sessionId !== currentSessionId) {
    return;
  }

  // Show that another client started a chat
  if (data.source && data.source !== "Chrome Extension") {
    // Add user message from other client
    emptyChat.style.display = "none";
    chatHistory.push({
      role: "user",
      content: data.message,
      time: new Date().toISOString(),
      source: data.source,
    });

    const el = document.createElement("div");
    el.className = "msg user";
    el.innerHTML = `
      <div>${escapeHtml(data.message)}</div>
      <div class="msg-time">${data.source}</div>
    `;
    chatMessages.appendChild(el);
    chatMessages.scrollTop = chatMessages.scrollHeight;
  }
}

function handleSSEContentIngested(data) {
  // Show toast notification
  console.log("SSE: Content ingested", data.title, "from", data.source);

  // Could show a toast notification here
  // For now, just update documents list if in docs view
  if (data.title) {
    documents.push({
      name: data.title,
      source: data.source,
      id: data.documentId,
    });
    updateStats();
  }
}

// ==================== VIEW SWITCHING ====================
function switchView(view) {
  // Update toolbar tabs (only chat and library are in toolbar)
  document.querySelectorAll(".tool-chip[data-view]").forEach((c) => c.classList.remove("selected"));
  document.querySelector(`.tool-chip[data-view="${view}"]`)?.classList.add("selected");

  // Hide all views
  chatView.classList.remove("active");
  graphView.classList.remove("active");
  docsView.classList.remove("active");
  document.getElementById("libraryView")?.classList.remove("active");

  // Show selected view
  if (view === "chat") {
    chatView.classList.add("active");
  } else if (view === "graph") {
    graphView.classList.add("active");
    renderGraph();
  } else if (view === "docs") {
    docsView.classList.add("active");
    renderDocsList();
  } else if (view === "library") {
    document.getElementById("libraryView")?.classList.add("active");
    renderLibraryContent();
  }
}

// ==================== MODALS ====================
function openModal(id) {
  document.getElementById(id)?.classList.add("open");
  if (id === "projectModal") renderProjectList();
  if (id === "sessionModal") renderSessionList();
}

function closeModal(id) {
  document.getElementById(id)?.classList.remove("open");
}

function renderProjectList() {
  projectList.innerHTML =
    projects.length === 0
      ? '<div style="padding:12px;color:var(--text-muted);text-align:center;font-size:10px;">No projects</div>'
      : projects
          .map(
            (p) => `
      <div class="modal-item ${p.id === currentProjectId ? "selected" : ""}" data-id="${p.id}">
        <div class="modal-item-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg></div>
        <div class="modal-item-info">
          <div class="modal-item-name">${p.name || "Project"}</div>
          <div class="modal-item-meta">${p.sessions_count || 0} sessions</div>
        </div>
      </div>
    `
          )
          .join("");

  projectList.querySelectorAll(".modal-item").forEach((item) => {
    item.addEventListener("click", () => selectProject(item.dataset.id));
  });
}

function renderSessionList() {
  sessionList.innerHTML =
    sessions.length === 0
      ? '<div style="padding:12px;color:var(--text-muted);text-align:center;font-size:10px;">No sessions</div>'
      : sessions
          .map(
            (s) => `
      <div class="modal-item ${s.id === currentSessionId ? "selected" : ""}" data-id="${s.id}">
        <div class="modal-item-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg></div>
        <div class="modal-item-info">
          <div class="modal-item-name">${s.name || s.title || "Session"}</div>
          <div class="modal-item-meta">${s.messages_count || 0} messages</div>
        </div>
      </div>
    `
          )
          .join("");

  sessionList.querySelectorAll(".modal-item").forEach((item) => {
    item.addEventListener("click", () => selectSession(item.dataset.id));
  });
}

async function selectProject(id) {
  currentProjectId = id;
  currentSessionId = null;
  chatHistory = [];
  await chrome.storage.local.set({
    currentProjectId: id,
    currentSessionId: null,
  });

  const project = projects.find((p) => p.id === id);
  projectName.textContent = truncate(project?.name || "Project", 12);
  sessionName.textContent = "Session";

  closeModal("projectModal");
  clearChat();
  await loadProjectSessions(id);
  updateStats();
}

async function selectSession(id) {
  currentSessionId = id;
  await chrome.storage.local.set({ currentSessionId: id });

  const session = sessions.find((s) => s.id === id);
  sessionName.textContent = truncate(session?.name || session?.title || "Session", 12);

  closeModal("sessionModal");
  await loadSessionMessages(id);
  updateSendButton();
}

function selectLanguage(lang) {
  setLanguage(lang);
  updateLangDisplay();
  closeModal("langModal");
}

function updateLangDisplay() {
  currentLangSpan.textContent = currentLanguage.toUpperCase();
}

// ==================== DATA LOADING ====================
async function loadProjects() {
  try {
    const response = await chrome.runtime.sendMessage({ type: "GET_PROJECTS" });
    if (response.success && response.data) {
      projects = response.data;
      updateStats();
    }
  } catch (e) {
    console.error("Load projects error:", e);
  }
}

async function loadProjectSessions(projectId) {
  try {
    const response = await chrome.runtime.sendMessage({
      type: "GET_PROJECT_SESSIONS",
      projectId,
    });
    if (response.success && response.data) {
      sessions = response.data;
      updateStats();
    }
  } catch (e) {
    console.error("Load sessions error:", e);
  }
}

async function loadSessionMessages(sessionId) {
  try {
    clearChat();
    const response = await chrome.runtime.sendMessage({
      type: "GET_SESSION_MESSAGES",
      sessionId,
    });
    if (response.success && response.data) {
      chatHistory = response.data.map((m) => ({
        role: m.role === "user" ? "user" : "assistant",
        content: m.content,
        time: m.created_at,
      }));
      renderMessages();
    }
  } catch (e) {
    console.error("Load messages error:", e);
  }
}

async function createNewSession() {
  if (!currentProjectId) return;
  const name = prompt("Session name:", "");
  if (name === null) return;

  try {
    const response = await chrome.runtime.sendMessage({
      type: "CREATE_PROJECT_SESSION",
      projectId: currentProjectId,
      name: name || "New Session",
    });

    if (response.success && response.data) {
      currentSessionId = response.data.id;
      await chrome.storage.local.set({ currentSessionId });
      sessions.push(response.data);
      sessionName.textContent = truncate(name || "New Session", 12);
      closeModal("sessionModal");
      clearChat();
      updateStats();
    }
  } catch (e) {
    console.error("Create session error:", e);
  }
}

// ==================== CONNECTION ====================
async function checkConnection() {
  try {
    const response = await chrome.runtime.sendMessage({
      type: "GET_HUB_STATUS",
    });
    updateConnectionStatus(response.connected);
    if (response.hubUrl) hubUrl = response.hubUrl;
  } catch (e) {
    updateConnectionStatus(false);
  }
}

function updateConnectionStatus(connected) {
  isConnected = connected;
  statusDot.classList.toggle("on", connected);
  statusDot.title = connected ? "Connected" : "Disconnected";
}

// ==================== CONTEXT ====================
async function updatePageContext() {
  try {
    const [tab] = await chrome.tabs.query({
      active: true,
      currentWindow: true,
    });
    if (tab && tab.title && !tab.url.startsWith("chrome://")) {
      currentContext = { url: tab.url, title: tab.title };
      contextPageTitle.textContent = truncate(tab.title, 20);
      contextBar.classList.remove("hidden");
    } else {
      contextBar.classList.add("hidden");
    }
  } catch (e) {
    contextBar.classList.add("hidden");
  }
}

function clearContext() {
  currentContext = null;
  contextBar.classList.add("hidden");
}

// ==================== CHAT ====================
function updateSendButton() {
  sendBtn.disabled = !inputField.value.trim() || !currentSessionId;
}

function clearChat() {
  chatMessages.innerHTML = "";
  chatHistory = [];
  emptyChat.style.display = "flex";
  chatMessages.appendChild(emptyChat);
}

function renderMessages() {
  chatMessages.innerHTML = "";
  if (chatHistory.length === 0) {
    emptyChat.style.display = "flex";
    chatMessages.appendChild(emptyChat);
    return;
  }
  emptyChat.style.display = "none";
  chatHistory.forEach((m) => addMessageEl(m.role, m.content, m.time));
  chatMessages.scrollTop = chatMessages.scrollHeight;
}

function addMessageEl(role, content, time) {
  const el = document.createElement("div");
  el.className = `msg ${role}`;
  const t = time
    ? new Date(time).toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
      })
    : "";
  el.innerHTML = `<div>${escapeHtml(content)}</div>${t ? `<div class="msg-time">${t}</div>` : ""}`;
  chatMessages.appendChild(el);
}

async function sendMessage() {
  const message = inputField.value.trim();
  if (!message || !currentSessionId) return;

  inputField.value = "";
  updateSendButton();
  emptyChat.style.display = "none";

  chatHistory.push({
    role: "user",
    content: message,
    time: new Date().toISOString(),
  });
  addMessageEl("user", message, new Date().toISOString());
  chatMessages.scrollTop = chatMessages.scrollHeight;

  // Typing indicator
  const typing = document.createElement("div");
  typing.className = "msg assistant";
  typing.id = "typing";
  typing.innerHTML = "<span style='opacity:0.5'>...</span>";
  chatMessages.appendChild(typing);
  chatMessages.scrollTop = chatMessages.scrollHeight;

  try {
    let context = currentContext
      ? { pageUrl: currentContext.url, pageTitle: currentContext.title }
      : null;

    const response = await chrome.runtime.sendMessage({
      type: "SEND_SESSION_CHAT",
      sessionId: currentSessionId,
      message,
      context,
    });

    document.getElementById("typing")?.remove();

    if (response.success) {
      const reply = response.data.reply || response.data.content || "";
      chatHistory.push({
        role: "assistant",
        content: reply,
        time: new Date().toISOString(),
      });
      addMessageEl("assistant", reply, new Date().toISOString());
    } else {
      addMessageEl("assistant", "Error: " + (response.error || "Unknown"));
    }
    chatMessages.scrollTop = chatMessages.scrollHeight;
  } catch (e) {
    document.getElementById("typing")?.remove();
    addMessageEl("assistant", "Error: " + e.message);
  }
}

// ==================== PAGE ACTIONS ====================
async function saveCurrentPage() {
  if (!currentContext) return;

  // Save to local library first (local-first approach)
  const capture = await savePageToLibrary();

  if (capture) {
    documents.push({ name: currentContext.title, url: currentContext.url });
    updateStats();
    // Refresh library if visible
    renderLibraryContent();
  }
}

async function captureScreenshot() {
  try {
    const response = await chrome.runtime.sendMessage({
      type: "CAPTURE_SCREENSHOT",
    });
    if (response.success) {
      // Could add to attachments or show preview
      alert("Screenshot captured!");
    }
  } catch (e) {
    console.error("Screenshot error:", e);
  }
}

function handleFileUpload(e) {
  const files = Array.from(e.target.files);
  files.forEach((file) => {
    documents.push({ name: file.name, type: "file" });
  });
  updateStats();
  renderDocsList();
  e.target.value = "";
}

// ==================== GRAPH VIEW ====================
function renderGraph() {
  graphCanvas.innerHTML = "";

  // Center project node
  if (currentProjectId) {
    const project = projects.find((p) => p.id === currentProjectId);
    if (project) {
      const pNode = createNode("project", project.name || "Project", "center", 20, 20);
      graphCanvas.appendChild(pNode);

      // Session nodes
      sessions.forEach((s, i) => {
        const sNode = createNode("session", s.name || s.title || "Session", s.id, 20, 70 + i * 50);
        graphCanvas.appendChild(sNode);

        // Link
        const link = document.createElement("div");
        link.className = "graph-link";
        link.style.left = "60px";
        link.style.top = 35 + "px";
        link.style.width = "30px";
        link.style.transform = `rotate(${(Math.atan2(70 + i * 50 - 35, 30) * 180) / Math.PI}deg)`;
        graphCanvas.appendChild(link);
      });

      // Doc nodes
      documents.forEach((d, i) => {
        const dNode = createNode("document", d.name || "Doc", d.url || i, 180, 20 + i * 50);
        graphCanvas.appendChild(dNode);
      });
    }
  } else {
    graphCanvas.innerHTML =
      '<div class="empty-chat"><span>Select a project to view graph</span></div>';
  }
}

function createNode(type, name, id, x, y) {
  const node = document.createElement("div");
  node.className = `node ${type}`;
  node.style.left = x + "px";
  node.style.top = y + "px";
  node.innerHTML = `
    <div class="node-type">${type}</div>
    <div class="node-name" title="${name}">${truncate(name, 15)}</div>
  `;
  return node;
}

function renderDocsList() {
  const docsList = document.getElementById("docsList");
  if (!docsList) return;

  if (documents.length === 0) {
    docsList.innerHTML =
      '<div class="empty-chat"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/></svg><span>No documents linked</span></div>';
    return;
  }

  docsList.innerHTML = documents
    .map(
      (d, i) => `
    <div class="modal-item">
      <div class="modal-item-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/></svg></div>
      <div class="modal-item-info">
        <div class="modal-item-name">${d.name || "Document"}</div>
        <div class="modal-item-meta">${d.type || "webpage"}</div>
      </div>
    </div>
  `
    )
    .join("");
}

// ==================== STATS ====================
function updateStats() {
  const totalNodes = projects.length + sessions.length + documents.length;
  graphCount.textContent = totalNodes;
  docsCount.textContent = documents.length;
  statProjects.textContent = projects.length + " projects";
  statSessions.textContent = sessions.length + " sessions";
  statDocs.textContent = documents.length + " docs";
}

// ==================== UTILITIES ====================
function truncate(str, len) {
  return str.length > len ? str.substring(0, len) + "..." : str;
}

function escapeHtml(text) {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML.replace(/\n/g, "<br>");
}

// ==================== LOCAL LIBRARY ====================

/**
 * Load all library data from local storage
 */
async function loadLibraryData() {
  if (!window.WhytCardStorage) {
    console.error("WhytCardStorage module not loaded");
    return;
  }

  try {
    libraryItems.highlights = await window.WhytCardStorage.Highlights.getAll();
    libraryItems.clips = await window.WhytCardStorage.Clips.getAll();
    libraryItems.notes = await window.WhytCardStorage.Notes.getAll();
    libraryItems.pages = await window.WhytCardStorage.PageCaptures.getAll();
    updateLibraryCount();
  } catch (error) {
    console.error("Failed to load library data:", error);
  }
}

/**
 * Update library count badge
 */
function updateLibraryCount() {
  const total =
    libraryItems.highlights.length +
    libraryItems.clips.length +
    libraryItems.notes.length +
    libraryItems.pages.length;
  document.getElementById("libraryCount").textContent = total;
}

/**
 * Render library content based on current tab
 * Uses per-item action buttons instead of checkboxes for better UX
 */
function renderLibraryContent() {
  const content = document.getElementById("libraryContent");
  if (!content) return;

  let items = [];
  let storageKey = "";
  let emptyMessage = "";

  switch (currentLibraryTab) {
    case "highlights":
      items = libraryItems.highlights;
      storageKey = window.WhytCardStorage?.STORAGE_KEYS.HIGHLIGHTS;
      emptyMessage = t("library.noHighlights");
      break;
    case "clips":
      items = libraryItems.clips;
      storageKey = window.WhytCardStorage?.STORAGE_KEYS.CLIPS;
      emptyMessage = t("library.noClips");
      break;
    case "notes":
      items = libraryItems.notes;
      storageKey = window.WhytCardStorage?.STORAGE_KEYS.NOTES;
      emptyMessage = t("library.noNotes");
      break;
    case "pages":
      items = libraryItems.pages;
      storageKey = window.WhytCardStorage?.STORAGE_KEYS.PAGE_CAPTURES;
      emptyMessage = t("library.noPages");
      break;
  }

  if (items.length === 0) {
    content.innerHTML = `
      <div class="library-empty">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
          <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
        </svg>
        <span>${emptyMessage}</span>
      </div>
    `;
    return;
  }

  content.innerHTML = items
    .map((item) => {
      const text = item.text || item.content || item.title || "Untitled";
      const truncatedText = truncate(text, 50);
      const date = new Date(item.createdAt).toLocaleDateString();

      // Determine which action buttons to show based on status
      const showSyncBtn = item.syncStatus === "local" || item.syncStatus === "failed";
      const showIndexBtn = item.syncStatus === "synced" && item.ragStatus !== "indexed";
      const showUnindexBtn = item.ragStatus === "indexed";

      return `
        <div class="library-item" data-id="${item.id}" data-storage-key="${storageKey}">
          <div class="library-item-content">
            <div class="library-item-text" title="${escapeHtml(text)}">${escapeHtml(truncatedText)}</div>
            <div class="library-item-meta">
              <span>${date}</span>
              <span class="badge ${item.syncStatus}">${item.syncStatus}</span>
              ${item.ragStatus !== "none" ? `<span class="badge ${item.ragStatus}">${item.ragStatus}</span>` : ""}
            </div>
          </div>
          <div class="library-item-actions">
            ${
              showSyncBtn
                ? `<button class="library-action-btn sync-btn" data-action="sync" title="${t("library.syncToHub")}">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 12a9 9 0 0 1-9 9m9-9a9 9 0 0 0-9-9m9 9H3m9 9a9 9 0 0 1-9-9m9 9V3m-9 9a9 9 0 0 1 9-9"/>
              </svg>
            </button>`
                : ""
            }
            ${
              showIndexBtn
                ? `<button class="library-action-btn index-btn" data-action="index" title="${t("library.addToRag")}">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"/>
                <path d="m21 21-4.35-4.35"/>
              </svg>
            </button>`
                : ""
            }
            ${
              showUnindexBtn
                ? `<button class="library-action-btn unindex-btn" data-action="unindex" title="${t("library.removeFromRag")}">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"/>
                <path d="m21 21-4.35-4.35"/>
                <path d="M8 11h6"/>
              </svg>
            </button>`
                : ""
            }
            <button class="library-action-btn delete-btn" data-action="delete" title="${t("library.delete")}">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 6h18"/>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/>
                <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
            </button>
          </div>
        </div>
      `;
    })
    .join("");

  // Add event listeners to action buttons
  content.querySelectorAll(".library-item").forEach((el) => {
    const id = el.dataset.id;
    const itemStorageKey = el.dataset.storageKey;

    el.querySelectorAll(".library-action-btn").forEach((btn) => {
      btn.addEventListener("click", async (e) => {
        e.stopPropagation();
        const action = btn.dataset.action;
        await handleLibraryItemAction(action, id, itemStorageKey);
      });
    });
  });

  updateLibraryStats();
}

/**
 * Handle action on a single library item
 */
async function handleLibraryItemAction(action, itemId, storageKey) {
  const Storage = window.WhytCardStorage;
  if (!Storage || !Storage.SyncManager) {
    console.error("Storage module not available");
    return;
  }

  try {
    switch (action) {
      case "sync": {
        if (!currentProjectId) {
          alert(t("selectProjectFirst"));
          return;
        }
        // Get the full item from storage
        const item = await Storage.getById(storageKey, itemId);
        if (!item) {
          console.error("Item not found:", itemId);
          return;
        }
        // Mark as pending
        await Storage.updateItem(storageKey, itemId, { syncStatus: "pending" });
        renderLibraryContent();

        // Sync to Hub
        const result = await Storage.SyncManager.syncItemToHub(storageKey, item, currentProjectId);
        if (result.success) {
          await Storage.updateItem(storageKey, itemId, {
            syncStatus: "synced",
            hubFileId: result.hubFileId,
          });
        } else {
          await Storage.updateItem(storageKey, itemId, { syncStatus: "failed" });
          console.error("Sync failed:", result.error);
        }
        break;
      }
      case "index":
        await Storage.SyncManager.requestRagIndexing(storageKey, itemId);
        break;
      case "unindex":
        await Storage.SyncManager.removeFromRagIndex(storageKey, itemId);
        break;
      case "delete":
        await deleteLibraryItem(storageKey, itemId);
        break;
    }
    // Refresh library content
    await loadLibraryData();
    renderLibraryContent();
  } catch (error) {
    console.error(`Failed to ${action} item:`, error);
  }
}

/**
 * Delete a library item from local storage
 */
async function deleteLibraryItem(storageKey, itemId) {
  return new Promise((resolve, reject) => {
    chrome.storage.local.get([storageKey], (result) => {
      const items = result[storageKey] || [];
      const filtered = items.filter((item) => item.id !== itemId);
      chrome.storage.local.set({ [storageKey]: filtered }, () => {
        if (chrome.runtime.lastError) {
          reject(chrome.runtime.lastError);
        } else {
          resolve();
        }
      });
    });
  });
}

/**
 * Update library stats display
 */
function updateLibraryStats() {
  const statsEl = document.getElementById("libraryStats");
  if (!statsEl || !window.WhytCardStorage) return;

  window.WhytCardStorage.SyncManager.getStats().then((stats) => {
    statsEl.textContent = `${stats.local} local | ${stats.synced} synced | ${stats.ragIndexed} indexed`;
  });
}

/**
 * Find item by ID across all library types
 */
function findItemById(id) {
  for (const type of ["highlights", "clips", "notes", "pages"]) {
    const item = libraryItems[type].find((i) => i.id === id);
    if (item) return item;
  }
  return null;
}

/**
 * Save current page to local library
 */
async function savePageToLibrary() {
  if (!currentContext || !window.WhytCardStorage) return;

  try {
    const [tab] = await chrome.tabs.query({
      active: true,
      currentWindow: true,
    });

    const pageContent = await chrome.tabs.sendMessage(tab.id, {
      type: "GET_PAGE_CONTENT",
    });

    if (!pageContent.success) {
      console.error("Failed to extract page content");
      return null;
    }

    const capture = await window.WhytCardStorage.PageCaptures.add({
      url: currentContext.url,
      title: currentContext.title,
      content: pageContent.data.content,
      metadata: pageContent.data.metadata,
    });

    await loadLibraryData();
    return capture;
  } catch (error) {
    console.error("Save to library error:", error);
    return null;
  }
}

/**
 * Save highlight to local library
 */
async function saveHighlightToLibrary(text, url, pageTitle, color = "yellow") {
  if (!window.WhytCardStorage) return null;

  try {
    const highlight = await window.WhytCardStorage.Highlights.add({
      text,
      url,
      pageTitle,
      color,
    });
    await loadLibraryData();
    return highlight;
  } catch (error) {
    console.error("Save highlight error:", error);
    return null;
  }
}

/**
 * Save clip to local library
 */
async function saveClipToLibrary(content, contentType, url, pageTitle) {
  if (!window.WhytCardStorage) return null;

  try {
    const clip = await window.WhytCardStorage.Clips.add({
      content,
      contentType,
      url,
      pageTitle,
    });
    await loadLibraryData();
    return clip;
  } catch (error) {
    console.error("Save clip error:", error);
    return null;
  }
}

/**
 * Save note to local library
 */
async function saveNoteToLibrary(content, url = null, pageTitle = null, tags = []) {
  if (!window.WhytCardStorage) return null;

  try {
    const note = await window.WhytCardStorage.Notes.add({
      content,
      url,
      pageTitle,
      tags,
    });
    await loadLibraryData();
    return note;
  } catch (error) {
    console.error("Save note error:", error);
    return null;
  }
}

// ==================== START ====================
init();
