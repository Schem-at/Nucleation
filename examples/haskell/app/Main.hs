module Main where

import qualified Data.ByteString as BS
import           System.Exit     (die)

import           Nucleation

main :: IO ()
main = withSchematic $ \schem -> do
  mapM_ (placeOrDie schem) $
    [ ((x, 0, 0), "minecraft:stone_bricks") | x <- [0 .. 5] ]

  bytes <- toLitematic schem
  BS.writeFile "demo.litematic" bytes
  putStrLn $ "wrote demo.litematic (" ++ show (BS.length bytes) ++ " bytes)"
  where
    placeOrDie schem (pos, blk) = do
      r <- setBlock schem pos blk
      case r of
        Left err -> die ("setBlock failed at " ++ show pos ++ ": " ++ err)
        Right () -> pure ()
