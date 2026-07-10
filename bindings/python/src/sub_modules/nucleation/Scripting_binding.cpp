#include "diplomat_nanobind_common.hpp"


#include "Scripting.hpp"

namespace nucleation {
void add_Scripting_binding(nb::module_ mod) {
    PyType_Slot nucleation_Scripting_slots[] = {
        {Py_tp_free, (void *)nucleation::Scripting::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Scripting> opaque(mod, "Scripting", nb::type_slots(nucleation_Scripting_slots));
    opaque
        .def_static("run_js_script", std::move(maybe_op_unwrap(&nucleation::Scripting::run_js_script)), "path"_a)
        .def_static("run_lua_script", std::move(maybe_op_unwrap(&nucleation::Scripting::run_lua_script)), "path"_a)
        .def_static("run_script", std::move(maybe_op_unwrap(&nucleation::Scripting::run_script)), "path"_a);
}

} 