#ifndef StoreIo_H
#define StoreIo_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "StoreIo.d.h"






typedef struct StoreIo_open_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} StoreIo_open_result;
StoreIo_open_result StoreIo_open(DiplomatStringView uri);

typedef struct StoreIo_save_result {union { NucleationError err;}; bool is_ok;} StoreIo_save_result;
StoreIo_save_result StoreIo_save(const Schematic* schematic, DiplomatStringView uri, DiplomatStringView version);

typedef struct StoreIo_export_settings_schema_result {union { NucleationError err;}; bool is_ok;} StoreIo_export_settings_schema_result;
StoreIo_export_settings_schema_result StoreIo_export_settings_schema(DiplomatStringView format, DiplomatWrite* write);

typedef struct StoreIo_import_settings_schema_result {union { NucleationError err;}; bool is_ok;} StoreIo_import_settings_schema_result;
StoreIo_import_settings_schema_result StoreIo_import_settings_schema(DiplomatStringView format, DiplomatWrite* write);

typedef struct StoreIo_supported_import_formats_result {union { NucleationError err;}; bool is_ok;} StoreIo_supported_import_formats_result;
StoreIo_supported_import_formats_result StoreIo_supported_import_formats(DiplomatWrite* write);

typedef struct StoreIo_supported_export_formats_result {union { NucleationError err;}; bool is_ok;} StoreIo_supported_export_formats_result;
StoreIo_supported_export_formats_result StoreIo_supported_export_formats(DiplomatWrite* write);

typedef struct StoreIo_format_versions_result {union { NucleationError err;}; bool is_ok;} StoreIo_format_versions_result;
StoreIo_format_versions_result StoreIo_format_versions(DiplomatStringView format, DiplomatWrite* write);

typedef struct StoreIo_default_format_version_result {union { NucleationError err;}; bool is_ok;} StoreIo_default_format_version_result;
StoreIo_default_format_version_result StoreIo_default_format_version(DiplomatStringView format, DiplomatWrite* write);

void StoreIo_destroy(StoreIo* self);





#endif // StoreIo_H
