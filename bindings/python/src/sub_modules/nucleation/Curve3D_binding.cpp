#include "diplomat_nanobind_common.hpp"


#include "Curve3D.hpp"

namespace nucleation {
void add_Curve3D_binding(nb::module_ mod) {
    PyType_Slot nucleation_Curve3D_slots[] = {
        {Py_tp_free, (void *)nucleation::Curve3D::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Curve3D> opaque(mod, "Curve3D", nb::type_slots(nucleation_Curve3D_slots));
    opaque
        .def_static("from_points", std::move(maybe_op_unwrap(&nucleation::Curve3D::from_points)), "coordinates"_a, "closed"_a)
        .def("is_closed", &nucleation::Curve3D::is_closed)
        .def("point_count", &nucleation::Curve3D::point_count);
}

} 