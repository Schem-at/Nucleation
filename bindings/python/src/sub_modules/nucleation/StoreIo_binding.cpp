#include "diplomat_nanobind_common.hpp"


#include "Schematic.hpp"
#include "StoreIo.hpp"

namespace nucleation {
void add_StoreIo_binding(nb::module_ mod) {
    PyType_Slot nucleation_StoreIo_slots[] = {
        {Py_tp_free, (void *)nucleation::StoreIo::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::StoreIo> opaque(mod, "StoreIo", nb::type_slots(nucleation_StoreIo_slots));
    opaque
        .def_static("default_format_version", &nucleation::StoreIo::default_format_version, "format"_a)
        .def_static("export_settings_schema", &nucleation::StoreIo::export_settings_schema, "format"_a)
        .def_static("format_versions", &nucleation::StoreIo::format_versions, "format"_a)
        .def_static("import_settings_schema", &nucleation::StoreIo::import_settings_schema, "format"_a)
        .def_static("open", std::move(maybe_op_unwrap(&nucleation::StoreIo::open)), "uri"_a)
        .def_static("save", &nucleation::StoreIo::save, "schematic"_a, "uri"_a, "version"_a)
        .def_static("supported_export_formats", &nucleation::StoreIo::supported_export_formats)
        .def_static("supported_import_formats", &nucleation::StoreIo::supported_import_formats);
}

} 