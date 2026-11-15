/** Interpolate the last completed tick, rather than predicting the next one.
 * Piston block entities have already advanced half a block when a step returns;
 * drawing completedTick + fraction skips the first half of every new stroke. */
export class PresentationClock {
  private simulation: unknown;
  private completed = 0;
  private shown = 0;

  sample(simulation: unknown, completed: number, fraction: number, running: boolean): number {
    if (simulation !== this.simulation || completed < this.completed) {
      this.simulation = simulation;
      this.completed = completed;
      this.shown = running ? Math.max(0, completed - 1) : completed;
    }
    if (running) {
      const interpolated = Math.max(0, completed - 1 + Math.min(1, Math.max(0, fraction)));
      this.shown = Math.max(this.shown, interpolated);
    } else if (completed !== this.completed) {
      // An explicit step/scrub shows its exact completed state. Pausing alone
      // holds the fractional position, so neither pause nor resume runs back.
      this.shown = completed;
    }
    this.completed = completed;
    return this.shown;
  }
}
