// Vibespeak Service Worker for PWA functionality
// Handles caching, offline functionality, and background tasks

const CACHE_NAME = 'vibespeak-v1.0.0';
const STATIC_CACHE = 'vibespeak-static-v1.0.0';
const DYNAMIC_CACHE = 'vibespeak-dynamic-v1.0.0';

// Files to cache for offline use
const STATIC_FILES = [
    '/',
    '/manifest.json',
    '/static/app.js',
    '/static/styles.css',
    // Add icon files when available
    // '/icon-192.png',
    // '/icon-512.png'
];

// Install event - cache static resources
self.addEventListener('install', event => {
    console.log('[SW] Installing service worker');
    event.waitUntil(
        caches.open(STATIC_CACHE)
            .then(cache => {
                console.log('[SW] Caching static files');
                return cache.addAll(STATIC_FILES);
            })
            .catch(error => {
                console.error('[SW] Failed to cache static files:', error);
            })
    );
    // Force activation of new service worker
    self.skipWaiting();
});

// Activate event - clean up old caches
self.addEventListener('activate', event => {
    console.log('[SW] Activating service worker');
    event.waitUntil(
        caches.keys()
            .then(cacheNames => {
                return Promise.all(
                    cacheNames.map(cacheName => {
                        if (cacheName !== STATIC_CACHE && cacheName !== DYNAMIC_CACHE) {
                            console.log('[SW] Deleting old cache:', cacheName);
                            return caches.delete(cacheName);
                        }
                    })
                );
            })
            .then(() => {
                // Take control of all clients
                return self.clients.claim();
            })
    );
});

// Fetch event - serve cached content when offline
self.addEventListener('fetch', event => {
    const { request } = event;
    const url = new URL(request.url);

    // Skip non-GET requests and external requests
    if (request.method !== 'GET' || !url.pathname.startsWith('/')) {
        return;
    }

    // Handle API requests differently
    if (url.pathname.startsWith('/api/')) {
        event.respondWith(
            fetch(request)
                .catch(() => {
                    // Return offline response for API calls
                    return new Response(
                        JSON.stringify({
                            error: 'Offline',
                            message: 'This feature requires an internet connection'
                        }),
                        {
                            status: 503,
                            headers: { 'Content-Type': 'application/json' }
                        }
                    );
                })
        );
        return;
    }

    // Cache-first strategy for static assets
    event.respondWith(
        caches.match(request)
            .then(response => {
                if (response) {
                    return response;
                }

                // Network-first for dynamic content
                return fetch(request)
                    .then(response => {
                        // Cache successful responses
                        if (response.status === 200) {
                            const responseClone = response.clone();
                            caches.open(DYNAMIC_CACHE)
                                .then(cache => {
                                    cache.put(request, responseClone);
                                });
                        }
                        return response;
                    })
                    .catch(() => {
                        // Return offline fallback for navigation requests
                        if (request.mode === 'navigate') {
                            return caches.match('/');
                        }

                        // Return error for other requests
                        return new Response(
                            'Offline - Content not available',
                            { status: 503 }
                        );
                    });
            })
    );
});

// Background sync for offline commands
self.addEventListener('sync', event => {
    if (event.tag === 'background-sync-commands') {
        event.waitUntil(syncPendingCommands());
    }
});

async function syncPendingCommands() {
    try {
        const cache = await caches.open('pending-commands');
        const keys = await cache.keys();

        for (const request of keys) {
            try {
                await fetch(request);
                await cache.delete(request);
                console.log('[SW] Synced pending command');
            } catch (error) {
                console.error('[SW] Failed to sync command:', error);
            }
        }
    } catch (error) {
        console.error('[SW] Background sync failed:', error);
    }
}

// Push notifications (for future use)
self.addEventListener('push', event => {
    if (!event.data) return;

    const data = event.data.json();
    const options = {
        body: data.body,
        icon: 'icon-192.png',
        badge: 'icon-192.png',
        vibrate: [100, 50, 100],
        data: data.url,
        requireInteraction: true,
        actions: [
            {
                action: 'view',
                title: 'View'
            },
            {
                action: 'dismiss',
                title: 'Dismiss'
            }
        ]
    };

    event.waitUntil(
        self.registration.showNotification(data.title, options)
    );
});

// Handle notification clicks
self.addEventListener('notificationclick', event => {
    event.notification.close();

    if (event.action === 'view') {
        const url = event.notification.data;
        event.waitUntil(
            clients.openWindow(url || '/')
        );
    }
});

// Message handling from main thread
self.addEventListener('message', event => {
    const { type, data } = event.data;

    switch (type) {
        case 'SKIP_WAITING':
            self.skipWaiting();
            break;

        case 'CACHE_COMMAND':
            // Cache command for offline execution
            cacheCommandForOffline(data);
            break;

        default:
            console.log('[SW] Unknown message type:', type);
    }
});

async function cacheCommandForOffline(commandData) {
    try {
        const cache = await caches.open('pending-commands');
        const request = new Request(`/api/remote/command/offline/${Date.now()}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(commandData)
        });

        await cache.put(request, new Response(JSON.stringify({ cached: true })));
        console.log('[SW] Command cached for offline execution');
    } catch (error) {
        console.error('[SW] Failed to cache command:', error);
    }
}

// Periodic cleanup (run every hour)
setInterval(async () => {
    try {
        const cache = await caches.open(DYNAMIC_CACHE);
        const keys = await cache.keys();

        // Remove old entries (older than 1 day)
        const oneDayAgo = Date.now() - (24 * 60 * 60 * 1000);

        for (const request of keys) {
            const response = await cache.match(request);
            if (response) {
                const date = response.headers.get('date');
                if (date && new Date(date).getTime() < oneDayAgo) {
                    await cache.delete(request);
                }
            }
        }
    } catch (error) {
        console.error('[SW] Cache cleanup failed:', error);
    }
}, 60 * 60 * 1000); // Run every hour