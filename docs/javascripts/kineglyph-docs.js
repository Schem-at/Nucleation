let controllers = [];
let visibilityCleanups = [];
let generation = 0;

async function mountKineglyphPage() {
  generation += 1;
  const current = generation;

  for (const cleanup of visibilityCleanups) cleanup();
  visibilityCleanups = [];
  for (const controller of controllers) controller.destroy();
  controllers = [];

  const runtime = await import("./vendor/kineglyph-web.js");
  if (current !== generation) return;

  controllers = runtime.autoMount();
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
