<!--
  MediaGallery.svelte — Zero-overlay DXGI Screenshot & Hardware Video Clip Gallery.
  Includes file deletion, fullscreen lightbox preview, and 3D Game GUI styling.
-->
<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import {
    deleteMediaItem,
    formatBytes,
    toggleClipRecording,
    triggerScreenshot,
    type MediaItem,
  } from '../stores/library';

  interface Props {
    items: MediaItem[];
    onRefresh: () => void;
  }

  let { items, onRefresh }: Props = $props();
  let recording = $state(false);
  let gameFilter = $state('all');
  let typeFilter = $state<'all' | 'screenshot' | 'clip'>('all');
  let deletingPath = $state<string | null>(null);
  let selectedPreview = $state<MediaItem | null>(null);
  let toastMessage = $state<string | null>(null);

  const gamesWithMedia = $derived(
    Array.from(new Set(items.map((i) => i.gameTitle ?? 'Desktop')))
  );

  const visibleItems = $derived(
    items.filter((i) => {
      const matchGame = gameFilter === 'all' || (i.gameTitle ?? 'Desktop') === gameFilter;
      const matchType = typeFilter === 'all' || i.kind === typeFilter;
      return matchGame && matchType;
    })
  );

  async function handleSnap() {
    try {
      await triggerScreenshot();
      toastMessage = 'Скриншот сохранён (F12)';
      onRefresh();
    } catch (err: any) {
      toastMessage = `Ошибка: ${err?.message ?? err}`;
    } finally {
      setTimeout(() => (toastMessage = null), 3000);
    }
  }

  async function handleRecord() {
    try {
      await toggleClipRecording();
      recording = !recording;
      toastMessage = recording ? 'Запись видеоклипа начата…' : 'Видеоклип сохранён!';
      onRefresh();
    } catch (err: any) {
      toastMessage = `Ошибка: ${err?.message ?? err}`;
    } finally {
      setTimeout(() => (toastMessage = null), 3000);
    }
  }

  async function handleDelete(path: string, event?: MouseEvent) {
    event?.stopPropagation();
    if (!confirm('Вы уверены, что хотите удалить этот файл с диска?')) {
      return;
    }
    deletingPath = path;
    try {
      await deleteMediaItem(path);
      toastMessage = 'Файл удалён с диска';
      if (selectedPreview?.path === path) {
        selectedPreview = null;
      }
      onRefresh();
    } catch (err: any) {
      toastMessage = `Ошибка удаления: ${err?.message ?? err}`;
    } finally {
      deletingPath = null;
      setTimeout(() => (toastMessage = null), 2500);
    }
  }

  function toAssetUrl(path: string): string {
    try {
      return convertFileSrc(path);
    } catch {
      return path;
    }
  }
</script>

<div class="space-y-6">
  <!-- Toast Notification -->
  {#if toastMessage}
    <div
      class="flex items-center justify-between rounded-xl border border-sky-500/30 bg-sky-950/70 px-4 py-2 text-xs font-mono text-sky-200 backdrop-blur-md shadow-lg"
    >
      <span>{toastMessage}</span>
      <button type="button" onclick={() => (toastMessage = null)} class="text-zinc-400 hover:text-white">✕</button>
    </div>
  {/if}

  <!-- Header with 3D Action Controls -->
  <div class="panel-game p-5">
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div>
        <h2 class="text-base font-black tracking-tight text-white">Медиагалерея • Zero-Overlay DXGI</h2>
        <p class="font-mono text-xs text-zinc-400 mt-0.5">
          Скриншоты по клавише F12 и видеоклипы NVENC/AMF (Ctrl+Shift+R) прямо из игр без нагрузки на FPS
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <button
          type="button"
          onclick={handleSnap}
          class="btn-game btn-game-white px-4 py-2 text-xs"
        >
          <svg class="h-3.5 w-3.5 fill-current" viewBox="0 0 24 24">
            <circle cx="12" cy="12" r="3.2"/>
            <path d="M9 2L7.17 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2h-3.17L15 2H9zm3 15c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5z"/>
          </svg>
          <span>Скриншот (F12)</span>
        </button>

        <button
          type="button"
          onclick={handleRecord}
          class="btn-game {recording ? 'btn-game-red' : 'btn-game-dark'} px-4 py-2 text-xs"
        >
          <span class="h-2 w-2 rounded-full {recording ? 'bg-white animate-ping' : 'bg-red-500'}"></span>
          <span>{recording ? '● Остановить запись' : 'Запись клипа (Ctrl+Shift+R)'}</span>
        </button>
      </div>
    </div>

    <!-- Filter Pills Row -->
    <div class="mt-4 flex flex-wrap items-center justify-between gap-3 pt-3 border-t border-white/5">
      <div class="flex items-center gap-1.5">
        <button
          type="button"
          onclick={() => (typeFilter = 'all')}
          class="btn-game px-3 py-1 text-xs {typeFilter === 'all' ? 'btn-game-gold text-black' : 'btn-game-dark text-zinc-400'}"
        >
          Все ({items.length})
        </button>
        <button
          type="button"
          onclick={() => (typeFilter = 'screenshot')}
          class="btn-game px-3 py-1 text-xs {typeFilter === 'screenshot' ? 'btn-game-gold text-black' : 'btn-game-dark text-zinc-400'}"
        >
          Скриншоты ({items.filter((i) => i.kind === 'screenshot').length})
        </button>
        <button
          type="button"
          onclick={() => (typeFilter = 'clip')}
          class="btn-game px-3 py-1 text-xs {typeFilter === 'clip' ? 'btn-game-gold text-black' : 'btn-game-dark text-zinc-400'}"
        >
          Видео ({items.filter((i) => i.kind === 'clip').length})
        </button>
      </div>

      <!-- Game filter select -->
      <select
        bind:value={gameFilter}
        class="rounded-xl border border-white/10 bg-[#18181b] px-3 py-1.5 font-mono text-xs text-zinc-200 outline-none"
      >
        <option value="all">Все игры</option>
        {#each gamesWithMedia as g}
          <option value={g}>{g}</option>
        {/each}
      </select>
    </div>
  </div>

  <!-- Media Grid -->
  {#if visibleItems.length === 0}
    <div
      class="panel-game flex flex-col items-center justify-center py-20 text-center text-zinc-400"
    >
      <p class="text-sm font-bold text-zinc-200">Медиафайлы ещё не записаны</p>
      <p class="mt-1 font-mono text-xs text-zinc-500">
        Нажмите F12 во время игры для снимка экрана или Ctrl+Shift+R для видео.
      </p>
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {#each visibleItems as item (item.path)}
        <div
          class="stat-box-game group relative overflow-hidden rounded-2xl cursor-pointer transition-all duration-200 hover:-translate-y-1 hover:border-white/30 hover:shadow-2xl"
          role="button"
          tabindex="0"
          onclick={() => (selectedPreview = item)}
          onkeydown={(e) => (e.key === 'Enter' ? (selectedPreview = item) : null)}
        >
          <!-- Media Preview (Image or Video thumbnail) -->
          <div class="relative aspect-video w-full overflow-hidden bg-black/60">
            {#if item.kind === 'screenshot'}
              <img
                src={toAssetUrl(item.path)}
                alt={item.gameTitle ?? 'Скриншот'}
                class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
                loading="lazy"
              />
            {:else}
              <video
                src={toAssetUrl(item.path)}
                preload="metadata"
                class="h-full w-full bg-black object-cover"
              >
                <track kind="captions" />
              </video>
              <div
                class="absolute inset-0 flex items-center justify-center bg-black/30 group-hover:bg-black/10 transition-colors"
              >
                <div class="flex h-12 w-12 items-center justify-center rounded-full bg-white/20 backdrop-blur-md border border-white/30 text-white">
                  ▶
                </div>
              </div>
            {/if}

            <!-- Delete Button (Top Right of thumbnail) -->
            <button
              type="button"
              onclick={(e) => handleDelete(item.path, e)}
              disabled={deletingPath === item.path}
              class="btn-game btn-game-red absolute right-2.5 top-2.5 flex h-8 w-8 items-center justify-center p-0 opacity-0 group-hover:opacity-100 transition-opacity"
              title="Удалить файл"
            >
              ✕
            </button>
          </div>

          <!-- Card Details -->
          <div class="flex items-center justify-between p-3.5">
            <div class="min-w-0 flex-1">
              <p class="truncate text-xs font-bold text-white group-hover:text-sky-300 transition-colors">
                {item.gameTitle ?? 'Desktop Capture'}
              </p>
              <p class="truncate font-mono text-[10px] text-zinc-400 mt-0.5">
                {item.kind.toUpperCase()} • {formatBytes(item.sizeBytes)}
              </p>
            </div>

            {#if item.encoder}
              <span class="rounded bg-white/5 border border-white/10 px-2 py-0.5 font-mono text-[9px] text-zinc-300">
                {item.encoder}
              </span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Fullscreen Lightbox Preview Modal -->
{#if selectedPreview}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/90 p-4 backdrop-blur-xl"
    role="dialog"
    aria-modal="true"
  >
    <div
      class="chrome-card chrome-glass relative flex max-h-[95vh] w-full max-w-5xl flex-col overflow-hidden text-white shadow-2xl"
    >
      <div class="absolute right-4 top-4 z-20 flex items-center gap-3">
        <span class="chrome-badge">
          {selectedPreview.kind === 'screenshot' ? 'ПРОСМОТР СКРИНШОТА' : 'ВИДЕОПЛЕЕР'}
        </span>
        <button
          type="button"
          onclick={() => (selectedPreview = null)}
          class="flex h-8 w-8 items-center justify-center rounded-lg border border-white/10 bg-black/60 text-zinc-400 backdrop-blur-md transition-colors hover:border-white/20 hover:bg-white/10 hover:text-white"
          aria-label="Закрыть"
        >
          <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      <!-- Media Content in Lightbox -->
      <div class="relative flex-1 w-full bg-black flex items-center justify-center min-h-[500px]">
        {#if selectedPreview.kind === 'screenshot'}
          <img
            src={toAssetUrl(selectedPreview.path)}
            alt=""
            class="max-h-[75vh] w-auto max-w-full object-contain"
          />
        {:else}
          <video
            src={toAssetUrl(selectedPreview.path)}
            controls
            autoplay
            class="max-h-[75vh] w-full bg-black object-contain"
          >
            <track kind="captions" />
          </video>
        {/if}
      </div>

      <!-- Lightbox Bottom Control Bar -->
      <div
        class="flex flex-wrap items-center justify-between gap-4 border-t border-white/10 bg-[#121215] p-4"
      >
        <div>
          <h4 class="text-sm font-bold text-white">{selectedPreview.gameTitle ?? 'Desktop Capture'}</h4>
          <p class="font-mono text-xs text-zinc-400">
            {formatBytes(selectedPreview.sizeBytes)} • {selectedPreview.createdAt}
          </p>
        </div>

        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={() => selectedPreview && handleDelete(selectedPreview.path)}
            class="btn-minimal px-4 py-2 text-xs text-rose-400 hover:text-rose-300 hover:border-rose-500/40"
          >
            Удалить файл 🗑
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
