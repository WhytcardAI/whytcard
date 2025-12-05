// WhytCard Content Script
// Handles page content extraction, floating button, and communication with background

console.log("WhytCard content script loaded");

const HUB_URL = "http://localhost:3000";

// ==================== UTILITY FUNCTIONS ====================

function showToast(message, type = "success") {
  const existing = document.querySelector(".whytcard-toast");
  if (existing) existing.remove();

  const toast = document.createElement("div");
  toast.className = `whytcard-toast ${type}`;
  toast.textContent = message;
  document.body.appendChild(toast);

  setTimeout(() => {
    toast.style.animation = "whytcard-toast-out 0.3s ease forwards";
    setTimeout(() => toast.remove(), 300);
  }, 3000);
}

// ==================== ADVANCED PAGE SCRAPING ====================

function getPageContent() {
  // Extract main content from the page
  const selectors = [
    "article",
    "main",
    '[role="main"]',
    ".content",
    ".post-content",
    ".article-content",
    "#content",
  ];

  let content = "";
  for (const selector of selectors) {
    const el = document.querySelector(selector);
    if (el) {
      content = el.innerText;
      break;
    }
  }

  // Fallback to body content (cleaned)
  if (!content) {
    const clone = document.body.cloneNode(true);
    clone
      .querySelectorAll(
        "script, style, nav, header, footer, aside, .ad, [class*='advertisement']"
      )
      .forEach((el) => el.remove());
    content = clone.innerText;
  }

  // Limit content length
  const MAX_LENGTH = 10000;
  if (content.length > MAX_LENGTH) {
    content = content.substring(0, MAX_LENGTH) + "... [truncated]";
  }

  return content.trim();
}

function getStructuredPageContent() {
  const result = {
    text: "",
    title: document.title,
    url: window.location.href,
    headings: [],
    links: [],
    images: [],
    tables: [],
    codeBlocks: [],
    lists: [],
    metadata: getPageMetadata(),
    pageType: detectPageType(),
  };

  // Extract headings with hierarchy
  document.querySelectorAll("h1, h2, h3, h4, h5, h6").forEach((h) => {
    result.headings.push({
      level: parseInt(h.tagName[1]),
      text: h.innerText.trim().substring(0, 200),
    });
  });

  // Extract links (limited to most important)
  const mainContent =
    document.querySelector("article, main, [role='main'], .content") ||
    document.body;
  const links = mainContent.querySelectorAll("a[href]");
  const seenUrls = new Set();
  links.forEach((a) => {
    const href = a.href;
    if (
      href &&
      !seenUrls.has(href) &&
      !href.startsWith("javascript:") &&
      result.links.length < 50
    ) {
      seenUrls.add(href);
      result.links.push({
        text: a.innerText.trim().substring(0, 100) || a.title || href,
        url: href,
        isExternal: !href.startsWith(window.location.origin),
      });
    }
  });

  // Extract images with alt text
  document.querySelectorAll("img[src]").forEach((img) => {
    if (result.images.length < 30) {
      const src = img.src;
      if (src && !src.startsWith("data:image/svg")) {
        result.images.push({
          src: src,
          alt: img.alt || "",
          width: img.naturalWidth || img.width,
          height: img.naturalHeight || img.height,
        });
      }
    }
  });

  // Extract tables
  document.querySelectorAll("table").forEach((table) => {
    if (result.tables.length < 5) {
      const headers = [];
      const rows = [];

      table.querySelectorAll("thead th, thead td").forEach((th) => {
        headers.push(th.innerText.trim());
      });

      table.querySelectorAll("tbody tr").forEach((tr, i) => {
        if (i < 20) {
          // Limit rows
          const row = [];
          tr.querySelectorAll("td, th").forEach((td) => {
            row.push(td.innerText.trim().substring(0, 200));
          });
          rows.push(row);
        }
      });

      if (headers.length > 0 || rows.length > 0) {
        result.tables.push({ headers, rows });
      }
    }
  });

  // Extract code blocks
  document.querySelectorAll("pre code, pre, code").forEach((code) => {
    if (result.codeBlocks.length < 10) {
      const text = code.innerText.trim();
      if (text.length > 20 && text.length < 5000) {
        const lang =
          code.className.match(/language-(\w+)/)?.[1] ||
          code.closest("pre")?.className.match(/language-(\w+)/)?.[1] ||
          "unknown";
        result.codeBlocks.push({
          language: lang,
          code: text,
        });
      }
    }
  });

  // Extract lists
  document.querySelectorAll("ul, ol").forEach((list) => {
    if (result.lists.length < 10) {
      const items = [];
      list.querySelectorAll(":scope > li").forEach((li, i) => {
        if (i < 20) {
          items.push(li.innerText.trim().substring(0, 300));
        }
      });
      if (items.length > 0) {
        result.lists.push({
          type: list.tagName.toLowerCase(),
          items: items,
        });
      }
    }
  });

  // Get main text content
  result.text = getPageContent();

  return result;
}

function detectPageType() {
  const url = window.location.href;
  const meta = document.querySelector('meta[property="og:type"]')?.content;

  if (meta) return meta;

  // Detection heuristics
  if (
    url.includes("/docs/") ||
    url.includes("/documentation/") ||
    document.querySelector(".docusaurus, .sphinx, .mkdocs")
  ) {
    return "documentation";
  }
  if (
    url.includes("/blog/") ||
    url.includes("/article/") ||
    document.querySelector("article.post, .blog-post")
  ) {
    return "article";
  }
  if (url.includes("github.com") && url.includes("/blob/")) {
    return "code";
  }
  if (
    url.includes("github.com") &&
    (url.includes("/issues/") || url.includes("/pull/"))
  ) {
    return "github-issue";
  }
  if (
    document.querySelector("pre code") &&
    document.querySelectorAll("pre code").length > 3
  ) {
    return "code-heavy";
  }
  if (
    document.querySelector("table") &&
    document.querySelectorAll("table").length > 2
  ) {
    return "data-table";
  }

  return "webpage";
}

function getSelectedText() {
  return window.getSelection().toString().trim();
}

function getPageMetadata() {
  return {
    url: window.location.href,
    title: document.title,
    description:
      document.querySelector('meta[name="description"]')?.content ||
      document.querySelector('meta[property="og:description"]')?.content ||
      "",
    author: document.querySelector('meta[name="author"]')?.content || "",
    keywords: document.querySelector('meta[name="keywords"]')?.content || "",
    favicon:
      document.querySelector('link[rel="icon"]')?.href ||
      document.querySelector('link[rel="shortcut icon"]')?.href ||
      `${window.location.origin}/favicon.ico`,
    ogImage: document.querySelector('meta[property="og:image"]')?.content || "",
    canonical:
      document.querySelector('link[rel="canonical"]')?.href ||
      window.location.href,
    language: document.documentElement.lang || "en",
    timestamp: new Date().toISOString(),
  };
}

// ==================== FLOATING ACTION BUTTON ====================

let fabElement = null;

function createFAB() {
  if (fabElement) return;

  fabElement = document.createElement("button");
  fabElement.className = "whytcard-fab";
  fabElement.title = "WhytCard";
  fabElement.innerHTML = `
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/>
    </svg>
  `;

  fabElement.addEventListener("click", () => {
    showQuickInput();
  });

  document.body.appendChild(fabElement);
}

function toggleFAB(show) {
  if (show && !fabElement) {
    createFAB();
  } else if (!show && fabElement) {
    fabElement.remove();
    fabElement = null;
  }
}

// ==================== QUICK INPUT MODAL ====================

function showQuickInput(prefilledText = "") {
  // Remove existing modal if any
  document.querySelector(".whytcard-quick-input-backdrop")?.remove();
  document.querySelector(".whytcard-quick-input")?.remove();

  const backdrop = document.createElement("div");
  backdrop.className = "whytcard-quick-input-backdrop";

  const modal = document.createElement("div");
  modal.className = "whytcard-quick-input";
  modal.innerHTML = `
    <h3>
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" stroke-width="2">
        <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/>
      </svg>
      Ask WhytCard
    </h3>
    <textarea id="whytcard-input" placeholder="Ask a question about this page or anything else...">${prefilledText}</textarea>
    <div class="whytcard-quick-input-actions">
      <button class="secondary" id="whytcard-cancel">Cancel</button>
      <button class="primary" id="whytcard-send">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="margin-right: 6px;">
          <line x1="22" y1="2" x2="11" y2="13"></line>
          <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
        </svg>
        Send
      </button>
    </div>
  `;

  document.body.appendChild(backdrop);
  document.body.appendChild(modal);

  const input = modal.querySelector("#whytcard-input");
  input.focus();
  input.setSelectionRange(input.value.length, input.value.length);

  // Event handlers
  backdrop.addEventListener("click", closeQuickInput);
  modal
    .querySelector("#whytcard-cancel")
    .addEventListener("click", closeQuickInput);
  modal
    .querySelector("#whytcard-send")
    .addEventListener("click", sendQuickMessage);
  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      sendQuickMessage();
    } else if (e.key === "Escape") {
      closeQuickInput();
    }
  });
}

function closeQuickInput() {
  document.querySelector(".whytcard-quick-input-backdrop")?.remove();
  document.querySelector(".whytcard-quick-input")?.remove();
}

async function sendQuickMessage() {
  const input = document.querySelector("#whytcard-input");
  const message = input?.value.trim();

  if (!message) {
    showToast("Please enter a message", "error");
    return;
  }

  const sendBtn = document.querySelector("#whytcard-send");
  sendBtn.disabled = true;
  sendBtn.innerHTML = "Sending...";

  try {
    const response = await fetch(`${HUB_URL}/api/chat`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        message: message,
        context: {
          pageUrl: window.location.href,
          pageTitle: document.title,
          source: "Chrome Extension",
        },
      }),
    });

    const data = await response.json();
    closeQuickInput();
    showToast("Message sent! Check the Hub for response.", "success");

    // Notify background to open side panel with response
    chrome.runtime.sendMessage({
      type: "CHAT_RESPONSE",
      data: data,
    });
  } catch (error) {
    showToast(`Error: ${error.message}`, "error");
    sendBtn.disabled = false;
    sendBtn.innerHTML = "Send";
  }
}

// ==================== MESSAGE HANDLERS ====================

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  switch (request.type) {
    case "GET_PAGE_CONTENT":
      sendResponse({
        success: true,
        data: {
          content: getPageContent(),
          metadata: getPageMetadata(),
        },
      });
      break;

    case "GET_STRUCTURED_CONTENT":
      sendResponse({
        success: true,
        data: getStructuredPageContent(),
      });
      break;

    case "GET_PAGE_IMAGES":
      const images = [];
      document.querySelectorAll("img[src]").forEach((img) => {
        if (images.length < 50 && img.src && !img.src.startsWith("data:")) {
          images.push({
            src: img.src,
            alt: img.alt || "",
            width: img.naturalWidth || img.width,
            height: img.naturalHeight || img.height,
          });
        }
      });
      sendResponse({ success: true, data: images });
      break;

    case "GET_PAGE_LINKS":
      const links = [];
      const seen = new Set();
      document.querySelectorAll("a[href]").forEach((a) => {
        if (
          links.length < 100 &&
          a.href &&
          !seen.has(a.href) &&
          !a.href.startsWith("javascript:")
        ) {
          seen.add(a.href);
          links.push({
            text: a.innerText.trim().substring(0, 100) || a.href,
            url: a.href,
            isExternal: !a.href.startsWith(window.location.origin),
          });
        }
      });
      sendResponse({ success: true, data: links });
      break;

    case "GET_SELECTED_TEXT":
      const selectedText = getSelectedText();
      sendResponse({
        success: !!selectedText,
        data: selectedText,
      });
      break;

    case "SHOW_TOAST":
      showToast(request.message, request.toastType || "success");
      sendResponse({ success: true });
      break;

    case "TOGGLE_FAB":
      toggleFAB(request.show);
      sendResponse({ success: true });
      break;

    case "SHOW_QUICK_INPUT":
      showQuickInput(request.prefill || "");
      sendResponse({ success: true });
      break;

    case "CAPTURE_AND_SEND":
      captureAndSend(request.mode || "page", request.sessionId);
      sendResponse({ success: true });
      break;

    default:
      sendResponse({ success: false, error: "Unknown message type" });
  }
  return true; // Async response
});

// ==================== CAPTURE FUNCTIONS ====================

async function captureAndSend(mode = "page", sessionId = null) {
  let content = "";
  let metadata = getPageMetadata();

  if (mode === "selection") {
    content = getSelectedText();
    if (!content) {
      showToast("No text selected", "error");
      return;
    }
  } else if (mode === "structured") {
    const structured = getStructuredPageContent();
    content = JSON.stringify(structured);
    metadata = structured.metadata;
  } else {
    content = getPageContent();
  }

  try {
    const payload = {
      content,
      metadata,
      source: "Chrome Extension",
      type:
        mode === "selection"
          ? "selection"
          : mode === "structured"
          ? "structured"
          : "page",
    };

    if (sessionId) {
      payload.session_id = sessionId;
    }

    const response = await fetch(`${HUB_URL}/api/ingest`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });

    if (response.ok) {
      showToast(
        `${mode === "selection" ? "Selection" : "Page"} sent to WhytCard!`,
        "success"
      );
    } else {
      throw new Error(`HTTP ${response.status}`);
    }
  } catch (error) {
    showToast(`Failed to send: ${error.message}`, "error");
  }
}

// ==================== INITIALIZATION ====================

// Check settings and initialize FAB if enabled
async function init() {
  try {
    const settings = await chrome.storage.local.get(["showFab"]);
    if (settings.showFab !== false) {
      // Default to showing FAB
      createFAB();
    }
  } catch (e) {
    // Storage not available, create FAB anyway
    createFAB();
  }
}

// Initialize when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
