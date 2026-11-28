let controllers = [];
let visibilityCleanups = [];
let playgroundHandle;
let themeObserver;
let generation = 0;
let modelViewerPromise;

function ensureModelViewer() {
  if (customElements.get("model-viewer") !== undefined) return Promise.resolve(true);
  modelViewerPromise ??= import(
    "https://ajax.googleapis.com/ajax/libs/model-viewer/4.3.1/model-viewer.min.js"
  )
    .then(() => customElements.whenDefined("model-viewer"))
    .then(() => true)
    .catch((error) => {
      console.warn("The live schematic viewer could not be loaded; keeping the static fallback.", error);
      return false;
    });
  return modelViewerPromise;
}

async function loadSdfSurface(runtime) {
  if (document.querySelector('[data-kineglyph="sdf-and-fields"]') === null) return undefined;
  try {
    const [viewerReady, adapter] = await Promise.all([
      ensureModelViewer(),
      import("./nucleation-sdf-live.js"),
    ]);
    return viewerReady ? adapter.nucleationSdfSurface(runtime) : undefined;
  } catch (error) {
    console.warn("Nucleation WASM could not start; keeping the static SDF illustration.", error);
    return undefined;
  }
}

async function mountPlaygroundsIfPresent() {
  if (
    document.querySelector("[data-sdf-workbench]") === null &&
    document.querySelector("[data-sdf-csg-lab]") === null
  ) {
    return undefined;
  }
  try {
    const mod = await import("./sdf-playground.js");
    return await mod.mountSdfPlaygrounds(ensureModelViewer);
  } catch (error) {
    console.warn("Could not initialize the interactive SDF playground.", error);
    return undefined;
  }
}

function activeKineglyphTheme(runtime) {
  const scheme = document.body.getAttribute("data-md-color-scheme");
  return runtime.themes[scheme === "vellum" ? "nucleation-light" : "nucleation-dark"];
}

function applyKineglyphTheme(runtime) {
  const theme = activeKineglyphTheme(runtime);
  for (const controller of controllers) controller.setTheme(theme);
  playgroundHandle?.onThemeChange?.();
}

async function mountKineglyphPage() {
  generation += 1;
  const current = generation;

  for (const cleanup of visibilityCleanups) cleanup();
  visibilityCleanups = [];
  themeObserver?.disconnect();
  playgroundHandle?.destroy?.();
  playgroundHandle = undefined;
  for (const controller of controllers) controller.destroy();
  controllers = [];

  const [runtime, nextPlaygrounds] = await Promise.all([
    import("./vendor/kineglyph-web.js"),
    mountPlaygroundsIfPresent(),
  ]);
  if (current !== generation) {
    nextPlaygrounds?.destroy?.();
    return;
  }
  playgroundHandle = nextPlaygrounds;

  const sdfSurface = await loadSdfSurface(runtime);
  if (current !== generation) return;

  controllers = runtime.autoMount({
    mountOptions: (element, sceneId) => {
      if (sceneId !== "sdf-and-fields" || sdfSurface === undefined) return {};
      return {
        liveSurfaces: { "sdf-live-build": sdfSurface },
        onSurfaceError: (_nodeId, error) => {
          element.dataset.nucleationLiveError = "true";
          console.warn("The live SDF build failed; keeping the static fallback.", error);
        },
      };
    },
  });
  applyKineglyphTheme(runtime);
  themeObserver = new MutationObserver(() => applyKineglyphTheme(runtime));
  themeObserver.observe(document.body, {
    attributes: true,
    attributeFilter: ["data-md-color-scheme"],
  });
  for (const controller of controllers) {
    visibilityCleanups.push(
      runtime.startWhenVisible(controller.element, () => controller.restart(true), {
        threshold: 0.08,
        rootMargin: "0px 0px -12% 0px",
        fallbackImmediately: true,
      }),
    );
  }
}

if (typeof document$ === "undefined") {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mountKineglyphPage, { once: true });
  } else {
    void mountKineglyphPage();
  }
} else {
  document$.subscribe(() => void mountKineglyphPage());
}
