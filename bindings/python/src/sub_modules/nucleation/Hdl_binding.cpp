#include "diplomat_nanobind_common.hpp"


#include "Hdl.hpp"

namespace nucleation {
void add_Hdl_binding(nb::module_ mod) {
    PyType_Slot nucleation_Hdl_slots[] = {
        {Py_tp_free, (void *)nucleation::Hdl::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::Hdl> opaque(mod, "Hdl", nb::type_slots(nucleation_Hdl_slots));
    opaque
        .def_static("compile_blif", std::move(maybe_op_unwrap(&nucleation::Hdl::compile_blif)), "blif"_a, "name"_a, "bake"_a)
        .def_static("compile_blif_contract", &nucleation::Hdl::compile_blif_contract, "blif"_a, "name"_a)
        .def_static("compile_blif_report", &nucleation::Hdl::compile_blif_report, "blif"_a, "name"_a);
}

}
