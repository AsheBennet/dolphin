// Copyright 2026 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later

#include "Core/Brawlback/BridgeLaunchArgs.h"

#include <cstdlib>
#include <stdexcept>
#include <picojson.h>

#include "Common/JsonUtil.h"
#include "Common/Logging/Log.h"

namespace BridgeLaunchArgs
{
namespace
{
Args s_args;
bool s_active = false;
}  // namespace

bool ParseEndpointsJson(const std::string& json, std::vector<Endpoint>* out, std::string* error)
{
  if (!out)
  {
    if (error)
      *error = "out is null";
    return false;
  }
  out->clear();

  picojson::value root;
  const std::string err = picojson::parse(root, json);
  if (!err.empty())
  {
    if (error)
      *error = err;
    return false;
  }
  if (!root.is<picojson::array>())
  {
    if (error)
      *error = "endpoints JSON must be an array";
    return false;
  }

  for (const auto& entry : root.get<picojson::array>())
  {
    if (!entry.is<picojson::object>())
    {
      if (error)
        *error = "endpoint entry must be an object";
      return false;
    }
    const auto& obj = entry.get<picojson::object>();
    Endpoint ep;
    auto player_id = ReadStringFromJson(obj, "playerId");
    auto host = ReadStringFromJson(obj, "host");
    auto port = ReadNumericFromJson<double>(obj, "port");
    if (!player_id || !host || !port)
    {
      if (error)
        *error = "endpoint requires playerId (string), host (string), port (number)";
      return false;
    }
    if (*port < 1 || *port > 65535)
    {
      if (error)
        *error = "endpoint port missing or out of range (expected 1-65535)";
      return false;
    }
    ep.player_id = std::move(*player_id);
    ep.host = std::move(*host);
    ep.port = static_cast<u16>(*port);
    out->push_back(std::move(ep));
  }
  return true;
}

bool IsActive()
{
  return s_active;
}

const Args& Get()
{
  return s_args;
}

void Clear()
{
  s_args = {};
  s_active = false;
}

bool SetFromCli(const std::optional<std::string>& match_id, const std::optional<std::string>& seed,
                const std::optional<std::string>& local_idx,
                const std::optional<std::string>& endpoints_json, std::string* error)
{
  Clear();

  // None of the --bb-* flags → inactive (normal Lylat path).
  if (!match_id && !seed && !local_idx && !endpoints_json)
    return true;

  // Partial set is an error: Dock always emits all four together.
  if (!match_id || !seed || !local_idx || !endpoints_json)
  {
    if (error)
      *error = "Bridge --bb-* flags must be provided together "
               "(--bb-match-id, --bb-seed, --bb-local-idx, --bb-endpoints)";
    return false;
  }

  Args args;
  args.match_id = *match_id;

  try
  {
    args.seed = static_cast<s64>(std::stoll(*seed));
  }
  catch (const std::exception&)
  {
    if (error)
      *error = "invalid --bb-seed (expected int64 decimal)";
    return false;
  }

  try
  {
    args.local_player_idx = std::stoi(*local_idx);
  }
  catch (const std::exception&)
  {
    if (error)
      *error = "invalid --bb-local-idx (expected int)";
    return false;
  }

  std::string ep_err;
  if (!ParseEndpointsJson(*endpoints_json, &args.endpoints, &ep_err))
  {
    if (error)
      *error = "invalid --bb-endpoints: " + ep_err;
    return false;
  }

  if (args.endpoints.empty())
  {
    if (error)
      *error = "--bb-endpoints must be a non-empty array";
    return false;
  }

  if (args.local_player_idx < 0 ||
      static_cast<size_t>(args.local_player_idx) >= args.endpoints.size())
  {
    if (error)
      *error = "--bb-local-idx out of range for --bb-endpoints";
    return false;
  }

  s_args = std::move(args);
  s_active = true;
  INFO_LOG_FMT(BRAWLBACK,
               "BridgeLaunchArgs active: match_id={} seed={} local_idx={} endpoints={}",
               s_args.match_id, s_args.seed, s_args.local_player_idx, s_args.endpoints.size());
  return true;
}

std::optional<Endpoint> LocalEndpoint()
{
  if (!s_active)
    return std::nullopt;
  const size_t idx = static_cast<size_t>(s_args.local_player_idx);
  if (idx >= s_args.endpoints.size())
    return std::nullopt;
  return s_args.endpoints[idx];
}

std::vector<Endpoint> RemoteEndpoints()
{
  std::vector<Endpoint> remotes;
  if (!s_active)
    return remotes;
  for (size_t i = 0; i < s_args.endpoints.size(); ++i)
  {
    if (static_cast<int>(i) == s_args.local_player_idx)
      continue;
    remotes.push_back(s_args.endpoints[i]);
  }
  return remotes;
}
}  // namespace BridgeLaunchArgs
