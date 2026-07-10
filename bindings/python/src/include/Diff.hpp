#ifndef NUCLEATION_Diff_HPP
#define NUCLEATION_Diff_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct Diff_compute_result {union {nucleation::capi::Diff* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Diff_compute_result;
    Diff_compute_result Diff_compute(const nucleation::capi::Schematic* a, const nucleation::capi::Schematic* b, nucleation::diplomat::capi::DiplomatStringView preset);

    typedef struct Diff_compute_with_opts_result {union {nucleation::capi::Diff* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Diff_compute_with_opts_result;
    Diff_compute_with_opts_result Diff_compute_with_opts(const nucleation::capi::Schematic* a, const nucleation::capi::Schematic* b, nucleation::diplomat::capi::DiplomatStringView preset, int32_t cost_add, int32_t cost_delete, int32_t cost_change, int32_t cost_swap, nucleation::diplomat::capi::DiplomatStringView symmetry);

    typedef struct Diff_from_json_result {union {nucleation::capi::Diff* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Diff_from_json_result;
    Diff_from_json_result Diff_from_json(nucleation::diplomat::capi::DiplomatStringView json);

    uint64_t Diff_distance(const nucleation::capi::Diff* self);

    float Diff_support(const nucleation::capi::Diff* self);

    void Diff_to_json(const nucleation::capi::Diff* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void Diff_summary_json(const nucleation::capi::Diff* self, nucleation::diplomat::capi::DiplomatWrite* write);

    nucleation::capi::Schematic* Diff_added(const nucleation::capi::Diff* self);

    nucleation::capi::Schematic* Diff_removed(const nucleation::capi::Diff* self);

    nucleation::capi::Schematic* Diff_changed(const nucleation::capi::Diff* self);

    nucleation::capi::Schematic* Diff_swapped(const nucleation::capi::Diff* self);

    nucleation::capi::Schematic* Diff_markers(const nucleation::capi::Diff* self);

    typedef struct Diff_to_overlay_glb_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Diff_to_overlay_glb_b64_result;
    Diff_to_overlay_glb_b64_result Diff_to_overlay_glb_b64(const nucleation::capi::Diff* self, nucleation::diplomat::capi::DiplomatU8View after_glb, nucleation::diplomat::capi::DiplomatWrite* write);

    void Diff_destroy(Diff* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Diff>, nucleation::NucleationError> nucleation::Diff::compute(const nucleation::Schematic& a, const nucleation::Schematic& b, std::string_view preset) {
    auto result = nucleation::capi::Diff_compute(a.AsFFI(),
        b.AsFFI(),
        {preset.data(), preset.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Diff>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Diff>>(std::unique_ptr<nucleation::Diff>(nucleation::Diff::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Diff>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Diff>, nucleation::NucleationError> nucleation::Diff::compute_with_opts(const nucleation::Schematic& a, const nucleation::Schematic& b, std::string_view preset, int32_t cost_add, int32_t cost_delete, int32_t cost_change, int32_t cost_swap, std::string_view symmetry) {
    auto result = nucleation::capi::Diff_compute_with_opts(a.AsFFI(),
        b.AsFFI(),
        {preset.data(), preset.size()},
        cost_add,
        cost_delete,
        cost_change,
        cost_swap,
        {symmetry.data(), symmetry.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Diff>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Diff>>(std::unique_ptr<nucleation::Diff>(nucleation::Diff::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Diff>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Diff>, nucleation::NucleationError> nucleation::Diff::from_json(std::string_view json) {
    auto result = nucleation::capi::Diff_from_json({json.data(), json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Diff>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Diff>>(std::unique_ptr<nucleation::Diff>(nucleation::Diff::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Diff>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline uint64_t nucleation::Diff::distance() const {
    auto result = nucleation::capi::Diff_distance(this->AsFFI());
    return result;
}

inline float nucleation::Diff::support() const {
    auto result = nucleation::capi::Diff_support(this->AsFFI());
    return result;
}

inline std::string nucleation::Diff::to_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Diff_to_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Diff::to_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Diff_to_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::Diff::summary_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Diff_summary_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Diff::summary_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Diff_summary_json(this->AsFFI(),
        &write);
}

inline std::unique_ptr<nucleation::Schematic> nucleation::Diff::added() const {
    auto result = nucleation::capi::Diff_added(this->AsFFI());
    return std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result));
}

inline std::unique_ptr<nucleation::Schematic> nucleation::Diff::removed() const {
    auto result = nucleation::capi::Diff_removed(this->AsFFI());
    return std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result));
}

inline std::unique_ptr<nucleation::Schematic> nucleation::Diff::changed() const {
    auto result = nucleation::capi::Diff_changed(this->AsFFI());
    return std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result));
}

inline std::unique_ptr<nucleation::Schematic> nucleation::Diff::swapped() const {
    auto result = nucleation::capi::Diff_swapped(this->AsFFI());
    return std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result));
}

inline std::unique_ptr<nucleation::Schematic> nucleation::Diff::markers() const {
    auto result = nucleation::capi::Diff_markers(this->AsFFI());
    return std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Diff::to_overlay_glb_b64(nucleation::diplomat::span<const uint8_t> after_glb) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Diff_to_overlay_glb_b64(this->AsFFI(),
        {after_glb.data(), after_glb.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Diff::to_overlay_glb_b64_write(nucleation::diplomat::span<const uint8_t> after_glb, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Diff_to_overlay_glb_b64(this->AsFFI(),
        {after_glb.data(), after_glb.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Diff* nucleation::Diff::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Diff*>(this);
}

inline nucleation::capi::Diff* nucleation::Diff::AsFFI() {
    return reinterpret_cast<nucleation::capi::Diff*>(this);
}

inline const nucleation::Diff* nucleation::Diff::FromFFI(const nucleation::capi::Diff* ptr) {
    return reinterpret_cast<const nucleation::Diff*>(ptr);
}

inline nucleation::Diff* nucleation::Diff::FromFFI(nucleation::capi::Diff* ptr) {
    return reinterpret_cast<nucleation::Diff*>(ptr);
}

inline void nucleation::Diff::operator delete(void* ptr) {
    nucleation::capi::Diff_destroy(reinterpret_cast<nucleation::capi::Diff*>(ptr));
}


#endif // NUCLEATION_Diff_HPP
