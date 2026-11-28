#include "diplomat_nanobind_common.hpp"


#include "MeshBytes.hpp"

namespace nucleation {
void add_MeshBytes_binding(nb::module_ mod) {
    PyType_Slot nucleation_MeshBytes_slots[] = {
        {Py_tp_free, (void *)nucleation::MeshBytes::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::MeshBytes> opaque(mod, "MeshBytes", nb::type_slots(nucleation_MeshBytes_slots));
    opaque
        .def("copy_into", &nucleation::MeshBytes::copy_into, "offset"_a, "out"_a)
        .def("len", &nucleation::MeshBytes::len);
}

}
