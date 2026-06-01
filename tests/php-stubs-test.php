<?php
/**
 * Simple test to validate PHP stubs
 */

require_once __DIR__ . '/../nucleation-stubs.php';

// Test that all classes and functions are defined in stubs
$functions = [
    'nucleation_hello',
    'nucleation_version',
    'nucleation_detect_format',
    'nucleation_convert_format',
    'nucleation_create_schematic',
    'nucleation_load_from_file',
    'nucleation_save_to_file'
];

$classes = [
    'Nucleation\\Schematic',
    'Nucleation\\Diff'
];

// Diff + fingerprint methods on the Schematic / Diff classes
$methods = [
    'Nucleation\\Schematic' => [
        'fingerprint',
        'signature',
        'footprintDistance',
        'isDuplicateOf',
        'diff',
    ],
    'Nucleation\\Diff' => [
        'fromJson',
        'distance',
        'support',
        'toJson',
        'summaryJson',
        'added',
        'removed',
        'changed',
        'swapped',
        'markers',
        'toOverlayGlb',
    ],
];

echo "Testing PHP stubs...\n";

foreach ($functions as $func) {
    if (!function_exists($func)) {
        echo "❌ Function $func not found in stubs\n";
        exit(1);
    }
}

foreach ($classes as $class) {
    if (!class_exists($class)) {
        echo "❌ Class $class not found in stubs\n";
        exit(1);
    }
}

foreach ($methods as $class => $classMethods) {
    foreach ($classMethods as $method) {
        if (!method_exists($class, $method)) {
            echo "❌ Method $class::$method not found in stubs\n";
            exit(1);
        }
    }
}

echo "✅ All PHP stubs validated successfully\n";