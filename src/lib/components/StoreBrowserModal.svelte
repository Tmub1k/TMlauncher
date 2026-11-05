<!--
  StoreBrowserModal.svelte — In-App Store Browser.
  Opens store pages (Steam, itch.io) directly within the launcher.
-->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface Props {
    open: boolean;
    url: string;
    title: string;
    installUri?: string;
    onClose: () => void;
  }

  let { open, url, title, installUri, onClose }: Props = $props();

  let iframeRef = $state<HTMLIFrameElement | null>(null);
  let loading = $state(true);
  let srcdocContent = $state<string | null>(null);

  $effect(() => {
    if (open && url) {
      loadPage();
    }
  });

  async function openInWebview() {
    try {
      await invoke('open_in_app_store_window', { url, title });
      onClose();
    } catch (e) {
      console.error('Failed to open native store window:', e);
    }
  }

  async function openInBrowser() {
    try {
      await invoke('open_external_url', { url });
      onClose();
    } catch {
      window.open(url, '_blank');
      onClose();
    }
  }

  async function loadPage() {
    // Epic Games and Cloudflare-protected sites cannot be embedded inside an iframe srcdoc without WAF blocks
    if (url.includes('epicgames.com') || url.includes('steamcommunity.com')) {
      await openInWebview();
      return;
    }

    loading = true;
    srcdocContent = null;
    try {
      // Fetch via backend to strip X-Frame-Options and embed properly in the same window
      const html = await invoke<string>('get_store_page_html', { url });
      srcdocContent = html;
    } catch (err) {
      console.warn('Backend store fetch error, falling back to direct iframe src:', err);
      if (iframeRef) {
        iframeRef.src = url;
      }
    } finally {
      loading = false;
    }
  }

  function reloadIframe() {
    loadPage();
  }
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/85 p-4 backdrop-blur-md"
    role="dialog"
    aria-modal="true"
  >
    <div
      class="chrome-card chrome-glass relative flex h-[92vh] w-full max-w-6xl flex-col overflow-hidden text-white shadow-2xl"
    >
      <!-- Store Browser Top Bar with Sleek Header & Clean Close Button -->
      <div
        class="flex flex-wrap items-center justify-between gap-3 border-b border-white/10 bg-[#121215] px-6 py-3.5"
      >
        <div class="flex items-center gap-3 min-w-0 flex-1">
          <span class="chrome-badge">ВСТРОЕННЫЙ МАГАЗИН</span>
          <span class="font-bold text-sm text-white truncate max-w-xs">{title}</span>
          <span
            class="hidden md:inline font-mono text-[11px] text-zinc-400 bg-black/50 px-3 py-1 rounded-lg border border-white/10 truncate max-w-md"
          >
            {url}
          </span>
        </div>

        <div class="flex items-center gap-2">
          {#if installUri}
            <a
              href={installUri}
              class="btn-chrome px-3.5 py-1.5 text-xs"
            >
              <svg class="h-3.5 w-3.5 fill-current" viewBox="0 0 24 24">
                <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
              </svg>
              <span>Скачать</span>
            </a>
          {/if}

          <button
            type="button"
            onclick={openInWebview}
            class="btn-chrome px-3 py-1.5 text-xs bg-white/10 hover:bg-white/20 text-white"
            title="Открыть во встроенном отдельном окне"
          >
            ❐ Окно Webview
          </button>

          <button
            type="button"
            onclick={openInBrowser}
            class="btn-minimal px-3 py-1.5 text-xs text-zinc-300 hover:text-white"
            title="Открыть в основном браузере"
          >
            В браузере ↗
          </button>

          <button
            type="button"
            onclick={reloadIframe}
            class="btn-minimal px-3 py-1.5 text-xs"
            title="Обновить страницу"
          >
            ↻ Обновить
          </button>

          <button
            type="button"
            onclick={onClose}
            class="flex h-8 w-8 items-center justify-center rounded-lg border border-white/10 bg-white/5 text-zinc-400 transition-colors hover:border-white/20 hover:bg-white/10 hover:text-white"
            aria-label="Закрыть"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>

      <!-- Embedded Web Frame Container -->
      <div class="relative flex-1 w-full bg-[#09090b]">
        {#if loading}
          <div
            class="absolute inset-0 flex flex-col items-center justify-center bg-[#09090b]/80 z-10"
          >
            <div class="h-8 w-8 animate-spin rounded-full border-2 border-white/20 border-t-white"></div>
            <p class="mt-3 font-mono text-xs text-zinc-400">Загрузка страницы магазина…</p>
          </div>
        {/if}

        <iframe
          bind:this={iframeRef}
          srcdoc={srcdocContent ?? undefined}
          src={srcdocContent ? undefined : url}
          title={title}
          class="h-full w-full border-0 bg-white"
          onload={() => (loading = false)}
        ></iframe>
      </div>
    </div>
  </div>
{/if}
