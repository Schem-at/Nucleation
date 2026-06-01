"""Store + transparent-I/O bindings.

    maturin develop --features python,simulation,rendering,meshing
    pytest tests/python_store_test.py -v
"""
from __future__ import annotations

import pytest

from nucleation import Schematic, Store


class DictHandlers:
    """A Python object backing a `Store.from_callbacks` store with a dict."""

    def __init__(self):
        self.d: dict[str, bytes] = {}

    def get(self, key):           # -> bytes | None
        return self.d.get(key)

    def put(self, key, data):
        self.d[key] = bytes(data)

    def has(self, key):           # -> bool
        return key in self.d

    def delete(self, key):
        self.d.pop(key, None)

    def list(self, prefix):       # -> list[str]
        return [k for k in self.d if k.startswith(prefix)]

    def health(self):
        pass


def test_mem_store_roundtrip():
    store = Store.open("mem://")
    assert store.get("absent") is None
    store.put("a/b", b"hello")
    assert store.get("a/b") == b"hello"
    assert store.exists("a/b") is True
    assert store.list("a/") == ["a/b"]
    store.health()  # must not raise
    store.delete("a/b")
    assert store.get("a/b") is None
    assert store.exists("a/b") is False


def test_put_if_absent():
    store = Store.open("mem://")
    assert store.put_if_absent("k", b"first") is True
    assert store.put_if_absent("k", b"second") is False
    assert store.get("k") == b"first"


def test_list_paginated_keyset():
    store = Store.open("mem://")
    for i in range(5):
        store.put(f"p/{i}", b"x")
    keys, nxt = store.list_paginated("p/", None, 2)
    assert len(keys) == 2
    assert keys == sorted(keys)
    assert nxt is not None
    # walk to exhaustion covers every key exactly once
    seen, cursor = [], None
    while True:
        page, cursor = store.list_paginated("p/", cursor, 2)
        seen.extend(page)
        if cursor is None:
            break
    assert sorted(seen) == [f"p/{i}" for i in range(5)]


def test_store_from_callbacks_roundtrip():
    handlers = DictHandlers()
    store = Store.from_callbacks(handlers)
    store.put("k1", b"one")
    store.put("k2", b"two")
    assert store.get("k1") == b"one"
    assert store.exists("k2") is True
    assert sorted(store.list("")) == ["k1", "k2"]
    store.delete("k1")
    assert store.get("k1") is None
    store.health()
    # the data really lives in the Python dict
    assert handlers.d == {"k2": b"two"}


def test_from_callbacks_propagates_python_errors():
    class Broken:
        def get(self, key):
            raise RuntimeError("boom")

        def put(self, key, data):
            pass

        def has(self, key):
            return False

        def delete(self, key):
            pass

        def list(self, prefix):
            return []

        def health(self):
            pass

    store = Store.from_callbacks(Broken())
    with pytest.raises(Exception):
        store.get("anything")


def _stone() -> Schematic:
    s = Schematic("store-io")
    s.set_block(0, 0, 0, "minecraft:stone")
    s.set_block(1, 0, 0, "minecraft:dirt")
    return s


def test_transparent_io_explicit_store():
    store = Store.open("mem://")
    _stone().save(store, "build.schem")
    loaded = Schematic.open(store, "build.schem")
    assert loaded.get_block_string(0, 0, 0) == "minecraft:stone"
    assert loaded.get_block_string(1, 0, 0) == "minecraft:dirt"


def test_transparent_io_over_callback_store():
    """Schematic save/open round-trips through a Python-callback-backed store."""
    handlers = DictHandlers()
    store = Store.from_callbacks(handlers)
    _stone().save(store, "nested/build.litematic")
    assert handlers.d  # bytes landed in the Python dict
    loaded = Schematic.open(store, "nested/build.litematic")
    assert loaded.get_block_string(0, 0, 0) == "minecraft:stone"
