module Config where

import Text.Read (readMaybe)
import Data.Char (toLower)

-- ============================================================================
-- NEWTYPES
-- Type-safe wrappers. Port and TimeoutSec are distinct types, not bare numbers.
-- ============================================================================

-- TODO: Define newtype Port (wrapping Int), deriving Show and Eq
-- TODO: Define newtype TimeoutSec (wrapping Double), deriving Show and Eq

-- ============================================================================
-- DEFAULTS
-- ============================================================================

-- TODO: Define defaultPort, defaultTimeout, defaultEnv

-- ============================================================================
-- CONFIG TYPE
-- Define a record type for Config. Consider:
--   - Which fields are always present?
--   - Which fields might not be set? (use Maybe)
-- ============================================================================

-- TODO: Define Config record type, deriving Show

-- ============================================================================
-- PARSE CONFIG
-- Takes a list of key-value pairs and returns Either String Config.
-- Left = error message, Right = parsed config.
-- ============================================================================

-- | Parse environment variables into a typed Config.
-- Returns Left with an error message if required fields are missing
-- or values can't be parsed.
parseConfig :: [(String, String)] -> Either String Config
parseConfig env = Left "not implemented"

-- ============================================================================
-- SHOW CONFIG
-- Human-readable summary of the config.
-- ============================================================================

-- TODO: showConfig :: Config -> String

-- ============================================================================
-- HELPERS (optional — add your own as needed)
-- ============================================================================

-- | Look up a key in the environment.
lookupEnv :: String -> [(String, String)] -> Maybe String
lookupEnv = lookup

-- | Parse a boolean from common string representations.
parseBool :: String -> Maybe Bool
parseBool s = case map toLower s of
  "true"  -> Just True
  "false" -> Just False
  "1"     -> Just True
  "0"     -> Just False
  _       -> Nothing
