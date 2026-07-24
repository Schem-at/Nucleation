#include "diplomat_nanobind_common.hpp"


#include "WsSegmentJob.hpp"

namespace nucleation {
void add_WsSegmentJob_binding(nb::module_ mod) {
    PyType_Slot nucleation_WsSegmentJob_slots[] = {
        {Py_tp_free, (void *)nucleation::WsSegmentJob::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::WsSegmentJob> opaque(mod, "WsSegmentJob", nb::type_slots(nucleation_WsSegmentJob_slots));
    opaque
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::WsSegmentJob::create)), "cell_size"_a, "closing_radius"_a, "min_cluster_blocks"_a, "source_id"_a, "snapshot_id"_a, "min_y"_a, "max_y"_a, "extracted_at"_a, "match_iou"_a, "hard_cut"_a);
}

} 