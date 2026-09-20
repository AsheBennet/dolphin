// Copyright 2026 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later

#pragma once

#include <optional>
#include <string>
#include <vector>

#include "Common/CommonTypes.h"

// Dock / Bridge match-launch argv contract (locked with brawlback-launcher matchLaunch.ts).
// No --bb-host / isHost: listen role is derived from local_player_idx + matching endpoint.
namespace BridgeLaunchArgs
{
struct Endpoint
{
  std::string player_id;
  std::string host;
  u16 port = 0;
};

struct Args
{
  std::string match_id;
  s64 seed = 0;
  int local_player_idx = 0;
  std::vector<Endpoint> endpoints;
};

// Pure JSON parser for --bb-endpoints. Returns false on malformed input.
bool ParseEndpointsJson(const std::string& json, std::vector<Endpoint>* out,
                        std::string* error = nullptr);

// True only after SetFromCli succeeds with all four --bb-* flags present.
// Partial CLI leaves inactive (fail closed).
bool IsActive();
const Args& Get();
void Clear();

// Populate process-wide stash from CLI string values. Any omitted optional keeps
// IsActive() false. On parse failure returns false and leaves stash cleared.
bool SetFromCli(const std::optional<std::string>& match_id,
                const std::optional<std::string>& seed,
                const std::optional<std::string>& local_idx,
                const std::optional<std::string>& endpoints_json, std::string* error = nullptr);

// Convenience: local listen endpoint for Get().local_player_idx, if in range.
std::optional<Endpoint> LocalEndpoint();
// Peers excluding local_player_idx.
std::vector<Endpoint> RemoteEndpoints();
}  // namespace BridgeLaunchArgs
