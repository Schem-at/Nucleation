#include "diplomat_nanobind_common.hpp"


#include "WsPartitionHints.hpp"
#include "WsProfile.hpp"
#include "WsRunResult.hpp"
#include "WsSegmentJob.hpp"

namespace nucleation {
void add_WsRunResult_binding(nb::module_ mod) {
    PyType_Slot nucleation_WsRunResult_slots[] = {
        {Py_tp_free, (void *)nucleation::WsRunResult::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::WsRunResult> opaque(mod, "WsRunResult", nb::type_slots(nucleation_WsRunResult_slots));
    opaque
        .def("bbox_max_of", &nucleation::WsRunResult::bbox_max_of, "index"_a)
        .def("bbox_min_of", &nucleation::WsRunResult::bbox_min_of, "index"_a)
        .def("block_count_of", &nucleation::WsRunResult::block_count_of, "index"_a)
        .def("build_count", &nucleation::WsRunResult::build_count)
        .def("builds", &nucleation::WsRunResult::builds)
        .def("cross_tile", &nucleation::WsRunResult::cross_tile)
        .def("fingerprint_hex", &nucleation::WsRunResult::fingerprint_hex, "index"_a)
        .def("largest_block_count", &nucleation::WsRunResult::largest_block_count)
        .def_static("run_dir", std::move(maybe_op_unwrap(&nucleation::WsRunResult::run_dir)), "job"_a, "hints"_a, "profile"_a, "world_dir"_a)
        .def("stable_id_hex", &nucleation::WsRunResult::stable_id_hex, "index"_a)
        .def("tier_confident", &nucleation::WsRunResult::tier_confident)
        .def("tier_debris", &nucleation::WsRunResult::tier_debris)
        .def("tier_of", &nucleation::WsRunResult::tier_of, "index"_a)
        .def("tier_probable", &nucleation::WsRunResult::tier_probable)
        .def("write_schem_to", &nucleation::WsRunResult::write_schem_to, "index"_a, "path"_a);
}

} 