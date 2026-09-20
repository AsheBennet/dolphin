// Copyright 2026 Dolphin Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later

#include <optional>

#include <gtest/gtest.h>

#include "Core/Brawlback/BridgeLaunchArgs.h"

TEST(BridgeLaunchArgs, ParseEndpointsEmptyArray)
{
  std::vector<BridgeLaunchArgs::Endpoint> eps;
  std::string err;
  ASSERT_TRUE(BridgeLaunchArgs::ParseEndpointsJson("[]", &eps, &err)) << err;
  EXPECT_TRUE(eps.empty());
}

TEST(BridgeLaunchArgs, ParseEndpointsTwoPeers)
{
  const char* json =
      R"([{"playerId":"p0","host":"10.0.0.1","port":4096},{"playerId":"p1","host":"10.0.0.2","port":4097}])";
  std::vector<BridgeLaunchArgs::Endpoint> eps;
  std::string err;
  ASSERT_TRUE(BridgeLaunchArgs::ParseEndpointsJson(json, &eps, &err)) << err;
  ASSERT_EQ(eps.size(), 2u);
  EXPECT_EQ(eps[0].player_id, "p0");
  EXPECT_EQ(eps[0].host, "10.0.0.1");
  EXPECT_EQ(eps[0].port, 4096);
  EXPECT_EQ(eps[1].player_id, "p1");
  EXPECT_EQ(eps[1].host, "10.0.0.2");
  EXPECT_EQ(eps[1].port, 4097);
}

TEST(BridgeLaunchArgs, ParseEndpointsRejectsMalformed)
{
  std::vector<BridgeLaunchArgs::Endpoint> eps;
  std::string err;
  EXPECT_FALSE(BridgeLaunchArgs::ParseEndpointsJson("{", &eps, &err));
  EXPECT_FALSE(BridgeLaunchArgs::ParseEndpointsJson("{}", &eps, &err));
  EXPECT_FALSE(BridgeLaunchArgs::ParseEndpointsJson(R"([{"host":"x","port":1}])", &eps, &err));
  EXPECT_FALSE(
      BridgeLaunchArgs::ParseEndpointsJson(R"([{"playerId":"a","host":"x","port":99999}])", &eps, &err));
  EXPECT_FALSE(
      BridgeLaunchArgs::ParseEndpointsJson(R"([{"playerId":"a","host":"x","port":0}])", &eps, &err));
}

TEST(BridgeLaunchArgs, SetFromCliRoundTrip)
{
  BridgeLaunchArgs::Clear();
  EXPECT_FALSE(BridgeLaunchArgs::IsActive());

  std::string err;
  ASSERT_TRUE(BridgeLaunchArgs::SetFromCli(
      std::string("match-abc"), std::string("123456789012345"), std::string("1"),
      std::string(
          R"([{"playerId":"p0","host":"10.0.0.1","port":4096},{"playerId":"p1","host":"10.0.0.2","port":4097}])"),
      &err))
      << err;

  ASSERT_TRUE(BridgeLaunchArgs::IsActive());
  const auto& args = BridgeLaunchArgs::Get();
  EXPECT_EQ(args.match_id, "match-abc");
  EXPECT_EQ(args.seed, 123456789012345LL);
  EXPECT_EQ(args.local_player_idx, 1);

  auto local = BridgeLaunchArgs::LocalEndpoint();
  ASSERT_TRUE(local.has_value());
  EXPECT_EQ(local->host, "10.0.0.2");
  EXPECT_EQ(local->port, 4097);

  auto remotes = BridgeLaunchArgs::RemoteEndpoints();
  ASSERT_EQ(remotes.size(), 1u);
  EXPECT_EQ(remotes[0].host, "10.0.0.1");

  BridgeLaunchArgs::Clear();
  EXPECT_FALSE(BridgeLaunchArgs::IsActive());
}

TEST(BridgeLaunchArgs, SetFromCliRejectsPartialAndBadIdx)
{
  BridgeLaunchArgs::Clear();
  std::string err;
  EXPECT_FALSE(BridgeLaunchArgs::SetFromCli(std::string("m"), std::nullopt, std::string("0"),
                                            std::string("[]"), &err));
  EXPECT_FALSE(BridgeLaunchArgs::IsActive());
  EXPECT_NE(err.find("--bb-* flags must be provided together"), std::string::npos);

  EXPECT_FALSE(BridgeLaunchArgs::SetFromCli(
      std::string("m"), std::string("1"), std::string("5"),
      std::string(R"([{"playerId":"p0","host":"127.0.0.1","port":1}])"), &err));
  EXPECT_FALSE(BridgeLaunchArgs::IsActive());
  EXPECT_NE(err.find("out of range"), std::string::npos);
}

TEST(BridgeLaunchArgs, SetFromCliRejectsBadJson)
{
  BridgeLaunchArgs::Clear();
  std::string err;
  EXPECT_FALSE(BridgeLaunchArgs::SetFromCli(std::string("m"), std::string("1"), std::string("0"),
                                            std::string("{"), &err));
  EXPECT_FALSE(BridgeLaunchArgs::IsActive());
  EXPECT_NE(err.find("invalid --bb-endpoints"), std::string::npos);

  EXPECT_FALSE(BridgeLaunchArgs::SetFromCli(std::string("m"), std::string("1"), std::string("0"),
                                            std::string("{}"), &err));
  EXPECT_FALSE(BridgeLaunchArgs::IsActive());
}

TEST(BridgeLaunchArgs, SetFromCliRejectsEmptyEndpoints)
{
  BridgeLaunchArgs::Clear();
  std::string err;
  EXPECT_FALSE(BridgeLaunchArgs::SetFromCli(std::string("m"), std::string("1"), std::string("0"),
                                            std::string("[]"), &err));
  EXPECT_FALSE(BridgeLaunchArgs::IsActive());
  EXPECT_NE(err.find("non-empty"), std::string::npos);
}

TEST(BridgeLaunchArgs, SetFromCliRejectsMissingPort)
{
  BridgeLaunchArgs::Clear();
  std::string err;
  EXPECT_FALSE(BridgeLaunchArgs::SetFromCli(
      std::string("m"), std::string("1"), std::string("0"),
      std::string(R"([{"playerId":"p0","host":"127.0.0.1","port":0}])"), &err));
  EXPECT_FALSE(BridgeLaunchArgs::IsActive());
  EXPECT_NE(err.find("invalid --bb-endpoints"), std::string::npos);
}

TEST(BridgeLaunchArgs, SetFromCliNoneKeepsInactive)
{
  BridgeLaunchArgs::Clear();
  std::string err;
  ASSERT_TRUE(BridgeLaunchArgs::SetFromCli(std::nullopt, std::nullopt, std::nullopt, std::nullopt,
                                           &err))
      << err;
  EXPECT_FALSE(BridgeLaunchArgs::IsActive());
}

TEST(BridgeLaunchArgs, LargeSeedAsString)
{
  BridgeLaunchArgs::Clear();
  std::string err;
  ASSERT_TRUE(BridgeLaunchArgs::SetFromCli(
      std::string("m"), std::string("9007199254740993"), std::string("0"),
      std::string(R"([{"playerId":"p0","host":"127.0.0.1","port":7777}])"), &err))
      << err;
  EXPECT_EQ(BridgeLaunchArgs::Get().seed, 9007199254740993LL);
  BridgeLaunchArgs::Clear();
}
