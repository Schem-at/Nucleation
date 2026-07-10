<?php
// End-to-end smoke test for the generated PHP bindings (same coverage as ../c).
require __DIR__ . '/../../../bindings/php/index.php';

use Stencil\Autostack;
use Stencil\DefinitionRegion;
use Stencil\Diff;
use Stencil\DiplomatError;
use Stencil\Lib;
use Stencil\Schematic;
use Stencil\SchematicBuilder;
use Stencil\SchematicRegions;
use Stencil\Store;

$lib = getenv('NUCLEATION_LIBRARY_PATH');
if ($lib === false) {
    $ext = PHP_OS_FAMILY === 'Darwin' ? 'dylib' : 'so';
    $lib = __DIR__ . "/../../../target/release/libnucleation.$ext";
}
Lib::init($lib);

function expect(bool $cond, string $what): void {
    if (!$cond) {
        throw new \Exception("FAILED: $what");
    }
}

// --- schematic: create/set/get + error path ---
$s = Schematic::create('smoke');
expect($s->setBlock(1, 2, 3, 'minecraft:stone') === true, 'setBlock places');
expect($s->getBlockName(1, 2, 3) === 'minecraft:stone', 'getBlockName reads back');
try {
    $s->getBlockName(40, 40, 40);
    expect(false, 'expected NotFound');
} catch (DiplomatError $e) {
    expect($e->errorName === 'NotFound', 'empty position raises NotFound');
}

// --- save/load roundtrip ---
$path = sys_get_temp_dir() . '/bridge_smoke_' . getmypid() . '.litematic';
$s->saveToFile($path);
$loaded = Schematic::loadFromFile($path);
unlink($path);

// --- builder: consuming build + AlreadyConsumed ---
$b = SchematicBuilder::create();
$b->map('s', 'minecraft:stone');
$b->layer('["s"]');
$built = $b->build();
try {
    $b->build();
    expect(false, 'expected AlreadyConsumed');
} catch (DiplomatError $e) {
    expect($e->errorName === 'AlreadyConsumed', 'second build raises AlreadyConsumed');
}

// --- diff: distance between original and its saved copy is 0 ---
$diff = Diff::compute($s, $loaded, 'exact');
expect($diff->distance() === 0, 'roundtripped schematic has diff distance 0');

// --- autostack: JSON out ---
$json = Autostack::detectStructures($s);
expect($json !== '' && $json[0] === '[', 'detectStructures writes a JSON array');

// --- definition regions ---
$r = DefinitionRegion::create();
$r->addPoint(1, 2, 3);
SchematicRegions::add($s, 'io', $r);
expect(SchematicRegions::namesJson($s) === '["io"]', 'region name registered');

// --- store: mem:// save/open roundtrip ---
$store = Store::open('mem://');
$store->saveSchematic($s, 'k1.litematic', '');
$reopened = $store->openSchematic('k1.litematic');
expect($reopened->getBlockName(1, 2, 3) === 'minecraft:stone', 'store roundtrip preserves block');

echo "bridge smoke (PHP) OK\n";
