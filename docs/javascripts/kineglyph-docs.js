let controllers = [];
let visibilityCleanups = [];
let themeObserver;
let generation = 0;

function activeKineglyphTheme(runtime) {
  const scheme = document.body.getAttribute("data-md-color-scheme");
  return runtime.themes[scheme === "vellum" ? "nucleation-light" : "nucleation-dark"];
}

function applyKineglyphTheme(runtime) {
  const theme = activeKineglyphTheme(runtime);
  for (const controller of controllers) controller.setTheme(theme);
}

async function mountKineglyphPage() {
  generation += 1;
  const current = generation;

  for (const cleanup of visibilityCleanups) cleanup();
  visibilityCleanups = [];
  themeObserver?.disconnect();
  for (const controller of controllers) controller.destroy();
  controllers = [];

  const runtime = await import("./vendor/kineglyph-web.js");
  if (current !== generation) return;

  controllers = runtime.autoMount();
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
