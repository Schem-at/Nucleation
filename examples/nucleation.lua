local ffi = require("ffi")

ffi.cdef[[
    typedef struct {
        unsigned char *data;
        size_t len;
    } ByteArray;

    typedef struct SchematicWrapper SchematicWrapper;

    SchematicWrapper *schematic_new(void);
    void schematic_free(SchematicWrapper *schematic);
    int schematic_set_block(SchematicWrapper *schematic, int x, int y, int z, const char *block_name);
    void schematic_set_name(SchematicWrapper *schematic, const char *name);
    void schematic_set_author(SchematicWrapper *schematic, const char *author);
    void schematic_set_description(SchematicWrapper *schematic, const char *description);
    ByteArray schematic_to_schematic(const SchematicWrapper *schematic);
    ByteArray schematic_to_litematic(const SchematicWrapper *schematic);
    ByteArray schematic_save_as(const SchematicWrapper *schematic, const char *format, const char *version, const char *settings);
    void free_byte_array(ByteArray array);
    char *schematic_last_error(void);
    void free_string(char *string);
]]

local lib = ffi.load("nucleation")

local Schematic = {}
Schematic.__index = Schematic

function Schematic.new(name)
    local ptr = lib.schematic_new()
    assert(ptr ~= nil, "Failed to create schematic")
    local self = setmetatable({ _ptr = ptr }, Schematic)
    if name then self:set_name(name) end
    return self
end

function Schematic:set_name(name)
    lib.schematic_set_name(self._ptr, name)
end

function Schematic:set_author(author)
    lib.schematic_set_author(self._ptr, author)
end

function Schematic:set_description(desc)
    lib.schematic_set_description(self._ptr, desc)
end

function Schematic:set_block(x, y, z, block)
    local rc = lib.schematic_set_block(self._ptr, x, y, z, block)
    assert(rc == 0, string.format("Failed to set block at (%d, %d, %d)", x, y, z))
end

function Schematic:fill(x1, y1, z1, x2, y2, z2, block)
    for x = x1, x2 do
        for y = y1, y2 do
            for z = z1, z2 do
                self:set_block(x, y, z, block)
            end
        end
    end
end

function Schematic:save(filename, format)
    format = format or filename:match("%.(%w+)$")

    local data
    if format == "schematic" then
        data = lib.schematic_to_schematic(self._ptr)
    elseif format == "litematic" then
        data = lib.schematic_to_litematic(self._ptr)
    else
        data = lib.schematic_save_as(self._ptr, format, nil, nil)
    end

    if data.data == nil or data.len == 0 then
        local err = lib.schematic_last_error()
        local msg = err ~= nil and ffi.string(err) or "unknown error"
        if err ~= nil then lib.free_string(err) end
        error("Failed to export: " .. msg)
    end

    local f = io.open(filename, "wb")
    assert(f, "Failed to open " .. filename)
    f:write(ffi.string(data.data, data.len))
    f:close()

    local bytes = tonumber(data.len)
    lib.free_byte_array(data)
    print(string.format("Saved %s (%d bytes)", filename, bytes))
end

function Schematic:free()
    if self._ptr ~= nil then
        lib.schematic_free(self._ptr)
        self._ptr = nil
    end
end

return Schematic
