// Bridge in-emu NDJSON frame log (spike).
// Join keys match Tools/FrameLogHarness / Rollback PR #1 README:
//   local, confirmed, predicting, rb_start, rb_stop, checksum
// Optional SyncData: sync_locX, sync_locY, sync_anim, sync_percent, sync_stocks, sync_facing
//
// Gating: compile with -DBRAWLBACK_FRAME_LOG=0 to disable, or set env
// BRAWLBACK_FRAME_LOG=0 at runtime. Spike default is ON.

#pragma once

#include <cstdlib>
#include <fstream>
#include <mutex>
#include <string>

#include <fmt/format.h>

#include "Common/CommonTypes.h"
#include "Common/FileUtil.h"
#include "Common/Logging/Log.h"
#include "brawlback-common/PlayerFrameData.h"
#include "brawlback-common/SyncData.h"

#ifndef BRAWLBACK_FRAME_LOG
#define BRAWLBACK_FRAME_LOG 1
#endif

namespace Brawlback::FrameLog
{
inline bool IsEnabled()
{
#if !BRAWLBACK_FRAME_LOG
  return false;
#else
  const char* env = std::getenv("BRAWLBACK_FRAME_LOG");
  if (env && (env[0] == '0') && env[1] == '\0')
    return false;
  return true;
#endif
}

inline std::string LogPath()
{
  return File::GetExeDirectory() + "/brawlback_frame_log.ndjson";
}

inline void AppendLine(const std::string& line)
{
  if (!IsEnabled())
    return;

  static std::mutex s_mutex;
  std::lock_guard<std::mutex> lock(s_mutex);

  std::fstream out;
  if (!File::OpenFStream(out, LogPath(), std::ios_base::out | std::ios_base::app))
  {
    ERROR_LOG_FMT(BRAWLBACK, "FrameLog: failed to open {}\n", LogPath());
    return;
  }
  out << line << '\n';
}

// Pad store event (after storeLocalInputs).
inline void EmitPad(u32 frame, u8 player_idx, u32 buttons)
{
  AppendLine(fmt::format(
      R"({{"event":"pad","local":{},"playerIdx":{},"buttons":{}}})", frame, player_idx, buttons));
}

// Frame-advance reply (handleFrameAdvanceRequest).
inline void EmitAdvance(u32 frames_to_advance)
{
  AppendLine(fmt::format(R"({{"event":"advance","frames_to_advance":{}}})", frames_to_advance));
}

// Join line after updateSync (optional SyncData + checksum when available).
inline void EmitSync(u32 local, u32 confirmed, bool predicting, bool has_rollback, u32 rb_start,
                     u32 rb_stop, s32 checksum, const SyncData* sync)
{
  std::string line = fmt::format(
      R"({{"event":"sync","local":{},"confirmed":{},"predicting":{},"checksum":{})", local,
      confirmed, predicting ? "true" : "false", checksum);

  if (has_rollback)
  {
    line += fmt::format(R"(,"rb_start":{},"rb_stop":{})", rb_start, rb_stop);
  }
  else
  {
    line += R"(,"rb_start":null,"rb_stop":null)";
  }

  if (sync)
  {
    line += fmt::format(
        R"(,"sync_locX":{},"sync_locY":{},"sync_anim":{},"sync_percent":{},"sync_stocks":{},"sync_facing":{})",
        sync->locX, sync->locY, sync->anim, sync->percent, static_cast<u32>(sync->stocks),
        static_cast<s32>(sync->facingDir));
  }

  line += '}';
  AppendLine(line);
}

}  // namespace Brawlback::FrameLog
