/**
 * Unified social state (Steam + Epic Games friends presence & Universal Discord RPC 2.0).
 */

import { invoke } from '@tauri-apps/api/core';
import type { PlatformKind } from './library';

export type PresenceState = 'ingame' | 'online' | 'away' | 'offline';

export interface UnifiedFriend {
  id: string;
  platform: PlatformKind;
  displayName: string;
  avatarUrl: string | null;
  state: PresenceState;
  currentGame: string | null;
  currentAppId: string | null;
  joinUri: string | null;
}

export interface DiscordPresencePayload {
  gameId: string;
  title: string;
  state: string;
  largeImage: string | null;
  startedUnix: number | null;
}

export interface DiscordRpcStatus {
  enabled: boolean;
  connected: boolean;
  clientId: string;
  current: DiscordPresencePayload | null;
}

export async function fetchUnifiedFriends(): Promise<UnifiedFriend[]> {
  return invoke<UnifiedFriend[]>('get_unified_friends');
}

export async function fetchDiscordRpcStatus(): Promise<DiscordRpcStatus> {
  return invoke<DiscordRpcStatus>('get_discord_rpc_status');
}

export async function setDiscordRpcEnabled(enabled: boolean): Promise<DiscordRpcStatus> {
  return invoke<DiscordRpcStatus>('set_discord_rpc_enabled', { enabled });
}
