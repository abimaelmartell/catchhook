/**
 * Catchhook UI - Modern JavaScript Application
 * Consumes the Catchhook API to display webhook requests
 */

class CatchhookApp {
  constructor() {
    this.apiBase = window.location.origin;
    this.currentRequest = null;
    this.requests = [];
    this.refreshInterval = null;
    
    this.init();
  }

  /**
   * Initialize the application
   */
  async init() {
    this.setupElements();
    this.setupEventListeners();
    this.updateWebhookUrl();
    await this.loadRequests();
    this.startAutoRefresh();
  }

  /**
   * Cache DOM elements for better performance
   */
  setupElements() {
    this.elements = {
      webhookUrl: document.getElementById('webhookUrl'),
      copyButton: document.getElementById('copyButton'),
      webhookList: document.getElementById('webhookList'),
      detailPanel: document.getElementById('detailPanel'),
      refreshButton: document.getElementById('refreshButton'),
      autoRefreshIndicator: document.getElementById('autoRefreshIndicator')
    };
  }

  /**
   * Setup event listeners
   */
  setupEventListeners() {
    this.elements.copyButton.addEventListener('click', () => this.copyWebhookUrl());
    this.elements.refreshButton.addEventListener('click', () => this.manualRefresh());
    
    // Handle visibility change to pause/resume auto-refresh
    document.addEventListener('visibilitychange', () => {
      if (document.hidden) {
        this.pauseAutoRefresh();
      } else {
        this.resumeAutoRefresh();
        this.loadRequests(); // Refresh immediately when tab becomes visible
      }
    });

    // Handle keyboard shortcuts
    document.addEventListener('keydown', (e) => {
      if (e.key === 'r' && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        this.manualRefresh();
      }
    });
  }

  /**
   * Update the webhook URL display
   */
  updateWebhookUrl() {
    const webhookUrl = `${this.apiBase}/webhook`;
    this.elements.webhookUrl.textContent = webhookUrl;
  }

  /**
   * Copy webhook URL to clipboard
   */
  async copyWebhookUrl() {
    try {
      const url = this.elements.webhookUrl.textContent;
      
      // Try modern clipboard API first
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(url);
      } else {
        // Fallback for non-secure contexts or older browsers
        prompt('Copy this URL:', url);
      }
      
      // Visual feedback
      const button = this.elements.copyButton;
      const originalText = button.textContent;
      button.textContent = 'Copied!';
      button.classList.add('copied');
      
      setTimeout(() => {
        button.textContent = originalText;
        button.classList.remove('copied');
      }, 2000);
    } catch (err) {
      console.error('Failed to copy URL:', err);
      this.showNotification('Failed to copy URL', 'error');
    }
  }

  /**
   * Load requests from the API
   */
  async loadRequests() {
    try {
      const response = await fetch(`${this.apiBase}/latest`);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
      const data = await response.json();
      this.requests = data.items || [];
      this.renderRequestList();
      
      // If we have a current request selected, refresh its details
      if (this.currentRequest) {
        const updatedRequest = this.requests.find(r => r.id === this.currentRequest.id);
        if (updatedRequest) {
          this.currentRequest = updatedRequest;
        }
      }
    } catch (error) {
      console.error('Failed to load requests:', error);
      this.showNotification('Failed to load requests', 'error');
    }
  }

  /**
   * Render the request list in the sidebar
   */
  renderRequestList() {
    const listElement = this.elements.webhookList;
    
    if (this.requests.length === 0) {
      listElement.innerHTML = `
        <div class="empty-state" style="padding: 2rem; text-align: center; color: #666;">
          <div style="font-size: 2rem; margin-bottom: 1rem;">📭</div>
          <p>No webhook requests yet</p>
          <p style="font-size: 0.875rem; margin-top: 0.5rem;">Send a request to your webhook URL to get started</p>
        </div>
      `;
      return;
    }

    const requestElements = this.requests.map(request => {
      const isActive = this.currentRequest && this.currentRequest.id === request.id;
      const timeAgo = this.formatTimeAgo(request.ts_ms);
      const method = request.method.toUpperCase();
      
      return `
        <li class="webhook-item ${isActive ? 'active' : ''}" 
            data-request-id="${request.id}"
            onclick="app.selectRequest(${request.id})">
          <div class="webhook-item-header">
            <span class="method-badge method-${method.toLowerCase()}">${method}</span>
            <span class="webhook-path">${this.escapeHtml(request.path)}</span>
          </div>
          <div class="webhook-time">${timeAgo}</div>
        </li>
      `;
    }).join('');

    listElement.innerHTML = requestElements;
  }

  /**
   * Select and display a specific request
   */
  async selectRequest(requestId) {
    try {
      // Find request in current list first (for immediate feedback)
      let request = this.requests.find(r => r.id === requestId);
      
      if (!request) {
        // If not found, try to fetch it directly
        const response = await fetch(`${this.apiBase}/req/${requestId}`);
        if (!response.ok) {
          throw new Error(`Request not found: ${requestId}`);
        }
        request = await response.json();
      }

      this.currentRequest = request;
      this.renderRequestDetails(request);
      this.renderRequestList(); // Re-render to update active state
    } catch (error) {
      console.error('Failed to load request details:', error);
      this.showNotification('Failed to load request details', 'error');
    }
  }

  /**
   * Render request details in the detail panel
   */
  renderRequestDetails(request) {
    const detailPanel = this.elements.detailPanel;
    const timestamp = new Date(request.ts_ms).toLocaleString();
    const method = request.method.toUpperCase();
    
    // Parse body content
    const { bodyContent, bodyType } = this.parseBody(request.body);
    
    // Format headers
    const headersHtml = request.headers.length > 0 
      ? request.headers.map(([key, value]) => `
          <div class="key-value-item">
            <div class="key-value-key">${this.escapeHtml(key)}</div>
            <div class="key-value-value">${this.escapeHtml(value)}</div>
          </div>
        `).join('')
      : '<div style="padding: 1rem; text-align: center; color: #666;">No headers</div>';

    detailPanel.innerHTML = `
      <div class="detail-header">
        <div class="detail-title">
          <span class="method-badge method-${method.toLowerCase()}">${method}</span>
          <h2>${this.escapeHtml(request.path)}</h2>
        </div>
        <div class="detail-time">
          Request #${request.id} • ${timestamp}
        </div>
      </div>

      <div class="detail-section">
        <h3 class="section-title">Headers</h3>
        <div class="key-value-list">
          ${headersHtml}
        </div>
      </div>

      <div class="detail-section">
        <h3 class="section-title">Body ${bodyType ? `(${bodyType})` : ''}</h3>
        <div class="code-block">
          <pre>${bodyContent}</pre>
        </div>
      </div>

      <div class="detail-section">
        <h3 class="section-title">Raw Data</h3>
        <div class="code-block">
          <pre>${JSON.stringify(request, null, 2)}</pre>
        </div>
      </div>
    `;
  }

  /**
   * Parse request body and determine content type
   */
  parseBody(bodyArray) {
    if (!bodyArray || bodyArray.length === 0) {
      return { bodyContent: '(empty)', bodyType: null };
    }

    try {
      // Convert byte array to string
      const bodyString = new TextDecoder().decode(new Uint8Array(bodyArray));
      
      // Try to parse as JSON
      try {
        const jsonData = JSON.parse(bodyString);
        return {
          bodyContent: JSON.stringify(jsonData, null, 2),
          bodyType: 'JSON'
        };
      } catch {
        // Not JSON, return as text
        return {
          bodyContent: bodyString || '(empty)',
          bodyType: this.detectContentType(bodyString)
        };
      }
    } catch (error) {
      // If decoding fails, show raw bytes
      return {
        bodyContent: `Binary data (${bodyArray.length} bytes): [${bodyArray.slice(0, 20).join(', ')}${bodyArray.length > 20 ? '...' : ''}]`,
        bodyType: 'Binary'
      };
    }
  }

  /**
   * Detect content type from string content
   */
  detectContentType(content) {
    if (!content) return null;
    
    const trimmed = content.trim();
    if (trimmed.startsWith('<') && trimmed.endsWith('>')) {
      return 'XML/HTML';
    }
    if (trimmed.includes('=') && trimmed.includes('&')) {
      return 'Form Data';
    }
    return 'Text';
  }

  /**
   * Format timestamp as "time ago"
   */
  formatTimeAgo(timestamp) {
    const now = Date.now();
    const diff = now - timestamp;
    
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);
    
    if (seconds < 60) return `${seconds}s ago`;
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    return `${days}d ago`;
  }

  /**
   * Escape HTML to prevent XSS
   */
  escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  /**
   * Show notification to user
   */
  showNotification(message, type = 'info') {
    // Simple console notification for now
    // Could be enhanced with toast notifications
    console.log(`[${type.toUpperCase()}] ${message}`);
  }

  /**
   * Manual refresh triggered by user
   */
  async manualRefresh() {
    const button = this.elements.refreshButton;
    button.style.transform = 'rotate(180deg)';
    
    await this.loadRequests();
    
    setTimeout(() => {
      button.style.transform = '';
    }, 300);
  }

  /**
   * Start auto-refresh of requests
   */
  startAutoRefresh() {
    this.stopAutoRefresh(); // Clear any existing interval
    this.refreshInterval = setInterval(() => {
      this.loadRequests();
    }, 5000); // Refresh every 5 seconds
    this.updateRefreshIndicator(true);
  }

  /**
   * Stop auto-refresh
   */
  stopAutoRefresh() {
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval);
      this.refreshInterval = null;
    }
    this.updateRefreshIndicator(false);
  }

  /**
   * Pause auto-refresh (when tab hidden)
   */
  pauseAutoRefresh() {
    this.stopAutoRefresh();
  }

  /**
   * Resume auto-refresh (when tab visible)
   */
  resumeAutoRefresh() {
    this.startAutoRefresh();
  }

  /**
   * Update the auto-refresh indicator
   */
  updateRefreshIndicator(isActive) {
    const indicator = this.elements.autoRefreshIndicator;
    if (isActive) {
      indicator.classList.remove('paused');
      indicator.title = 'Auto-refresh active (every 5s)';
    } else {
      indicator.classList.add('paused');
      indicator.title = 'Auto-refresh paused';
    }
  }
}

// Initialize the application when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
  window.app = new CatchhookApp();
});

// Handle page unload to clean up intervals
window.addEventListener('beforeunload', () => {
  if (window.app) {
    window.app.stopAutoRefresh();
  }
});
