# Builds the nucleation Python extension from the Diplomat-generated nanobind
# sources in src/, statically linking the Rust core. Invoked via scikit-build-core
# (`pip install bindings/python`), not directly.
cmake_minimum_required(VERSION 3.15...3.27)
project(nucleation_ext LANGUAGES CXX)

if (NOT SKBUILD)
  message(WARNING "Build via 'pip install ${CMAKE_CURRENT_SOURCE_DIR}' (scikit-build-core), not raw CMake.")
endif()

find_package(Python 3.9
  REQUIRED COMPONENTS Interpreter Development.Module
  OPTIONAL_COMPONENTS Development.SABIModule)

# nanobind from the pip build env (declared in pyproject build-system.requires)
execute_process(
  COMMAND "${Python_EXECUTABLE}" -m nanobind --cmake_dir
  OUTPUT_STRIP_TRAILING_WHITESPACE OUTPUT_VARIABLE nanobind_ROOT)
find_package(nanobind CONFIG REQUIRED)

set(REPO_ROOT ${CMAKE_CURRENT_SOURCE_DIR}/../..)
set(RUST_LIB ${REPO_ROOT}/target/release/${CMAKE_STATIC_LIBRARY_PREFIX}nucleation${CMAKE_STATIC_LIBRARY_SUFFIX})

# NUCLEATION_FEATURES lets CI choose the feature set baked into the wheel.
if(NOT DEFINED ENV{NUCLEATION_FEATURES})
  set(NUCLEATION_FEATURES "bridge-full")
else()
  set(NUCLEATION_FEATURES $ENV{NUCLEATION_FEATURES})
endif()

add_custom_command(OUTPUT ${RUST_LIB}
  COMMAND cargo build --release --lib --features ${NUCLEATION_FEATURES}
  WORKING_DIRECTORY ${REPO_ROOT})
add_custom_target(nucleation_rust_target DEPENDS ${RUST_LIB})
add_library(nucleation_rust STATIC IMPORTED GLOBAL)
add_dependencies(nucleation_rust nucleation_rust_target)
set_target_properties(nucleation_rust PROPERTIES IMPORTED_LOCATION ${RUST_LIB})
if(WIN32)
  target_link_libraries(nucleation_rust INTERFACE kernel32.lib advapi32.lib ntdll.lib userenv.lib ws2_32.lib dbghelp.lib)
endif()
if(APPLE)
  # wgpu (rendering feature) needs the system frameworks
  # Foundation: wgpu_hal's Metal layer observer uses NSKeyValueChange* symbols
  target_link_libraries(nucleation_rust INTERFACE "-framework Metal" "-framework QuartzCore" "-framework CoreFoundation" "-framework Foundation")
endif()

file(GLOB_RECURSE SUBMODULE_FILES src/sub_modules/*.cpp)
nanobind_add_module(
  nucleation
  STABLE_ABI
  NB_STATIC
  src/nucleation_ext.cpp
  ${SUBMODULE_FILES}
)
target_include_directories(nucleation PUBLIC "src/include")
# The Diplomat-generated dealloc shim reaches into nanobind's internal
# nb_inst struct (the reason for the ==2.12.0 pin), so the generated
# sources compile against nb_internals.h, which needs nanobind's bundled
# robin_map headers — private to the nanobind target, so add them here.
execute_process(
  COMMAND "${Python_EXECUTABLE}" -c "import nanobind, os; print(os.path.dirname(nanobind.__file__))"
  OUTPUT_STRIP_TRAILING_WHITESPACE OUTPUT_VARIABLE NB_PKG_DIR)
target_include_directories(nucleation PRIVATE "${NB_PKG_DIR}/ext/robin_map/include")
target_compile_features(nucleation PUBLIC cxx_std_20)
target_link_libraries(nucleation PUBLIC nucleation_rust ${Python3_SABI_LIBRARY})

install(TARGETS nucleation LIBRARY DESTINATION .)
