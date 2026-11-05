<!--
  SaveVaultModal.svelte — Save-State Time Machine UI.
  Provides snapshot timeline, 1-click restore with automated safeguard rollback,
  manual bookmark creation, and direct save folder opener in Explorer.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { resolveArtUrl, type Game } from '../stores/library';

  interface SaveSnapshot {
    snapshot_id: string;
    game_id: string;
    timestamp: number;
    archive_size_bytes: number;
    sha256_checksum: string;
    title: string;
    label: string;
    kind: string;
    file_count: number;
    uncompressed_bytes: number;
    source_path: string;
    archive_path: string;
    pinned: boolean;
    note: string | null;
  }

  interface RestoreReport {
    success: boolean;
    snapshot_id: string;
    target_path: string;
    restored_files_count: number;
    safety_snapshot_id: string | null;
    restored_at: number;
  }

  interface Props {
    open: boolean;
    onClose: () => void;
    game: Game | null;
  }

  let { open, onClose, game }: Props = $props();

  let snapshots = $state<SaveSnapshot[]>([]);
  let loading = $state(false);
  let actionInProgress = $state(false);
  let statusMessage = $state<{ text: string; isError: boolean } | null>(null);
  let customLabelInput = $state('');
  let showCreateInput = $state(false);

  $effect(() => {
    if (open && game) {
      void loadSnapshots();
    }
  });

  async function loadSnapshots() {
    if (!game) return;
    loading = true;
    try {
      snapshots = await invoke<SaveSnapshot[]>('list_save_snapshots', {
        gameId: game.id.toString(),
      });
    } catch (err: any) {
      statusMessage = { text: err?.message ?? 'Failed to load snapshots', isError: true };
    } finally {
      loading = false;
    }
  }

  async function createSnapshot() {
    if (!game || actionInProgress) return;
    actionInProgress = true;
    try {
      const snap = await invoke<SaveSnapshot>('create_save_snapshot', {
        gameId: game.id.toString(),
        label: customLabelInput.trim() || 'Manual Bookmark',
        kind: 'manual',
        customPath: null,
      });
      snapshots = [snap, ...snapshots];
      customLabelInput = '';
      showCreateInput = false;
      statusMessage = { text: 'Snapshot created successfully!', isError: false };
    } catch (err: any) {
      statusMessage = { text: err?.message ?? 'Failed to create snapshot', isError: true };
    } finally {
      actionInProgress = false;
    }
  }

  async function restoreSnapshot(snapshotId: string) {
    if (!game || actionInProgress) return;
    if (!confirm('Restore save to this point in time? An emergency safeguard backup will be created automatically.')) {
      return;
    }

    actionInProgress = true;
    try {
      const report = await invoke<RestoreReport>('restore_save_snapshot', {
        gameId: game.id.toString(),
        snapshotId,
      });
      statusMessage = {
        text: `Restored ${report.restored_files_count} files successfully! Safeguard: ${report.safety_snapshot_id ?? 'None'}`,
        isError: false,
      };
      await loadSnapshots();
    } catch (err: any) {
      statusMessage = { text: err?.message ?? 'Failed to restore snapshot', isError: true };
    } finally {
      actionInProgress = false;
    }
  }

  async function deleteSnapshot(snapshotId: string) {
    if (!game || actionInProgress) return;
    actionInProgress = true;
    try {
      await invoke('delete_save_snapshot', {
        gameId: game.id.toString(),
        snapshotId,
      });
      snapshots = snapshots.filter((s) => s.snapshot_id !== snapshotId);
      statusMessage = { text: 'Snapshot deleted', isError: false };
    } catch (err: any) {
      statusMessage = { text: err?.message ?? 'Failed to delete snapshot', isError: true };
    } finally {
      actionInProgress = false;
    }
  }

  async function togglePin(snapshotId: string) {
    if (!game) return;
    try {
      const pinned = await invoke<boolean>('toggle_snapshot_pin', {
        gameId: game.id.toString(),
        snapshotId,
      });
      snapshots = snapshots.map((s) => (s.snapshot_id === snapshotId ? { ...s, pinned } : s));
    } catch (err: any) {
      statusMessage = { text: err?.message ?? 'Failed to toggle pin', isError: true };
    }
  }

  async function openSaveFolder() {
    if (!game) return;
    try {
      await invoke('open_save_folder', { gameId: game.id.toString() });
    } catch (err: any) {
      statusMessage = { text: err?.message ?? 'Failed to open save folder', isError: true };
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

{#if open && game}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/85 p-4 backdrop-blur-md"
    role="dialog"
    aria-modal="true"
  >
    <div
      class="w-full max-w-2xl max-h-[90vh] flex flex-col rounded-2xl border border-white/15 bg-[#121215] text-white shadow-2xl overflow-hidden"
    >
      <!-- Header Zone -->
      <div class="flex items-center justify-between border-b border-white/10 p-5 bg-white/[0.02]">
        <div class="flex items-center gap-3.5">
          {#if resolveArtUrl(game.art?.poster)}
            <img
              src={resolveArtUrl(game.art?.poster)}
              alt={game.title}
              class="h-12 w-9 rounded object-cover border border-white/15 shadow-sm"
            />
          {/if}
          <div>
            <div class="flex items-center gap-2">
              <span class="rounded bg-white/10 px-1.5 py-0.5 font-mono text-[9px] uppercase tracking-wider text-zinc-300">
                Save Vault
              </span>
              <h2 class="text-base font-bold tracking-tight">{game.title}</h2>
            </div>
            <p class="font-mono text-[11px] text-zinc-400 mt-0.5">
              Save-State Time Machine • Automated Pre/Post-session snapshots
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={openSaveFolder}
            class="flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 font-mono text-xs text-zinc-300 hover:border-white/25 hover:text-white transition-all"
            title="Open Save Directory in Windows Explorer"
          >
            <span>📁</span>
            <span>Open Folder</span>
          </button>
          <button
            type="button"
            onclick={onClose}
            class="rounded-lg border border-white/10 px-2.5 py-1.5 font-mono text-xs text-zinc-400 hover:border-white/30 hover:text-white"
          >
            ESC
          </button>
        </div>
      </div>

      <!-- Action & Notification Bar -->
      <div class="flex items-center justify-between border-b border-white/10 px-5 py-3 bg-black/40">
        <div class="flex items-center gap-2">
          {#if !showCreateInput}
            <button
              type="button"
              onclick={() => (showCreateInput = true)}
              class="flex items-center gap-1.5 rounded-lg bg-white px-3 py-1.5 font-mono text-xs font-bold text-black shadow hover:bg-zinc-200 transition-all"
            >
              <span>💾</span>
              <span>Create Snapshot</span>
            </button>
          {:else}
            <div class="flex items-center gap-2">
              <input
                type="text"
                bind:value={customLabelInput}
                placeholder="Snapshot label (e.g. Before Boss)..."
                class="rounded-lg border border-white/20 bg-zinc-900 px-3 py-1 font-mono text-xs text-white outline-none focus:border-white w-64"
                onkeydown={(e) => e.key === 'Enter' && createSnapshot()}
              />
              <button
                type="button"
                onclick={createSnapshot}
                disabled={actionInProgress}
                class="rounded-lg bg-white px-3 py-1 font-mono text-xs font-bold text-black hover:bg-zinc-200"
              >
                Save
              </button>
              <button
                type="button"
                onclick={() => (showCreateInput = false)}
                class="text-xs font-mono text-zinc-400 hover:text-white"
              >
                Cancel
              </button>
            </div>
          {/if}
        </div>

        {#if statusMessage}
          <div
            class="font-mono text-xs px-2.5 py-1 rounded border {statusMessage.isError
              ? 'border-rose-500/40 bg-rose-950/40 text-rose-300'
              : 'border-emerald-500/40 bg-emerald-950/40 text-emerald-300'}"
          >
            {statusMessage.text}
          </div>
        {/if}
      </div>

      <!-- Snapshot Timeline -->
      <div class="flex-1 overflow-y-auto p-5 space-y-3">
        {#if loading}
          <div class="py-12 text-center font-mono text-xs text-zinc-400">
            Scanning Save Vault snapshots…
          </div>
        {:else if snapshots.length === 0}
          <div class="py-12 text-center space-y-2">
            <p class="text-zinc-400 text-sm">No snapshots recorded for this title yet.</p>
            <p class="font-mono text-xs text-zinc-500">
              Snapshots are captured automatically when launching and exiting game sessions, or manually via "Create Snapshot".
            </p>
          </div>
        {:else}
          {#each snapshots as snap}
            <div
              class="relative flex items-center justify-between rounded-xl border border-white/10 bg-white/[0.03] p-4 transition-all hover:border-white/20 hover:bg-white/[0.05]"
            >
              <div class="space-y-1">
                <div class="flex items-center gap-2">
                  <span
                    class="rounded px-2 py-0.5 font-mono text-[9px] font-bold uppercase {snap.kind === 'safeguard'
                      ? 'border border-amber-400/40 bg-amber-500/20 text-amber-300'
                      : snap.kind === 'pre_launch'
                        ? 'border border-cyan-400/40 bg-cyan-500/20 text-cyan-300'
                        : snap.kind === 'post_launch'
                          ? 'border border-emerald-400/40 bg-emerald-500/20 text-emerald-300'
                          : 'border border-purple-400/40 bg-purple-500/20 text-purple-300'}"
                  >
                    {snap.kind}
                  </span>

                  {#if snap.pinned}
                    <span class="font-mono text-xs text-amber-400" title="Pinned (Immune to rotation purge)">📌</span>
                  {/if}

                  <h4 class="font-bold text-sm text-white">{snap.label}</h4>
                </div>

                <div class="flex items-center gap-3 font-mono text-[11px] text-zinc-400">
                  <span>{new Date(snap.timestamp * 1000).toLocaleString()}</span>
                  <span>•</span>
                  <span>{formatBytes(snap.archive_size_bytes)}</span>
                  <span>•</span>
                  <span>{snap.file_count} files</span>
                </div>
              </div>

              <!-- Controls -->
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  onclick={() => togglePin(snap.snapshot_id)}
                  class="rounded-lg border border-white/10 bg-white/5 p-1.5 text-xs text-zinc-400 hover:text-white hover:border-white/30"
                  title={snap.pinned ? 'Unpin' : 'Pin Snapshot'}
                >
                  {snap.pinned ? '📌' : '📍'}
                </button>

                <button
                  type="button"
                  onclick={() => restoreSnapshot(snap.snapshot_id)}
                  disabled={actionInProgress}
                  class="rounded-lg border border-emerald-500/40 bg-emerald-500/10 px-3 py-1.5 font-mono text-xs font-semibold text-emerald-300 hover:bg-emerald-500/20 transition-all"
                >
                  Restore ↺
                </button>

                <button
                  type="button"
                  onclick={() => deleteSnapshot(snap.snapshot_id)}
                  disabled={actionInProgress}
                  class="rounded-lg border border-rose-500/30 bg-rose-500/5 px-2.5 py-1.5 font-mono text-xs text-rose-400 hover:bg-rose-500/20 hover:text-rose-200 transition-all"
                  title="Delete snapshot"
                >
                  ✕
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}
