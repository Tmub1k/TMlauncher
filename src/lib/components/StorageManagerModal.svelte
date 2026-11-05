<!--
  StorageManagerModal.svelte — Disk Footprint & Shader Cache Reclaimer.
  Visualizes per-game storage consumption and purges orphaned Steam/GPU shader caches.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { fetchLibrary, type Game } from '../stores/library';

  interface CacheItem {
    id: string;
    app_id: number | null;
    game_title: string;
    path: string;
    size_bytes: number;
    is_installed: boolean;
    category: string;
    is_safe_to_delete: boolean;
  }

  interface ShaderCacheSummary {
    total_bytes: number;
    item_count: number;
    caches: CacheItem[];
    scanned_at: number;
  }

  interface CleanReport {
    total_freed_bytes: number;
    cleaned_items_count: number;
    failed_items_count: number;
    skipped_locked_files: number;
    cleaned_at: number;
    message: string;
  }

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  let activeTab = $state<'shader' | 'games'>('shader');
  let summary = $state<ShaderCacheSummary | null>(null);
  let games = $state<Game[]>([]);
  let selectedIds = $state<Set<string>>(new Set());
  let scanning = $state(false);
  let cleaning = $state(false);
  let report = $state<CleanReport | null>(null);
  let statusError = $state<string | null>(null);

  $effect(() => {
    if (open) {
      void runScan();
      void fetchLibrary().then((g) => {
        games = g;
      }).catch(() => {});
    }
  });

  async function runScan() {
    scanning = true;
    statusError = null;
    report = null;
    try {
      summary = await invoke<ShaderCacheSummary>('scan_shader_caches');
      // Auto-select all orphaned caches
      const orphans = new Set<string>();
      for (const c of summary.caches) {
        if (!c.is_installed && c.is_safe_to_delete) {
          orphans.add(c.id);
        }
      }
      selectedIds = orphans;
    } catch (err: any) {
      statusError = err?.message ?? 'Failed to scan shader caches';
    } finally {
      scanning = false;
    }
  }

  async function purgeSelected() {
    if (cleaning || selectedIds.size === 0) return;
    cleaning = true;
    statusError = null;
    try {
      report = await invoke<CleanReport>('clean_shader_caches', {
        request: {
          item_ids: Array.from(selectedIds),
          only_orphans: false,
        },
      });
      await runScan();
    } catch (err: any) {
      statusError = err?.message ?? 'Failed to clean shader caches';
    } finally {
      cleaning = false;
    }
  }

  function toggleSelect(id: string) {
    const next = new Set(selectedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    selectedIds = next;
  }

  function selectAllOrphans() {
    if (!summary) return;
    const orphans = new Set<string>();
    for (const c of summary.caches) {
      if (!c.is_installed) {
        orphans.add(c.id);
      }
    }
    selectedIds = orphans;
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  let selectedTotalBytes = $derived.by(() => {
    if (!summary) return 0;
    let sum = 0;
    for (const c of summary.caches) {
      if (selectedIds.has(c.id)) {
        sum += c.size_bytes;
      }
    }
    return sum;
  });

  let gamesList = $derived.by(() => {
    return games.slice().sort((a, b) => (b.playtimeMinutes ?? 0) - (a.playtimeMinutes ?? 0));
  });
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/85 p-4 backdrop-blur-md"
    role="dialog"
    aria-modal="true"
  >
    <div
      class="w-full max-w-3xl max-h-[90vh] flex flex-col rounded-2xl border border-white/15 bg-[#121215] text-white shadow-2xl overflow-hidden"
    >
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-white/10 p-5 bg-white/[0.02]">
        <div class="flex items-center gap-3">
          <div class="flex h-9 w-9 items-center justify-center rounded-xl border border-white/20 bg-white/10 text-base">
            💽
          </div>
          <div>
            <h2 class="text-base font-bold tracking-tight">Storage & Shader Cache Manager</h2>
            <p class="font-mono text-[11px] text-zinc-400">
              Recover gigabytes from orphaned Steam caches, DirectX, and GPU driver dumps
            </p>
          </div>
        </div>

        <div class="flex items-center gap-3">
          <!-- Master Tabs -->
          <div class="flex rounded-lg border border-white/10 bg-black/40 p-0.5 font-mono text-xs">
            <button
              type="button"
              onclick={() => (activeTab = 'shader')}
              class="rounded-md px-3 py-1 transition-all {activeTab === 'shader'
                ? 'bg-white text-black font-bold shadow'
                : 'text-zinc-400 hover:text-white'}"
            >
              Shader Reclaimer
            </button>
            <button
              type="button"
              onclick={() => (activeTab = 'games')}
              class="rounded-md px-3 py-1 transition-all {activeTab === 'games'
                ? 'bg-white text-black font-bold shadow'
                : 'text-zinc-400 hover:text-white'}"
            >
              Games Footprint
            </button>
          </div>

          <button
            type="button"
            onclick={onClose}
            class="rounded-lg border border-white/10 px-2.5 py-1.5 font-mono text-xs text-zinc-400 hover:border-white/30 hover:text-white"
          >
            ESC
          </button>
        </div>
      </div>

      <!-- Tab 1: Shader Reclaimer -->
      {#if activeTab === 'shader'}
        <!-- Metrics / Summary Banner -->
        <div class="flex items-center justify-between border-b border-white/10 bg-black/40 px-5 py-3">
          <div class="flex items-center gap-4">
            <div>
              <span class="block font-mono text-[10px] uppercase text-zinc-400">Total Scanned</span>
              <strong class="font-mono text-sm text-white">
                {summary ? formatBytes(summary.total_bytes) : '0 B'}
              </strong>
            </div>
            <div class="h-6 w-px bg-white/10"></div>
            <div>
              <span class="block font-mono text-[10px] uppercase text-emerald-400">Selected to Reclaim</span>
              <strong class="font-mono text-sm text-emerald-300">
                {formatBytes(selectedTotalBytes)} ({selectedIds.size} items)
              </strong>
            </div>
          </div>

          <div class="flex items-center gap-2">
            <button
              type="button"
              onclick={selectAllOrphans}
              class="rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 font-mono text-xs text-zinc-300 hover:border-white/20 hover:text-white"
            >
              Select Orphans
            </button>
            <button
              type="button"
              onclick={purgeSelected}
              disabled={cleaning || selectedIds.size === 0}
              class="rounded-lg bg-emerald-500 px-4 py-1.5 font-mono text-xs font-bold text-black shadow-md hover:bg-emerald-400 disabled:opacity-50"
            >
              {cleaning ? 'Purging…' : `Purge Selected (${formatBytes(selectedTotalBytes)}) 🧹`}
            </button>
          </div>
        </div>

        {#if report}
          <div class="border-b border-emerald-500/30 bg-emerald-950/30 px-5 py-2.5 font-mono text-xs text-emerald-300">
            ✔ {report.message} (Skipped {report.skipped_locked_files} active/locked driver files)
          </div>
        {:else if statusError}
          <div class="border-b border-rose-500/30 bg-rose-950/30 px-5 py-2.5 font-mono text-xs text-rose-300">
            ⚠ {statusError}
          </div>
        {/if}

        <!-- Cache List Table -->
        <div class="flex-1 overflow-y-auto p-5 space-y-2">
          {#if scanning}
            <div class="py-12 text-center font-mono text-xs text-zinc-400">
              Scanning Steam and GPU shader caches…
            </div>
          {:else if !summary || summary.caches.length === 0}
            <div class="py-12 text-center font-mono text-xs text-zinc-400">
              No reclaimable shader caches found. System drives are clean.
            </div>
          {:else}
            {#each summary.caches as item}
              <div
                class="flex items-center justify-between rounded-xl border border-white/10 bg-white/[0.03] p-3 transition-all hover:border-white/20 hover:bg-white/[0.05]"
              >
                <div class="flex items-center gap-3">
                  <input
                    type="checkbox"
                    checked={selectedIds.has(item.id)}
                    onchange={() => toggleSelect(item.id)}
                    class="h-4 w-4 rounded border-white/20 bg-black accent-emerald-500 cursor-pointer"
                  />
                  <div>
                    <div class="flex items-center gap-2">
                      <span
                        class="rounded px-1.5 py-0.5 font-mono text-[9px] font-bold uppercase {item.is_installed
                          ? 'border border-blue-400/40 bg-blue-500/20 text-blue-300'
                          : 'border border-amber-400/40 bg-amber-500/20 text-amber-300'}"
                      >
                        {item.is_installed ? 'Installed' : '⚠ Orphaned'}
                      </span>
                      <span class="rounded bg-white/10 px-1.5 py-0.5 font-mono text-[9px] uppercase text-zinc-400">
                        {item.category}
                      </span>
                      <h4 class="font-bold text-sm text-white">{item.game_title}</h4>
                    </div>
                    <p class="font-mono text-[10px] text-zinc-400 mt-0.5 truncate max-w-lg">{item.path}</p>
                  </div>
                </div>

                <div class="font-mono text-sm font-semibold text-zinc-200">
                  {formatBytes(item.size_bytes)}
                </div>
              </div>
            {/each}
          {/if}
        </div>

      <!-- Tab 2: Games Footprint -->
      {:else}
        <div class="flex-1 overflow-y-auto p-5 space-y-3">
          <div class="rounded-xl border border-white/10 bg-white/[0.02] p-4">
            <h3 class="font-bold text-sm text-white">Visual Disk Breakdown</h3>
            <p class="font-mono text-[11px] text-zinc-400 mt-0.5">
              Installed games sorted by active session activity & estimated disk space
            </p>
          </div>

          {#each gamesList as game}
            <div class="flex items-center justify-between rounded-xl border border-white/10 bg-white/[0.03] p-3">
              <div class="flex items-center gap-3">
                <span class="rounded bg-white/10 px-2 py-0.5 font-mono text-[10px] uppercase text-zinc-300">
                  {game.platform}
                </span>
                <div>
                  <h4 class="font-bold text-sm text-white">{game.title}</h4>
                  <div class="flex items-center gap-2 font-mono text-[10px] text-zinc-400 mt-0.5">
                    <span>Playtime: {Math.floor((game.playtimeMinutes ?? 0) / 60)}h {(game.playtimeMinutes ?? 0) % 60}m</span>
                    <span>•</span>
                    <span class="text-emerald-400">Status: Installed</span>
                  </div>
                </div>
              </div>

              <div class="flex items-center gap-3 font-mono text-xs">
                <span class="rounded bg-white/5 border border-white/10 px-2 py-1 text-zinc-300">
                  Base Install (~25 GB)
                </span>
                <span class="rounded bg-white/5 border border-white/10 px-2 py-1 text-zinc-400">
                  Saves (~15 MB)
                </span>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/if}
