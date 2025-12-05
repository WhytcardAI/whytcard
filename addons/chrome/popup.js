// WhytCard Popup Script
// Handles popup UI, connection status, and quick actions

let hubUrl = "http://localhost:3000";
let isConnected = false;

// ==================== DOM ELEMENTS ====================

const dot = document.getElementById("dot");
const connectionText = document.getElementById("connection-text");
const statusEl = document.getElementById("status");
const pingBtn = document.getElementById("pingBtn");
const pageFavicon = document.getElementById("pageFavicon");
const pageTitle = document.getElementById("pageTitle");
const settingsPanel = document.getElementById("settingsPanel");
const fabToggle = document.getElementById("fabToggle");
const languageSelect = document.getElementById("languageSelect");

// Action buttons
const chatAction = document.getElementById("chatAction");
const saveAction = document.getElementById("saveAction");
const summarizeAction = document.getElementById("summarizeAction");
const settingsAction = document.getElementById("settingsAction");

// ==================== INITIALIZATION ====================

async function init() {
  // Load language first
  await loadLanguage();
  updateUI();

  // Load settings
  const settings = await chrome.storage.local.get([
    "showFab",
    "activeHubUrl",
    "isHubConnected",
  ]);

  if (settings.activeHubUrl) {
    hubUrl = settings.activeHubUrl;
  }

  // Initialize FAB toggle
  const showFab = settings.showFab !== false;
  fabToggle.classList.toggle("active", showFab);

  // Update connection status
  updateConnectionState(settings.isHubConnected);

  // Get current page info
  loadPageInfo();

  // Setup event listeners
  setupEventListeners();

  // Check connection
  checkConnection();
}

// ==================== EVENT LISTENERS ====================

function setupEventListeners() {
  // Connect button
  pingBtn.addEventListener("click", handleConnect);

  // Quick actions
  chatAction.addEventListener("click", openSidePanel);
  saveAction.addEventListener("click", savePage);
  summarizeAction.addEventListener("click", summarizePage);
  settingsAction.addEventListener("click", toggleSettings);

  // FAB toggle
  fabToggle.addEventListener("click", toggleFab);

  // Language selector
  languageSelect.addEventListener("change", async (e) => {
    await setLanguage(e.target.value);
    updateConnectionState(isConnected);
  });
}

// ==================== CONNECTION ====================

async function checkConnection() {
  try {
    const response = await chrome.runtime.sendMessage({
      type: "GET_HUB_STATUS",
    });
    updateConnectionState(response.connected);
    if (response.hubUrl) {
      hubUrl = response.hubUrl;
    }
  } catch (error) {
    updateConnectionState(false);
  }
}

async function handleConnect() {
  pingBtn.disabled = true;
  const btnSpan = pingBtn.querySelector("span");
  if (btnSpan) btnSpan.textContent = "...";

  try {
    const response = await fetch(`${hubUrl}/api/ping?source=chrome`);
    const data = await response.json();

    updateConnectionState(true);
    showStatus(`${t("connected")}!`, "success");
  } catch (error) {
    updateConnectionState(false);
    showStatus(`${t("error")} ${error.message}`, "error");
  } finally {
    pingBtn.disabled = false;
    updateConnectionState(isConnected);
  }
}

function updateConnectionState(connected) {
  isConnected = connected;
  dot.classList.toggle("connected", connected);
  connectionText.textContent = connected ? t("connected") : t("disconnected");

  // Update ping button text
  const btnSpan = pingBtn.querySelector("span");
  if (btnSpan) {
    btnSpan.textContent = t("connectHub");
  }
}

// ==================== PAGE INFO ====================

async function loadPageInfo() {
  try {
    const [tab] = await chrome.tabs.query({
      active: true,
      currentWindow: true,
    });

    if (tab) {
      // Set favicon
      if (tab.favIconUrl) {
        pageFavicon.src = tab.favIconUrl;
      } else {
        pageFavicon.src = "icons/icon16.png";
      }

      // Set title (truncated)
      const title = tab.title || "Unknown Page";
      pageTitle.textContent =
        title.length > 40 ? title.substring(0, 40) + "..." : title;
      pageTitle.title = title; // Full title on hover
    }
  } catch (error) {
    pageTitle.textContent = "Unable to load page info";
  }
}

// ==================== QUICK ACTIONS ====================

async function openSidePanel() {
  try {
    const [tab] = await chrome.tabs.query({
      active: true,
      currentWindow: true,
    });
    if (tab) {
      await chrome.sidePanel.open({ tabId: tab.id });
      window.close();
    }
  } catch (error) {
    showStatus(`${t("error")} ${error.message}`, "error");
  }
}

async function savePage() {
  if (!isConnected) {
    showStatus(t("disconnected"), "error");
    return;
  }

  const span = saveAction.querySelector("span");
  const originalText = span.textContent;
  span.textContent = "...";

  try {
    const [tab] = await chrome.tabs.query({
      active: true,
      currentWindow: true,
    });

    // Get page content from content script
    const pageContent = await chrome.tabs.sendMessage(tab.id, {
      type: "GET_PAGE_CONTENT",
    });

    if (!pageContent.success) {
      throw new Error(t("failedToExtract"));
    }

    // Send to Hub
    const response = await fetch(`${hubUrl}/api/ingest`, {
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
      showStatus(t("pageSaved"), "success");
    } else {
      throw new Error(`HTTP ${response.status}`);
    }
  } catch (error) {
    showStatus(`${t("failedToSave")} ${error.message}`, "error");
  } finally {
    span.textContent = originalText;
  }
}

async function summarizePage() {
  if (!isConnected) {
    showStatus(t("disconnected"), "error");
    return;
  }

  const span = summarizeAction.querySelector("span");
  const originalText = span.textContent;
  span.textContent = "...";

  try {
    const [tab] = await chrome.tabs.query({
      active: true,
      currentWindow: true,
    });

    // Get page content
    const pageContent = await chrome.tabs.sendMessage(tab.id, {
      type: "GET_PAGE_CONTENT",
    });

    if (!pageContent.success) {
      throw new Error(t("failedToExtract"));
    }

    // Request summary from Hub
    const response = await fetch(`${hubUrl}/api/chat`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        message: `${t("promptSummarize")}\n\nTitle: ${
          pageContent.data.metadata.title
        }\n\nContent:\n${pageContent.data.content.substring(0, 5000)}`,
        source: "Chrome Extension",
      }),
    });

    const data = await response.json();

    // Open side panel and show summary
    await chrome.sidePanel.open({ tabId: tab.id });

    setTimeout(() => {
      chrome.runtime.sendMessage({
        type: "SHOW_SUMMARY",
        summary: data.reply,
        pageTitle: pageContent.data.metadata.title,
      });
    }, 500);

    window.close();
  } catch (error) {
    showStatus(`${t("error")} ${error.message}`, "error");
  } finally {
    span.textContent = originalText;
  }
}

function toggleSettings() {
  settingsPanel.classList.toggle("visible");
}

async function toggleFab() {
  const isActive = fabToggle.classList.toggle("active");
  await chrome.storage.local.set({ showFab: isActive });

  // Notify all tabs
  const tabs = await chrome.tabs.query({});
  for (const tab of tabs) {
    try {
      await chrome.tabs.sendMessage(tab.id, {
        type: "TOGGLE_FAB",
        show: isActive,
      });
    } catch (e) {
      // Content script not loaded
    }
  }
}

// ==================== UTILITIES ====================

function showStatus(message, type) {
  statusEl.textContent = message;
  statusEl.className = `visible ${type}`;

  if (type === "success") {
    setTimeout(() => {
      statusEl.className = "";
    }, 3000);
  }
}

// ==================== START ====================

init();
