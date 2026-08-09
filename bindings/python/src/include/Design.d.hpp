#ifndef NUCLEATION_Design_D_HPP
#define NUCLEATION_Design_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"
namespace nucleation {
namespace capi { struct Design; }
class Design;
namespace capi { struct Schematic; }
class Schematic;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Design;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A composition document: loose blocks + cell instances + bus layers
 * over a shared coordinate space.
 */
class Design {
public:

  /**
   * An empty design.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError> create(std::string_view name);

  /**
   * A design whose loose block layer is a copy of `base` (endpoint
   * hardware placed with raw `set_block`).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError> for_schematic(std::string_view name, const nucleation::Schematic& base);

  /**
   * Register a library cell; its contract is resolved from the
   * schematic (embedded metadata first, Insign signs as fallback)
   * and registration fails loudly when no source defines one.
   * Writes resolution warnings as a JSON array of strings.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> add_cell(std::string_view name, const nucleation::Schematic& cell);
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_cell_write(std::string_view name, const nucleation::Schematic& cell, W& writeable_output);

  /**
   * Place an instance layer referencing a library cell. `rot_y` is
   * in degrees, a multiple of 90.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> place(std::string_view name, std::string_view cell, int32_t x, int32_t y, int32_t z, int32_t rot_y);

  /**
   * Declare a drivable input port: anchor = bit-0 connection cell,
   * step to the next bit, `width` bits of `ty` (`"uint"` or
   * `"bool"`). The hardware is scanned (adjacent lever per bit) and
   * validated loudly.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> declare_input(std::string_view name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, std::string_view ty);

  /**
   * Declare a readable output port (adjacent lamp per bit); same
   * shape as `declare_input`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> declare_output(std::string_view name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, std::string_view ty);

  /**
   * Declare AND realize a bus. `sinks_json` is a JSON array of port
   * names; `gates_json` an array of `{"name", "anchor": [x,y,z],
   * "step": [x,y,z]}` (pass `[]` for none); `style_json` an object
   * with optional `bus_block` / `transparent_block`. Declaration
   * errors are error returns; geometric unroutability is the
   * written STATE: `"routed"` or `"failed: reason"` — realization
   * is atomic, never half-routed.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> route_bus(std::string_view name, std::string_view driver, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json);
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> route_bus_write(std::string_view name, std::string_view driver, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json, W& writeable_output);

  /**
   * Declare AND realize a wired-OR bus: `drivers_json` is a JSON
   * array of port names — multiple drivers are legal ONLY through
   * this explicit merge (`merge="or"`). Extra drivers join the
   * trunk as diode-isolated dust-merge branches; the LVS intent
   * stays ONE net per bit. Same shapes as `route_bus` otherwise.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> route_bus_or(std::string_view name, std::string_view drivers_json, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json);
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> route_bus_or_write(std::string_view name, std::string_view drivers_json, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json, W& writeable_output);

  /**
   * Edit the loose block layer: plain `set_block` on the base
   * schematic (participates in occupancy and flatten).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_block(int32_t x, int32_t y, int32_t z, std::string_view block);

  /**
   * Drag an instance layer to a new position/rotation. The move
   * itself ALWAYS succeeds (the document's truth); the affected bus
   * set — fragments intersecting the old or new footprint +
   * influence halo, plus every already-failed bus — is ripped and
   * co-rerouted deterministically with bounded retry rounds.
   * Writes `{"rerouted": [...], "failed": {name: reason}}`.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> move_instance(std::string_view name, int32_t x, int32_t y, int32_t z, int32_t rot_y);
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> move_instance_write(std::string_view name, int32_t x, int32_t y, int32_t z, int32_t rot_y, W& writeable_output);

  /**
   * Remove an instance layer. Buses that terminate on one of its
   * ports are DELETED (they lost an endpoint); buses that merely
   * crossed its space are ripped and co-rerouted. Writes
   * `{"removed_buses": [...], "rerouted": [...], "failed": {...}}`.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> remove_instance(std::string_view name);
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> remove_instance_write(std::string_view name, W& writeable_output);

  /**
   * Re-realize a bus from its stored declaration (the counterpart to
   * `rip`); writes the resulting bus state.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> reroute(std::string_view name);
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> reroute_write(std::string_view name, W& writeable_output);

  /**
   * Delete a bus outright — fragment AND declaration, freeing the
   * name. `rip` keeps the declaration so the bus can be rerouted.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> remove_bus(std::string_view name);

  /**
   * The flattened artifact as `.schem` bytes, base64. Unlike
   * `flatten()` + the schematic writer, this composites the layer
   * stack into ONE region first: `.schem` has no layers, and the
   * region merge drops named-layer cells that the loose layer's
   * bounding box shadows.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_schem_b64() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_schem_b64_write(W& writeable_output) const;

  /**
   * The flattened artifact composited into ONE region (see
   * `to_schem_b64`) — the shape an interchange export wants.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> flatten_composite() const;

  /**
   * Every routing endpoint the placed instances expose, as a JSON
   * array of `{name, instance, port, role, ty, width, hardware,
   * wires, step, routable, blocked}`. `name` is `{instance}.{port}`
   * — exactly what `route_bus` accepts; `role` is the CELL-facing
   * direction, so `"output"` drives a bus and `"input"` receives
   * one. A port whose bits have no dust connection cell (a lever
   * input, say) reports `routable: false` and why in `blocked`.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> instance_ports() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> instance_ports_write(W& writeable_output) const;

  /**
   * Resolve one routing endpoint name — a declared design port or an
   * instance port `{instance}.{port}` — to the geometry a bus would
   * use: `{"name","anchor","step","width","direction","connectable"}`.
   * `direction` is DESIGN-facing (`"input"` drives buses).
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> resolve_port(std::string_view name) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> resolve_port_write(std::string_view name, W& writeable_output) const;

  /**
   * Add a gate to an existing bus (splitting the segment it lands
   * in) and re-realize it. Writes the resulting bus state.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> add_gate(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z, int32_t sx, int32_t sy, int32_t sz);
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_gate_write(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z, int32_t sx, int32_t sy, int32_t sz, W& writeable_output);

  /**
   * Drag a gate: the anchor moves unconditionally, then EXACTLY the
   * two adjacent segments are ripped and rerouted atomically. An
   * unroutable move leaves the bus `failed: reason` — visible,
   * never half-routed. Writes `{"state": "...",
   * "rerouted_segments": n}`.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> move_gate(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z);
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> move_gate_write(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z, W& writeable_output);

  /**
   * Attach a net-class discipline to a bus (JSON `NetClassRule`:
   * optional `max_len_rt` delay budget, `y_band` layer band, …);
   * `check()` enforces it.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_bus_rule(std::string_view bus, std::string_view rule_json);

  /**
   * Per-bus skew from the routed fragment: writes
   * `{"per_bit_rt": [...], "skew_rt": n, "max_rt": n}`.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> bus_skew(std::string_view name) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> bus_skew_write(std::string_view name, W& writeable_output) const;

  /**
   * The lifecycle state of a bus: `"intended"`, `"routed"` or
   * `"failed: reason"`.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> bus_state(std::string_view name) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> bus_state_write(std::string_view name, W& writeable_output) const;

  /**
   * Rip a bus: clear its fragment, back to `intended`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rip(std::string_view name);

  /**
   * Collapse the layer stack into ONE self-describing schematic:
   * named regions per layer (`inst:x`, `bus:y`) and the merged
   * contract embedded in the metadata — itself placeable as a cell.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> flatten() const;

  /**
   * DRC + LVS over the flattened artifact. Writes `{"clean",
   * "drc": [...], "lvs": {...}, "buses": {...}}`.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> check() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> check_write(W& writeable_output) const;

  /**
   * Settle the flattened artifact in the vanilla-accurate tick
   * engine and return it with every settled state written back and
   * `InitialState::Baked` stamped into the embedded contract (needs
   * the `mc-tick` feature, else errors).
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> bake(uint32_t budget) const;

  /**
   * Serialize the FULL design document to `.nucm` project-tier
   * bytes (magic `NUCM`): cells deduped by content hash, instance
   * transforms, ports with scanned hardware, every bus layer with
   * its fragment, runs and `intended`/`routed`/`failed: reason`
   * state, and the loose base layer. Base64 across the bridge.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_nucm_b64() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_nucm_b64_write(W& writeable_output) const;

  /**
   * Reopen a `.nucm` design document from raw bytes. The reloaded
   * design is the same model mid-edit: rerouting works.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError> from_nucm(nucleation::diplomat::span<const uint8_t> data);

  /**
   * Save the `.nucm` project document to a file. Not available in
   * JS: the WASM build has no filesystem — use `to_nucm_b64`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> save_nucm(std::string_view path) const;

  /**
   * Load a `.nucm` project document from a file. Not available in
   * JS — read the bytes yourself and use `from_nucm`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError> load_nucm(std::string_view path);

  /**
   * Export the design as a LAYERED `.litematic` (interchange tier):
   * one named region per layer (`inst:{name}`, `bus:{name}`, loose
   * base) plus the design manifest as a root-level
   * `NucleationDesign` tag. Opens in Litematica as a plain
   * multi-region litematic; reimports as a design whose cell
   * references have degraded to embedded copies. Base64 across the
   * bridge.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_litematic_b64() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_litematic_b64_write(W& writeable_output) const;

  /**
   * Import a layered `.litematic` (with a `NucleationDesign`
   * manifest) from raw bytes; a plain litematic errors loudly —
   * open those with `Schematic.from_litematic`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError> from_litematic(nucleation::diplomat::span<const uint8_t> data);

  /**
   * Export the layered `.litematic` to a file. Not available in JS
   * — use `to_litematic_b64`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> export_litematic(std::string_view path) const;

  /**
   * Import a layered `.litematic` from a file. Not available in JS
   * — read the bytes yourself and use `from_litematic`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError> import_litematic(std::string_view path);

    inline const nucleation::capi::Design* AsFFI() const;
    inline nucleation::capi::Design* AsFFI();
    inline static const nucleation::Design* FromFFI(const nucleation::capi::Design* ptr);
    inline static nucleation::Design* FromFFI(nucleation::capi::Design* ptr);
    inline static void operator delete(void* ptr);
private:
    Design() = delete;
    Design(const nucleation::Design&) = delete;
    Design(nucleation::Design&&) noexcept = delete;
    Design operator=(const nucleation::Design&) = delete;
    Design operator=(nucleation::Design&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Design_D_HPP
