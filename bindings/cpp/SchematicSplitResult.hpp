#ifndef SchematicSplitResult_HPP
#define SchematicSplitResult_HPP

#include "SchematicSplitResult.d.hpp"

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

    uint32_t SchematicSplitResult_len(const diplomat::capi::SchematicSplitResult* self);

    typedef struct SchematicSplitResult_piece_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} SchematicSplitResult_piece_result;
    SchematicSplitResult_piece_result SchematicSplitResult_piece(const diplomat::capi::SchematicSplitResult* self, uint32_t index);

    void SchematicSplitResult_destroy(SchematicSplitResult* self);

    } // extern "C"
} // namespace capi
} // namespace

inline uint32_t SchematicSplitResult::len() const {
    auto result = diplomat::capi::SchematicSplitResult_len(this->AsFFI());
    return result;
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> SchematicSplitResult::piece(uint32_t index) const {
    auto result = diplomat::capi::SchematicSplitResult_piece(this->AsFFI(),
        index);
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::SchematicSplitResult* SchematicSplitResult::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::SchematicSplitResult*>(this);
}

inline diplomat::capi::SchematicSplitResult* SchematicSplitResult::AsFFI() {
    return reinterpret_cast<diplomat::capi::SchematicSplitResult*>(this);
}

inline const SchematicSplitResult* SchematicSplitResult::FromFFI(const diplomat::capi::SchematicSplitResult* ptr) {
    return reinterpret_cast<const SchematicSplitResult*>(ptr);
}

inline SchematicSplitResult* SchematicSplitResult::FromFFI(diplomat::capi::SchematicSplitResult* ptr) {
    return reinterpret_cast<SchematicSplitResult*>(ptr);
}

inline void SchematicSplitResult::operator delete(void* ptr) {
    diplomat::capi::SchematicSplitResult_destroy(reinterpret_cast<diplomat::capi::SchematicSplitResult*>(ptr));
}


#endif // SchematicSplitResult_HPP
