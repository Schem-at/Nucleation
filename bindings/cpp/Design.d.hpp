#ifndef Design_D_HPP
#define Design_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Schematic; }
class Schematic;
class NucleationError;




namespace diplomat {
namespace capi {
    struct Design;
} // namespace capi
} // namespace

/**
 * A composition document: loose blocks + cell instances + bus layers
 * over a shared coordinate space.
 */
class Design {
public:

  /**
   * An empty design.
   */
  inline static diplomat::result<std::unique_ptr<Design>, NucleationError> create(std::string_view name);

  /**
   * A design whose loose block layer is a copy of `base` (endpoint
   * hardware placed with raw `set_block`).
   */
  inline static diplomat::result<std::unique_ptr<Design>, NucleationError> for_schematic(std::string_view name, const Schematic& base);

  /**
   * Register a library cell; its contract is resolved from the
   * schematic (embedded metadata first, Insign signs as fallback)
   * and registration fails loudly when no source defines one.
   * Writes resolution warnings as a JSON array of strings.
   */
  inline diplomat::result<std::string, NucleationError> add_cell(std::string_view name, const Schematic& cell);
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> add_cell_write(std::string_view name, const Schematic& cell, W& writeable_output);

  /**
   * Place an instance layer referencing a library cell. `rot_y` is
   * in degrees, a multiple of 90.
   */
  inline diplomat::result<std::monostate, NucleationError> place(std::string_view name, std::string_view cell, int32_t x, int32_t y, int32_t z, int32_t rot_y);

  /**
   * Declare a drivable input port: anchor = bit-0 connection cell,
   * step to the next bit, `width` bits of `ty` (`"uint"` or
   * `"bool"`). The hardware is scanned (adjacent lever per bit) and
   * validated loudly.
   */
  inline diplomat::result<std::monostate, NucleationError> declare_input(std::string_view name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, std::string_view ty);

  /**
   * Declare a readable output port (adjacent lamp per bit); same
   * shape as `declare_input`.
   */
  inline diplomat::result<std::monostate, NucleationError> declare_output(std::string_view name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, std::string_view ty);

  /**
   * Declare AND realize a bus. `sinks_json` is a JSON array of port
   * names; `gates_json` an array of `{"name", "anchor": [x,y,z],
   * "step": [x,y,z]}` (pass `[]` for none); `style_json` an object
   * with optional `bus_block` / `transparent_block`. Declaration
   * errors are error returns; geometric unroutability is the
   * written STATE: `"routed"` or `"failed: reason"` — realization
   * is atomic, never half-routed.
   */
  inline diplomat::result<std::string, NucleationError> route_bus(std::string_view name, std::string_view driver, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json);
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> route_bus_write(std::string_view name, std::string_view driver, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json, W& writeable_output);

  /**
   * Declare AND realize a wired-OR bus: `drivers_json` is a JSON
   * array of port names — multiple drivers are legal ONLY through
   * this explicit merge (`merge="or"`). Extra drivers join the
   * trunk as diode-isolated dust-merge branches; the LVS intent
   * stays ONE net per bit. Same shapes as `route_bus` otherwise.
   */
  inline diplomat::result<std::string, NucleationError> route_bus_or(std::string_view name, std::string_view drivers_json, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json);
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> route_bus_or_write(std::string_view name, std::string_view drivers_json, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json, W& writeable_output);

  /**
   * Edit the loose block layer: plain `set_block` on the base
   * schematic (participates in occupancy and flatten).
   */
  inline diplomat::result<std::monostate, NucleationError> set_block(int32_t x, int32_t y, int32_t z, std::string_view block);

  /**
   * Drag an instance layer to a new position/rotation. The move
   * itself ALWAYS succeeds (the document's truth); the affected bus
   * set — fragments intersecting the old or new footprint +
   * influence halo, plus every already-failed bus — is ripped and
   * co-rerouted deterministically with bounded retry rounds.
   * Writes `{"rerouted": [...], "failed": {name: reason}}`.
   */
  inline diplomat::result<std::string, NucleationError> move_instance(std::string_view name, int32_t x, int32_t y, int32_t z, int32_t rot_y);
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> move_instance_write(std::string_view name, int32_t x, int32_t y, int32_t z, int32_t rot_y, W& writeable_output);

  /**
   * Add a gate to an existing bus (splitting the segment it lands
   * in) and re-realize it. Writes the resulting bus state.
   */
  inline diplomat::result<std::string, NucleationError> add_gate(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z, int32_t sx, int32_t sy, int32_t sz);
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> add_gate_write(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z, int32_t sx, int32_t sy, int32_t sz, W& writeable_output);

  /**
   * Drag a gate: the anchor moves unconditionally, then EXACTLY the
   * two adjacent segments are ripped and rerouted atomically. An
   * unroutable move leaves the bus `failed: reason` — visible,
   * never half-routed. Writes `{"state": "...",
   * "rerouted_segments": n}`.
   */
  inline diplomat::result<std::string, NucleationError> move_gate(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z);
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> move_gate_write(std::string_view bus, std::string_view gate, int32_t x, int32_t y, int32_t z, W& writeable_output);

  /**
   * Attach a net-class discipline to a bus (JSON `NetClassRule`:
   * optional `max_len_rt` delay budget, `y_band` layer band, …);
   * `check()` enforces it.
   */
  inline diplomat::result<std::monostate, NucleationError> set_bus_rule(std::string_view bus, std::string_view rule_json);

  /**
   * Per-bus skew from the routed fragment: writes
   * `{"per_bit_rt": [...], "skew_rt": n, "max_rt": n}`.
   */
  inline diplomat::result<std::string, NucleationError> bus_skew(std::string_view name) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> bus_skew_write(std::string_view name, W& writeable_output) const;

  /**
   * The lifecycle state of a bus: `"intended"`, `"routed"` or
   * `"failed: reason"`.
   */
  inline diplomat::result<std::string, NucleationError> bus_state(std::string_view name) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> bus_state_write(std::string_view name, W& writeable_output) const;

  /**
   * Rip a bus: clear its fragment, back to `intended`.
   */
  inline diplomat::result<std::monostate, NucleationError> rip(std::string_view name);

  /**
   * Collapse the layer stack into ONE self-describing schematic:
   * named regions per layer (`inst:x`, `bus:y`) and the merged
   * contract embedded in the metadata — itself placeable as a cell.
   */
  inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> flatten() const;

  /**
   * DRC + LVS over the flattened artifact. Writes `{"clean",
   * "drc": [...], "lvs": {...}, "buses": {...}}`.
   */
  inline diplomat::result<std::string, NucleationError> check() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> check_write(W& writeable_output) const;

  /**
   * Settle the flattened artifact in the vanilla-accurate tick
   * engine and return it with every settled state written back and
   * `InitialState::Baked` stamped into the embedded contract (needs
   * the `mc-tick` feature, else errors).
   */
  inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> bake(uint32_t budget) const;

  /**
   * Serialize the FULL design document to `.nucm` project-tier
   * bytes (magic `NUCM`): cells deduped by content hash, instance
   * transforms, ports with scanned hardware, every bus layer with
   * its fragment, runs and `intended`/`routed`/`failed: reason`
   * state, and the loose base layer. Base64 across the bridge.
   */
  inline diplomat::result<std::string, NucleationError> to_nucm_b64() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> to_nucm_b64_write(W& writeable_output) const;

  /**
   * Reopen a `.nucm` design document from raw bytes. The reloaded
   * design is the same model mid-edit: rerouting works.
   */
  inline static diplomat::result<std::unique_ptr<Design>, NucleationError> from_nucm(diplomat::span<const uint8_t> data);

  /**
   * Save the `.nucm` project document to a file. Not available in
   * JS: the WASM build has no filesystem — use `to_nucm_b64`.
   */
  inline diplomat::result<std::monostate, NucleationError> save_nucm(std::string_view path) const;

  /**
   * Load a `.nucm` project document from a file. Not available in
   * JS — read the bytes yourself and use `from_nucm`.
   */
  inline static diplomat::result<std::unique_ptr<Design>, NucleationError> load_nucm(std::string_view path);

  /**
   * Export the design as a LAYERED `.litematic` (interchange tier):
   * one named region per layer (`inst:{name}`, `bus:{name}`, loose
   * base) plus the design manifest as a root-level
   * `NucleationDesign` tag. Opens in Litematica as a plain
   * multi-region litematic; reimports as a design whose cell
   * references have degraded to embedded copies. Base64 across the
   * bridge.
   */
  inline diplomat::result<std::string, NucleationError> to_litematic_b64() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> to_litematic_b64_write(W& writeable_output) const;

  /**
   * Import a layered `.litematic` (with a `NucleationDesign`
   * manifest) from raw bytes; a plain litematic errors loudly —
   * open those with `Schematic.from_litematic`.
   */
  inline static diplomat::result<std::unique_ptr<Design>, NucleationError> from_litematic(diplomat::span<const uint8_t> data);

  /**
   * Export the layered `.litematic` to a file. Not available in JS
   * — use `to_litematic_b64`.
   */
  inline diplomat::result<std::monostate, NucleationError> export_litematic(std::string_view path) const;

  /**
   * Import a layered `.litematic` from a file. Not available in JS
   * — read the bytes yourself and use `from_litematic`.
   */
  inline static diplomat::result<std::unique_ptr<Design>, NucleationError> import_litematic(std::string_view path);

    inline const diplomat::capi::Design* AsFFI() const;
    inline diplomat::capi::Design* AsFFI();
    inline static const Design* FromFFI(const diplomat::capi::Design* ptr);
    inline static Design* FromFFI(diplomat::capi::Design* ptr);
    inline static void operator delete(void* ptr);
private:
    Design() = delete;
    Design(const Design&) = delete;
    Design(Design&&) noexcept = delete;
    Design operator=(const Design&) = delete;
    Design operator=(Design&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Design_D_HPP
