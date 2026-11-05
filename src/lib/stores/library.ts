/**
 * Unified game library & media store backed by Tauri v2 IPC commands and events.
 */

import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type PlatformKind = 'steam' | 'epic' | 'roblox' | 'itch' | 'emulator';

export interface ArtRef {
  kind: 'local' | 'remote';
  value: string;
}

export interface Artwork {
  poster: ArtRef | null;
  hero: ArtRef | null;
  logo: ArtRef | null;
  icon: ArtRef | null;
}

export interface Game {
  id: string;
  platform: PlatformKind;
  nativeId: string;
  title: string;
  installed: boolean;
  installDir: string | null;
  executable: string | null;
  sizeBytes: number | null;
  playtimeMinutes: number | null;
  lastPlayed: string | null;
  art: Artwork;
  tags: string[];
}

export interface PlatformFailure {
  platform: PlatformKind;
  error: string;
}

export interface ScanReport {
  games: Game[];
  errors: PlatformFailure[];
  scannedAt: string;
}

export interface LaunchHandle {
  gameId: string;
  pid: number | null;
  startedAt: string;
}

export interface MediaItem {
  path: string;
  kind: 'screenshot' | 'clip';
  gameId: string | null;
  gameTitle: string | null;
  createdAt: string;
  sizeBytes: number;
  encoder: string | null;
  durationSecs: number | null;
}

/** Resolve an `ArtRef` into a browser-loadable URL (converting local paths via Tauri's asset protocol). */
export function resolveArtUrl(ref: ArtRef | null | undefined): string | null {
  if (!ref) return null;
  if (ref.kind === 'remote') return ref.value;
  try {
    return convertFileSrc(ref.value);
  } catch {
    return ref.value;
  }
}

/** Format minutes into a compact human-readable string (`42m` or `128.4 hrs`). */
export function formatPlaytime(minutes: number | null | undefined): string {
  if (!minutes || minutes <= 0) return 'Unplayed';
  if (minutes < 60) return `${minutes}m`;
  return `${(minutes / 60).toFixed(1)} hrs`;
}

/** Format byte counts into `GB` / `MB`. */
export function formatBytes(bytes: number | null | undefined): string {
  if (!bytes || bytes <= 0) return '—';
  const gb = bytes / (1024 * 1024 * 1024);
  if (gb >= 1) return `${gb.toFixed(1)} GB`;
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(0)} MB`;
}

export async function fetchLibrary(): Promise<Game[]> {
  return invoke<Game[]>('get_library');
}

export async function rescanLibrary(): Promise<ScanReport> {
  return invoke<ScanReport>('refresh_library');
}

export async function launchGameById(gameId: string): Promise<LaunchHandle> {
  return invoke<LaunchHandle>('launch_game', { gameId });
}

export async function installGameById(gameId: string): Promise<LaunchHandle> {
  return invoke<LaunchHandle>('install_game', { gameId });
}

// ---- Steam Store Showcase ---------------------------------------------------
export interface SteamStoreItem {
  id: number;
  name: string;
  discounted: boolean;
  discountPercent: number;
  originalPrice: number | null;
  finalPrice: number | null;
  currency: string;
  formattedOriginalPrice: string | null;
  formattedFinalPrice: string | null;
  headerImage: string | null;
  largeCapsuleImage: string | null;
  storeUrl: string;
  installUri: string;
}

export interface SteamStoreShowcase {
  specials: SteamStoreItem[];
  topSellers: SteamStoreItem[];
  newReleases: SteamStoreItem[];
  comingSoon: SteamStoreItem[];
  syncedAt: string;
}

export async function fetchSteamStoreShowcase(): Promise<SteamStoreShowcase> {
  return invoke<SteamStoreShowcase>('get_steam_store_showcase');
}

// ---- itch.io Store Showcase -------------------------------------------------
export interface ItchStoreGame {
  id: string;
  title: string;
  price: string;
  currency: string;
  coverUrl: string | null;
  url: string;
  description: string;
  platforms: string[];
}

export async function fetchItchStoreShowcase(category?: string): Promise<ItchStoreGame[]> {
  return invoke<ItchStoreGame[]>('get_itch_store_showcase', { category });
}

// ---- Roblox Experiences ----------------------------------------------------
export interface RobloxExperiencePreset {
  placeId: number;
  title: string;
  category: string;
  description: string;
  iconUrl: string;
}

export async function fetchPopularRobloxExperiences(): Promise<RobloxExperiencePreset[]> {
  return invoke<RobloxExperiencePreset[]>('roblox_get_popular_experiences');
}

export async function launchRobloxPlace(placeId: number): Promise<LaunchHandle> {
  return invoke<LaunchHandle>('roblox_launch_place', { placeId });
}

export async function triggerScreenshot(): Promise<MediaItem> {
  return invoke<MediaItem>('take_screenshot');
}

export async function toggleClipRecording(): Promise<unknown> {
  return invoke('toggle_recording');
}

export async function fetchMediaGallery(): Promise<MediaItem[]> {
  return invoke<MediaItem[]>('list_media');
}

export async function deleteMediaItem(path: string): Promise<boolean> {
  return invoke<boolean>('delete_media', { path });
}

export interface InAppStoreState {
  open: boolean;
  url: string;
  title: string;
  installUri?: string;
}

export const inAppStore = {
  state: {
    open: false,
    url: '',
    title: '',
    installUri: undefined as string | undefined,
  },
  listeners: new Set<(state: InAppStoreState) => void>(),
  open(url: string, title = 'Магазин', installUri?: string) {
    this.state = { open: true, url, title, installUri };
    this.notify();
  },
  close() {
    this.state = { open: false, url: '', title: '', installUri: undefined };
    this.notify();
  },
  subscribe(fn: (state: InAppStoreState) => void) {
    this.listeners.add(fn);
    fn(this.state);
    return () => {
      this.listeners.delete(fn);
    };
  },
  notify() {
    for (const fn of this.listeners) fn(this.state);
  },
};

export function openInAppStore(url: string, title = 'Store', installUri?: string): void {
  // If it's Epic Games or Steam profile, directly open the high-fidelity native webview window
  if (url.includes('epicgames.com') || url.includes('steamcommunity.com')) {
    invoke('open_in_app_store_window', { url, title }).catch(() => {
      inAppStore.open(url, title, installUri);
    });
    return;
  }
  inAppStore.open(url, title, installUri);
}

export function closeInAppStore(): void {
  inAppStore.close();
}

export async function onLibraryUpdated(
  handler: (report: ScanReport) => void
): Promise<UnlistenFn> {
  return listen<ScanReport>('library://updated', (event) => handler(event.payload));
}

// ---------------------------------------------------------------------------
// Milestone 2: Platform Integrations, Steam Modal, Epic & Roblox, Deduplication
// ---------------------------------------------------------------------------

export interface SteamProfile {
  steam_id: string;
  persona_name: string;
  profile_url: string;
  avatar_url: string;
  is_cached?: boolean;
  last_synced?: number;
}

export interface SteamAchievement {
  id: string;
  api_name: string;
  name: string;
  display_name?: string | null;
  description?: string | null;
  achieved: boolean;
  unlock_time: number;
  icon_url?: string | null;
  is_hidden?: boolean | null;
}

export interface SteamRepairResult {
  app_id: number;
  cache_cleared: boolean;
  achievements_reconciled: number;
  message: string;
}

export interface CloudSyncResult {
  app_id: number;
  synced: boolean;
  local_files: string[];
  remote_files: string[];
  last_sync_timestamp: number;
  snapshot_id?: string | null;
}

export interface EpicGame {
  app_name: string;
  display_name: string;
  catalog_item_id: string;
  installed: boolean;
  install_location?: string | null;
  launch_uri?: string | null;
  install_uri?: string | null;
}

export interface RobloxPlace {
  place_id: number;
  name: string;
  last_played_timestamp: number;
  thumbnail_url?: string | null;
  universe_id?: number | null;
  launch_url?: string | null;
}

export interface UnifiedGameSource {
  id: string;
  platform: string;
  native_id: string;
  title: string;
  installed: boolean;
  install_dir?: string | null;
  executable?: string | null;
  size_bytes?: number | null;
  playtime_minutes?: number | null;
  last_played?: string | null;
  launch_action?: string | null;
}

export interface UnifiedGame {
  id: string;
  primary_title: string;
  title: string;
  platforms: string[];
  preferred_platform: string;
  installed: boolean;
  total_playtime_minutes: number;
  poster_url?: string | null;
  sources: UnifiedGameSource[];
}

export interface GameProperties {
  game_id: string;
  launch_args: string;
  custom_executable?: string | null;
  cloud_enabled: boolean;
  compatibility_tool?: string | null;
  env_vars: Record<string, string>;
  high_dpi_scaling: boolean;
  install_dir?: string | null;
  total_size_bytes: number;
  file_count: number;
  verify_integrity_available: boolean;
  quota_bytes: number;
  used_bytes: number;
  last_sync?: number | null;
}

export interface VerifyResult {
  status: string;
  file_count: number;
  verified_files: number;
  corrupted_files: number;
  is_locked: boolean;
}

export async function fetchSteamProfile(): Promise<SteamProfile> {
  return invoke<SteamProfile>('get_steam_profile');
}

export async function fetchSteamAchievements(appId: number): Promise<SteamAchievement[]> {
  return invoke<SteamAchievement[]>('get_steam_achievements', { appId });
}

export async function repairSteamAchievements(appId: number): Promise<SteamRepairResult> {
  return invoke<SteamRepairResult>('repair_steam_achievements', { appId });
}

export async function syncSteamCloud(appId: number): Promise<CloudSyncResult> {
  return invoke<CloudSyncResult>('sync_steam_cloud', { appId });
}

export async function fetchEpicLibrary(includeUninstalled = true): Promise<EpicGame[]> {
  return invoke<EpicGame[]>('get_epic_library', { includeUninstalled });
}

export async function fetchRobloxRecentPlaces(): Promise<RobloxPlace[]> {
  return invoke<RobloxPlace[]>('get_roblox_recent_places');
}

export async function fetchUnifiedLibrary(): Promise<UnifiedGame[]> {
  return invoke<UnifiedGame[]>('get_unified_library');
}

export async function getGameProperties(gameId: string): Promise<GameProperties> {
  return invoke<GameProperties>('get_game_properties', { gameId });
}

export async function saveGameProperties(gameId: string, properties: GameProperties): Promise<void> {
  return invoke<void>('save_game_properties', { gameId, properties });
}

export async function verifyGameFiles(gameId: string): Promise<VerifyResult> {
  return invoke<VerifyResult>('verify_game_files', { gameId });
}

export function sanitizeLaunchArgs(rawInput: string): string[] {
  if (!rawInput) return [];
  // Strip shell injection metacharacters, redirections, escapes, env expansion, newlines, and null bytes
  const sanitized = rawInput.replace(/[;&|`$><^%()\r\n\0]/g, '');

  // Parse argv tokens preserving spaces inside single and double quotes
  const tokens: string[] = [];
  let current = '';
  let inQuote: '"' | "'" | null = null;

  for (let i = 0; i < sanitized.length; i++) {
    const char = sanitized[i];

    if (char === '"' || char === "'") {
      if (inQuote === char) {
        current += char;
        inQuote = null;
      } else if (!inQuote) {
        inQuote = char;
        current += char;
      } else {
        current += char;
      }
    } else if (/\s/.test(char) && !inQuote) {
      if (current.length > 0) {
        tokens.push(current);
        current = '';
      }
    } else {
      current += char;
    }
  }

  if (current.length > 0) {
    tokens.push(current);
  }

  return tokens;
}

export function normalizeGameTitle(title: string): string {
  if (!title) return '';
  return title
    .toLowerCase()
    .replace(/®|™|©/g, '')
    .replace(/\b(standard|deluxe|game of the year|goty|edition|complete|ultimate|remastered|anniversary|definitive|bundle|director's cut)\b/gu, '')
    .replace(/[^\p{L}\p{N}]/gu, '')
    .trim();
}
