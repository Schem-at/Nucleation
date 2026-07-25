#ifndef NUCLEATION_FieldRange_D_HPP
#define NUCLEATION_FieldRange_D_HPP

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
namespace capi {
    struct FieldRange {
      float min;
      float max;
    };

    typedef struct FieldRange_option {union { FieldRange ok; }; bool is_ok; } FieldRange_option;
} // namespace capi
} // namespace


namespace nucleation {
/**
 * The closed interval a field's values are analytically proven to lie in.
 */
struct FieldRange {
    float min;
    float max;

    inline nucleation::capi::FieldRange AsFFI() const;
    inline static nucleation::FieldRange FromFFI(nucleation::capi::FieldRange c_struct);
};

} // namespace
#endif // NUCLEATION_FieldRange_D_HPP
