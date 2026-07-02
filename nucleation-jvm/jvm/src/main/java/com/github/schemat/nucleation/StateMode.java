package com.github.schemat.nucleation;

/**
 * State management for a {@link TypedCircuitExecutor} between executions.
 */
public enum StateMode {
    /** Always reset before execution (default). */
    STATELESS(0),
    /** Preserve state between executions. */
    STATEFUL(1),
    /** Caller controls {@link TypedCircuitExecutor#reset()} manually. */
    MANUAL(2);

    final int code;

    StateMode(int code) {
        this.code = code;
    }
}
