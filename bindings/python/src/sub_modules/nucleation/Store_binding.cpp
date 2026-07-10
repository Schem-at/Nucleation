#include "diplomat_nanobind_common.hpp"


#include "Schematic.hpp"
#include "Store.hpp"

namespace nucleation {
void add_Store_binding(nb::module_ mod) {
    PyType_Slot nucleation_Store_slots[] = {
        {Py_tp_free, (void *)nucleation::Store::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Store> opaque(mod, "Store", nb::type_slots(nucleation_Store_slots));
    opaque
        .def("delete", &nucleation::Store::delete_, "key"_a)
        .def("exists", &nucleation::Store::exists, "key"_a)
        .def("get_b64", &nucleation::Store::get_b64, "key"_a)
        .def("health", &nucleation::Store::health)
        .def("list", &nucleation::Store::list, "prefix"_a)
        .def("list_paginated", &nucleation::Store::list_paginated, "prefix"_a, "after"_a, "limit"_a)
        .def_static("open", std::move(maybe_op_unwrap(&nucleation::Store::open)), "url"_a)
        .def("open_schematic", std::move(maybe_op_unwrap(&nucleation::Store::open_schematic)), "key"_a)
        .def("put", &nucleation::Store::put, "key"_a, "data"_a)
        .def("put_if_absent", &nucleation::Store::put_if_absent, "key"_a, "data"_a)
        .def("save_schematic", &nucleation::Store::save_schematic, "schematic"_a, "key"_a, "version"_a);
}

} 