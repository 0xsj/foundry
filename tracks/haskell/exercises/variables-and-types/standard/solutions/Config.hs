module Config where

import Text.Read (readMaybe)
import Data.Char (toLower)

-- ============================================================================
-- NEWTYPES
-- Distinct types for Port and TimeoutSec. The compiler prevents mixing them.
-- Zero runtime cost — erased at compile time.
-- ============================================================================

newtype Port = Port Int deriving (Show, Eq)
newtype TimeoutSec = TimeoutSec Double deriving (Show, Eq)

-- ============================================================================
-- DEFAULTS
-- These are bindings, not variables. They never change.
-- ============================================================================

defaultPort :: Port
defaultPort = Port 8080

defaultTimeout :: TimeoutSec
defaultTimeout = TimeoutSec 30.0

defaultEnv :: String
defaultEnv = "development"

-- ============================================================================
-- CONFIG TYPE
-- Record syntax gives us named fields + accessor functions for free.
-- cfgMaxRetries is Maybe Int — Nothing means "not set", Just 0 means
-- "explicitly set to zero". No ambiguity, unlike Go's zero values.
-- ============================================================================

data Config = Config
  { cfgHost       :: String
  , cfgPort       :: Port
  , cfgDebug      :: Bool
  , cfgMaxRetries :: Maybe Int
  , cfgTimeout    :: TimeoutSec
  , cfgEnv        :: String
  } deriving (Show, Eq)

-- ============================================================================
-- PARSE CONFIG
-- Returns Either String Config:
--   Left "error message"  — like Go's (Config{}, fmt.Errorf(...))
--   Right config          — like Go's (config, nil)
-- ============================================================================

parseConfig :: [(String, String)] -> Either String Config
parseConfig env = do
  -- 'do' notation with Either: Left short-circuits, Right continues.
  -- This is like Go's early-return-on-error pattern, but compositional.

  -- Host (required)
  host <- case lookupEnv "HOST" env of
    Nothing -> Left "HOST is required"
    Just "" -> Left "HOST is required"
    Just h  -> Right h

  -- Port (optional, must be valid int if present)
  port <- case lookupEnv "PORT" env of
    Nothing -> Right defaultPort
    Just s  -> case readMaybe s of
      Nothing -> Left ("invalid PORT: " ++ s)
      Just p  -> Right (Port p)

  -- Debug (optional, parses true/false/1/0)
  debug <- case lookupEnv "DEBUG" env of
    Nothing -> Right False
    Just s  -> case parseBool s of
      Nothing -> Left ("invalid DEBUG: " ++ s)
      Just b  -> Right b

  -- MaxRetries (optional — Nothing if not set, Just n if set)
  maxRetries <- case lookupEnv "MAX_RETRIES" env of
    Nothing -> Right Nothing
    Just s  -> case readMaybe s of
      Nothing -> Left ("invalid MAX_RETRIES: " ++ s)
      Just n  -> Right (Just n)  -- Just 0 is valid!

  -- Timeout (optional, must be valid double if present)
  timeout <- case lookupEnv "TIMEOUT_SEC" env of
    Nothing -> Right defaultTimeout
    Just s  -> case readMaybe s of
      Nothing -> Left ("invalid TIMEOUT_SEC: " ++ s)
      Just t  -> Right (TimeoutSec t)

  -- Environment (optional, defaults to development)
  let environment = case lookupEnv "ENVIRONMENT" env of
        Nothing -> defaultEnv
        Just "" -> defaultEnv
        Just e  -> e

  Right Config
    { cfgHost       = host
    , cfgPort       = port
    , cfgDebug      = debug
    , cfgMaxRetries = maxRetries
    , cfgTimeout    = timeout
    , cfgEnv        = environment
    }

-- ============================================================================
-- SHOW CONFIG
-- ============================================================================

showConfig :: Config -> String
showConfig c =
  "Config{host=" ++ cfgHost c
  ++ ", port=" ++ showPort (cfgPort c)
  ++ ", debug=" ++ show (cfgDebug c)
  ++ ", maxRetries=" ++ showRetries (cfgMaxRetries c)
  ++ ", timeout=" ++ showTimeout (cfgTimeout c)
  ++ ", env=" ++ cfgEnv c
  ++ "}"
  where
    showPort (Port p) = show p
    showTimeout (TimeoutSec t) = show t ++ "s"
    showRetries Nothing  = "<not set>"
    showRetries (Just n) = show n

-- ============================================================================
-- HELPERS
-- ============================================================================

lookupEnv :: String -> [(String, String)] -> Maybe String
lookupEnv = lookup

parseBool :: String -> Maybe Bool
parseBool s = case map toLower s of
  "true"  -> Just True
  "false" -> Just False
  "1"     -> Just True
  "0"     -> Just False
  _       -> Nothing
