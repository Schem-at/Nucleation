#include <cassert>
#include <cstdint>
#include <iostream>
#include <utility>

#include "AnimationEffect.hpp"
#include "BuildAnimation.hpp"
#include "Field.hpp"
#include "FieldDistanceKind.hpp"
#include "FieldExprBuilder.hpp"
#include "Material.hpp"
#include "VoxelPipeline.hpp"

int main() {
    auto animation = BuildAnimation::create("fluent");
    auto effect = AnimationEffect::spin_in(600.0f, 1.0f);

    auto animated = animation->with_effect(*effect).set_block(
        0, 0, 0, "minecraft:stone"
    );
    assert(animated.is_ok());
    assert(std::move(animated).ok().value() == 0);

    auto plain = animation->set_block(1, 0, 0, "minecraft:dirt");
    assert(plain.is_ok());
    assert(std::move(plain).ok().value() == 1);
    assert(animation->group_count() == 2);

    auto eb = FieldExprBuilder::create();
    assert(eb->set_bounds(-2.0f, -2.0f, -2.0f, 2.0f, 2.0f, 2.0f).is_ok());
    eb->set_distance_kind(FieldDistanceKind::Exact);
    auto field_res = eb->compile("length(p) - 2.0");
    assert(field_res.is_ok());
    auto field = std::move(field_res).ok().value();
    assert(field->eval_at(0.0f, 0.0f, 0.0f) < 0.0f);

    auto mat_res = Material::solid("minecraft:stone");
    assert(mat_res.is_ok());
    auto mat = std::move(mat_res).ok().value();
    auto pipe = VoxelPipeline::create(*field, *mat);
    auto schem_res = pipe->to_schematic("expr_sphere");
    assert(schem_res.is_ok());

    std::cout << "bridge smoke (C++) OK\n";
    return 0;
}
