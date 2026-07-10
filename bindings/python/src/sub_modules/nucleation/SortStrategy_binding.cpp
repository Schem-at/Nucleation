#include "diplomat_nanobind_common.hpp"


#include "SortStrategy.hpp"

namespace nucleation {
void add_SortStrategy_binding(nb::module_ mod) {
    PyType_Slot nucleation_SortStrategy_slots[] = {
        {Py_tp_free, (void *)nucleation::SortStrategy::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::SortStrategy> opaque(mod, "SortStrategy", nb::type_slots(nucleation_SortStrategy_slots));
    opaque
        .def_static("descending", std::move(maybe_op_unwrap(&nucleation::SortStrategy::descending)))
        .def_static("distance_from", std::move(maybe_op_unwrap(&nucleation::SortStrategy::distance_from)), "x"_a, "y"_a, "z"_a)
        .def_static("distance_from_desc", std::move(maybe_op_unwrap(&nucleation::SortStrategy::distance_from_desc)), "x"_a, "y"_a, "z"_a)
        .def_static("from_string", std::move(maybe_op_unwrap(&nucleation::SortStrategy::from_string)), "s"_a)
        .def("name", &nucleation::SortStrategy::name)
        .def_static("preserve", std::move(maybe_op_unwrap(&nucleation::SortStrategy::preserve)))
        .def_static("reverse", std::move(maybe_op_unwrap(&nucleation::SortStrategy::reverse)))
        .def_static("x_desc_yz", std::move(maybe_op_unwrap(&nucleation::SortStrategy::x_desc_yz)))
        .def_static("xyz", std::move(maybe_op_unwrap(&nucleation::SortStrategy::xyz)))
        .def_static("y_desc_xz", std::move(maybe_op_unwrap(&nucleation::SortStrategy::y_desc_xz)))
        .def_static("yxz", std::move(maybe_op_unwrap(&nucleation::SortStrategy::yxz)))
        .def_static("z_desc_yx", std::move(maybe_op_unwrap(&nucleation::SortStrategy::z_desc_yx)))
        .def_static("zyx", std::move(maybe_op_unwrap(&nucleation::SortStrategy::zyx)));
}

} 