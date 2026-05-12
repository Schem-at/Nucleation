-- | Minimal Haskell bindings to the Nucleation C FFI.
--
-- Exposes just enough to build a schematic, set/get blocks by string,
-- and serialize to the .litematic format. Designed as a starting point;
-- extend this module with more FFI imports as needed.
{-# LANGUAGE ForeignFunctionInterface #-}
{-# LANGUAGE OverloadedStrings #-}
module Nucleation
  ( -- * Opaque handle
    Schematic
    -- * Lifecycle
  , newSchematic
  , freeSchematic
  , withSchematic
    -- * Block I/O
  , setBlock
  , getBlock
    -- * Serialization
  , toLitematic
    -- * Diagnostics
  , lastError
  ) where

import           Control.Exception     (bracket)
import qualified Data.ByteString       as BS
import           Data.ByteString       (ByteString)
import           Data.Word             (Word8)
import           Foreign.C.String      (CString, newCString, peekCString)
import           Foreign.C.Types       (CInt(..), CSize(..))
import           Foreign.Marshal.Alloc (alloca, free)
import           Foreign.Marshal.Array (peekArray)
import           Foreign.Ptr           (Ptr, nullPtr)
import           Foreign.Storable      (peek)


-- | Opaque pointer to the Rust-owned schematic.
data SchematicHandle
type Schematic = Ptr SchematicHandle


--------------------------------------------------------------------------------
-- Raw FFI imports
--------------------------------------------------------------------------------

foreign import ccall unsafe "schematic_new"
  c_schematic_new :: IO Schematic

foreign import ccall unsafe "schematic_free"
  c_schematic_free :: Schematic -> IO ()

foreign import ccall unsafe "schematic_set_block_from_string"
  c_schematic_set_block_from_string
    :: Schematic -> CInt -> CInt -> CInt -> CString -> IO CInt

foreign import ccall unsafe "schematic_get_block_string"
  c_schematic_get_block_string
    :: Schematic -> CInt -> CInt -> CInt -> IO CString

-- ByteArray-returning FFI functions are wrapped via cbits/nucleation_shim.c
-- because GHC's foreign-import ccall doesn't accept struct-by-value
-- results.
foreign import ccall unsafe "hs_schematic_to_litematic"
  c_schematic_to_litematic
    :: Schematic -> Ptr (Ptr Word8) -> Ptr CSize -> IO ()

foreign import ccall unsafe "hs_free_byte_array"
  c_free_byte_array :: Ptr Word8 -> CSize -> IO ()

foreign import ccall unsafe "schematic_last_error"
  c_schematic_last_error :: IO CString

foreign import ccall unsafe "free_string"
  c_free_string :: CString -> IO ()


--------------------------------------------------------------------------------
-- Public API
--------------------------------------------------------------------------------

-- | Allocate a fresh, empty schematic. Pair every successful 'newSchematic'
--   with 'freeSchematic' (or use 'withSchematic').
newSchematic :: IO Schematic
newSchematic = c_schematic_new

-- | Free a schematic returned by 'newSchematic'.
freeSchematic :: Schematic -> IO ()
freeSchematic = c_schematic_free

-- | Bracket-style helper that frees the schematic on any exit.
withSchematic :: (Schematic -> IO a) -> IO a
withSchematic = bracket newSchematic freeSchematic

-- | Set a block from a full block string, e.g.
--   @"minecraft:lever[face=floor,facing=east,powered=false]"@.
--   Returns 'Right' on success and 'Left' with the C-side error on failure.
setBlock :: Schematic -> (Int, Int, Int) -> String -> IO (Either String ())
setBlock h (x, y, z) blk =
  bracket (newCString blk) free $ \cstr -> do
    rc <- c_schematic_set_block_from_string
            h (fromIntegral x) (fromIntegral y) (fromIntegral z) cstr
    if rc == 0
      then pure (Right ())
      else Left <$> lastError

-- | Read the block at @(x, y, z)@ as its full block string. 'Nothing' if no
--   block is recorded at that position.
getBlock :: Schematic -> (Int, Int, Int) -> IO (Maybe String)
getBlock h (x, y, z) = do
  cstr <- c_schematic_get_block_string
            h (fromIntegral x) (fromIntegral y) (fromIntegral z)
  if cstr == nullPtr
    then pure Nothing
    else do
      s <- peekCString cstr
      c_free_string cstr
      pure (Just s)

-- | Serialize the schematic to a .litematic byte string.
toLitematic :: Schematic -> IO ByteString
toLitematic h =
  alloca $ \pData ->
  alloca $ \pLen -> do
    c_schematic_to_litematic h pData pLen
    dat <- peek pData
    n   <- peek pLen
    if dat == nullPtr || n == 0
      then pure BS.empty
      else do
        bytes <- peekArray (fromIntegral n) dat
        c_free_byte_array dat n
        pure (BS.pack bytes)

-- | Last error message produced by the FFI (empty string if none).
lastError :: IO String
lastError = do
  cstr <- c_schematic_last_error
  if cstr == nullPtr
    then pure ""
    else do
      s <- peekCString cstr
      c_free_string cstr
      pure s
