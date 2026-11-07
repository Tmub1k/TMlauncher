<!--
  StoreBrowserModal.svelte — Store Gateway & In-App Browser Selector.
  Allows 1-click seamless opening in:
  - Steam / Epic Games desktop client (already signed in)
  - Dedicated In-Launcher WebView window (preserves cookies/sessions)
  - Default Windows system browser
-->
<script lang="ts">
  import { storeLinkManager, type StoreOpenMode } from '$lib/utils/storeLink';

  interface Props {
    open: boolean;
    url: string;
    title: string;
    installUri?: string;
    onClose: () => void;
  }

  let { open, url, title, installUri, onClose }: Props = $props();

  const isSteam = $derived(url.includes('steampowered.com') || url.includes('steamcommunity.com'));
  const isEpic = $derived(url.includes('epicgames.com'));
  const platformName = $derived(isSteam ? 'Steam' : isEpic ? 'Epic Games' : 'Магазин');

  async function handleOpen(mode: StoreOpenMode) {
    await storeLinkManager.open({
      url,
      title,
      installUri,
      mode,
    });
    onClose();
  }

  function handleSetDefaultAndOpen(mode: StoreOpenMode) {
    storeLinkManager.setMode(mode);
    handleOpen(mode);
  }
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 p-4 backdrop-blur-md"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onkeydown={(e) => {
      if (e.key === 'Escape') onClose();
    }}
  >
    <div
      class="panel-game relative flex w-full max-w-xl flex-col overflow-hidden text-[var(--text-primary)] shadow-2xl p-6 sm:p-8"
      style="background: var(--bg-surface-base); border: 1px solid var(--border-subtle); border-radius: var(--radius-xl);"
    >
      <!-- Close Button -->
      <button
        type="button"
        onclick={onClose}
        class="btn-close-game"
        aria-label="Закрыть"
      >
        ✕
      </button>

      <!-- Header -->
      <div class="mb-6 space-y-1.5">
        <div class="flex items-center gap-2">
          <span class="chrome-badge">ПЕРЕХОД В МАГАЗИН</span>
          {#if isSteam}
            <span class="rounded bg-sky-500/20 px-2 py-0.5 font-mono text-[10px] font-bold text-sky-300">STEAM</span>
          {:else if isEpic}
            <span class="rounded bg-zinc-700 px-2 py-0.5 font-mono text-[10px] font-bold text-white">EPIC</span>
          {/if}
        </div>
        <h3 class="text-xl font-black tracking-tight text-[var(--text-primary)]">
          {title || 'Страница игры'}
        </h3>
        <p class="font-mono text-xs text-[var(--text-secondary)] truncate max-w-md">
          {url}
        </p>
      </div>

      <!-- Action Choices -->
      <div class="space-y-3">
        <!-- Option 1: Native Client (Steam / Epic) -->
        <button
          type="button"
          onclick={() => handleOpen('client')}
          class="w-full flex items-center justify-between p-4 rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] hover:border-[var(--accent-primary)] hover:bg-[var(--bg-surface-floating)] transition-all group text-left"
        >
          <div class="flex items-center gap-3.5">
            <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-[var(--accent-primary)]/15 text-[var(--accent-primary)] border border-[var(--accent-primary)]/30 group-hover:scale-105 transition-transform">
              {#if isSteam}
                <svg class="h-5 w-5 fill-current" viewBox="0 0 24 24"><path d="M12 2C6.47 2 2 6.47 2 12c0 4.19 2.58 7.78 6.24 9.25l2.42-3.48A4.954 4.954 0 0 1 10 15c0-.62.11-1.21.32-1.76L8.43 11.8A7.03 7.03 0 0 1 12 5c3.87 0 7 3.13 7 7 0 3.2-2.15 5.9-5.11 6.74l1.17 1.68C18.9 19.16 22 15.93 22 12c0-5.53-4.47-10-10-10zm0 5a5 5 0 0 0-4.9 4.02l2.49 1.74A2.99 2.99 0 0 1 12 11c1.66 0 3 1.34 3 3 0 .76-.28 1.45-.75 1.98l2.4 1.68A5.002 5.002 0 0 0 12 7z"/></svg>
              {:else}
                <span class="font-bold text-base">🎮</span>
              {/if}
            </div>
            <div>
              <div class="flex items-center gap-2">
                <span class="font-bold text-sm text-[var(--text-primary)]">
                  В приложении {platformName}
                </span>
                <span class="rounded bg-emerald-500/20 text-emerald-400 font-mono text-[9px] px-1.5 py-0.5 font-bold">
                  РЕКОМЕНДУЕТСЯ
                </span>
              </div>
              <p class="text-xs text-[var(--text-secondary)] mt-0.5">
                Мгновенно открывает в клиенте, где вы уже авторизованы в своём аккаунте
              </p>
            </div>
          </div>
          <span class="text-xs font-mono text-[var(--accent-primary)] opacity-0 group-hover:opacity-100 transition-opacity">
            Открыть →
          </span>
        </button>

        <!-- Option 2: Launcher WebView Window -->
        <button
          type="button"
          onclick={() => handleOpen('launcher')}
          class="w-full flex items-center justify-between p-4 rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] hover:border-[var(--accent-primary)] hover:bg-[var(--bg-surface-floating)] transition-all group text-left"
        >
          <div class="flex items-center gap-3.5">
            <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-white/5 text-zinc-300 border border-white/10 group-hover:scale-105 transition-transform">
              <svg class="h-5 w-5 fill-none stroke-current" stroke-width="2" viewBox="0 0 24 24">
                <rect x="2" y="3" width="20" height="14" rx="2" />
                <line x1="8" y1="21" x2="16" y2="21" />
                <line x1="12" y1="17" x2="12" y2="21" />
              </svg>
            </div>
            <div>
              <div class="font-bold text-sm text-[var(--text-primary)]">
                Во встроенном окне лаунчера
              </div>
              <p class="text-xs text-[var(--text-secondary)] mt-0.5">
                Отдельное окно WebView лаунчера с сохранением куки и сессии
              </p>
            </div>
          </div>
          <span class="text-xs font-mono text-[var(--accent-primary)] opacity-0 group-hover:opacity-100 transition-opacity">
            Открыть →
          </span>
        </button>

        <!-- Option 3: Default OS Browser -->
        <button
          type="button"
          onclick={() => handleOpen('browser')}
          class="w-full flex items-center justify-between p-4 rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] hover:border-[var(--accent-primary)] hover:bg-[var(--bg-surface-floating)] transition-all group text-left"
        >
          <div class="flex items-center gap-3.5">
            <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-white/5 text-zinc-300 border border-white/10 group-hover:scale-105 transition-transform">
              <svg class="h-5 w-5 fill-none stroke-current" stroke-width="2" viewBox="0 0 24 24">
                <circle cx="12" cy="12" r="10"/>
                <line x1="2" y1="12" x2="22" y2="12"/>
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
              </svg>
            </div>
            <div>
              <div class="font-bold text-sm text-[var(--text-primary)]">
                В браузере по умолчанию
              </div>
              <p class="text-xs text-[var(--text-secondary)] mt-0.5">
                В вашем стандартном системном веб-браузере Windows
              </p>
            </div>
          </div>
          <span class="text-xs font-mono text-[var(--accent-primary)] opacity-0 group-hover:opacity-100 transition-opacity">
            Открыть ↗
          </span>
        </button>
      </div>

      <!-- Quick Set as Default Preferences -->
      <div class="mt-6 flex flex-wrap items-center justify-between gap-3 pt-4 border-t border-[var(--border-hairline)] text-xs text-[var(--text-secondary)]">
        <div class="flex items-center gap-2">
          <span>Действие по умолчанию:</span>
          <select
            value={storeLinkManager.mode}
            onchange={(e) => storeLinkManager.setMode((e.currentTarget as HTMLSelectElement).value as StoreOpenMode)}
            class="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] px-2.5 py-1 font-mono text-xs text-[var(--text-primary)] outline-none"
          >
            <option value="client">В клиенте Steam/Epic</option>
            <option value="launcher">Во встроенном окне</option>
            <option value="browser">В браузере Windows</option>
          </select>
        </div>

        {#if installUri}
          <a
            href={installUri}
            class="btn-game btn-game-green px-4 py-1.5 text-xs font-bold"
          >
            Установить сразу
          </a>
        {/if}
      </div>
    </div>
  </div>
{/if}
