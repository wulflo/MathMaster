// Service worker de MathMaster: shell y assets para funcionar offline.
// Este archivo es pegamento de plataforma (no lógica de producto): es JS estático.
//
// Estrategia: red primero, caché como respaldo, para todo.
// - Con red, una versión nueva se ve en la siguiente carga (sin wasm viejo en caché).
// - Sin red, se sirve lo precacheado; una navegación cae al shell `index.html`.
// El HTTP cache del navegador sigue evitando descargas repetidas cuando aplica.
const VERSION = "mathmaster-v11";
const ASSETS = [
  "./",
  "./index.html",
  "./anzan_app.js",
  "./anzan_app_bg.wasm",
  "./manifest.webmanifest",
  "./icon.svg",
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(VERSION)
      // `reload` ignora la caché HTTP: GitHub Pages sirve los assets con max-age y, si
      // no, el precache podría guardar un wasm viejo tras un deploy.
      .then((cache) =>
        cache.addAll(ASSETS.map((url) => new Request(url, { cache: "reload" }))),
      )
      .then(() => self.skipWaiting()),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys.filter((key) => key !== VERSION).map((key) => caches.delete(key)),
        ),
      )
      .then(() => self.clients.claim()),
  );
});

async function fromCache(request) {
  const cached = await caches.match(request);
  if (cached) return cached;
  if (request.mode === "navigate") {
    const shell = await caches.match("./index.html");
    if (shell) return shell;
  }
  return undefined;
}

self.addEventListener("fetch", (event) => {
  const { request } = event;
  if (request.method !== "GET") return;
  if (new URL(request.url).origin !== self.location.origin) return;

  // `no-cache` revalida con el origen (304 si no cambió): una versión nueva se ve sin
  // esperar al max-age de GitHub Pages.
  const network = new Request(request, { cache: "no-cache" });

  event.respondWith(
    fetch(network)
      .then((response) => {
        if (response.ok) {
          const copy = response.clone();
          caches.open(VERSION).then((cache) => cache.put(request, copy));
          return response;
        }
        // Sin red, algunos proxies responden 502 en vez de rechazar la conexión.
        if (response.status >= 500) {
          return fromCache(request).then((cached) => cached || response);
        }
        return response;
      })
      .catch(() =>
        fromCache(request).then((cached) => cached || Response.error()),
      ),
  );
});
