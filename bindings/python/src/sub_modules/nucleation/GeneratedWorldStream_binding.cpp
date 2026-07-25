#include "diplomat_nanobind_common.hpp"


#include "GeneratedWorldStream.hpp"

namespace nucleation {
void add_GeneratedWorldStream_binding(nb::module_ mod) {
    PyType_Slot nucleation_GeneratedWorldStream_slots[] = {
        {Py_tp_free, (void *)nucleation::GeneratedWorldStream::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::GeneratedWorldStream> opaque(mod, "GeneratedWorldStream", nb::type_slots(nucleation_GeneratedWorldStream_slots));
    opaque
        .def("next", std::move(maybe_op_unwrap(&nucleation::GeneratedWorldStream::next)))
        .def("remaining", &nucleation::GeneratedWorldStream::remaining);
}

}
