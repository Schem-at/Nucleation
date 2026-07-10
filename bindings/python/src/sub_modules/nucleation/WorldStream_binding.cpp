#include "diplomat_nanobind_common.hpp"


#include "WorldStream.hpp"

namespace nucleation {
void add_WorldStream_binding(nb::module_ mod) {
    PyType_Slot nucleation_WorldStream_slots[] = {
        {Py_tp_free, (void *)nucleation::WorldStream::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::WorldStream> opaque(mod, "WorldStream", nb::type_slots(nucleation_WorldStream_slots));
    opaque
        .def_static("from_zip", std::move(maybe_op_unwrap(&nucleation::WorldStream::from_zip)), "data"_a)
        .def_static("from_zip_bounded", std::move(maybe_op_unwrap(&nucleation::WorldStream::from_zip_bounded)), "data"_a, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def("next", std::move(maybe_op_unwrap(&nucleation::WorldStream::next)))
        .def_static("open_dir", std::move(maybe_op_unwrap(&nucleation::WorldStream::open_dir)), "path"_a)
        .def_static("open_dir_bounded", std::move(maybe_op_unwrap(&nucleation::WorldStream::open_dir_bounded)), "path"_a, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a);
}

} 