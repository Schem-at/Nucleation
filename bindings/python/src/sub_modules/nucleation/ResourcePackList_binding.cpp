#include "diplomat_nanobind_common.hpp"


#include "ResourcePackList.hpp"

namespace nucleation {
void add_ResourcePackList_binding(nb::module_ mod) {
    PyType_Slot nucleation_ResourcePackList_slots[] = {
        {Py_tp_free, (void *)nucleation::ResourcePackList::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::ResourcePackList> opaque(mod, "ResourcePackList", nb::type_slots(nucleation_ResourcePackList_slots));
    opaque
        .def("add", &nucleation::ResourcePackList::add, "data"_a)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::ResourcePackList::create)))
        .def("len", &nucleation::ResourcePackList::len);
}

} 