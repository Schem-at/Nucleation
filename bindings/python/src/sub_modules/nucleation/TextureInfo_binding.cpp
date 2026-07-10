#include "diplomat_nanobind_common.hpp"


#include "TextureInfo.hpp"

namespace nucleation {
void add_TextureInfo_binding(nb::module_ mod) {
    nb::class_<nucleation::TextureInfo> st(mod, "TextureInfo");
    st
        .def(nb::init<>())
        .def(nb::init<uint32_t, uint32_t, bool, uint32_t>(), "width"_a.none(),  "height"_a.none(),  "animated"_a.none(),  "frame_count"_a.none())
        .def_rw("width", &nucleation::TextureInfo::width)
        .def_rw("height", &nucleation::TextureInfo::height)
        .def_rw("animated", &nucleation::TextureInfo::animated)
        .def_rw("frame_count", &nucleation::TextureInfo::frame_count);
}

} 