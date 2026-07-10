#ifndef NUCLEATION_IoType_D_HPP
#define NUCLEATION_IoType_D_HPP

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
namespace capi { struct IoType; }
class IoType;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct IoType;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * The wire type of a circuit input/output (PORTING rule 10).
 */
class IoType {
public:

  inline static std::unique_ptr<nucleation::IoType> unsigned_int(uint32_t bits);

  inline static std::unique_ptr<nucleation::IoType> signed_int(uint32_t bits);

  inline static std::unique_ptr<nucleation::IoType> float32();

  inline static std::unique_ptr<nucleation::IoType> boolean();

  inline static std::unique_ptr<nucleation::IoType> ascii(uint32_t chars);

    inline const nucleation::capi::IoType* AsFFI() const;
    inline nucleation::capi::IoType* AsFFI();
    inline static const nucleation::IoType* FromFFI(const nucleation::capi::IoType* ptr);
    inline static nucleation::IoType* FromFFI(nucleation::capi::IoType* ptr);
    inline static void operator delete(void* ptr);
private:
    IoType() = delete;
    IoType(const nucleation::IoType&) = delete;
    IoType(nucleation::IoType&&) noexcept = delete;
    IoType operator=(const nucleation::IoType&) = delete;
    IoType operator=(nucleation::IoType&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_IoType_D_HPP
