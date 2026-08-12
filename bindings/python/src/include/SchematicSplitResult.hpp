#ifndef NUCLEATION_SchematicSplitResult_HPP
#define NUCLEATION_SchematicSplitResult_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    uint32_t SchematicSplitResult_len(const nucleation::capi::SchematicSplitResult* self);

    typedef struct SchematicSplitResult_piece_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} SchematicSplitResult_piece_result;
    SchematicSplitResult_piece_result SchematicSplitResult_piece(const nucleation::capi::SchematicSplitResult* self, uint32_t index);

    void SchematicSplitResult_destroy(SchematicSplitResult* self);

    } // extern "C"
} // namespace capi
} // namespace

inline uint32_t nucleation::SchematicSplitResult::len() const {
    auto result = nucleation::capi::SchematicSplitResult_len(this->AsFFI());
    return result;
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::SchematicSplitResult::piece(uint32_t index) const {
    auto result = nucleation::capi::SchematicSplitResult_piece(this->AsFFI(),
        index);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::SchematicSplitResult* nucleation::SchematicSplitResult::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::SchematicSplitResult*>(this);
}

inline nucleation::capi::SchematicSplitResult* nucleation::SchematicSplitResult::AsFFI() {
    return reinterpret_cast<nucleation::capi::SchematicSplitResult*>(this);
}

inline const nucleation::SchematicSplitResult* nucleation::SchematicSplitResult::FromFFI(const nucleation::capi::SchematicSplitResult* ptr) {
    return reinterpret_cast<const nucleation::SchematicSplitResult*>(ptr);
}

inline nucleation::SchematicSplitResult* nucleation::SchematicSplitResult::FromFFI(nucleation::capi::SchematicSplitResult* ptr) {
    return reinterpret_cast<nucleation::SchematicSplitResult*>(ptr);
}

inline void nucleation::SchematicSplitResult::operator delete(void* ptr) {
    nucleation::capi::SchematicSplitResult_destroy(reinterpret_cast<nucleation::capi::SchematicSplitResult*>(ptr));
}


#endif // NUCLEATION_SchematicSplitResult_HPP
