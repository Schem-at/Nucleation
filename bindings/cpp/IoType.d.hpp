#ifndef IoType_D_HPP
#define IoType_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    struct IoType;
} // namespace capi
} // namespace

/**
 * The wire type of a circuit input/output (PORTING rule 10).
 */
class IoType {
public:

  inline static std::unique_ptr<IoType> unsigned_int(uint32_t bits);

  inline static std::unique_ptr<IoType> signed_int(uint32_t bits);

  inline static std::unique_ptr<IoType> float32();

  inline static std::unique_ptr<IoType> boolean();

  inline static std::unique_ptr<IoType> ascii(uint32_t chars);

    inline const diplomat::capi::IoType* AsFFI() const;
    inline diplomat::capi::IoType* AsFFI();
    inline static const IoType* FromFFI(const diplomat::capi::IoType* ptr);
    inline static IoType* FromFFI(diplomat::capi::IoType* ptr);
    inline static void operator delete(void* ptr);
private:
    IoType() = delete;
    IoType(const IoType&) = delete;
    IoType(IoType&&) noexcept = delete;
    IoType operator=(const IoType&) = delete;
    IoType operator=(IoType&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // IoType_D_HPP
