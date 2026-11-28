#ifndef FieldExprBuilder_D_HPP
#define FieldExprBuilder_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Field; }
class Field;
class FieldDistanceKind;
class NucleationError;




namespace diplomat {
namespace capi {
    struct FieldExprBuilder;
} // namespace capi
} // namespace

/**
 * Builder for compiling infix expressions that reference named external {@link Field}s.
 */
class FieldExprBuilder {
public:

  inline static std::unique_ptr<FieldExprBuilder> create();

  inline diplomat::result<std::monostate, NucleationError> bind(std::string_view name, const Field& field);

  inline diplomat::result<std::monostate, NucleationError> set_bounds(float min_x, float min_y, float min_z, float max_x, float max_y, float max_z);

  inline void set_distance_kind(FieldDistanceKind kind);

  inline diplomat::result<std::unique_ptr<Field>, NucleationError> compile(std::string_view source) const;

    inline const diplomat::capi::FieldExprBuilder* AsFFI() const;
    inline diplomat::capi::FieldExprBuilder* AsFFI();
    inline static const FieldExprBuilder* FromFFI(const diplomat::capi::FieldExprBuilder* ptr);
    inline static FieldExprBuilder* FromFFI(diplomat::capi::FieldExprBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    FieldExprBuilder() = delete;
    FieldExprBuilder(const FieldExprBuilder&) = delete;
    FieldExprBuilder(FieldExprBuilder&&) noexcept = delete;
    FieldExprBuilder operator=(const FieldExprBuilder&) = delete;
    FieldExprBuilder operator=(FieldExprBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // FieldExprBuilder_D_HPP
