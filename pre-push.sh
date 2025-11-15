#!/bin/bash
# Pre-push verification script
# Run this before pushing to catch issues that CI would catch

set -e

echo "=== Pre-Push Verification ==="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track overall status
OVERALL_STATUS=0

# Function to run a check
run_check() {
    local name="$1"
    local cmd="$2"
    
    echo -e "${YELLOW}Checking:${NC} $name"
    if eval "$cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} $name passed"
    else
        echo -e "${RED}✗${NC} $name FAILED"
        OVERALL_STATUS=1
    fi
    echo ""
}

# 1. Format check
echo -e "${YELLOW}Running cargo fmt check...${NC}"
if cargo fmt -- --check 2>&1 | grep -q "Diff"; then
    echo -e "${RED}✗${NC} Code is not formatted. Run: cargo fmt"
    OVERALL_STATUS=1
else
    echo -e "${GREEN}✓${NC} Code is formatted"
fi
echo ""

# 2. Build checks
run_check "Build (default features)" "cargo build --release"
run_check "Build (simulation feature)" "cargo build --release --features simulation"
run_check "Build (WASM + simulation)" "cargo build --release --target wasm32-unknown-unknown --features wasm,simulation"
run_check "Build (Python + simulation)" "cargo build --release --features python,simulation"

# 3. Test checks
run_check "Tests (default)" "cargo test"
run_check "Tests (simulation)" "cargo test --features simulation"
run_check "Tests (Insign IO integration)" "cargo test --lib --features simulation typed_executor::insign_io"

# 4. Python build check (if maturin is available)
if command -v maturin &> /dev/null; then
    echo -e "${YELLOW}Checking:${NC} Python build with maturin"
    if maturin build --features python,simulation > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} Python build with maturin passed"
    else
        echo -e "${RED}✗${NC} Python build with maturin FAILED"
        echo "Run: maturin build --features python,simulation"
        OVERALL_STATUS=1
    fi
    echo ""
else
    echo -e "${YELLOW}⚠${NC}  Maturin not installed, skipping Python build check"
    echo "   Install with: pip install maturin"
    echo ""
fi

# 5. WASM build check
if [ -f "./build-wasm.sh" ]; then
    echo -e "${YELLOW}Checking:${NC} WASM build script"
    if ./build-wasm.sh > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} WASM build script passed"
    else
        echo -e "${RED}✗${NC} WASM build script FAILED"
        OVERALL_STATUS=1
    fi
    echo ""
fi

# 6. WASM tests check
if [ -f "./tests/node_wasm_test.js" ]; then
    echo -e "${YELLOW}Checking:${NC} WASM JavaScript tests"
    if node tests/node_wasm_test.js > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} WASM JavaScript tests passed"
    else
        echo -e "${RED}✗${NC} WASM JavaScript tests FAILED"
        OVERALL_STATUS=1
    fi
    echo ""
fi

# 7. Check for version consistency
echo -e "${YELLOW}Checking:${NC} Version consistency"
CARGO_VERSION=$(grep -m1 'version = ' Cargo.toml | cut -d '"' -f2)
PYPROJECT_VERSION=$(grep -m1 'version = ' pyproject.toml | cut -d '"' -f2)

if [ "$CARGO_VERSION" = "$PYPROJECT_VERSION" ]; then
    echo -e "${GREEN}✓${NC} Versions match: $CARGO_VERSION"
else
    echo -e "${RED}✗${NC} Version mismatch!"
    echo "   Cargo.toml:     $CARGO_VERSION"
    echo "   pyproject.toml: $PYPROJECT_VERSION"
    OVERALL_STATUS=1
fi
echo ""

# Final summary
echo "=================================="
if [ $OVERALL_STATUS -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    echo "Ready to push."
    exit 0
else
    echo -e "${RED}✗ Some checks failed!${NC}"
    echo "Please fix the issues above before pushing."
    exit 1
fi
