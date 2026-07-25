#include "diplomat_nanobind_common.hpp"


#include "CellularSdfConfig.hpp"

namespace nucleation {
void add_CellularSdfConfig_binding(nb::module_ mod) {
    PyType_Slot nucleation_CellularSdfConfig_slots[] = {
        {Py_tp_free, (void *)nucleation::CellularSdfConfig::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::CellularSdfConfig> opaque(mod, "CellularSdfConfig", nb::type_slots(nucleation_CellularSdfConfig_slots));
    opaque
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::CellularSdfConfig::create)), "cell_size_x"_a, "cell_size_z"_a, "seed"_a, "max_jitter_x"_a, "max_jitter_z"_a, "max_yaw_degrees"_a, "min_scale"_a, "max_scale"_a, "min_y_offset"_a, "max_y_offset"_a, "presence_numerator"_a, "presence_denominator"_a, "feature_salt"_a);
}

}
