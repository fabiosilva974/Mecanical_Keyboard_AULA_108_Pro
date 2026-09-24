/**
 * bridge.js - Asynchronous IPC Bridge between WebKit frontend and Python host.
 *
 * Provides a modern Promise-based API for communicating with app.py.
 * Handles request-response correlation and event streaming.
 */

class HostBridge {
    constructor() {
        this.reqCounter = 0;
        this.pendingRequests = new Map();
        this.eventListeners = new Map();

        // Setup global callbacks invoked by Python evaluate_javascript
        window.__onApiResponse = (response) => {
            const { id, success, data, error } = response;
            if (this.pendingRequests.has(id)) {
                const { resolve, reject } = this.pendingRequests.get(id);
                this.pendingRequests.delete(id);
                if (success) {
                    resolve(data);
                } else {
                    reject(new Error(error || "Unknown IPC error"));
                }
            }
        };

        window.__onApiEvent = (eventName, data) => {
            if (this.eventListeners.has(eventName)) {
                this.eventListeners.get(eventName).forEach(cb => {
                    try { cb(data); } catch (e) { console.error(`Event callback error (${eventName}):`, e); }
                });
            }
        };
    }

    /**
     * Invokes an asynchronous action on the Python host.
     * @param {string} action - Action name (e.g. 'execute_cli', 'get_device_status')
     * @param {object} payload - Optional arguments object
     * @returns {Promise<any>}
     */
    invoke(action, payload = {}) {
        return new Promise((resolve, reject) => {
            const reqId = `req_${++this.reqCounter}_${Date.now()}`;
            this.pendingRequests.set(reqId, { resolve, reject });

            try {
                if (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.api) {
                    window.webkit.messageHandlers.api.postMessage({
                        id: reqId,
                        action: action,
                        payload: payload
                    });
                } else {
                    // Running in standard browser outside WebKitGTK container (mock mode)
                    console.warn(`[Mock IPC] ${action}`, payload);
                    setTimeout(() => {
                        this.pendingRequests.delete(reqId);
                        resolve({ mock: true, action, payload });
                    }, 50);
                }
            } catch (err) {
                this.pendingRequests.delete(reqId);
                reject(err);
            }
        });
    }

    /**
     * Registers a listener for events pushed by the host.
     * @param {string} eventName
     * @param {function} callback
     */
    on(eventName, callback) {
        if (!this.eventListeners.has(eventName)) {
            this.eventListeners.set(eventName, []);
        }
        this.eventListeners.get(eventName).push(callback);
    }

    /**
     * Removes an event listener.
     */
    off(eventName, callback) {
        if (this.eventListeners.has(eventName)) {
            const list = this.eventListeners.get(eventName).filter(cb => cb !== callback);
            this.eventListeners.set(eventName, list);
        }
    }
}

// Global singleton instance
window.hostBridge = new HostBridge();
