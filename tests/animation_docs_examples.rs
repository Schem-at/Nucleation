//! Executable Rust source embedded in docs/features/animation.md.

#[test]
fn records_and_samples_the_engine_walkthrough() -> Result<(), String> {
    // --8<-- [start:record]
    use nucleation::animation::{presets, BuildAnimation};

    let mut animation = BuildAnimation::new("engine_walkthrough");
    animation.set_step_ms(300.0);

    // Calls inside a group share one target, effect, and start time.
    animation.begin_group(None)?;
    for x in 0..5 {
        animation.set_block(x, 0, 0, "minecraft:stone_bricks")?;
    }
    animation.end_group()?;

    // with_effect changes exactly the next recorded target.
    animation
        .with_effect(presets::spin_in(700.0, 1.0))
        .set_block(4, 1, 0, "minecraft:diamond_block")?;
    animation.set_block(0, 1, 0, "minecraft:furnace[facing=south]")?;

    // The camera is another target on the same clock.
    animation.animate_camera(presets::turntable(3_000.0), 0.0);
    // --8<-- [end:record]

    // --8<-- [start:sample]
    let frame = animation.frame_at(450.0);
    println!("{}", animation.groups().len()); // 3
    println!("{}", animation.duration_ms()); // 3000, set by the camera track
    println!("{}", frame.poses.len()); // 3 group poses at t=450 ms

    // --8<-- [end:sample]

    assert_eq!(animation.groups().len(), 3);
    assert_eq!(animation.duration_ms(), 3_000.0);
    assert_eq!(frame.poses.len(), 3);
    Ok(())
}
