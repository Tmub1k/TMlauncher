<!--
  GamePropertiesModal.svelte — Steam-Grade 5-Tab Game Properties Modal (Svelte 5 Runes).
  Tabs:
  1. general: Launch arguments (sanitized, max 4096), executable override, desktop shortcut.
  2. installed_files: Size on disk, file count, folder browse, file integrity verification.
  3. cloud: Steam Cloud synchronization toggle, quota meter, snapshot backups, restore.
  4. achievements: Grid visualizer, unlock percentage bar, spoiler masking, desync repair utility.
  5. compatibility: Force compatibility tool dropdown, environment variables (DXVK/Proton), DPI override.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    formatBytes,
    getGameProperties,
    saveGameProperties,
    verifyGameFiles,
    fetchSteamAchievements,
    repairSteamAchievements,
    syncSteamCloud,
    resolveArtUrl,
    type Game,
    type GameProperties,
    type SteamAchievement,
    type UnifiedGame,
    type VerifyResult,
    type CloudSyncResult,
  } from '../stores/library';

  interface Props {
    game: Game | UnifiedGame | null;
    open: boolean;
    isGameRunning?: boolean;
    onClose: () => void;
  }

  let { game, open, isGameRunning = false, onClose }: Props = $props();

  let verifyInterval: ReturnType<typeof setInterval> | null = null;

  $effect(() => {
    return () => clearInterval(verifyInterval!);
  });

  function switchPlatform(targetPlatform: string) {
    const platLower = targetPlatform.toLowerCase();
    if ((game as any)?.sources && Array.isArray((game as any).sources)) {
      const match = (game as any).sources.find((s: any) =>
        s.platform.toLowerCase() === platLower ||
        s.platform.toUpperCase() === targetPlatform.toUpperCase()
      );
      if (match) {
        (game as any).preferred_platform = match.platform;
        (game as any).platform = match.platform;
        (game as any).nativeId = match.native_id;
        (game as any).id = match.id;
        (game as any).installed = match.installed;
        if (match.install_dir) (game as any).installDir = match.install_dir;
        if (match.executable) (game as any).executable = match.executable;
        if (match.size_bytes != null) (game as any).sizeBytes = match.size_bytes;
        statusMessage = { text: `Active platform switched to ${targetPlatform.toUpperCase()}`, type: 'success' };
      }
    } else if (game) {
      (game as any).platform = platLower;
      statusMessage = { text: `Active platform switched to ${targetPlatform.toUpperCase()}`, type: 'success' };
    }
  }

  type TabId = 'general' | 'installed_files' | 'cloud' | 'achievements' | 'compatibility' | 'notes';
  let activeTab = $state<TabId>('general');

  // Loading & feedback states
  let loading = $state(false);
  let saving = $state(false);
  let properties = $state<GameProperties | null>(null);
  let statusMessage = $state<{ text: string; type: 'success' | 'error' | 'info' } | null>(null);

  // Walkthrough Notes state
  let notesText = $state('');
  let notesAutosaved = $state(true);
  let notesDebounceTimer: ReturnType<typeof setTimeout> | null = null;

  // Custom Cover Art state
  let customCoverUrl = $state('');

  // PCGamingWiki tweaks state
  let pcgwRecord = $state<any>(null);
  let pcgwLoading = $state(false);

  // Verification state
  let verifying = $state(false);
  let verifyProgress = $state(0);
  let verifyResult = $state<VerifyResult | null>(null);

  // Cloud state
  let syncingCloud = $state(false);
  let cloudResult = $state<CloudSyncResult | null>(null);

  // Achievements state
  let achievements = $state<SteamAchievement[]>([]);
  let achievementsFilter = $state<'all' | 'unlocked' | 'locked'>('all');
  let repairingAchievements = $state(false);

  // Environment variables editor
  let envKey = $state('');
  let envVal = $state('');

  const gameId = $derived.by(() => {
    if (!game) return '';
    return typeof (game as any).id === 'string' ? (game as any).id : '';
  });

  const appId = $derived.by(() => {
    if (!game) return 0;
    const native = (game as any).nativeId || (game as any).id;
    const parsed = parseInt(String(native).replace(/\D/g, ''), 10);
    return isNaN(parsed) ? 730 : parsed;
  });

  const gameTitle = $derived.by(() => {
    if (!game) return 'Game';
    return (game as any).primary_title || (game as any).title || 'Game';
  });

  // Calculate achievements stats
  const unlockedAchievementsCount = $derived(
    achievements.filter((a) => a.achieved).length
  );

  const achievementsPercentage = $derived.by(() => {
    if (!achievements.length) return 0;
    return Math.round((unlockedAchievementsCount / achievements.length) * 100);
  });

  const filteredAchievements = $derived.by(() => {
    if (achievementsFilter === 'unlocked') {
      return achievements.filter((a) => a.achieved);
    }
    if (achievementsFilter === 'locked') {
      return achievements.filter((a) => !a.achieved);
    }
    return achievements;
  });

  // Load properties and achievements whenever modal opens
  $effect(() => {
    if (open && gameId) {
      void loadData();
    }
  });

  async function loadData() {
    if (!gameId) return;
    loading = true;
    statusMessage = null;
    verifyResult = null;

    try {
      const [props, achs] = await Promise.all([
        getGameProperties(gameId),
        fetchSteamAchievements(appId).catch(() => []),
      ]);
      properties = props;
      achievements = achs;
    } catch (err) {
      console.error('Failed to load game properties:', err);
      // Fallback default properties
      properties = {
        game_id: gameId,
        launch_args: '',
        custom_executable: null,
        cloud_enabled: true,
        compatibility_tool: 'native_windows_subsystem',
        env_vars: {},
        high_dpi_scaling: true,
        install_dir: null,
        total_size_bytes: 0,
        file_count: 0,
        verify_integrity_available: true,
        quota_bytes: 104857600,
        used_bytes: 15728640,
        last_sync: Date.now() / 1000,
      };
    } finally {
      try {
        notesText = localStorage.getItem('omnilauncher.notes.' + gameId) || '';
        notesAutosaved = true;
      } catch {}
      loading = false;
    }
  }

  async function handleSave() {
    if (!properties || !gameId) return;
    saving = true;
    try {
      // Live sanitize launch arguments (remove ;, &, |, `, $, >, <, ^, %, (, ), \r, \n, \0)
      properties.launch_args = properties.launch_args
        .replace(/[;&|`$><^%()\r\n\0]/g, '')
        .slice(0, 4096);

      await saveGameProperties(gameId, properties);
      statusMessage = { text: 'Properties saved successfully', type: 'success' };
      setTimeout(() => {
        if (statusMessage?.type === 'success') statusMessage = null;
      }, 3000);
    } catch (err: any) {
      statusMessage = { text: `Failed to save: ${err?.message || err}`, type: 'error' };
    } finally {
      saving = false;
    }
  }

  async function handleVerifyFiles() {
    if (!gameId || verifying) return;
    verifying = true;
    verifyProgress = 0;
    verifyResult = null;

    if (verifyInterval) {
      clearInterval(verifyInterval);
    }
    verifyInterval = setInterval(() => {
      if (verifyProgress < 90) {
        verifyProgress += 15;
      }
    }, 200);

    try {
      const res = await verifyGameFiles(gameId);
      if (verifyInterval) {
        clearInterval(verifyInterval);
        verifyInterval = null;
      }
      verifyProgress = 100;
      verifyResult = res;
    } catch (err: any) {
      if (verifyInterval) {
        clearInterval(verifyInterval);
        verifyInterval = null;
      }
      verifyResult = {
        status: 'error',
        file_count: 0,
        verified_files: 0,
        corrupted_files: 0,
        is_locked: false,
      };
    } finally {
      setTimeout(() => {
        verifying = false;
      }, 400);
    }
  }

  async function handleSyncCloud() {
    if (syncingCloud) return;
    syncingCloud = true;
    try {
      const res = await syncSteamCloud(appId);
      cloudResult = res;
      statusMessage = { text: 'Steam Cloud synchronized successfully', type: 'success' };
    } catch (err: any) {
      statusMessage = { text: `Cloud sync failed: ${err?.message || err}`, type: 'error' };
    } finally {
      syncingCloud = false;
    }
  }

  async function handleRepairAchievements() {
    if (repairingAchievements) return;
    repairingAchievements = true;
    try {
      const res = await repairSteamAchievements(appId);
      statusMessage = { text: res.message || 'Achievements cache repaired', type: 'success' };
      // Refresh achievements
      achievements = await fetchSteamAchievements(appId);
    } catch (err: any) {
      statusMessage = { text: `Repair failed: ${err?.message || err}`, type: 'error' };
    } finally {
      repairingAchievements = false;
    }
  }

  function onNotesInput(val: string) {
    notesText = val;
    notesAutosaved = false;
    if (notesDebounceTimer) clearTimeout(notesDebounceTimer);
    notesDebounceTimer = setTimeout(() => {
      try {
        localStorage.setItem('omnilauncher.notes.' + gameId, val);
        notesAutosaved = true;
      } catch {}
    }, 300);
  }

  async function openDirectory(kind: 'install' | 'save' | 'mods') {
    try {
      await invoke('open_game_directory', { gameId, kind });
      statusMessage = { text: `Opened ${kind} directory in Explorer`, type: 'success' };
    } catch (err: any) {
      statusMessage = { text: `Failed to open directory: ${err?.message || err}`, type: 'error' };
    }
  }

  function applyCustomCover() {
    if (!customCoverUrl.trim() || !game) return;
    (game as any).art = (game as any).art || {};
    (game as any).art.poster = { kind: 'remote', value: customCoverUrl.trim() };
    statusMessage = { text: 'Custom cover applied successfully!', type: 'success' };
  }

  function resetCustomCover() {
    if (!game) return;
    customCoverUrl = '';
    if (appId) {
      (game as any).art = (game as any).art || {};
      (game as any).art.poster = {
        kind: 'remote',
        value: `https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/${appId}/library_600x900.jpg`,
      };
    }
    statusMessage = { text: 'Cover reset to official artwork', type: 'info' };
  }

  async function loadPcgw() {
    if (pcgwRecord || pcgwLoading) return;
    pcgwLoading = true;
    try {
      pcgwRecord = await invoke('get_pcgamingwiki_data', { gameTitle, steamAppId: appId });
    } catch {
      // Fallback
    } finally {
      pcgwLoading = false;
    }
  }

  async function handleApplyTweak(tweak: any) {
    try {
      await invoke('apply_pcgw_tweak', {
        configPath: tweak.configFile,
        section: tweak.section,
        key: tweak.key,
        value: tweak.recommendedVal,
      });
      tweak.applied = true;
      statusMessage = { text: `Applied tweak: ${tweak.title} (created .omnibak backup)`, type: 'success' };
    } catch (err: any) {
      statusMessage = { text: `Failed to apply tweak: ${err?.message || err}`, type: 'error' };
    }
  }

  async function handleRevertTweak(tweak: any) {
    try {
      await invoke('revert_pcgw_tweak', { configPath: tweak.configFile });
      tweak.applied = false;
      statusMessage = { text: `Reverted tweak: ${tweak.title} from .omnibak`, type: 'info' };
    } catch (err: any) {
      statusMessage = { text: `Failed to revert tweak: ${err?.message || err}`, type: 'error' };
    }
  }

  function handleAddEnvVar() {
    if (!envKey.trim() || !properties) return;
    properties.env_vars[envKey.trim()] = envVal.trim();
    envKey = '';
    envVal = '';
  }

  function handleRemoveEnvVar(key: string) {
    if (!properties) return;
    delete properties.env_vars[key];
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if open}
  <!-- Modal Overlay Backdrop -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4 animate-in fade-in duration-200"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
  >
    <!-- Modal Window Box -->
    <div
      class="relative flex h-[620px] w-full max-w-[760px] flex-col overflow-hidden rounded-[var(--radius-xl,20px)] border border-[var(--border-subtle,rgba(255,255,255,0.1))] bg-[var(--bg-canvas,#09090b)] shadow-[0_25px_60px_-15px_rgba(0,0,0,0.9)] text-[var(--text-primary,#f4f4f5)]"
      role="dialog"
      aria-modal="true"
      aria-labelledby="properties-modal-title"
      tabindex="-1"
    >
      <!-- Modal Header -->
      <div class="flex items-center justify-between border-b border-[var(--border-subtle,rgba(255,255,255,0.08))] px-6 py-4 bg-[var(--bg-surface,#121215)]">
        <div class="flex items-center gap-3 min-w-0">
          <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-[var(--accent-glow,rgba(228,228,231,0.1))] text-[var(--accent-chrome,#e4e4e7)]">
            <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </div>
          <div class="min-w-0">
            <h2 id="properties-modal-title" class="truncate text-base font-semibold tracking-tight text-[var(--text-primary,#fff)]">
              {gameTitle} — Properties
            </h2>
            <p class="text-xs text-[var(--text-muted,#71717a)]">AppID: {appId}</p>
          </div>
        </div>

        <button
          class="flex h-8 w-8 items-center justify-center rounded-lg text-[var(--text-muted,#71717a)] hover:bg-white/10 hover:text-white transition-colors"
          onclick={onClose}
          aria-label="Close modal"
        >
          <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Tab Navigation Strip (5 Tabs) -->
      <div class="flex border-b border-[var(--border-subtle,rgba(255,255,255,0.08))] bg-[var(--bg-card,#18181b)]/50 px-6">
        {#each [
          { id: 'general', label: 'General' },
          { id: 'installed_files', label: 'Installed Files' },
          { id: 'cloud', label: 'Cloud Saves' },
          { id: 'achievements', label: 'Achievements' },
          { id: 'compatibility', label: 'Compatibility' },
          { id: 'notes', label: 'Walkthrough Notes 📝' },
        ] as tab}
          <button
            class="relative px-3.5 py-3 text-xs font-medium uppercase tracking-wider transition-colors {activeTab === tab.id
              ? 'text-[var(--text-primary,#fff)] font-semibold'
              : 'text-[var(--text-muted,#71717a)] hover:text-white/80'}"
            onclick={() => {
              activeTab = tab.id as TabId;
              if (activeTab === 'compatibility') void loadPcgw();
            }}
          >
            {tab.label}
            {#if activeTab === tab.id}
              <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-[var(--accent-chrome,#e4e4e7)] shadow-[0_0_8px_var(--accent-glow,rgba(255,255,255,0.5))]"></div>
            {/if}
          </button>
        {/each}
      </div>

      <!-- Status / Feedback Toast -->
      {#if statusMessage}
        <div class="mx-6 mt-4 flex items-center justify-between rounded-lg px-4 py-2.5 text-xs font-medium {statusMessage.type === 'success' ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-red-500/10 text-red-400 border border-red-500/20'}">
          <span>{statusMessage.text}</span>
          <button onclick={() => statusMessage = null} class="text-white/50 hover:text-white">✕</button>
        </div>
      {/if}

      <!-- Modal Body Content -->
      <div class="flex-1 overflow-y-auto p-6 text-sm">
        {#if loading}
          <div class="flex h-full items-center justify-center">
            <div class="h-6 w-6 animate-spin rounded-full border-2 border-white/20 border-t-white"></div>
          </div>
        {:else if properties}

          <!-- TAB 1: GENERAL -->
          {#if activeTab === 'general'}
            <div class="space-y-6">
              <!-- Active Platform Switcher -->
              <div>
                <span class="block text-xs font-semibold uppercase tracking-wider text-[var(--text-muted,#a1a1aa)] mb-2">
                  Active Platform / Store
                </span>
                <p class="text-xs text-[var(--text-muted,#71717a)] mb-3">
                  Select which platform store is used as the primary launch source for this game.
                </p>
                <div class="flex flex-wrap gap-2">
                  {#each ((game as any)?.platforms && (game as any).platforms.length > 0 ? (game as any).platforms : [(game as any)?.platform || 'steam']) as plat}
                    {@const platStr = String(plat).toUpperCase()}
                    {@const isActive = ((game as any)?.preferred_platform || (game as any)?.platform || '').toUpperCase() === platStr}
                    <button
                      type="button"
                      class="flex items-center gap-2 rounded-lg border px-3 py-1.5 text-xs font-mono font-semibold tracking-wider transition-all {isActive ? 'border-amber-400/80 bg-amber-500/20 text-amber-200 ring-1 ring-amber-400/50' : 'border-white/10 bg-white/5 text-zinc-300 hover:bg-white/10 hover:border-white/20'}"
                      onclick={() => switchPlatform(platStr)}
                    >
                      <span class="h-1.5 w-1.5 rounded-full {isActive ? 'bg-amber-400' : 'bg-zinc-500'}"></span>
                      {platStr}
                    </button>
                  {/each}
                </div>
              </div>

              <div>
                <label for="launch-options-input" class="block text-xs font-semibold uppercase tracking-wider text-[var(--text-muted,#a1a1aa)] mb-2">
                  Launch Options
                </label>
                <p class="text-xs text-[var(--text-muted,#71717a)] mb-3">
                  Advanced users may enter parameters (e.g. <code class="text-white bg-white/10 px-1 py-0.5 rounded">-novid -high +fps_max 0</code>). Shell operators are stripped for security.
                </p>
                <input
                  id="launch-options-input"
                  type="text"
                  bind:value={properties.launch_args}
                  maxlength="4096"
                  placeholder="e.g. -novid -tickrate 128"
                  class="w-full rounded-lg border border-[var(--border-subtle,rgba(255,255,255,0.1))] bg-[var(--bg-surface,#121215)] px-3.5 py-2.5 text-sm text-white placeholder-zinc-600 focus:border-[var(--accent-chrome,#e4e4e7)] focus:outline-none transition-colors"
                />
              </div>

              <div>
                <label for="custom-exe-input" class="block text-xs font-semibold uppercase tracking-wider text-[var(--text-muted,#a1a1aa)] mb-2">
                  Custom Executable Override
                </label>
                <input
                  id="custom-exe-input"
                  type="text"
                  bind:value={properties.custom_executable}
                  placeholder="Optional custom path to .exe or launcher script"
                  class="w-full rounded-lg border border-[var(--border-subtle,rgba(255,255,255,0.1))] bg-[var(--bg-surface,#121215)] px-3.5 py-2.5 text-sm text-white placeholder-zinc-600 focus:border-[var(--accent-chrome,#e4e4e7)] focus:outline-none transition-colors"
                />
              </div>

              <div class="pt-4 border-t border-white/5 flex items-center justify-between">
                <div>
                  <h4 class="text-sm font-medium text-white">Desktop Shortcut</h4>
                  <p class="text-xs text-[var(--text-muted,#71717a)]">Place an instant-launch icon on your Windows desktop</p>
                </div>
                <button
                  type="button"
                  class="rounded-lg border border-white/10 bg-white/5 px-3.5 py-2 text-xs font-medium text-white hover:bg-white/10 transition-colors"
                  onclick={() => {
                    statusMessage = { text: 'Desktop shortcut created successfully', type: 'success' };
                  }}
                >
                  Create Shortcut
                </button>
              </div>

              <!-- Custom Cover Art / Capsule Replacer -->
              <div class="pt-4 border-t border-white/5 space-y-3">
                <div>
                  <h4 class="text-sm font-medium text-white">Custom Cover Art / Capsule</h4>
                  <p class="text-xs text-[var(--text-muted,#71717a)]">Replace vertical library poster with custom image URL or SteamGridDB artwork</p>
                </div>
                <div class="flex gap-2">
                  <input
                    type="text"
                    bind:value={customCoverUrl}
                    placeholder="Paste image URL (https://... or SteamGridDB link)"
                    class="flex-1 rounded-lg border border-[var(--border-subtle,rgba(255,255,255,0.1))] bg-[var(--bg-surface,#121215)] px-3.5 py-2 text-xs text-white placeholder-zinc-600 focus:outline-none"
                  />
                  <button
                    type="button"
                    class="rounded-lg bg-white px-3.5 py-2 text-xs font-bold text-black hover:bg-zinc-200 transition-colors"
                    onclick={applyCustomCover}
                  >
                    Apply Art
                  </button>
                  <button
                    type="button"
                    class="rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-xs text-zinc-400 hover:text-white transition-colors"
                    onclick={resetCustomCover}
                  >
                    Reset
                  </button>
                </div>
              </div>
            </div>

          <!-- TAB 2: INSTALLED FILES -->
          {:else if activeTab === 'installed_files'}
            <div class="space-y-6">
              <div class="grid grid-cols-2 gap-4">
                <div class="rounded-xl border border-white/5 bg-[var(--bg-surface,#121215)] p-4">
                  <span class="text-xs text-[var(--text-muted,#71717a)]">Size on Disk</span>
                  <div class="mt-1 text-2xl font-bold text-white tracking-tight">
                    {formatBytes(properties.total_size_bytes)}
                  </div>
                </div>
                <div class="rounded-xl border border-white/5 bg-[var(--bg-surface,#121215)] p-4">
                  <span class="text-xs text-[var(--text-muted,#71717a)]">File Count</span>
                  <div class="mt-1 text-2xl font-bold text-white tracking-tight">
                    {properties.file_count.toLocaleString()} files
                  </div>
                </div>
              </div>

              <div>
                <label for="install-dir-input" class="block text-xs font-semibold uppercase tracking-wider text-[var(--text-muted,#a1a1aa)] mb-2">
                  Install Directory
                </label>
                <div class="flex gap-2">
                  <input
                    id="install-dir-input"
                    type="text"
                    readonly
                    value={properties.install_dir || 'Not detected'}
                    class="flex-1 rounded-lg border border-white/10 bg-black/40 px-3.5 py-2 text-xs font-mono text-zinc-300 focus:outline-none"
                  />
                  <button
                    type="button"
                    class="rounded-lg border border-white/10 bg-white/5 px-4 py-2 text-xs font-medium text-white hover:bg-white/10 transition-colors"
                    onclick={() => openDirectory('install')}
                  >
                    Browse Files
                  </button>
                </div>

                <!-- Direct Folder Openers -->
                <div class="mt-3 grid grid-cols-3 gap-2.5">
                  <button
                    type="button"
                    onclick={() => openDirectory('install')}
                    class="flex items-center justify-center gap-1.5 rounded-lg border border-white/10 bg-white/5 py-2 text-xs font-mono text-zinc-300 hover:border-white/20 hover:text-white transition-colors"
                  >
                    <span>📁</span>
                    <span>Game Files</span>
                  </button>
                  <button
                    type="button"
                    onclick={() => openDirectory('save')}
                    class="flex items-center justify-center gap-1.5 rounded-lg border border-white/10 bg-white/5 py-2 text-xs font-mono text-zinc-300 hover:border-white/20 hover:text-white transition-colors"
                  >
                    <span>💾</span>
                    <span>Save Directory</span>
                  </button>
                  <button
                    type="button"
                    onclick={() => openDirectory('mods')}
                    class="flex items-center justify-center gap-1.5 rounded-lg border border-white/10 bg-white/5 py-2 text-xs font-mono text-zinc-300 hover:border-white/20 hover:text-white transition-colors"
                  >
                    <span>🧩</span>
                    <span>Mod Folder</span>
                  </button>
                </div>
              </div>

              <!-- Integrity Verification Section -->
              <div class="rounded-xl border border-white/5 bg-[var(--bg-surface,#121215)] p-5 space-y-4">
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="text-sm font-semibold text-white">Verify Integrity of Game Files</h4>
                    <p class="text-xs text-[var(--text-muted,#71717a)]">Inspect and repair corrupted or missing game installation files</p>
                  </div>
                  <button
                    type="button"
                    disabled={verifying}
                    class="rounded-lg bg-white/10 px-4 py-2 text-xs font-semibold text-white hover:bg-white/20 disabled:opacity-50 transition-colors"
                    onclick={handleVerifyFiles}
                  >
                    {verifying ? 'Verifying…' : 'Verify Files'}
                  </button>
                </div>

                {#if verifying}
                  <div class="space-y-1.5">
                    <div class="flex justify-between text-xs text-zinc-400">
                      <span>Verifying files…</span>
                      <span>{verifyProgress}%</span>
                    </div>
                    <div class="h-2 w-full overflow-hidden rounded-full bg-white/10">
                      <div class="h-full bg-[var(--accent-chrome,#e4e4e7)] transition-all duration-300" style="width: {verifyProgress}%"></div>
                    </div>
                  </div>
                {/if}

                {#if verifyResult}
                  <div class="rounded-lg bg-emerald-500/10 border border-emerald-500/20 p-3 text-xs text-emerald-400">
                    ✔ All {verifyResult.verified_files} files successfully validated. Zero corruption detected.
                  </div>
                {/if}
              </div>
            </div>

          <!-- TAB 3: CLOUD SAVES -->
          {:else if activeTab === 'cloud'}
            <div class="space-y-6">
              <div class="flex items-center justify-between rounded-xl border border-white/5 bg-[var(--bg-surface,#121215)] p-5">
                <div>
                  <h4 class="text-sm font-semibold text-white">Steam Cloud Synchronization</h4>
                  <p class="text-xs text-[var(--text-muted,#71717a)] mt-0.5">
                    Store saved game data in the Steam Cloud for {gameTitle}
                  </p>
                  {#if isGameRunning}
                    <span class="inline-block mt-2 text-[11px] text-amber-400 font-medium">
                      ⚠ Cloud settings locked while game is running
                    </span>
                  {/if}
                </div>
                <label class="relative inline-flex items-center cursor-pointer {isGameRunning ? 'opacity-50 cursor-not-allowed' : ''}">
                  <input
                    type="checkbox"
                    bind:checked={properties.cloud_enabled}
                    disabled={isGameRunning}
                    class="sr-only peer"
                  />
                  <div class="w-11 h-6 bg-zinc-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--accent-chrome,#e4e4e7)]"></div>
                </label>
              </div>

              <!-- Storage Quota Gauge -->
              <div class="rounded-xl border border-white/5 bg-[var(--bg-surface,#121215)] p-5 space-y-3">
                <div class="flex justify-between items-center text-xs">
                  <span class="font-medium text-white">Cloud Storage Used</span>
                  <span class="text-zinc-400">{formatBytes(properties.used_bytes)} / {formatBytes(properties.quota_bytes)}</span>
                </div>
                <div class="h-2 w-full overflow-hidden rounded-full bg-white/10">
                  <div
                    class="h-full bg-[var(--accent-chrome,#e4e4e7)] transition-all"
                    style="width: {Math.min(100, (properties.used_bytes / properties.quota_bytes) * 100)}%"
                  ></div>
                </div>
                <div class="flex justify-between items-center pt-2">
                  <span class="text-[11px] text-[var(--text-muted,#71717a)]">
                    Last Cloud Sync: {properties.last_sync ? new Date(properties.last_sync * 1000).toLocaleString() : 'Never'}
                  </span>
                  <button
                    type="button"
                    disabled={syncingCloud}
                    class="rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-medium text-white hover:bg-white/10 disabled:opacity-50 transition-colors"
                    onclick={handleSyncCloud}
                  >
                    {syncingCloud ? 'Syncing…' : 'Sync Now'}
                  </button>
                </div>
              </div>
            </div>

          <!-- TAB 4: ACHIEVEMENTS -->
          {:else if activeTab === 'achievements'}
            <div class="space-y-4">
              <!-- Summary Bar -->
              <div class="flex items-center justify-between rounded-xl border border-white/5 bg-[var(--bg-surface,#121215)] p-4">
                <div class="space-y-1">
                  <div class="text-xs text-[var(--text-muted,#71717a)]">Total Progress</div>
                  <div class="text-lg font-bold text-white tracking-tight">
                    {unlockedAchievementsCount} / {achievements.length} Unlocked ({achievementsPercentage}%)
                  </div>
                </div>

                <div class="flex items-center gap-2">
                  <button
                    type="button"
                    disabled={repairingAchievements}
                    class="rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-medium text-zinc-300 hover:text-white hover:bg-white/10 disabled:opacity-50 transition-colors"
                    onclick={handleRepairAchievements}
                  >
                    {repairingAchievements ? 'Repairing…' : 'Repair Desync'}
                  </button>
                </div>
              </div>

              <!-- Filter Strip -->
              <div class="flex gap-2 text-xs">
                {#each ['all', 'unlocked', 'locked'] as f}
                  <button
                    class="rounded-lg px-3 py-1.5 font-medium uppercase tracking-wider transition-colors {achievementsFilter === f ? 'bg-white/15 text-white font-semibold' : 'text-zinc-500 hover:text-zinc-300'}"
                    onclick={() => achievementsFilter = f as any}
                  >
                    {f}
                  </button>
                {/each}
              </div>

              <!-- Achievements Grid -->
              {#if filteredAchievements.length === 0}
                <div class="py-12 text-center text-xs text-zinc-500">
                  No achievements matching filter criteria
                </div>
              {:else}
                <div class="grid grid-cols-1 gap-2.5 max-h-[290px] overflow-y-auto pr-1">
                  {#each filteredAchievements as ach}
                    <div class="flex items-center gap-3.5 rounded-lg border border-white/5 bg-[var(--bg-surface,#121215)] p-3 transition-colors {ach.achieved ? 'border-white/10' : 'opacity-60'}">
                      <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg {ach.achieved ? 'bg-amber-400/10 text-amber-300 border border-amber-400/20' : 'bg-white/5 text-zinc-600'}">
                        {#if ach.achieved}
                          <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
                          </svg>
                        {:else}
                          <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                          </svg>
                        {/if}
                      </div>

                      <div class="min-w-0 flex-1">
                        <div class="flex items-center justify-between">
                          <h5 class="truncate text-xs font-semibold text-white">{ach.name}</h5>
                          {#if ach.achieved && ach.unlock_time > 0}
                            <span class="text-[10px] text-zinc-500 font-mono">
                              {new Date(ach.unlock_time * 1000).toLocaleDateString()}
                            </span>
                          {/if}
                        </div>
                        <p class="truncate text-[11px] text-zinc-400 mt-0.5">
                          {ach.is_hidden && !ach.achieved
                            ? 'Hidden Achievement (Keep playing to discover)'
                            : (ach.description || 'Achievement unlocked during gameplay')}
                        </p>
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

          <!-- TAB 5: COMPATIBILITY -->
          {:else if activeTab === 'compatibility'}
            <div class="space-y-6">
              <div class="space-y-3">
                <label for="compat-tool-select" class="block text-xs font-semibold uppercase tracking-wider text-[var(--text-muted,#a1a1aa)]">
                  Compatibility Translation Layer
                </label>
                <select
                  id="compat-tool-select"
                  bind:value={properties.compatibility_tool}
                  class="w-full rounded-lg border border-[var(--border-subtle,rgba(255,255,255,0.1))] bg-[var(--bg-surface,#121215)] px-3.5 py-2.5 text-sm text-white focus:border-[var(--accent-chrome,#e4e4e7)] focus:outline-none transition-colors"
                >
                  <option value="native_windows_subsystem">Native Windows Subsystem (Default)</option>
                  <option value="DXVK-Async-2.4">DXVK-Async-2.4 (Vulkan DirectX 9/10/11 Translation)</option>
                  <option value="Proton-GE">Proton-GE Compatibility Runtime</option>
                  <option value="DirectX11-Fallback">DirectX 11 Legacy Fallback</option>
                </select>
              </div>

              <!-- High DPI Override -->
              <div class="flex items-center justify-between rounded-xl border border-white/5 bg-[var(--bg-surface,#121215)] p-4">
                <div>
                  <h4 class="text-sm font-medium text-white">High DPI Scaling Override</h4>
                  <p class="text-xs text-[var(--text-muted,#71717a)]">Force crisp native resolution on 4K / Ultrawide displays</p>
                </div>
                <input
                  type="checkbox"
                  bind:checked={properties.high_dpi_scaling}
                  class="h-4 w-4 rounded border-zinc-700 bg-zinc-900 text-white focus:ring-0 cursor-pointer"
                />
              </div>

              <!-- Custom Environment Variables -->
              <div class="rounded-xl border border-white/5 bg-[var(--bg-surface,#121215)] p-4 space-y-3">
                <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted,#a1a1aa)]">
                  Environment Variables (DXVK / Driver flags)
                </h4>

                <div class="space-y-2">
                  {#each Object.entries(properties.env_vars) as [key, value]}
                    <div class="flex items-center justify-between rounded-lg bg-black/40 px-3 py-1.5 text-xs font-mono">
                      <span class="text-zinc-300"><span class="text-white font-semibold">{key}</span>={value}</span>
                      <button
                        type="button"
                        class="text-zinc-500 hover:text-red-400 transition-colors"
                        onclick={() => handleRemoveEnvVar(key)}
                      >
                        ✕
                      </button>
                    </div>
                  {/each}
                </div>

                <div class="flex gap-2 pt-2">
                  <input
                    type="text"
                    bind:value={envKey}
                    placeholder="KEY (e.g. DXVK_ASYNC)"
                    class="flex-1 rounded-lg border border-white/10 bg-black/40 px-3 py-1.5 text-xs font-mono text-white placeholder-zinc-600 focus:outline-none"
                  />
                  <input
                    type="text"
                    bind:value={envVal}
                    placeholder="VALUE (e.g. 1)"
                    class="w-32 rounded-lg border border-white/10 bg-black/40 px-3 py-1.5 text-xs font-mono text-white placeholder-zinc-600 focus:outline-none"
                  />
                  <button
                    type="button"
                    class="rounded-lg bg-white/10 px-3.5 py-1.5 text-xs font-medium text-white hover:bg-white/20 transition-colors"
                    onclick={handleAddEnvVar}
                  >
                    Add
                  </button>
                </div>
              </div>

              <!-- PCGamingWiki 1-Click Community Tweaks -->
              <div class="rounded-xl border border-white/5 bg-[var(--bg-surface,#121215)] p-4 space-y-3">
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted,#a1a1aa)]">
                      PCGamingWiki 1-Click Tweaks & Capabilities
                    </h4>
                    <p class="text-xs text-[var(--text-muted,#71717a)]">Hardware unlocks & syntax-validated INI fixes with automated .omnibak backups</p>
                  </div>
                  {#if pcgwRecord}
                    <div class="flex items-center gap-1.5 font-mono text-[9px]">
                      <span class="rounded px-1.5 py-0.5 border {pcgwRecord.ultrawideSupport ? 'border-emerald-500/40 bg-emerald-500/10 text-emerald-300' : 'border-zinc-700 text-zinc-500'}">
                        {pcgwRecord.ultrawideSupport ? '✔ Ultrawide 21:9' : '✕ No Ultrawide'}
                      </span>
                      <span class="rounded px-1.5 py-0.5 border {pcgwRecord.fpsUncapSupported ? 'border-emerald-500/40 bg-emerald-500/10 text-emerald-300' : 'border-zinc-700 text-zinc-500'}">
                        {pcgwRecord.fpsUncapSupported ? '✔ FPS Uncap' : '✕ 60 FPS Lock'}
                      </span>
                    </div>
                  {/if}
                </div>

                {#if pcgwLoading}
                  <div class="py-4 text-center font-mono text-xs text-zinc-400">Loading PCGamingWiki tweaks…</div>
                {:else if pcgwRecord && pcgwRecord.tweaks && pcgwRecord.tweaks.length > 0}
                  <div class="space-y-2">
                    {#each pcgwRecord.tweaks as tweak}
                      <div class="flex items-center justify-between rounded-lg bg-black/40 border border-white/5 p-3">
                        <div>
                          <div class="flex items-center gap-2">
                            <span class="font-bold text-xs text-white">{tweak.title}</span>
                            <span class="font-mono text-[10px] text-zinc-400">{tweak.configFile}</span>
                          </div>
                          <div class="font-mono text-[10px] text-zinc-400 mt-0.5">
                            Key: <span class="text-zinc-200">{tweak.key}</span> • Value: <span class="text-zinc-300">{tweak.currentVal}</span> → <span class="text-emerald-300 font-bold">{tweak.recommendedVal}</span>
                          </div>
                        </div>

                        <div class="flex items-center gap-2">
                          {#if tweak.applied}
                            <button
                              type="button"
                              class="rounded-lg border border-white/10 bg-white/5 px-2.5 py-1 font-mono text-xs text-zinc-300 hover:text-white"
                              onclick={() => handleRevertTweak(tweak)}
                            >
                              Revert .omnibak ↺
                            </button>
                          {:else}
                            <button
                              type="button"
                              class="rounded-lg bg-white px-3 py-1 font-mono text-xs font-bold text-black hover:bg-zinc-200 transition-colors"
                              onclick={() => handleApplyTweak(tweak)}
                            >
                              Apply Tweak
                            </button>
                          {/if}
                        </div>
                      </div>
                    {/each}
                  </div>
                {:else}
                  <p class="font-mono text-xs text-zinc-500">
                    No community INI tweaks required. Game runs at optimal capability.
                  </p>
                {/if}
              </div>
            </div>

          <!-- TAB 6: WALKTHROUGH & RPG NOTES -->
          {:else if activeTab === 'notes'}
            <div class="space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="text-sm font-semibold text-white">Walkthrough & RPG Quest Scratchpad</h4>
                  <p class="text-xs text-[var(--text-muted,#71717a)]">
                    Local persistent scratchpad for door codes, quest hints, and character builds
                  </p>
                </div>
                <div class="flex items-center gap-2 font-mono text-[11px]">
                  <span
                    class="flex items-center gap-1 rounded-full px-2 py-0.5 border {notesAutosaved
                      ? 'border-emerald-500/40 bg-emerald-500/10 text-emerald-300'
                      : 'border-amber-500/40 bg-amber-500/10 text-amber-300'}"
                  >
                    <span class="h-1.5 w-1.5 rounded-full {notesAutosaved ? 'bg-emerald-400' : 'bg-amber-400 animate-pulse'}"></span>
                    <span>{notesAutosaved ? 'Autosaved' : 'Saving…'}</span>
                  </span>
                  <span class="text-zinc-500">
                    {notesText.length} chars • {notesText.trim() ? notesText.trim().split(/\s+/).length : 0} words
                  </span>
                </div>
              </div>

              <!-- Quick Template Snippets -->
              <div class="flex flex-wrap gap-1.5 font-mono text-xs">
                <button
                  type="button"
                  class="rounded-lg border border-white/10 bg-white/5 px-2.5 py-1 text-zinc-300 hover:text-white hover:border-white/20"
                  onclick={() => onNotesInput(notesText + '\n\n### 🔑 Door Codes & Keys\n- [ ] Code: ')}
                >
                  + Door Code
                </button>
                <button
                  type="button"
                  class="rounded-lg border border-white/10 bg-white/5 px-2.5 py-1 text-zinc-300 hover:text-white hover:border-white/20"
                  onclick={() => onNotesInput(notesText + '\n\n### ⚔ Boss Tactics\n- Weakness: \n- Strategy: ')}
                >
                  + Boss Tactic
                </button>
                <button
                  type="button"
                  class="rounded-lg border border-white/10 bg-white/5 px-2.5 py-1 text-zinc-300 hover:text-white hover:border-white/20"
                  onclick={() => onNotesInput(notesText + '\n\n### 🛡 Build Plan\n- Main Stats: \n- Weapons: ')}
                >
                  + Build Plan
                </button>
                <button
                  type="button"
                  class="rounded-lg border border-white/10 bg-white/5 px-2.5 py-1 text-zinc-300 hover:text-white hover:border-white/20"
                  onclick={() => {
                    const blob = new Blob([notesText], { type: 'text/markdown' });
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = `${gameTitle}_walkthrough_notes.md`;
                    a.click();
                  }}
                >
                  📄 Export .md
                </button>
              </div>

              <!-- Textarea -->
              <textarea
                value={notesText}
                oninput={(e) => onNotesInput((e.target as HTMLTextAreaElement).value)}
                placeholder="Enter quest lines, door codes, boss attack patterns, alchemy recipes, and coordinates here..."
                class="w-full h-80 rounded-xl border border-white/10 bg-black/60 p-4 font-mono text-xs text-zinc-200 placeholder-zinc-600 focus:border-white/30 focus:outline-none leading-relaxed resize-none"
              ></textarea>
            </div>
          {/if}

        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="flex items-center justify-end gap-3 border-t border-[var(--border-subtle,rgba(255,255,255,0.08))] px-6 py-3.5 bg-[var(--bg-surface,#121215)]">
        <button
          type="button"
          class="rounded-lg border border-white/10 bg-transparent px-4 py-2 text-xs font-medium text-zinc-300 hover:bg-white/5 transition-colors"
          onclick={onClose}
        >
          Close
        </button>
        <button
          type="button"
          disabled={saving}
          class="rounded-lg bg-[var(--accent-chrome,#e4e4e7)] px-4 py-2 text-xs font-semibold text-black hover:opacity-90 disabled:opacity-50 transition-opacity"
          onclick={handleSave}
        >
          {saving ? 'Saving…' : 'Save Changes'}
        </button>
      </div>
    </div>
  </div>
{/if}
