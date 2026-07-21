#!/usr/bin/env python3
import importlib.util
import pathlib
import sys
import unittest

MODULE_PATH = pathlib.Path(__file__).with_name("compose_code.py")
spec = importlib.util.spec_from_file_location("compose_code", MODULE_PATH)
assert spec is not None and spec.loader is not None
compose_code = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = compose_code
spec.loader.exec_module(compose_code)


class SyntaxTokenTests(unittest.TestCase):
    def test_tokenises_receiver_method_numbers_and_string(self):
        spans = compose_code.syntax_spans(
            's.set_block(-1, 2, 0, "minecraft:chiseled_stone_bricks")'
        )
        by_kind = {}
        for text, kind in spans:
            by_kind.setdefault(kind, "")
            by_kind[kind] += text

        self.assertIn("set_block", by_kind["call"])
        self.assertIn("-1", by_kind["number"])
        self.assertIn('"minecraft:chiseled_stone_bricks"', by_kind["string"])
        self.assertIn("s.", by_kind[None])


class LayoutTests(unittest.TestCase):
    def test_layout_expands_for_code_without_crowding_the_scene(self):
        code = [
            's = Schematic("signal_gate")',
            's.set_block(-1, 2, 0, "minecraft:chiseled_stone_bricks")',
        ]
        layout = compose_code.calculate_layout(code, anim_w=420, anim_h=420)

        self.assertGreaterEqual(layout.panel_w, 590)
        self.assertEqual(layout.scene_w, 420)
        self.assertGreaterEqual(layout.width, layout.panel_w + layout.scene_w)
        self.assertGreaterEqual(layout.height, 468)

    def test_svg_has_editor_chrome_line_numbers_and_scene_divider(self):
        spec = {
            "title": "Build a signal gate",
            "filename": "signal_gate.py",
            "code": ['s = Schematic("signal_gate")', 's.save("gate.schem")'],
            "anim_w": 420,
            "anim_h": 420,
            "active": [0],
        }
        svg = compose_code.build_svg(spec, "data:image/png;base64,AAAA", active=1)

        self.assertIn("signal_gate.py", svg)
        self.assertIn('data-role="window-dot"', svg)
        self.assertIn('data-role="line-number"', svg)
        self.assertIn('data-role="scene-divider"', svg)
        self.assertIn('data-role="active-line"', svg)
        self.assertNotIn("LIVE RENDER", svg)
        self.assertNotIn("#1f6feb", svg)


if __name__ == "__main__":
    unittest.main()
