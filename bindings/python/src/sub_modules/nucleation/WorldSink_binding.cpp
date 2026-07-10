#include "diplomat_nanobind_common.hpp"


#include "WorldChunkView.hpp"
#include "WorldSink.hpp"

namespace nucleation {
void add_WorldSink_binding(nb::module_ mod) {
    PyType_Slot nucleation_WorldSink_slots[] = {
        {Py_tp_free, (void *)nucleation::WorldSink::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::WorldSink> opaque(mod, "WorldSink", nb::type_slots(nucleation_WorldSink_slots));
    opaque
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::WorldSink::create)), "dir"_a, "options_json"_a)
        .def("finish", &nucleation::WorldSink::finish)
        .def_static("open_existing", std::move(maybe_op_unwrap(&nucleation::WorldSink::open_existing)), "dir"_a)
        .def("put_chunk", &nucleation::WorldSink::put_chunk, "view"_a)
        .def("write_chunk", &nucleation::WorldSink::write_chunk, "view"_a);
}

} 