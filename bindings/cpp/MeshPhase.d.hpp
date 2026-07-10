#ifndef MeshPhase_D_HPP
#define MeshPhase_D_HPP

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
    enum MeshPhase {
      MeshPhase_BuildingAtlas = 0,
      MeshPhase_MeshingChunks = 1,
      MeshPhase_Complete = 2,
      MeshPhase_Failed = 3,
    };

    typedef struct MeshPhase_option {union { MeshPhase ok; }; bool is_ok; } MeshPhase_option;
} // namespace capi
} // namespace

/**
 * Phase of a running {@link MeshJob}.
 */
class MeshPhase {
public:
    enum Value {
        /**
         * Scanning palettes / building the shared texture atlas.
         */
        BuildingAtlas = 0,
        /**
         * Meshing individual chunks (`current` / `total` advance here).
         */
        MeshingChunks = 1,
        /**
         * All chunks meshed; `take_result` will not block.
         */
        Complete = 2,
        /**
         * The job failed; `take_result` returns the error.
         */
        Failed = 3,
    };

    MeshPhase(): value(Value::BuildingAtlas) {}

    // Implicit conversions between enum and ::Value
    constexpr MeshPhase(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline diplomat::capi::MeshPhase AsFFI() const;
    inline static MeshPhase FromFFI(diplomat::capi::MeshPhase c_enum);
private:
    Value value;
};


#endif // MeshPhase_D_HPP
