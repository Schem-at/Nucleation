import base64
import json
import unittest
from player_head_skins import profile_urls, skin_hash, validate_png


class Skins(unittest.TestCase):
    def test_saved_legacy_and_current_profiles(self):
        url = 'http://textures.minecraft.net/texture/' + 'ab'*32
        value = base64.b64encode(json.dumps({'textures':{'SKIN':{'url':url}}}).encode()).decode()
        legacy = {'SkullOwner':{'Properties':{'textures':[{'Value':value}]}}}
        current = {'profile':{'properties':[{'name':'textures','value':value}]}}
        self.assertEqual(profile_urls(legacy), {url.replace('http:', 'https:')})
        self.assertEqual(profile_urls(current), profile_urls(legacy))
        current['profile']['texture'] = 'minecraft:custom'
        self.assertEqual(profile_urls(current), set(), 'pack overrides should not download a different skin')

    def test_no_arbitrary_urls_or_path_traversal(self):
        for url in ['http://localhost/texture/abc', 'file:///etc/passwd',
                    'https://textures.minecraft.net.evil/texture/abc',
                    'https://user@textures.minecraft.net/texture/abc',
                    'https://textures.minecraft.net/texture/../abc',
                    'https://textures.minecraft.net:123/texture/abc',
                    'https://textures.minecraft.net/texture/abc?redirect=elsewhere']:
            with self.subTest(url=url), self.assertRaises(ValueError): skin_hash(url)

    def test_missing_and_malformed_saved_textures_fall_back(self):
        self.assertEqual(profile_urls({'profile':{'name':'UnresolvedName'}}), set())
        self.assertEqual(profile_urls({'SkullOwner':{'Properties':{'textures':[{'Value':'not base64'}]}}}), set())
        with self.assertRaises(ValueError): validate_png(b'<html>failure</html>')

if __name__ == '__main__': unittest.main()
