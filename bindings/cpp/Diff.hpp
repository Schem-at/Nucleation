#ifndef Diff_HPP
#define Diff_HPP

#include "Diff.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Diff_compute_result {union {diplomat::capi::Diff* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Diff_compute_result;
    Diff_compute_result Diff_compute(const diplomat::capi::Schematic* a, const diplomat::capi::Schematic* b, diplomat::capi::DiplomatStringView preset);

    typedef struct Diff_compute_with_opts_result {union {diplomat::capi::Diff* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Diff_compute_with_opts_result;
    Diff_compute_with_opts_result Diff_compute_with_opts(const diplomat::capi::Schematic* a, const diplomat::capi::Schematic* b, diplomat::capi::DiplomatStringView preset, int32_t cost_add, int32_t cost_delete, int32_t cost_change, int32_t cost_swap, diplomat::capi::DiplomatStringView symmetry);

    typedef struct Diff_from_json_result {union {diplomat::capi::Diff* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Diff_from_json_result;
    Diff_from_json_result Diff_from_json(diplomat::capi::DiplomatStringView json);

    uint64_t Diff_distance(const diplomat::capi::Diff* self);

    float Diff_support(const diplomat::capi::Diff* self);

    void Diff_to_json(const diplomat::capi::Diff* self, diplomat::capi::DiplomatWrite* write);

    void Diff_summary_json(const diplomat::capi::Diff* self, diplomat::capi::DiplomatWrite* write);

    diplomat::capi::Schematic* Diff_added(const diplomat::capi::Diff* self);

    diplomat::capi::Schematic* Diff_removed(const diplomat::capi::Diff* self);

    diplomat::capi::Schematic* Diff_changed(const diplomat::capi::Diff* self);

    diplomat::capi::Schematic* Diff_swapped(const diplomat::capi::Diff* self);

    diplomat::capi::Schematic* Diff_markers(const diplomat::capi::Diff* self);

    typedef struct Diff_to_overlay_glb_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Diff_to_overlay_glb_b64_result;
    Diff_to_overlay_glb_b64_result Diff_to_overlay_glb_b64(const diplomat::capi::Diff* self, diplomat::capi::DiplomatU8View after_glb, diplomat::capi::DiplomatWrite* write);

    void Diff_destroy(Diff* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Diff>, NucleationError> Diff::compute(const Schematic& a, const Schematic& b, std::string_view preset) {
    auto result = diplomat::capi::Diff_compute(a.AsFFI(),
        b.AsFFI(),
        {preset.data(), preset.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Diff>, NucleationError>(diplomat::Ok<std::unique_ptr<Diff>>(std::unique_ptr<Diff>(Diff::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Diff>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Diff>, NucleationError> Diff::compute_with_opts(const Schematic& a, const Schematic& b, std::string_view preset, int32_t cost_add, int32_t cost_delete, int32_t cost_change, int32_t cost_swap, std::string_view symmetry) {
    auto result = diplomat::capi::Diff_compute_with_opts(a.AsFFI(),
        b.AsFFI(),
        {preset.data(), preset.size()},
        cost_add,
        cost_delete,
        cost_change,
        cost_swap,
        {symmetry.data(), symmetry.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Diff>, NucleationError>(diplomat::Ok<std::unique_ptr<Diff>>(std::unique_ptr<Diff>(Diff::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Diff>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Diff>, NucleationError> Diff::from_json(std::string_view json) {
    auto result = diplomat::capi::Diff_from_json({json.data(), json.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Diff>, NucleationError>(diplomat::Ok<std::unique_ptr<Diff>>(std::unique_ptr<Diff>(Diff::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Diff>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline uint64_t Diff::distance() const {
    auto result = diplomat::capi::Diff_distance(this->AsFFI());
    return result;
}

inline float Diff::support() const {
    auto result = diplomat::capi::Diff_support(this->AsFFI());
    return result;
}

inline std::string Diff::to_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Diff_to_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Diff::to_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Diff_to_json(this->AsFFI(),
        &write);
}

inline std::string Diff::summary_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Diff_summary_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Diff::summary_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Diff_summary_json(this->AsFFI(),
        &write);
}

inline std::unique_ptr<Schematic> Diff::added() const {
    auto result = diplomat::capi::Diff_added(this->AsFFI());
    return std::unique_ptr<Schematic>(Schematic::FromFFI(result));
}

inline std::unique_ptr<Schematic> Diff::removed() const {
    auto result = diplomat::capi::Diff_removed(this->AsFFI());
    return std::unique_ptr<Schematic>(Schematic::FromFFI(result));
}

inline std::unique_ptr<Schematic> Diff::changed() const {
    auto result = diplomat::capi::Diff_changed(this->AsFFI());
    return std::unique_ptr<Schematic>(Schematic::FromFFI(result));
}

inline std::unique_ptr<Schematic> Diff::swapped() const {
    auto result = diplomat::capi::Diff_swapped(this->AsFFI());
    return std::unique_ptr<Schematic>(Schematic::FromFFI(result));
}

inline std::unique_ptr<Schematic> Diff::markers() const {
    auto result = diplomat::capi::Diff_markers(this->AsFFI());
    return std::unique_ptr<Schematic>(Schematic::FromFFI(result));
}

inline diplomat::result<std::string, NucleationError> Diff::to_overlay_glb_b64(diplomat::span<const uint8_t> after_glb) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Diff_to_overlay_glb_b64(this->AsFFI(),
        {after_glb.data(), after_glb.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Diff::to_overlay_glb_b64_write(diplomat::span<const uint8_t> after_glb, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Diff_to_overlay_glb_b64(this->AsFFI(),
        {after_glb.data(), after_glb.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Diff* Diff::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Diff*>(this);
}

inline diplomat::capi::Diff* Diff::AsFFI() {
    return reinterpret_cast<diplomat::capi::Diff*>(this);
}

inline const Diff* Diff::FromFFI(const diplomat::capi::Diff* ptr) {
    return reinterpret_cast<const Diff*>(ptr);
}

inline Diff* Diff::FromFFI(diplomat::capi::Diff* ptr) {
    return reinterpret_cast<Diff*>(ptr);
}

inline void Diff::operator delete(void* ptr) {
    diplomat::capi::Diff_destroy(reinterpret_cast<diplomat::capi::Diff*>(ptr));
}


#endif // Diff_HPP
