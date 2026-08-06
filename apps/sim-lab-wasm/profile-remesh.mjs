/** What does a tick actually cost the lab, and how much of it is meshing?
 *
 * Measures the real loop in situ — step, applyChanges, flush — rather than a
 * reimplementation of it. An earlier version of this probe timed the stages of
 * one re-mesh in a loop of its own, leaked wasm objects doing it, and reported
 * numbers that disagreed with the running app. If this ever disagrees with the
 * tps readout again, trust the app.
 *
 * Usage: node profile-remesh.mjs <schematic>
 */
