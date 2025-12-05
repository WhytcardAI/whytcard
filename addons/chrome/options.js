// WhytCard - Options Page Script

document.addEventListener("DOMContentLoaded", () => {
  initOptions();
});

async function initOptions() {
  await loadSettings();
  setupEventListeners();
  checkConnectionStatus();
}

// ==================== SETTINGS MANAGEMENT ====================

async function loadSettings() {
  const settings = await chrome.storage.local.get([
    "apiKey",
    "showFab",
    "autoConnect",
    "showContextMenu",
  ]);

  // API Key (masked)
  const apiKeyInput = document.getElementById("apiKey");
  if (settings.apiKey) {
    apiKeyInput.value = settings.apiKey;
  }

  // Toggles
  document.getElementById("showFab").checked = settings.showFab !== false;
  document.getElementById("autoConnect").checked =
    settings.autoConnect !== false;
  document.getElementById("showContextMenu").checked =
    settings.showContextMenu !== false;
}

async function saveSettings(key, value) {
  await chrome.storage.local.set({ [key]: value });

  // Notify background script of settings change
  chrome.runtime.sendMessage({
    type: "SETTINGS_CHANGED",
    key,
    value,
  });
}

// ==================== EVENT LISTENERS ====================

function setupEventListeners() {
  // API Key - Save
  document.getElementById("saveApiKey").addEventListener("click", async () => {
    const apiKey = document.getElementById("apiKey").value.trim();

    if (!apiKey) {
      showAlert("error", "Please enter an API key");
      return;
    }

    // Validate token with Hub first
    const isValid = await validateToken(apiKey);
    if (!isValid) {
      showAlert("error", "Invalid token - could not authenticate with Hub");
      return;
    }

    try {
      await chrome.runtime.sendMessage({
        type: "SET_API_KEY",
        apiKey: apiKey,
      });

      showAlert("success", "API Token saved and validated successfully!");

      // Refresh connection status
      setTimeout(checkConnectionStatus, 500);
    } catch (error) {
      showAlert("error", `Error saving API Token: ${error.message}`);
    }
  });

  // API Key - Clear
  document.getElementById("clearApiKey").addEventListener("click", async () => {
    try {
      await chrome.runtime.sendMessage({ type: "CLEAR_API_KEY" });
      document.getElementById("apiKey").value = "";
      showAlert("success", "API Token cleared");
      setTimeout(checkConnectionStatus, 500);
    } catch (error) {
      showAlert("error", `Error clearing API Token: ${error.message}`);
    }
  });

  // API Key - Toggle Visibility
  let isVisible = false;
  document.getElementById("toggleVisibility").addEventListener("click", () => {
    const input = document.getElementById("apiKey");
    const icon = document.getElementById("eyeIcon");

    isVisible = !isVisible;
    input.type = isVisible ? "text" : "password";

    // Update icon
    if (isVisible) {
      icon.innerHTML = `
        <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
        <line x1="1" y1="1" x2="23" y2="23"></line>
      `;
    } else {
      icon.innerHTML = `
        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
        <circle cx="12" cy="12" r="3"></circle>
      `;
    }
  });

  // Refresh button
  document.getElementById("refreshBtn").addEventListener("click", () => {
    checkConnectionStatus();
  });

  // Toggles
  document.getElementById("showFab").addEventListener("change", async (e) => {
    await saveSettings("showFab", e.target.checked);
  });

  document
    .getElementById("autoConnect")
    .addEventListener("change", async (e) => {
      await saveSettings("autoConnect", e.target.checked);
    });

  document
    .getElementById("showContextMenu")
    .addEventListener("change", async (e) => {
      await saveSettings("showContextMenu", e.target.checked);
    });
}

// ==================== TOKEN VALIDATION ====================

async function validateToken(token) {
  try {
    const state = await chrome.storage.local.get(["activeHubUrl"]);
    const hubUrl = state.activeHubUrl || "http://localhost:3000";

    const response = await fetch(`${hubUrl}/api/tokens/validate`, {
      method: "GET",
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    });

    return response.ok;
  } catch (error) {
    console.error("Token validation error:", error);
    return false;
  }
}

// ==================== CONNECTION STATUS ====================

async function checkConnectionStatus() {
  const statusDot = document.getElementById("statusDot");
  const statusLabel = document.getElementById("statusLabel");
  const statusDetail = document.getElementById("statusDetail");

  // Show pending state
  statusDot.className = "status-dot pending";
  statusLabel.textContent = "Checking...";
  statusDetail.textContent = "Connecting to WhytCard Hub";

  try {
    const response = await chrome.runtime.sendMessage({ type: "PING_HUB" });

    if (response.success) {
      // Also check SSE status
      const sseStatus = await chrome.runtime.sendMessage({
        type: "GET_SSE_STATUS",
      });

      statusDot.className = "status-dot connected";
      statusLabel.textContent = "Connected";
      statusDetail.textContent = sseStatus.connected
        ? `Hub: ${response.hubUrl} (SSE active)`
        : `Hub: ${response.hubUrl} (SSE inactive)`;
    } else {
      statusDot.className = "status-dot disconnected";
      statusLabel.textContent = "Disconnected";
      statusDetail.textContent = "WhytCard Hub is not running";
    }
  } catch (error) {
    statusDot.className = "status-dot disconnected";
    statusLabel.textContent = "Error";
    statusDetail.textContent = error.message;
  }
}

// ==================== ALERTS ====================

function showAlert(type, message) {
  const successAlert = document.getElementById("alertSuccess");
  const errorAlert = document.getElementById("alertError");

  // Hide both first
  successAlert.classList.remove("show");
  errorAlert.classList.remove("show");

  if (type === "success") {
    document.getElementById("successMessage").textContent = message;
    successAlert.classList.add("show");
    setTimeout(() => successAlert.classList.remove("show"), 3000);
  } else {
    document.getElementById("errorMessage").textContent = message;
    errorAlert.classList.add("show");
    setTimeout(() => errorAlert.classList.remove("show"), 5000);
  }
}
