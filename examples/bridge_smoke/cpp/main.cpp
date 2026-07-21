#include <cassert>
#include <cstdint>
#include <iostream>
#include <utility>

#include "AnimationEffect.hpp"
#include "BuildAnimation.hpp"

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

    std::cout << "bridge smoke (C++) OK\n";
    return 0;
}
