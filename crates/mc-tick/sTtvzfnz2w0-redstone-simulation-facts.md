# Copper-bulb inversion device: simulation facts extracted from the video

Source video: `sTtvzfnz2w0`  
Scope: factual technical claims made in the transcript only. Narrative, failed brainstorming, sponsor material, and personal commentary are omitted. Terminology follows the video; in particular, it calls the scheduled-tick stage the **tile tick phase**.

## Target behavior

Build a device in which:

- the comparator is **off** while the copper bulb appears **lit**;
- the comparator is **on** while the copper bulb appears **unlit**;
- the final presentation should not leave the bulb's powered-state red dot visible;
- the note-block/player input must still be able to change the bulb's logical state.

Normally, a comparator reading a copper bulb agrees with its lit state: comparator on for a lit bulb and off for an unlit bulb. The device creates the apparent inversion by changing the bulb at carefully separated points inside each game tick.

## State that must be represented

### Copper bulb

The video distinguishes two independent block states:

- **toggled/lit state** — whether the bulb is visually on;
- **powered state** — whether it is currently receiving redstone power, shown by the red dot.

A simulator must not collapse these into one Boolean. A pulse can toggle the lit state, while the red dot depends on whether power remains present at the client-visible boundary.

### Comparator

- A comparator reads the bulb's state during scheduled/tile-tick processing.
- Its result does not automatically re-evaluate after every later change in the same game tick.
- If the bulb changes after that comparator evaluation, the player can therefore receive a bulb state that disagrees with the comparator result.

### Input

- The demonstrated player input is a note block observed by an observer.
- Player actions and the observer's subsequent on/off scheduled events can enter the event order at different points.
- Correct operation depends on the parity of bulb toggles caused by the input plus the clock.

## Tick rate and required pulse rate

- Minecraft runs at **20 game ticks per second**.
- To maintain the illusion continuously, the bulb must be toggled twice per game tick: once to expose the desired visible state and once to restore the state that the comparator will read on the next cycle.
- Two transitions per 20 Hz game tick amount to a **40 Hz toggle clock**.
- Multiple clock modules can be offset by one tick so that the process occurs every game tick.

## Relevant game-loop model given by the video

The video uses this simplified phase order:

1. **Tile tick / scheduled-tick phase**
   - comparator and scheduled observer/repeater work is processed;
   - multiple operations can occur in a defined priority and scheduling order.
2. **Block changes are sent to the player**
   - this is the client-visible snapshot boundary used by the trick;
   - the player sees the bulb's lit state, comparator state, dust state, and powered red dot as they exist here.
3. **Block event phase**
   - piston actions are processed here in the video's model.
4. **Player-action scheduling must also be modeled**
   - the video separately adds a player-actions stage when explaining the note-block input;
   - an observer transition initiated by the player's note-block action is scheduled later than events already scheduled from tile-tick processing.

The transcript does not provide a complete ordering of every vanilla phase. The simulator only needs the relative boundaries used here: scheduled/tile ticks, the client-visible update boundary, block events, and player-action-induced scheduling.

## Core inversion mechanism

For one illustrated state, the video gives this sequence:

1. During the tile-tick phase, the comparator reads the bulb while the bulb is unlit, so comparator output remains off.
2. Later in the same tile-tick phase, a clock observer powers the redstone line and toggles the bulb on.
3. The comparator does not read again before the client update boundary.
4. Block changes are sent to the player. The player sees a lit bulb and an off comparator.
5. A second clock transition resets the bulb before the next comparator evaluation.

To produce the opposite apparent state, the same mechanism runs with the bulb's parity reversed: the comparator retains the result from one state while the client-visible transition exposes the other state.

## Why power must be removed before the client snapshot

An early working version toggled the bulb before the client update but left its redstone line powered at that boundary. It therefore showed the bulb's red powered-state dot.

The corrected requirement is:

- power the line and then unpower it **within the tile-tick phase**;
- both transitions must finish before block changes are sent to the player;
- at the client-visible boundary, the bulb is lit but its powered flag and redstone line are off.

## Two-clock implementation described before the final one-comparator solution

The video describes a working two-comparator implementation with two 20 Hz clocks:

- **Sand clock:** observers power through sand during the tile-tick phase.
- The sand setup uses boats aligned at a specific height so falling sand snaps instantly to a piston.
- This clock powers and unpowers the redstone line entirely within the tile-tick phase.
- **Slime/block-event clock:** slime blocks cut and reconnect the redstone line; this clock pulses during the block-event phase.

Per-cycle sequence:

1. Comparator reads the bulb and gets the pre-visible state.
2. Sand clock powers and unpowers the line in the tile-tick phase, toggling the bulb.
3. Client snapshot: opposite bulb appearance, retained comparator output, line unpowered, no red dot.
4. Block-event clock powers and unpowers the line again, restoring the bulb before the next cycle.

This is a valid intermediate mechanism, but it is not the video's final one-comparator/note-block-compatible answer.

## Scheduled-tick priority rules stated in the video

Within the tile-tick phase, the video gives the following descending priority order:

1. Repeaters facing into a repeater or comparator.
2. Repeaters turning off.
3. Repeaters turning on, and comparators facing into a repeater or comparator.
4. Everything else.

Additional ordering rule:

- When two scheduled/tile-tick operations have equal priority, execution order is determined by the order in which they were scheduled.

Effect of the second comparator:

- Adding a second comparator gives the comparator reading the copper bulb priority over the clock observer.
- The bulb comparator therefore evaluates first.
- Removing that comparator removes the priority advantage, allowing the clock observer to execute first and breaking the intended ordering.

A simulator must therefore order scheduled events by at least:

```text
(execution_tick, priority_class, scheduling_sequence)
```

where a higher-priority class executes before lower-priority work and `scheduling_sequence` preserves insertion order among equal-priority events.

## Two-tick scheduling relationship

The video repeatedly describes observer/scheduled-tick effects being queued and then executing **two ticks later**. The scheduling phase and source affect their insertion order when they eventually execute.

One attempted one-comparator design powered clock observers from pistons rather than from observers:

- comparator work was scheduled during the tile-tick phase;
- piston/block-event activity scheduled the clock observers later;
- two ticks later, equal-priority scheduled operations executed comparator first and clock observers afterward because of their scheduling order.

This fixed the comparator-before-clock relation but broke note-block input parity.

## Note-block input and toggle parity

The bulb's final state is determined by whether the number of toggle transitions is odd or even:

- odd number of toggles: bulb parity changes;
- even number of toggles: bulb parity is preserved.

For the working two-comparator ordering described in the video:

1. A clock observer is scheduled from the tile-tick phase.
2. Comparator is scheduled and moves ahead because of its priority.
3. Sand begins falling.
4. In the player-action stage, the note-block observer is scheduled to turn on.
5. Two ticks later, the bulb transitions on, off, on in the stated order while the next tick's event list is being constructed.
6. The observer's turn-off event is scheduled from its own tile-tick turn-on, not directly from the original player action, so it occupies a different position in the following list.
7. Relative to the clock's normal single toggle, the note-block input contributes the parity change required to toggle the bulb logically.

For the attempted one-comparator/block-event-scheduled clock:

- moving the clock after the comparator also moved it after the note-block observer's turn-off event;
- the bulb then toggled three times in the relevant sequence;
- this failed to change the intended parity in that arrangement, so the note-block input could not operate the device correctly.

The simulator must preserve both observer edges as separately scheduled events and must not model an observer pulse as one atomic state change.

## Final solution reported by the video

The final solution differs from the earlier tile-tick-plus-block-event design:

- it uses a **40 Hz clock entirely within the tile-tick phase**;
- it does **not** rely on a block-event-phase clock for the final timing;
- it consists of two clock sections offset from one another;
- one set of falling-sand updates is caused by observers;
- another set is caused by note blocks;
- observers and note blocks update their respective sand at slightly different times;
- that timing difference produces the two tile-tick-phase transitions needed to turn a 20 Hz mechanism into an effective 40 Hz bulb-toggle clock;
- the note block updates sand so that it falls sooner/differently in the tile-tick ordering;
- the final answer is therefore achieved by ordering both pulses inside scheduled/tile-tick processing rather than splitting them across tile ticks and block events.

## Minimum simulator behavior needed to test the mechanism

The simulator needs all of the following; a game-tick-only Boolean redstone model is insufficient:

1. **Sub-tick phases**
   - scheduled/tile-tick execution;
   - client-visible block-change snapshot;
   - block events/piston actions;
   - player-action-induced scheduling.
2. **Stable scheduled-event ordering**
   - execution tick;
   - priority class;
   - insertion/scheduling order for equal priority.
3. **Separate copper-bulb flags**
   - lit/toggled;
   - currently powered.
4. **Edge-sensitive bulb behavior**
   - each qualifying power pulse toggles lit parity;
   - multiple pulses in one game tick must all be retained in order.
5. **Comparator sampling**
   - comparator samples at its scheduled execution point;
   - it does not continuously follow later same-tick bulb changes.
6. **Observer pulse decomposition**
   - observer turn-on and turn-off are separate scheduled events;
   - their scheduling source and insertion order are preserved.
7. **Piston/block events**
   - piston motion can schedule later tile-tick work;
   - scheduling from block events occurs after tile-tick-scheduled work for equal-priority execution at the later target tick.
8. **Falling-sand timing**
   - observer-triggered and note-block-triggered sand updates can enter scheduled processing at different positions;
   - the final design depends on that difference.
9. **Client snapshot semantics**
   - visible bulb, red dot, comparator, and dust states are sampled at the block-change-send boundary, not simply at the end of the whole game tick.
10. **Parity tracing**
    - log every bulb toggle in execution order;
    - assert the bulb has the opposite visible state from the comparator at the client boundary;
    - assert the clock restores the state needed for the next comparator sample;
    - assert a note-block input changes long-term bulb parity.

## Suggested event trace schema

```text
Event {
  execution_tick
  phase                 // TILE_TICK, CLIENT_SNAPSHOT, BLOCK_EVENT, PLAYER_ACTION
  priority_class        // one of the scheduled-tick priority tiers above
  scheduling_sequence   // monotonic insertion index
  source_block
  target_block
  edge                   // ON or OFF where applicable
  action                 // SAMPLE, POWER, UNPOWER, TOGGLE, MOVE, UPDATE_SAND, SCHEDULE
}
```

At every client snapshot, record:

```text
bulb.lit
bulb.powered
comparator.output
redstone_line.powered
```

Success conditions:

```text
comparator.output == !bulb.lit
bulb.powered == false
redstone_line.powered == false
```

and, after a note-block input, the device's persistent bulb parity must change while those snapshot invariants continue to hold.

## Information the video does not provide

The transcript alone is insufficient to reconstruct the exact build geometry. It does not provide:

- Minecraft edition or exact version;
- block coordinates, orientations, or full wiring layout;
- exact delay values for every repeater/observer/piston/sand transition beyond the stated two-tick scheduled relationship;
- exact vanilla implementation names or numeric priority constants;
- a frame-by-frame event list for the final compact build;
- the exact positional mechanism by which observer-updated sand and note-block-updated sand receive different timing;
- chunk, location, or update-order dependencies.

Those details must come from the video frames/world download or independent version-specific Minecraft research. This file deliberately does not invent them.
