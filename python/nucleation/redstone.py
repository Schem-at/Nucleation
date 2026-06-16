"""Helpers for working with extracted redstone graphs.

The native :class:`RedstoneGraph` exposes ``nodes`` / ``edges`` as plain lists
of dicts (json / pandas friendly). :func:`to_networkx` converts that into a
``networkx.DiGraph`` for arbitrary graph analysis.
"""

from __future__ import annotations

from typing import Any


def to_networkx(graph: Any) -> Any:
    """Build a ``networkx.DiGraph`` from a ``RedstoneGraph``.

    Each node id becomes a graph node; remaining node-dict keys become node
    attributes. Each edge becomes a directed ``source -> target`` edge with
    ``kind`` and ``strength`` attributes.

    Requires the optional ``networkx`` dependency.
    """
    try:
        import networkx as nx
    except ImportError as e:  # pragma: no cover - optional dependency
        raise ImportError(
            "to_networkx() requires networkx: pip install networkx"
        ) from e

    g = nx.DiGraph()
    for n in graph.nodes:
        # graph.nodes returns a freshly-built dict each access, so mutating is safe.
        nid = n.pop("id")
        g.add_node(nid, **n)
    for e in graph.edges:
        g.add_edge(
            e["source"],
            e["target"],
            kind=e["kind"],
            strength=e["strength"],
        )
    return g
