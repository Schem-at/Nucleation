#include "diplomat_nanobind_common.hpp"


#include "ItemModelPackBuilder.hpp"

namespace nucleation {
void add_ItemModelPackBuilder_binding(nb::module_ mod) {
    PyType_Slot nucleation_ItemModelPackBuilder_slots[] = {
        {Py_tp_free, (void *)nucleation::ItemModelPackBuilder::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::ItemModelPackBuilder> opaque(mod, "ItemModelPackBuilder", nb::type_slots(nucleation_ItemModelPackBuilder_slots));
    opaque
        .def("build_zip_b64", &nucleation::ItemModelPackBuilder::build_zip_b64)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::ItemModelPackBuilder::create)))
        .def("len", &nucleation::ItemModelPackBuilder::len);
}

} 