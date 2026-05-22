// Un Service Worker básico para cumplir con el requisito de instalación PWA
self.addEventListener("install", (event) => {
  console.log("[CubePWA] Service Worker Instalado");
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  console.log("[CubePWA] Service Worker Activado");
  return self.clients.claim();
});

self.addEventListener("fetch", (event) => {
  // Modo "Network Only": dejamos que HTMX y Axum manejen todo en tiempo real
  event.respondWith(fetch(event.request));
});
