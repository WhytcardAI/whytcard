// WhytCard Background Service Worker
// Handles context menus, alarms, commands, Hub communication, and SSE sync

const HUB_URL = "http://localhost:3000"; // Primary - WhytCard Hub (Tauri)
const HUB_URL_ALT = "http://localhost:1420"; // Alternative port for dev

let isHubConnected = false;
let activeHubUrl = HUB_URL;
let apiKey = null; // Bearer token for authentication

// ==================== SSE CONNECTION ====================

let sseController = null; // AbortController for SSE fetch
let sseRetryCount = 0;
const SSE_MAX_RETRIES = 10;
const SSE_RETRY_DELAY_BASE = 1000; // 1 second base delay
let currentSessionId = null; // Current active session for filtering events

/**
 * Connect to Hub SSE endpoint for real-time events
 */
async function connectSSE() {
  // Don't connect if no API key
  if (!apiKey) {
    console.log("SSE: No API key configured, skipping connection");
    return;
  }

  // Abort existing connection if any
  if (sseController) {
    sseController.abort();
    sseController = null;
  }

  sseController = new AbortController();

  try {
    console.log(`SSE: Connecting to ${activeHubUrl}/api/events`);

    const response = await fetch(`${activeHubUrl}/api/events`, {
      headers: {
        Authorization: `Bearer ${apiKey}`,
        Accept: "text/event-stream",
        "Cache-Control": "no-cache",
      },
      signal: sseController.signal,
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    // Reset retry count on successful connection
    sseRetryCount = 0;
    console.log("SSE: Connected successfully");

    // Notify UI of connection
    broadcastToExtension({
      type: "SSE_CONNECTED",
      hubUrl: activeHubUrl,
    });

    // Read the stream
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = "";

    while (true) {
      const { done, value } = await reader.read();

      if (done) {
        console.log("SSE: Stream ended");
        break;
      }

      buffer += decoder.decode(value, { stream: true });

      // Process complete events (separated by double newlines)
      const events = buffer.split("\n\n");
      buffer = events.pop() || ""; // Keep incomplete event in buffer

      for (const eventText of events) {
        if (eventText.trim()) {
          processSSEEvent(eventText);
        }
      }
    }
  } catch (error) {
    if (error.name === "AbortError") {
      console.log("SSE: Connection aborted");
      return;
    }

    console.error("SSE: Connection error:", error.message);

    // Notify UI of disconnection
    broadcastToExtension({
      type: "SSE_DISCONNECTED",
      error: error.message,
    });

    // Retry with exponential backoff
    if (sseRetryCount < SSE_MAX_RETRIES) {
      const delay = SSE_RETRY_DELAY_BASE * Math.pow(2, sseRetryCount);
      sseRetryCount++;
      console.log(
        `SSE: Retrying in ${delay}ms (attempt ${sseRetryCount}/${SSE_MAX_RETRIES})`
      );
      setTimeout(connectSSE, delay);
    } else {
      console.log("SSE: Max retries reached, giving up");
    }
  }
}

/**
 * Disconnect SSE stream
 */
function disconnectSSE() {
  if (sseController) {
    sseController.abort();
    sseController = null;
  }
  sseRetryCount = 0;
}

/**
 * Process a single SSE event
 */
function processSSEEvent(eventText) {
  try {
    // Parse SSE format: "event: type\ndata: json"
    const lines = eventText.split("\n");
    let eventType = "message";
    let data = null;

    for (const line of lines) {
      if (line.startsWith("event:")) {
        eventType = line.substring(6).trim();
      } else if (line.startsWith("data:")) {
        const jsonStr = line.substring(5).trim();
        if (jsonStr) {
          data = JSON.parse(jsonStr);
        }
      }
    }

    if (!data) return;

    console.log(`SSE Event: ${eventType}`, data);

    // Handle different event types
    switch (data.type || eventType) {
      case "chat_chunk":
        handleChatChunk(data);
        break;
      case "chat_complete":
        handleChatComplete(data);
        break;
      case "chat_started":
        handleChatStarted(data);
        break;
      case "content_ingested":
        handleContentIngested(data);
        break;
      case "action_requested":
        handleActionRequested(data);
        break;
      case "workflow_started":
      case "workflow_completed":
      case "workflow_failed":
        handleWorkflowEvent(data);
        break;
      case "client_connected":
      case "client_disconnected":
        handleClientEvent(data);
        break;
      case "heartbeat":
        // Ignore heartbeats
        break;
      default:
        console.log("SSE: Unknown event type:", data.type || eventType);
    }
  } catch (error) {
    console.error("SSE: Error processing event:", error, eventText);
  }
}

/**
 * Handle streaming chat chunk
 */
function handleChatChunk(data) {
  // Only forward if it's for our current session or no session filter
  if (
    currentSessionId &&
    data.session_id &&
    data.session_id !== currentSessionId
  ) {
    return;
  }

  broadcastToExtension({
    type: "CHAT_CHUNK",
    chunk: data.chunk || data.content,
    sessionId: data.session_id,
    messageId: data.message_id,
  });
}

/**
 * Handle chat completion
 */
function handleChatComplete(data) {
  if (
    currentSessionId &&
    data.session_id &&
    data.session_id !== currentSessionId
  ) {
    return;
  }

  broadcastToExtension({
    type: "CHAT_COMPLETE",
    content: data.content || data.reply,
    sessionId: data.session_id,
    messageId: data.message_id,
    sources: data.sources,
  });
}

/**
 * Handle chat started (from another client)
 */
function handleChatStarted(data) {
  if (
    currentSessionId &&
    data.session_id &&
    data.session_id !== currentSessionId
  ) {
    return;
  }

  broadcastToExtension({
    type: "CHAT_STARTED",
    message: data.message,
    sessionId: data.session_id,
    source: data.source,
  });
}

/**
 * Handle content ingested notification
 */
function handleContentIngested(data) {
  broadcastToExtension({
    type: "CONTENT_INGESTED",
    title: data.title,
    source: data.source,
    documentId: data.document_id,
  });
}

/**
 * Handle action request from Hub
 */
function handleActionRequested(data) {
  // Check if this action is for Chrome extension
  if (data.target_client !== "chrome" && data.target_client !== "all") {
    return;
  }

  broadcastToExtension({
    type: "ACTION_REQUESTED",
    actionId: data.action_id,
    actionType: data.action_type,
    params: data.params,
    jobId: data.job_id,
  });
}

/**
 * Handle workflow events
 */
function handleWorkflowEvent(data) {
  broadcastToExtension({
    type: "WORKFLOW_EVENT",
    eventType: data.type,
    workflowId: data.workflow_id,
    instanceId: data.instance_id,
    status: data.status,
  });
}

/**
 * Handle client connection events
 */
function handleClientEvent(data) {
  broadcastToExtension({
    type: "CLIENT_EVENT",
    eventType: data.type,
    clientType: data.client_type,
    clientId: data.client_id,
  });
}

/**
 * Broadcast message to all extension contexts (popup, sidepanel, content scripts)
 */
function broadcastToExtension(message) {
  // Send to runtime (popup, sidepanel, options)
  chrome.runtime.sendMessage(message).catch(() => {
    // No listeners, ignore
  });

  // Send to all tabs (content scripts)
  chrome.tabs.query({}, (tabs) => {
    for (const tab of tabs) {
      if (tab.id) {
        chrome.tabs.sendMessage(tab.id, message).catch(() => {
          // Content script not loaded, ignore
        });
      }
    }
  });
}

// ==================== INSTALLATION ====================

chrome.runtime.onInstalled.addListener(() => {
  console.log("WhytCard extension installed");
  setupContextMenus();
  setupAlarms();
  initializeStorage();
  loadApiKey();
});

chrome.runtime.onStartup.addListener(() => {
  setupAlarms();
  loadApiKey();
  checkHubConnection();
});

// ==================== API KEY MANAGEMENT ====================

async function loadApiKey() {
  const result = await chrome.storage.local.get(["apiKey", "currentSessionId"]);
  apiKey = result.apiKey || null;
  currentSessionId = result.currentSessionId || null;
  console.log("API Key loaded:", apiKey ? "***" + apiKey.slice(-4) : "none");

  // Connect to SSE if we have an API key
  if (apiKey) {
    // Small delay to ensure Hub connection is established first
    setTimeout(() => {
      if (isHubConnected) {
        connectSSE();
      }
    }, 1000);
  }
}

async function setApiKey(key) {
  apiKey = key;
  await chrome.storage.local.set({ apiKey: key });
  await checkHubConnection();

  // Reconnect SSE with new key
  if (key && isHubConnected) {
    connectSSE();
  } else {
    disconnectSSE();
  }
}

function getAuthHeaders() {
  const headers = { "Content-Type": "application/json" };
  if (apiKey) {
    headers["Authorization"] = `Bearer ${apiKey}`;
  }
  return headers;
}

// ==================== CONTEXT MENUS ====================

function setupContextMenus() {
  // Remove existing menus first
  chrome.contextMenus.removeAll(() => {
    // Main parent menu
    chrome.contextMenus.create({
      id: "whytcard-parent",
      title: "WhytCard",
      contexts: ["all"],
    });

    // Ask about selection
    chrome.contextMenus.create({
      id: "whytcard-ask-selection",
      parentId: "whytcard-parent",
      title: "Ask about selection",
      contexts: ["selection"],
    });

    // Explain selection
    chrome.contextMenus.create({
      id: "whytcard-explain",
      parentId: "whytcard-parent",
      title: "Explain this",
      contexts: ["selection"],
    });

    // Translate selection
    chrome.contextMenus.create({
      id: "whytcard-translate",
      parentId: "whytcard-parent",
      title: "Translate this",
      contexts: ["selection"],
    });

    // Separator
    chrome.contextMenus.create({
      id: "whytcard-separator1",
      parentId: "whytcard-parent",
      type: "separator",
      contexts: ["all"],
    });

    // Save page to WhytCard
    chrome.contextMenus.create({
      id: "whytcard-save-page",
      parentId: "whytcard-parent",
      title: "Save page to WhytCard",
      contexts: ["page"],
    });

    // Summarize page
    chrome.contextMenus.create({
      id: "whytcard-summarize",
      parentId: "whytcard-parent",
      title: "Summarize this page",
      contexts: ["page"],
    });

    // Separator
    chrome.contextMenus.create({
      id: "whytcard-separator2",
      parentId: "whytcard-parent",
      type: "separator",
      contexts: ["all"],
    });

    // Open Side Panel
    chrome.contextMenus.create({
      id: "whytcard-open-panel",
      parentId: "whytcard-parent",
      title: "Open WhytCard Chat",
      contexts: ["all"],
    });
  });
}

chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  switch (info.menuItemId) {
    case "whytcard-ask-selection":
      await handleAskSelection(info.selectionText, tab);
      break;
    case "whytcard-explain":
      await handleExplain(info.selectionText, tab);
      break;
    case "whytcard-translate":
      await handleTranslate(info.selectionText, tab);
      break;
    case "whytcard-save-page":
      await handleSavePage(tab);
      break;
    case "whytcard-summarize":
      await handleSummarize(tab);
      break;
    case "whytcard-open-panel":
      await openSidePanel(tab);
      break;
  }
});

// ==================== CONTEXT MENU HANDLERS ====================

async function handleAskSelection(text, tab) {
  if (!text) return;

  // Open side panel with the selection as context
  await openSidePanel(tab);

  // Send message to side panel
  setTimeout(() => {
    chrome.runtime.sendMessage({
      type: "PREFILL_CHAT",
      text: `About this text:\n"${text}"\n\nMy question: `,
    });
  }, 500);
}

async function handleExplain(text, tab) {
  if (!text) return;

  const response = await sendToHub({
    message: `Please explain the following in simple terms:\n\n"${text}"`,
    context: {
      pageUrl: tab.url,
      pageTitle: tab.title,
    },
  });

  if (response.success) {
    notifyTab(tab.id, response.data.reply, "success");
  } else {
    notifyTab(tab.id, "Could not connect to WhytCard Hub", "error");
  }
}

async function handleTranslate(text, tab) {
  if (!text) return;

  const response = await sendToHub({
    message: `Please translate the following text to English (if it's already English, translate to French):\n\n"${text}"`,
    context: {
      pageUrl: tab.url,
      pageTitle: tab.title,
    },
  });

  if (response.success) {
    notifyTab(tab.id, response.data.reply, "success");
  } else {
    notifyTab(tab.id, "Could not connect to WhytCard Hub", "error");
  }
}

async function handleSavePage(tab) {
  try {
    const pageContent = await getPageContent(tab.id);
    if (!pageContent.success) {
      notifyTab(tab.id, "Failed to extract page content", "error");
      return;
    }

    const response = await fetch(`${activeHubUrl}/api/ingest`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        content: pageContent.data.content,
        metadata: pageContent.data.metadata,
        source: "Chrome Extension",
        type: "page",
      }),
    });

    if (response.ok) {
      notifyTab(tab.id, "Page saved to WhytCard!", "success");
    } else {
      throw new Error(`HTTP ${response.status}`);
    }
  } catch (error) {
    notifyTab(tab.id, `Failed to save: ${error.message}`, "error");
  }
}

async function handleSummarize(tab) {
  try {
    const pageContent = await getPageContent(tab.id);
    if (!pageContent.success) {
      notifyTab(tab.id, "Failed to extract page content", "error");
      return;
    }

    notifyTab(tab.id, "Summarizing page...", "success");

    const response = await sendToHub({
      message: `Please provide a concise summary of the following web page content:\n\nTitle: ${
        pageContent.data.metadata.title
      }\nURL: ${
        pageContent.data.metadata.url
      }\n\nContent:\n${pageContent.data.content.substring(0, 5000)}`,
    });

    if (response.success) {
      // Open side panel and show summary
      await openSidePanel(tab);
      setTimeout(() => {
        chrome.runtime.sendMessage({
          type: "SHOW_SUMMARY",
          summary: response.data.reply,
          pageTitle: pageContent.data.metadata.title,
        });
      }, 500);
    } else {
      notifyTab(tab.id, "Could not summarize page", "error");
    }
  } catch (error) {
    notifyTab(tab.id, `Error: ${error.message}`, "error");
  }
}

// ==================== COMMANDS (Keyboard Shortcuts) ====================

chrome.commands.onCommand.addListener(async (command, tab) => {
  switch (command) {
    case "open-sidepanel":
      await openSidePanel(tab);
      break;
    case "capture-page":
      await handleSavePage(tab);
      break;
  }
});

// ==================== ALARMS ====================

function setupAlarms() {
  // Ping hub every 30 seconds to maintain connection state
  chrome.alarms.create("pingHub", { periodInMinutes: 0.5 });
}

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === "pingHub") {
    checkHubConnection();
  }
});

// ==================== HUB COMMUNICATION ====================

async function checkHubConnection() {
  // Try primary URL first
  try {
    const response = await fetch(`${HUB_URL}/api/ping?source=chrome`, {
      signal: AbortSignal.timeout(2000),
    });
    if (response.ok) {
      activeHubUrl = HUB_URL;
      updateConnectionState(true);
      return;
    }
  } catch (e) {
    // Try alternative port
  }

  // Try alternative URL
  try {
    const response = await fetch(`${HUB_URL_ALT}/api/ping?source=chrome`, {
      signal: AbortSignal.timeout(2000),
    });
    if (response.ok) {
      activeHubUrl = HUB_URL_ALT;
      updateConnectionState(true);
      return;
    }
  } catch (e) {
    // Both failed
  }

  updateConnectionState(false);
}

function updateConnectionState(connected) {
  const wasConnected = isHubConnected;
  isHubConnected = connected;

  // Update badge
  chrome.action.setBadgeText({ text: connected ? "" : "!" });
  chrome.action.setBadgeBackgroundColor({
    color: connected ? "#22c55e" : "#ef4444",
  });

  // Store state
  chrome.storage.local.set({ isHubConnected: connected, activeHubUrl });

  // Connect/disconnect SSE based on connection state
  if (connected && !wasConnected && apiKey) {
    connectSSE();
  } else if (!connected && wasConnected) {
    disconnectSSE();
  }
}

async function sendToHub(payload) {
  try {
    const response = await fetch(`${activeHubUrl}/api/chat`, {
      method: "POST",
      headers: getAuthHeaders(),
      body: JSON.stringify({
        ...payload,
        source: "Chrome Extension",
      }),
    });

    if (response.status === 401) {
      return {
        success: false,
        error:
          "Unauthorized - Please configure your API key in extension options",
      };
    }

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const data = await response.json();
    return { success: true, data };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

// ==================== SIDE PANEL ====================

async function openSidePanel(tab) {
  try {
    await chrome.sidePanel.open({ tabId: tab.id });
  } catch (error) {
    console.error("Failed to open side panel:", error);
  }
}

// ==================== SCREENSHOT ====================

async function captureScreenshot(windowId = null) {
  try {
    const dataUrl = await chrome.tabs.captureVisibleTab(windowId, {
      format: "png",
      quality: 100,
    });
    return { success: true, data: dataUrl };
  } catch (error) {
    console.error("Screenshot capture failed:", error);
    return { success: false, error: error.message };
  }
}

// ==================== SESSIONS/PROJECTS ====================

async function getProjects() {
  try {
    const response = await fetch(`${activeHubUrl}/api/projects`, {
      method: "GET",
      headers: getAuthHeaders(),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const data = await response.json();
    return { success: true, data: data.projects || data };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

async function getProjectSessions(projectId) {
  try {
    const response = await fetch(
      `${activeHubUrl}/api/projects/${projectId}/sessions`,
      {
        method: "GET",
        headers: getAuthHeaders(),
      }
    );

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const data = await response.json();
    return { success: true, data: data.sessions || data };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

async function createProjectSession(projectId, name) {
  try {
    const response = await fetch(
      `${activeHubUrl}/api/projects/${projectId}/sessions`,
      {
        method: "POST",
        headers: getAuthHeaders(),
        body: JSON.stringify({ name: name || "New Session" }),
      }
    );

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const data = await response.json();
    return { success: true, data };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

async function getSessionMessages(sessionId) {
  try {
    const response = await fetch(
      `${activeHubUrl}/api/sessions/${sessionId}/messages`,
      {
        method: "GET",
        headers: getAuthHeaders(),
      }
    );

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const data = await response.json();
    return { success: true, data: data.messages || data };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

async function sendSessionChat(sessionId, message, context = null) {
  try {
    const payload = {
      message: message,
      source: "Chrome Extension",
    };

    if (context) {
      payload.context = context;
    }

    const response = await fetch(
      `${activeHubUrl}/api/sessions/${sessionId}/chat`,
      {
        method: "POST",
        headers: getAuthHeaders(),
        body: JSON.stringify(payload),
      }
    );

    if (response.status === 401) {
      return {
        success: false,
        error:
          "Unauthorized - Please configure your API key in extension options",
      };
    }

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const data = await response.json();
    return { success: true, data };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

async function getSessions() {
  try {
    const response = await fetch(`${activeHubUrl}/api/sessions`, {
      method: "GET",
      headers: getAuthHeaders(),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const data = await response.json();
    return { success: true, data: data.sessions || data };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

// ==================== DOCUMENT UPLOAD ====================

async function uploadDocument(content, metadata, sessionId = null) {
  try {
    const payload = {
      content: content,
      metadata: metadata,
      source: "Chrome Extension",
      type: metadata?.type || "document",
    };

    if (sessionId) {
      payload.session_id = sessionId;
    }

    const response = await fetch(`${activeHubUrl}/api/ingest`, {
      method: "POST",
      headers: getAuthHeaders(),
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const data = await response.json();
    return { success: true, data };
  } catch (error) {
    return { success: false, error: error.message };
  }
}

// ==================== TAB COMMUNICATION ====================

async function getPageContent(tabId) {
  try {
    const response = await chrome.tabs.sendMessage(tabId, {
      type: "GET_PAGE_CONTENT",
    });
    return response;
  } catch (error) {
    return { success: false, error: error.message };
  }
}

function notifyTab(tabId, message, type) {
  chrome.tabs
    .sendMessage(tabId, {
      type: "SHOW_TOAST",
      message,
      toastType: type,
    })
    .catch(() => {
      // Content script might not be loaded
    });
}

// ==================== STORAGE ====================

function initializeStorage() {
  chrome.storage.local.get(["apiKey"], (result) => {
    // Preserve API key if it exists
    chrome.storage.local.set({
      showFab: true,
      isHubConnected: false,
      activeHubUrl: HUB_URL,
      chatHistory: [],
      apiKey: result.apiKey || null,
    });
  });
}

// ==================== MESSAGE HANDLERS ====================

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  switch (request.type) {
    case "PING_HUB":
      checkHubConnection().then(() => {
        sendResponse({ success: isHubConnected, hubUrl: activeHubUrl });
      });
      return true;

    case "GET_HUB_STATUS":
      sendResponse({
        connected: isHubConnected,
        hubUrl: activeHubUrl,
        hasApiKey: !!apiKey,
      });
      break;

    case "SEND_TO_HUB":
      sendToHub(request.payload).then(sendResponse);
      return true;

    case "OPEN_SIDE_PANEL":
      chrome.tabs.query({ active: true, currentWindow: true }, async (tabs) => {
        if (tabs[0]) {
          await openSidePanel(tabs[0]);
        }
        sendResponse({ success: true });
      });
      return true;

    case "SET_API_KEY":
      setApiKey(request.apiKey).then(() => {
        sendResponse({ success: true });
      });
      return true;

    case "GET_API_KEY":
      sendResponse({ apiKey: apiKey });
      break;

    case "CLEAR_API_KEY":
      setApiKey(null).then(() => {
        sendResponse({ success: true });
      });
      return true;

    // ==================== NEW HANDLERS ====================

    case "CAPTURE_SCREENSHOT":
      captureScreenshot(request.windowId).then(sendResponse);
      return true;

    case "GET_SESSIONS":
      getSessions().then(sendResponse);
      return true;

    case "GET_PROJECTS":
      getProjects().then(sendResponse);
      return true;

    case "GET_PROJECT_SESSIONS":
      getProjectSessions(request.projectId).then(sendResponse);
      return true;

    case "CREATE_PROJECT_SESSION":
      createProjectSession(request.projectId, request.name).then(sendResponse);
      return true;

    case "GET_SESSION_MESSAGES":
      getSessionMessages(request.sessionId).then(sendResponse);
      return true;

    case "SEND_SESSION_CHAT":
      sendSessionChat(request.sessionId, request.message, request.context).then(
        sendResponse
      );
      return true;

    case "SET_CURRENT_SESSION":
      currentSessionId = request.sessionId;
      chrome.storage.local
        .set({ currentSessionId: request.sessionId })
        .then(() => {
          sendResponse({ success: true });
        });
      return true;

    case "GET_CURRENT_SESSION":
      chrome.storage.local.get(["currentSessionId"]).then((result) => {
        sendResponse({ sessionId: result.currentSessionId || null });
      });
      return true;

    case "GET_SSE_STATUS":
      sendResponse({
        connected: sseController !== null,
        retryCount: sseRetryCount,
      });
      break;

    case "RECONNECT_SSE":
      if (apiKey && isHubConnected) {
        disconnectSSE();
        connectSSE();
        sendResponse({ success: true });
      } else {
        sendResponse({
          success: false,
          error: "No API key or not connected to Hub",
        });
      }
      break;

    case "UPLOAD_DOCUMENT":
      uploadDocument(request.content, request.metadata, request.sessionId).then(
        sendResponse
      );
      return true;

    case "GET_STRUCTURED_CONTENT":
      chrome.tabs.query({ active: true, currentWindow: true }, async (tabs) => {
        if (tabs[0]) {
          try {
            const response = await chrome.tabs.sendMessage(tabs[0].id, {
              type: "GET_STRUCTURED_CONTENT",
            });
            sendResponse(response);
          } catch (error) {
            sendResponse({ success: false, error: error.message });
          }
        } else {
          sendResponse({ success: false, error: "No active tab" });
        }
      });
      return true;

    case "INGEST_WITH_SCREENSHOT":
      (async () => {
        try {
          // Get page content
          const [tab] = await chrome.tabs.query({
            active: true,
            currentWindow: true,
          });
          const pageContent = await chrome.tabs.sendMessage(tab.id, {
            type: "GET_PAGE_CONTENT",
          });

          // Get screenshot
          const screenshot = await captureScreenshot();

          // Upload both
          const payload = {
            content: pageContent.data?.content || "",
            metadata: {
              ...pageContent.data?.metadata,
              screenshot: screenshot.success ? screenshot.data : null,
            },
            source: "Chrome Extension",
            type: "page-with-screenshot",
          };

          if (request.sessionId) {
            payload.session_id = request.sessionId;
          }

          const response = await fetch(`${activeHubUrl}/api/ingest`, {
            method: "POST",
            headers: getAuthHeaders(),
            body: JSON.stringify(payload),
          });

          if (response.ok) {
            const data = await response.json();
            sendResponse({ success: true, data });
          } else {
            throw new Error(`HTTP ${response.status}`);
          }
        } catch (error) {
          sendResponse({ success: false, error: error.message });
        }
      })();
      return true;

    case "CHAT_RESPONSE":
      // Forward to side panel if open
      // This is handled by the side panel's own message listener
      break;

    default:
      sendResponse({ success: false, error: "Unknown message type" });
  }
});

// Initial connection check
loadApiKey().then(() => checkHubConnection());
