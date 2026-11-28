/**
 * Standalone bootstrap for Nucleation's interactive SDF, CSG, and WorldGenerator
 * browser playgrounds + Basalt/Vellum adaptive diagram syncing.
 *
 * Resolves module URLs from `document.currentScript.src` so dynamic imports
 * always point to `/javascripts/sdf-playground.js` regardless of page depth or
 * GitHub Pages base path.
 */
(function () {
  const scriptEl = document.currentScript;
  const baseUrl = scriptEl ? new URL("./", scriptEl.src) : new URL("./javascripts/", window.location.href);
  const playgroundModuleUrl = new URL("sdf-playground.js", baseUrl).href;

  let activeHandle = null;
  let themeObserver = null;
  let mountGen = 0;
  let modelViewerPromise = null;

  function ensureModelViewer() {
    if (customElements.get("model-viewer") !== undefined) return Promise.resolve(true);
    modelViewerPromise ??= import(
      "https://ajax.googleapis.com/ajax/libs/model-viewer/4.3.1/model-viewer.min.js"
    )
      .then(() => customElements.whenDefined("model-viewer"))
      .then(() => true)
      .catch((error) => {
        console.warn("Could not load <model-viewer>:", error);
        return false;
      });
    return modelViewerPromise;
  }

  function syncAdaptiveDiagrams() {
    const scheme = document.body.getAttribute("data-md-color-scheme");
    const isLight = scheme === "vellum";
    for (const fig of document.querySelectorAll("[data-adaptive-diagram]")) {
      const darkSrc = fig.dataset.dark;
      const lightSrc = fig.dataset.light;
      if (!darkSrc || !lightSrc) continue;
      const desired = isLight ? lightSrc : darkSrc;
      const source = fig.querySelector("picture > source");
      const img = fig.querySelector("picture > img");
      if (source) {
        source.media = isLight ? "not all" : "all";
        source.srcset = darkSrc;
      }
      if (img && img.getAttribute("src") !== desired) {
        img.setAttribute("src", desired);
      }
    }
  }

  async function mountAll() {
    mountGen += 1;
    const gen = mountGen;

    themeObserver?.disconnect();
    activeHandle?.destroy?.();
    activeHandle = null;

    syncAdaptiveDiagrams();

    const hasInteractive =
      document.querySelector("[data-sdf-workbench]") !== null ||
      document.querySelector("[data-sdf-csg-lab]") !== null ||
      document.querySelector("[data-worldgen-lab]") !== null;

    if (hasInteractive) {
      try {
        const mod = await import(playgroundModuleUrl);
        if (gen !== mountGen) return;
        activeHandle = await mod.mountSdfPlaygrounds(ensureModelViewer);
        if (gen !== mountGen) {
          activeHandle?.destroy?.();
          activeHandle = null;
          return;
        }
      } catch (error) {
        console.warn("Could not initialize Nucleation interactive studio:", error);
      }
    }

    themeObserver = new MutationObserver(() => {
      syncAdaptiveDiagrams();
      activeHandle?.onThemeChange?.();
    });
    themeObserver.observe(document.body, {
      attributes: true,
      attributeFilter: ["data-md-color-scheme"],
    });
  }

  if (typeof document$ === "undefined") {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", () => void mountAll(), { once: true });
    } else {
      void mountAll();
    }
  } else {
    document$.subscribe(() => void mountAll());
  }
})();
