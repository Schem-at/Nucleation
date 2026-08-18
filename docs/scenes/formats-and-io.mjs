// Nucleation owns this scene. Kineglyph supplies only the rendering primitives.
import { defineScene, drawEdge, flow, material, reveal, caption, code, eyebrow, heading, motif, row, stack, title, timeline, } from "kineglyph";

function eventName(key) {
    return `FOCUS_${key.replaceAll("-", "_").toUpperCase()}`;
}
function focusModel(id, specs, overview) {
    const transitions = {
        ...Object.fromEntries(specs.map((spec) => [eventName(spec.key), spec.key])),
        RESET: "overview",
    };
    const signals = {
        detailTitle: {
            match: { var: "focus" },
            cases: Object.fromEntries(specs.map((spec) => [spec.key, spec.title])),
            default: overview.title,
        },
        detailBody: {
            match: { var: "focus" },
            cases: Object.fromEntries(specs.map((spec) => [spec.key, spec.body])),
            default: overview.body,
        },
    };
    for (const spec of specs) {
        signals[`${spec.key}Focus`] = {
            when: { var: "focus", op: "eq", value: spec.key },
            then: 1,
            else: 0,
        };
        signals[`${spec.key}Dim`] = {
            when: { var: "focus", op: "in", value: ["none", spec.key] },
            then: 1,
            else: 0.32,
        };
    }
    return {
        machine: {
            id,
            initial: "overview",
            variables: { focus: "none" },
            states: {
                overview: {
                    entry: [
                        { type: "set", var: "focus", value: "none" },
                        { type: "select", node: null },
                    ],
                    on: transitions,
                },
                ...Object.fromEntries(specs.map((spec) => [
                    spec.key,
                    {
                        entry: [
                            { type: "set", var: "focus", value: spec.key },
                            { type: "select", node: spec.node },
                        ],
                        on: transitions,
                    },
                ])),
            },
            signals,
        },
        controls: [
            ...specs.map((spec) => ({
                id: `${id}-${spec.key}`,
                label: spec.label,
                event: eventName(spec.key),
                activeWhen: { var: "focus", op: "eq", value: spec.key },
            })),
            { id: `${id}-reset`, kind: "reset", label: "Show all" },
        ],
    };
}
function interactive(spec) {
    return {
        interactive: true,
        onActivate: eventName(spec.key),
        bind: { highlight: `${spec.key}Focus`, opacity: `${spec.key}Dim` },
        label: spec.title,
        description: spec.body,
    };
}
function detailRail(id) {
    return row(`${id}-detail`, [
        motif(`${id}-detail-mark`, "target", { tone: "accent", size: 18 }),
        stack(`${id}-detail-copy`, [
            heading(`${id}-detail-title`, "", { bind: { text: "detailTitle" }, width: "fill" }),
            caption(`${id}-detail-body`, "", {
                bind: { text: "detailBody" },
                maxLines: { wide: 2, compact: 3, narrow: 5 },
                width: "fill",
            }),
        ], { gap: 2, width: "fill" }),
    ], {
        gap: 12,
        align: "center",
        padding: [11, 14],
        frame: material("inset", { radius: 6 }),
        width: "fill",
    });
}
function artboard(id, label, headline, visual, detail = true) {
    return stack(`${id}-root`, [
        row(`${id}-head`, [
            stack(`${id}-head-copy`, [eyebrow(`${id}-label`, label, { tone: "accent" }), title(`${id}-title`, headline)], { gap: 3, width: "fill" }),
            code(`${id}-stamp`, "NUCLEATION / KINEGLYPH", {
                tone: "muted",
                hidden: { wide: false, compact: true },
            }),
        ], { align: "end", justify: "between", width: "fill" }),
        visual,
        ...(detail ? [detailRail(id)] : []),
    ], {
        gap: { wide: 20, compact: 16, narrow: 14 },
        padding: { wide: 24, compact: 20, narrow: 16 },
        frame: material("flat"),
        width: "fill",
    });
}
function sceneTimeline(nodes, edges = []) {
    const nodeTracks = nodes.flatMap((node, index) => reveal(node, 80 + index * 120, 440 + index * 120, { offset: 8, scale: 0.985 }));
    const edgeBase = 360 + nodes.length * 80;
    const edgeTracks = edges.flatMap((edge, index) => [
        ...drawEdge(edge, edgeBase + index * 140, edgeBase + 360 + index * 140),
        flow(edge, edgeBase + 360 + index * 140),
    ]);
    return timeline([...nodeTracks, ...edgeTracks], Math.max(1_200, edgeBase + edges.length * 140 + 480));
}

// Formats and I/O ----------------------------------------------------------------------------
const FORMAT_FOCUS = [
    {
        key: "detect",
        label: "Detect",
        node: "format-inputs",
        title: "Content detection precedes parsing",
        body: "Bytes and container structure select the parser; a filename extension is only a hint.",
    },
    {
        key: "model",
        label: "Model",
        node: "format-model",
        title: "Every parser converges on one editable model",
        body: "Blocks, entities, metadata, regions, and bounds use the same in-memory schematic contract.",
    },
    {
        key: "export",
        label: "Export",
        node: "format-outputs",
        title: "Export is an explicit destination choice",
        body: "Structure, snapshot, and world formats keep their own capabilities and loss boundaries visible.",
    },
];
const formatFocus = focusModel("format-hub", FORMAT_FOCUS, {
    title: "Many containers, one model, explicit destinations",
    body: "Nucleation isolates format quirks at the edge so edits and analysis operate on one schematic representation.",
});
function formatChip(id, text, tone) {
    return stack(id, [code(`${id}-text`, text, { tone, align: "center", width: "fill" })], {
        padding: [10, 8],
        frame: material("raised", { radius: 4 }),
        width: "fill",
    });
}
const formatInputs = {
    ...stack("format-inputs", [
        eyebrow("format-inputs-label", "DETECT + PARSE"),
        {
            id: "format-input-grid",
            type: "group",
            layout: "grid",
            columns: { wide: 3, narrow: 2 },
            gap: 7,
            width: "fill",
            children: [".schem", ".litematic", ".mcstructure", ".nusn", ".snbt", "world/"].map((name, index) => formatChip(`format-in-${index}`, name, index % 2 === 0 ? "info" : "accent")),
        },
    ], { gap: 10, padding: 16, frame: material("raised", { radius: 8 }), width: "fill" }),
    ...interactive(FORMAT_FOCUS[0]),
};
const formatModel = {
    ...stack("format-model", [
        motif("format-model-cube", "cube", { tone: "accent", size: 84 }),
        title("format-model-title", "Schematic", { align: "center", width: "fill" }),
        caption("format-model-note", "blocks · entities · metadata · regions", {
            align: "center",
            width: "fill",
            maxLines: 2,
        }),
    ], {
        gap: 8,
        align: "center",
        padding: [24, 18],
        frame: material("glass", { radius: 12 }),
        width: "fill",
    }),
    ...interactive(FORMAT_FOCUS[1]),
};
const formatOutputs = {
    ...stack("format-outputs", [
        eyebrow("format-outputs-label", "EXPORT"),
        ...[
            ["STRUCTURE", ".schem · .litematic"],
            ["SNAPSHOT", ".nusn · .snbt"],
            ["WORLD", "region · chunk"],
        ].map(([name, note], index) => row(`format-out-${index}`, [
            heading(`format-out-${index}-name`, name ?? ""),
            code(`format-out-${index}-note`, note ?? "", {
                tone: index === 2 ? "success" : "accent",
                align: "end",
                width: "fill",
            }),
        ], { gap: 12, align: "center", width: "fill" })),
    ], { gap: 12, padding: 16, frame: material("raised", { radius: 8 }), width: "fill" }),
    ...interactive(FORMAT_FOCUS[2]),
};
const formatEdges = [
    {
        id: "format-read",
        from: { node: "format-inputs", side: { wide: "right", compact: "bottom" } },
        to: { node: "format-model", side: { wide: "left", compact: "top" } },
        route: "straight",
        head: "arrow",
        tail: "dot",
        stroke: "flow",
        label: "detect",
        packets: { count: 2, period: 1700 },
    },
    {
        id: "format-write",
        from: { node: "format-model", side: { wide: "right", compact: "bottom" } },
        to: { node: "format-outputs", side: { wide: "left", compact: "top" } },
        route: "orthogonal",
        head: "bar",
        stroke: "dashed",
        packets: { count: 2, period: 1700 },
    },
];

export const formatsAndIoScene = defineScene({
    schemaVersion: 2,
    id: "formats-and-io",
    title: "Formats and I/O",
    description: "Container detection and parsers converge on one editable schematic model before explicit export branches out again.",
    breakpoints: { wide: 900, compact: 520 },
    background: "canvas",
    root: artboard("format", "FORMATS + I/O", "Format quirks stay at the boundary.", {
        id: "format-map",
        type: "group",
        layout: { wide: "row", compact: "stack" },
        gap: { wide: 36, compact: 34 },
        align: "stretch",
        width: "fill",
        children: [formatInputs, formatModel, formatOutputs],
    }),
    edges: formatEdges,
    machine: formatFocus.machine,
    controls: formatFocus.controls,
    timeline: sceneTimeline(["format-inputs", "format-model", "format-outputs"], formatEdges.map((edge) => edge.id)),
    metadata: { source: "formats-and-io/format-pipeline.svg", revision: 2 },
});

export default formatsAndIoScene;
