#pragma once

#include "Brush.hpp"
#include "Schematic.hpp"

#include <cmath>
#include <cstdint>
#include <exception>
#include <stdexcept>

namespace nucleation {

extern "C" std::uint8_t nucleation_python_fill_sdf_function(
    capi::Schematic *schematic,
    const capi::Brush *brush,
    std::int32_t min_x,
    std::int32_t min_y,
    std::int32_t min_z,
    std::int32_t max_x,
    std::int32_t max_y,
    std::int32_t max_z,
    double epsilon,
    void *context,
    bool (*eval)(void *, double, double, double, double *),
    bool (*normal)(void *, double, double, double, double *, double *, double *));

struct PythonSdfCallbackState {
    nb::callable eval;
    nb::object normal;
    std::exception_ptr error;
};

inline bool python_sdf_eval(
    void *context, double x, double y, double z, double *output) noexcept {
    auto &state = *static_cast<PythonSdfCallbackState *>(context);
    try {
        *output = nb::cast<double>(state.eval(x, y, z));
        if (!std::isfinite(*output)) {
            throw std::invalid_argument("SDF callback must return a finite number");
        }
        return true;
    } catch (...) {
        state.error = std::current_exception();
        return false;
    }
}

inline bool python_sdf_normal(
    void *context,
    double x,
    double y,
    double z,
    double *output_x,
    double *output_y,
    double *output_z) noexcept {
    auto &state = *static_cast<PythonSdfCallbackState *>(context);
    try {
        nb::object value = state.normal(x, y, z);
        nb::sequence sequence = nb::cast<nb::sequence>(value);
        if (nb::len(sequence) != 3) {
            throw std::invalid_argument("SDF normal callback must return three numbers");
        }
        *output_x = nb::cast<double>(sequence[0]);
        *output_y = nb::cast<double>(sequence[1]);
        *output_z = nb::cast<double>(sequence[2]);
        if (!std::isfinite(*output_x) || !std::isfinite(*output_y) || !std::isfinite(*output_z)) {
            throw std::invalid_argument("SDF normal callback must return finite numbers");
        }
        return true;
    } catch (...) {
        state.error = std::current_exception();
        return false;
    }
}

inline void fill_sdf_function(
    Schematic &schematic,
    const Brush &brush,
    std::int32_t min_x,
    std::int32_t min_y,
    std::int32_t min_z,
    std::int32_t max_x,
    std::int32_t max_y,
    std::int32_t max_z,
    nb::callable eval,
    nb::object normal,
    double epsilon) {
    PythonSdfCallbackState state{std::move(eval), std::move(normal), nullptr};
    auto normal_callback = state.normal.is_none() ? nullptr : &python_sdf_normal;
    const std::uint8_t status = nucleation_python_fill_sdf_function(
        schematic.AsFFI(),
        brush.AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z,
        epsilon,
        &state,
        &python_sdf_eval,
        normal_callback);

    if (status == 0) {
        return;
    }
    if (state.error) {
        std::rethrow_exception(state.error);
    }
    if (status == 3) {
        throw std::runtime_error("internal panic while evaluating SDF callback");
    }
    throw nb::value_error(
        "invalid SDF callback bounds/epsilon or resulting volume exceeds 16,777,216 blocks");
}

} // namespace nucleation
