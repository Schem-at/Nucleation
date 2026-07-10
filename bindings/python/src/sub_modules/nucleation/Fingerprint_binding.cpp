#include "diplomat_nanobind_common.hpp"


#include "Fingerprint.hpp"
#include "Schematic.hpp"

namespace nucleation {
void add_Fingerprint_binding(nb::module_ mod) {
    PyType_Slot nucleation_Fingerprint_slots[] = {
        {Py_tp_free, (void *)nucleation::Fingerprint::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Fingerprint> opaque(mod, "Fingerprint", nb::type_slots(nucleation_Fingerprint_slots));
    opaque
        .def_static("compute", &nucleation::Fingerprint::compute, "schematic"_a, "preset"_a)
        .def_static("footprint_distance", &nucleation::Fingerprint::footprint_distance, "a"_a, "b"_a, "preset"_a)
        .def_static("footprint_json", &nucleation::Fingerprint::footprint_json, "schematic"_a, "preset"_a)
        .def_static("is_duplicate", &nucleation::Fingerprint::is_duplicate, "a"_a, "b"_a, "preset"_a)
        .def_static("signature_json", &nucleation::Fingerprint::signature_json, "schematic"_a, "preset"_a);
}

} 