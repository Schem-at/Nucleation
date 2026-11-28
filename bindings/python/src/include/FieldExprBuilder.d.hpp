#ifndef NUCLEATION_FieldExprBuilder_D_HPP
#define NUCLEATION_FieldExprBuilder_D_HPP

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
namespace capi { struct Field; }
class Field;
namespace capi { struct FieldExprBuilder; }
class FieldExprBuilder;
class FieldDistanceKind;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct FieldExprBuilder;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Builder for compiling infix expressions that reference named external {@link Field}s.
 */
class FieldExprBuilder {
public:

  inline static std::unique_ptr<nucleation::FieldExprBuilder> create();

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> bind(std::string_view name, const nucleation::Field& field);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_bounds(float min_x, float min_y, float min_z, float max_x, float max_y, float max_z);

  inline void set_distance_kind(nucleation::FieldDistanceKind kind);

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> compile(std::string_view source) const;

    inline const nucleation::capi::FieldExprBuilder* AsFFI() const;
    inline nucleation::capi::FieldExprBuilder* AsFFI();
    inline static const nucleation::FieldExprBuilder* FromFFI(const nucleation::capi::FieldExprBuilder* ptr);
    inline static nucleation::FieldExprBuilder* FromFFI(nucleation::capi::FieldExprBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    FieldExprBuilder() = delete;
    FieldExprBuilder(const nucleation::FieldExprBuilder&) = delete;
    FieldExprBuilder(nucleation::FieldExprBuilder&&) noexcept = delete;
    FieldExprBuilder operator=(const nucleation::FieldExprBuilder&) = delete;
    FieldExprBuilder operator=(nucleation::FieldExprBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_FieldExprBuilder_D_HPP
