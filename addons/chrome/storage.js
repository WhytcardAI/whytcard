// WhytCard Chrome Extension - Local-First Storage Module
// All data is stored locally first. User explicitly chooses what to sync to Hub.

// ==================== DATA MODELS ====================

/**
 * @typedef {'local' | 'pending' | 'synced' | 'failed'} SyncStatus
 * @typedef {'none' | 'pending' | 'indexed' | 'failed'} RagStatus
 */

/**
 * @typedef {Object} Highlight
 * @property {string} id - Unique ID
 * @property {string} text - Highlighted text
 * @property {string} url - Source URL
 * @property {string} pageTitle - Page title
 * @property {string|null} note - User note
 * @property {string} color - Highlight color
 * @property {string} createdAt - ISO timestamp
 * @property {string} updatedAt - ISO timestamp
 * @property {SyncStatus} syncStatus - Local/synced state
 * @property {RagStatus} ragStatus - RAG indexing state
 * @property {string|null} hubFileId - ID in Hub after sync
 */

/**
 * @typedef {Object} Clip
 * @property {string} id - Unique ID
 * @property {string} content - Clipped content (text or HTML)
 * @property {string} contentType - 'text' | 'html' | 'image'
 * @property {string} url - Source URL
 * @property {string} pageTitle - Page title
 * @property {string} createdAt - ISO timestamp
 * @property {string} updatedAt - ISO timestamp
 * @property {SyncStatus} syncStatus - Local/synced state
 * @property {RagStatus} ragStatus - RAG indexing state
 * @property {string|null} hubFileId - ID in Hub after sync
 */

/**
 * @typedef {Object} Note
 * @property {string} id - Unique ID
 * @property {string} content - Note content (markdown)
 * @property {string|null} url - Associated URL (optional)
 * @property {string|null} pageTitle - Associated page title
 * @property {string[]} tags - User tags
 * @property {string} createdAt - ISO timestamp
 * @property {string} updatedAt - ISO timestamp
 * @property {SyncStatus} syncStatus - Local/synced state
 * @property {RagStatus} ragStatus - RAG indexing state
 * @property {string|null} hubFileId - ID in Hub after sync
 */

/**
 * @typedef {Object} PageCapture
 * @property {string} id - Unique ID
 * @property {string} url - Page URL
 * @property {string} title - Page title
 * @property {string} content - Extracted text content
 * @property {string|null} screenshot - Base64 screenshot (optional)
 * @property {Object} metadata - Page metadata
 * @property {string} createdAt - ISO timestamp
 * @property {SyncStatus} syncStatus - Local/synced state
 * @property {RagStatus} ragStatus - RAG indexing state
 * @property {string|null} hubFileId - ID in Hub after sync
 */

/**
 * @typedef {Object} LocalStorage
 * @property {Highlight[]} highlights
 * @property {Clip[]} clips
 * @property {Note[]} notes
 * @property {PageCapture[]} pageCaptures
 * @property {Object} syncConfig - Sync configuration
 */

// ==================== STORAGE KEYS ====================

const STORAGE_KEYS = {
  HIGHLIGHTS: "whytcard_highlights",
  CLIPS: "whytcard_clips",
  NOTES: "whytcard_notes",
  PAGE_CAPTURES: "whytcard_pageCaptures",
  SYNC_CONFIG: "whytcard_syncConfig",
  LAST_SYNC: "whytcard_lastSync",
};

// ==================== ID GENERATION ====================

function generateId() {
  return `${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
}

// ==================== GENERIC STORAGE OPERATIONS ====================

/**
 * Get all items from a storage key
 * @param {string} key - Storage key
 * @returns {Promise<any[]>}
 */
async function getAll(key) {
  const result = await chrome.storage.local.get(key);
  return result[key] || [];
}

/**
 * Save all items to a storage key
 * @param {string} key - Storage key
 * @param {any[]} items - Items to save
 */
async function saveAll(key, items) {
  await chrome.storage.local.set({ [key]: items });
}

/**
 * Get item by ID from a storage key
 * @param {string} key - Storage key
 * @param {string} id - Item ID
 * @returns {Promise<any|null>}
 */
async function getById(key, id) {
  const items = await getAll(key);
  return items.find((item) => item.id === id) || null;
}

/**
 * Add a new item to storage
 * @param {string} key - Storage key
 * @param {Object} item - Item to add (without id)
 * @returns {Promise<Object>} - Created item with id
 */
async function addItem(key, item) {
  const items = await getAll(key);
  const now = new Date().toISOString();
  const newItem = {
    id: generateId(),
    ...item,
    createdAt: now,
    updatedAt: now,
    syncStatus: "local",
    ragStatus: "none",
    hubFileId: null,
  };
  items.push(newItem);
  await saveAll(key, items);
  return newItem;
}

/**
 * Update an existing item
 * @param {string} key - Storage key
 * @param {string} id - Item ID
 * @param {Object} updates - Fields to update
 * @returns {Promise<Object|null>}
 */
async function updateItem(key, id, updates) {
  const items = await getAll(key);
  const index = items.findIndex((item) => item.id === id);
  if (index === -1) return null;

  const updatedItem = {
    ...items[index],
    ...updates,
    updatedAt: new Date().toISOString(),
  };

  // If content changed and was synced, mark as pending
  if (
    items[index].syncStatus === "synced" &&
    (updates.content !== undefined || updates.text !== undefined)
  ) {
    updatedItem.syncStatus = "pending";
  }

  items[index] = updatedItem;
  await saveAll(key, items);
  return updatedItem;
}

/**
 * Delete an item
 * @param {string} key - Storage key
 * @param {string} id - Item ID
 * @returns {Promise<boolean>}
 */
async function deleteItem(key, id) {
  const items = await getAll(key);
  const filtered = items.filter((item) => item.id !== id);
  if (filtered.length === items.length) return false;
  await saveAll(key, filtered);
  return true;
}

// ==================== HIGHLIGHTS API ====================

const Highlights = {
  /**
   * Get all highlights
   * @returns {Promise<Highlight[]>}
   */
  async getAll() {
    return getAll(STORAGE_KEYS.HIGHLIGHTS);
  },

  /**
   * Get highlights for a specific URL
   * @param {string} url - Page URL
   * @returns {Promise<Highlight[]>}
   */
  async getByUrl(url) {
    const all = await this.getAll();
    return all.filter((h) => h.url === url);
  },

  /**
   * Get highlight by ID
   * @param {string} id
   * @returns {Promise<Highlight|null>}
   */
  async getById(id) {
    return getById(STORAGE_KEYS.HIGHLIGHTS, id);
  },

  /**
   * Add a new highlight
   * @param {Object} data - Highlight data
   * @returns {Promise<Highlight>}
   */
  async add(data) {
    return addItem(STORAGE_KEYS.HIGHLIGHTS, {
      text: data.text,
      url: data.url,
      pageTitle: data.pageTitle || "",
      note: data.note || null,
      color: data.color || "yellow",
    });
  },

  /**
   * Update a highlight
   * @param {string} id
   * @param {Object} updates
   * @returns {Promise<Highlight|null>}
   */
  async update(id, updates) {
    return updateItem(STORAGE_KEYS.HIGHLIGHTS, id, updates);
  },

  /**
   * Delete a highlight
   * @param {string} id
   * @returns {Promise<boolean>}
   */
  async delete(id) {
    return deleteItem(STORAGE_KEYS.HIGHLIGHTS, id);
  },

  /**
   * Get highlights pending sync
   * @returns {Promise<Highlight[]>}
   */
  async getPendingSync() {
    const all = await this.getAll();
    return all.filter((h) => h.syncStatus === "local" || h.syncStatus === "pending");
  },

  /**
   * Get highlights by RAG status
   * @param {RagStatus} status
   * @returns {Promise<Highlight[]>}
   */
  async getByRagStatus(status) {
    const all = await this.getAll();
    return all.filter((h) => h.ragStatus === status);
  },
};

// ==================== CLIPS API ====================

const Clips = {
  /**
   * Get all clips
   * @returns {Promise<Clip[]>}
   */
  async getAll() {
    return getAll(STORAGE_KEYS.CLIPS);
  },

  /**
   * Get clips for a specific URL
   * @param {string} url
   * @returns {Promise<Clip[]>}
   */
  async getByUrl(url) {
    const all = await this.getAll();
    return all.filter((c) => c.url === url);
  },

  /**
   * Get clip by ID
   * @param {string} id
   * @returns {Promise<Clip|null>}
   */
  async getById(id) {
    return getById(STORAGE_KEYS.CLIPS, id);
  },

  /**
   * Add a new clip
   * @param {Object} data
   * @returns {Promise<Clip>}
   */
  async add(data) {
    return addItem(STORAGE_KEYS.CLIPS, {
      content: data.content,
      contentType: data.contentType || "text",
      url: data.url,
      pageTitle: data.pageTitle || "",
    });
  },

  /**
   * Update a clip
   * @param {string} id
   * @param {Object} updates
   * @returns {Promise<Clip|null>}
   */
  async update(id, updates) {
    return updateItem(STORAGE_KEYS.CLIPS, id, updates);
  },

  /**
   * Delete a clip
   * @param {string} id
   * @returns {Promise<boolean>}
   */
  async delete(id) {
    return deleteItem(STORAGE_KEYS.CLIPS, id);
  },

  /**
   * Get clips pending sync
   * @returns {Promise<Clip[]>}
   */
  async getPendingSync() {
    const all = await this.getAll();
    return all.filter((c) => c.syncStatus === "local" || c.syncStatus === "pending");
  },
};

// ==================== NOTES API ====================

const Notes = {
  /**
   * Get all notes
   * @returns {Promise<Note[]>}
   */
  async getAll() {
    return getAll(STORAGE_KEYS.NOTES);
  },

  /**
   * Get notes for a specific URL
   * @param {string} url
   * @returns {Promise<Note[]>}
   */
  async getByUrl(url) {
    const all = await this.getAll();
    return all.filter((n) => n.url === url);
  },

  /**
   * Get notes by tag
   * @param {string} tag
   * @returns {Promise<Note[]>}
   */
  async getByTag(tag) {
    const all = await this.getAll();
    return all.filter((n) => n.tags && n.tags.includes(tag));
  },

  /**
   * Get note by ID
   * @param {string} id
   * @returns {Promise<Note|null>}
   */
  async getById(id) {
    return getById(STORAGE_KEYS.NOTES, id);
  },

  /**
   * Add a new note
   * @param {Object} data
   * @returns {Promise<Note>}
   */
  async add(data) {
    return addItem(STORAGE_KEYS.NOTES, {
      content: data.content,
      url: data.url || null,
      pageTitle: data.pageTitle || null,
      tags: data.tags || [],
    });
  },

  /**
   * Update a note
   * @param {string} id
   * @param {Object} updates
   * @returns {Promise<Note|null>}
   */
  async update(id, updates) {
    return updateItem(STORAGE_KEYS.NOTES, id, updates);
  },

  /**
   * Delete a note
   * @param {string} id
   * @returns {Promise<boolean>}
   */
  async delete(id) {
    return deleteItem(STORAGE_KEYS.NOTES, id);
  },

  /**
   * Get notes pending sync
   * @returns {Promise<Note[]>}
   */
  async getPendingSync() {
    const all = await this.getAll();
    return all.filter((n) => n.syncStatus === "local" || n.syncStatus === "pending");
  },

  /**
   * Get all unique tags
   * @returns {Promise<string[]>}
   */
  async getAllTags() {
    const all = await this.getAll();
    const tagsSet = new Set();
    all.forEach((n) => {
      if (n.tags) {
        n.tags.forEach((t) => tagsSet.add(t));
      }
    });
    return Array.from(tagsSet).sort();
  },
};

// ==================== PAGE CAPTURES API ====================

const PageCaptures = {
  /**
   * Get all page captures
   * @returns {Promise<PageCapture[]>}
   */
  async getAll() {
    return getAll(STORAGE_KEYS.PAGE_CAPTURES);
  },

  /**
   * Get page capture by URL
   * @param {string} url
   * @returns {Promise<PageCapture|null>}
   */
  async getByUrl(url) {
    const all = await this.getAll();
    return all.find((p) => p.url === url) || null;
  },

  /**
   * Get page capture by ID
   * @param {string} id
   * @returns {Promise<PageCapture|null>}
   */
  async getById(id) {
    return getById(STORAGE_KEYS.PAGE_CAPTURES, id);
  },

  /**
   * Add a new page capture
   * @param {Object} data
   * @returns {Promise<PageCapture>}
   */
  async add(data) {
    return addItem(STORAGE_KEYS.PAGE_CAPTURES, {
      url: data.url,
      title: data.title,
      content: data.content,
      screenshot: data.screenshot || null,
      metadata: data.metadata || {},
    });
  },

  /**
   * Update a page capture
   * @param {string} id
   * @param {Object} updates
   * @returns {Promise<PageCapture|null>}
   */
  async update(id, updates) {
    return updateItem(STORAGE_KEYS.PAGE_CAPTURES, id, updates);
  },

  /**
   * Delete a page capture
   * @param {string} id
   * @returns {Promise<boolean>}
   */
  async delete(id) {
    return deleteItem(STORAGE_KEYS.PAGE_CAPTURES, id);
  },

  /**
   * Get page captures pending sync
   * @returns {Promise<PageCapture[]>}
   */
  async getPendingSync() {
    const all = await this.getAll();
    return all.filter((p) => p.syncStatus === "local" || p.syncStatus === "pending");
  },
};

// ==================== SYNC OPERATIONS ====================

const SyncManager = {
  /**
   * Get Hub URL and API key
   * @returns {Promise<{hubUrl: string, apiKey: string|null}>}
   */
  async getHubConfig() {
    const result = await chrome.storage.local.get(["activeHubUrl", "apiKey"]);
    return {
      hubUrl: result.activeHubUrl || "http://localhost:3000",
      apiKey: result.apiKey || null,
    };
  },

  /**
   * Get auth headers for Hub requests
   * @returns {Promise<Object>}
   */
  async getAuthHeaders() {
    const config = await this.getHubConfig();
    const headers = { "Content-Type": "application/json" };
    if (config.apiKey) {
      headers["Authorization"] = `Bearer ${config.apiKey}`;
    }
    return headers;
  },

  /**
   * Sync a single item to Hub
   * @param {string} storageKey - Storage key for the item type
   * @param {Object} item - Item to sync
   * @param {string} projectId - Target project ID
   * @returns {Promise<{success: boolean, hubFileId?: string, error?: string}>}
   */
  async syncItemToHub(storageKey, item, projectId) {
    const config = await this.getHubConfig();
    const headers = await this.getAuthHeaders();

    // Determine content type based on storage key
    let type = "document";
    if (storageKey === STORAGE_KEYS.HIGHLIGHTS) type = "highlight";
    else if (storageKey === STORAGE_KEYS.CLIPS) type = "clip";
    else if (storageKey === STORAGE_KEYS.NOTES) type = "note";
    else if (storageKey === STORAGE_KEYS.PAGE_CAPTURES) type = "page";

    // Prepare content for Hub
    const content = item.content || item.text || "";
    const metadata = {
      url: item.url,
      title: item.pageTitle || item.title,
      type: type,
      sourceExtension: "chrome",
      originalId: item.id,
      createdAt: item.createdAt,
    };

    if (item.tags) metadata.tags = item.tags;
    if (item.color) metadata.color = item.color;
    if (item.note) metadata.note = item.note;

    try {
      const response = await fetch(`${config.hubUrl}/api/ingest`, {
        method: "POST",
        headers: headers,
        body: JSON.stringify({
          content: content,
          metadata: metadata,
          source: "Chrome Extension",
          type: type,
          project_id: projectId,
        }),
      });

      if (response.status === 401) {
        return {
          success: false,
          error: "Unauthorized - Please configure API key",
        };
      }

      if (!response.ok) {
        const errorText = await response.text();
        return { success: false, error: `HTTP ${response.status}: ${errorText}` };
      }

      const data = await response.json();
      return {
        success: true,
        hubFileId: data.file_id || data.id,
      };
    } catch (error) {
      return { success: false, error: error.message };
    }
  },

  /**
   * Sync selected items to Hub
   * @param {Array<{storageKey: string, id: string}>} items - Items to sync
   * @param {string} projectId - Target project ID
   * @returns {Promise<{synced: number, failed: number, errors: string[]}>}
   */
  async syncSelectedItems(items, projectId) {
    const results = { synced: 0, failed: 0, errors: [] };

    for (const { storageKey, id } of items) {
      const item = await getById(storageKey, id);
      if (!item) {
        results.failed++;
        results.errors.push(`Item ${id} not found`);
        continue;
      }

      // Mark as pending before sync attempt
      await updateItem(storageKey, id, { syncStatus: "pending" });

      const result = await this.syncItemToHub(storageKey, item, projectId);

      if (result.success) {
        await updateItem(storageKey, id, {
          syncStatus: "synced",
          hubFileId: result.hubFileId,
        });
        results.synced++;
      } else {
        await updateItem(storageKey, id, { syncStatus: "failed" });
        results.failed++;
        results.errors.push(`${id}: ${result.error}`);
      }
    }

    // Update last sync timestamp
    await chrome.storage.local.set({
      [STORAGE_KEYS.LAST_SYNC]: new Date().toISOString(),
    });

    return results;
  },

  /**
   * Request RAG indexing for a synced item
   * @param {string} storageKey - Storage key
   * @param {string} id - Item ID
   * @returns {Promise<{success: boolean, error?: string}>}
   */
  async requestRagIndexing(storageKey, id) {
    const item = await getById(storageKey, id);
    if (!item) {
      return { success: false, error: "Item not found" };
    }

    if (item.syncStatus !== "synced" || !item.hubFileId) {
      return { success: false, error: "Item must be synced to Hub first" };
    }

    const config = await this.getHubConfig();
    const headers = await this.getAuthHeaders();

    try {
      // Mark as pending
      await updateItem(storageKey, id, { ragStatus: "pending" });

      const response = await fetch(`${config.hubUrl}/api/library/files/${item.hubFileId}/index`, {
        method: "POST",
        headers: headers,
      });

      if (!response.ok) {
        await updateItem(storageKey, id, { ragStatus: "failed" });
        return { success: false, error: `HTTP ${response.status}` };
      }

      await updateItem(storageKey, id, { ragStatus: "indexed" });
      return { success: true };
    } catch (error) {
      await updateItem(storageKey, id, { ragStatus: "failed" });
      return { success: false, error: error.message };
    }
  },

  /**
   * Remove item from RAG index
   * @param {string} storageKey - Storage key
   * @param {string} id - Item ID
   * @returns {Promise<{success: boolean, error?: string}>}
   */
  async removeFromRagIndex(storageKey, id) {
    const item = await getById(storageKey, id);
    if (!item) {
      return { success: false, error: "Item not found" };
    }

    if (!item.hubFileId) {
      return { success: false, error: "Item not synced to Hub" };
    }

    const config = await this.getHubConfig();
    const headers = await this.getAuthHeaders();

    try {
      const response = await fetch(`${config.hubUrl}/api/library/files/${item.hubFileId}/unindex`, {
        method: "DELETE",
        headers: headers,
      });

      if (!response.ok) {
        return { success: false, error: `HTTP ${response.status}` };
      }

      await updateItem(storageKey, id, { ragStatus: "none" });
      return { success: true };
    } catch (error) {
      return { success: false, error: error.message };
    }
  },

  /**
   * Get sync statistics
   * @returns {Promise<Object>}
   */
  async getStats() {
    const highlights = await Highlights.getAll();
    const clips = await Clips.getAll();
    const notes = await Notes.getAll();
    const pages = await PageCaptures.getAll();

    const all = [...highlights, ...clips, ...notes, ...pages];

    return {
      total: all.length,
      local: all.filter((i) => i.syncStatus === "local").length,
      pending: all.filter((i) => i.syncStatus === "pending").length,
      synced: all.filter((i) => i.syncStatus === "synced").length,
      failed: all.filter((i) => i.syncStatus === "failed").length,
      ragIndexed: all.filter((i) => i.ragStatus === "indexed").length,
      highlights: highlights.length,
      clips: clips.length,
      notes: notes.length,
      pages: pages.length,
    };
  },
};

// ==================== SEARCH OPERATIONS ====================

const Search = {
  /**
   * Search across all local data
   * @param {string} query - Search query
   * @returns {Promise<Array>}
   */
  async searchAll(query) {
    const normalizedQuery = query.toLowerCase().trim();
    if (!normalizedQuery) return [];

    const results = [];

    // Search highlights
    const highlights = await Highlights.getAll();
    highlights.forEach((h) => {
      if (
        h.text.toLowerCase().includes(normalizedQuery) ||
        (h.note && h.note.toLowerCase().includes(normalizedQuery)) ||
        h.pageTitle.toLowerCase().includes(normalizedQuery)
      ) {
        results.push({ type: "highlight", item: h });
      }
    });

    // Search clips
    const clips = await Clips.getAll();
    clips.forEach((c) => {
      if (
        c.content.toLowerCase().includes(normalizedQuery) ||
        c.pageTitle.toLowerCase().includes(normalizedQuery)
      ) {
        results.push({ type: "clip", item: c });
      }
    });

    // Search notes
    const notes = await Notes.getAll();
    notes.forEach((n) => {
      if (
        n.content.toLowerCase().includes(normalizedQuery) ||
        (n.pageTitle && n.pageTitle.toLowerCase().includes(normalizedQuery)) ||
        (n.tags && n.tags.some((t) => t.toLowerCase().includes(normalizedQuery)))
      ) {
        results.push({ type: "note", item: n });
      }
    });

    // Search page captures
    const pages = await PageCaptures.getAll();
    pages.forEach((p) => {
      if (
        p.title.toLowerCase().includes(normalizedQuery) ||
        p.content.toLowerCase().includes(normalizedQuery) ||
        p.url.toLowerCase().includes(normalizedQuery)
      ) {
        results.push({ type: "page", item: p });
      }
    });

    // Sort by most recent
    results.sort((a, b) => new Date(b.item.createdAt) - new Date(a.item.createdAt));

    return results;
  },
};

// ==================== EXPORT MODULE ====================

// Make available globally in extension context
if (typeof window !== "undefined") {
  window.WhytCardStorage = {
    Highlights,
    Clips,
    Notes,
    PageCaptures,
    SyncManager,
    Search,
    STORAGE_KEYS,
    // Generic storage operations
    getById,
    updateItem,
    deleteItem,
  };
}

// Export for ES modules (if used)
if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    Highlights,
    Clips,
    Notes,
    PageCaptures,
    SyncManager,
    Search,
    STORAGE_KEYS,
  };
}
