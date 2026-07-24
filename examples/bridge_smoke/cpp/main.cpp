#include <cassert>
#include <cstdint>
#include <iostream>
#include <utility>

#include "AnimationEffect.hpp"
#include "BuildAnimation.hpp"
#include "FieldProgramBinaryOp.hpp"
#include "FieldProgramBuilder.hpp"
#include "FieldProgramDistanceKind.hpp"
#include "FieldProgramUnaryOp.hpp"
#include "FieldProgramValueType.hpp"
#include "Sdf.hpp"

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

    auto program_builder = FieldProgramBuilder::create();
    auto distance_result = program_builder->add_slot(FieldProgramValueType::Scalar);
    assert(distance_result.is_ok());
    const auto distance = std::move(distance_result).ok().value();
    assert(program_builder->set_output(distance).is_ok());
    assert(program_builder->set_bounds(-2.0f, -2.0f, -2.0f, 2.0f, 2.0f, 2.0f).is_ok());
    assert(program_builder->set_distance_kind(FieldProgramDistanceKind::Exact).is_ok());
    assert(program_builder->push_pos().is_ok());
    assert(program_builder->unary_op(FieldProgramUnaryOp::Length).is_ok());
    assert(program_builder->push_const_scalar(2.0f).is_ok());
    assert(program_builder->binary_op(FieldProgramBinaryOp::Sub).is_ok());
    assert(program_builder->store_local(distance).is_ok());
    auto program_result = program_builder->build();
    assert(program_result.is_ok());
    auto program = std::move(program_result).ok().value();
    auto program_sdf = Sdf::from_program(*program);
    assert(program_sdf->eval_at(0.0f, 0.0f, 0.0f) < 0.0f);

    std::cout << "bridge smoke (C++) OK\n";
    return 0;
}
