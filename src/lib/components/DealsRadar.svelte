<!--
  DealsRadar.svelte — Real-time 100% Free Giveaways (Epic, Steam, GOG) & Historical Low Price Radar.
  Styled with modern theme adaptation, multi-currency support, image fallbacks, and 1-click client store opening.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { currencyStore } from '$lib/stores/store';
  import { storeLinkManager, type StoreOpenMode } from '$lib/utils/storeLink';

  export interface DealOffer {
    id: string;
    title: string;
    store: string;
    isFreebie: boolean;
    normalPriceUsd: number;
    salePriceUsd: number;
    savingsPercent: number;
    badge: string;
    thumbUrl: string | null;
    url: string;
    expiresAt: string | null;
  }

  export interface DealsRadarReport {
    freebies: DealOffer[];
    priceDrops: DealOffer[];
  }

  let report = $state<DealsRadarReport>({ freebies: [], priceDrops: [] });
  let loading = $state(true);
  let searchQuery = $state('');
  let imageAttempts = $state<Record<string, number>>({});
  let lastCurrency = $state<'USD' | 'KZT'>(currencyStore.code);

  function extractSteamId(offer: DealOffer): string | null {
    const match = offer.url.match(/(?:store\.steampowered\.com\/app|steamcommunity\.com\/app)\/(\d+)/i);
    if (match) return match[1];
    if (offer.store.toLowerCase().includes('steam') && /^\d+$/.test(offer.id)) return offer.id;
    return null;
  }

  function getDealImage(deal: DealOffer): string | null {
    const attempts = imageAttempts[deal.id] || 0;
    if (attempts === 0 && deal.thumbUrl) return deal.thumbUrl;
    const steamId = extractSteamId(deal);
    if (steamId) {
      if (attempts <= 1) {
        return `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${steamId}/header.jpg`;
      }
      if (attempts <= 2) {
        return `https://cdn.akamai.steamstatic.com/steam/apps/${steamId}/capsule_616x353.jpg`;
      }
    }
    return null;
  }

  async function loadDeals(curr = currencyStore.code) {
    loading = true;
    try {
      report = await invoke<DealsRadarReport>('get_deals_radar', { currency: curr });
    } catch (err) {
      console.warn('Deals radar fallback active:', err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadDeals();
  });

  // Refetch when currency changes
  $effect(() => {
    const currentCode = currencyStore.code;
    if (currentCode !== lastCurrency) {
      lastCurrency = currentCode;
      loadDeals(currentCode);
    }
  });

  const filteredFreebies = $derived(
    report.freebies.filter(
      (f) => !searchQuery || f.title.toLowerCase().includes(searchQuery.toLowerCase())
    )
  );

  const filteredPriceDrops = $derived(
    report.priceDrops.filter(
      (d) => !searchQuery || d.title.toLowerCase().includes(searchQuery.toLowerCase())
    )
  );

  function handleOpenDeal(offer: DealOffer, modeOverride?: StoreOpenMode) {
    storeLinkManager.open({
      url: offer.url,
      title: `${offer.title} • ${offer.isFreebie ? 'Раздача' : 'Скидка'}`,
      appId: extractSteamId(offer),
      platform: offer.store,
      mode: modeOverride,
    });
  }
</script>

<div class="space-y-8 text-[var(--text-primary)]">
  <!-- Top Header Panel with Currency & Store Open Mode Switcher -->
  <div class="panel-game p-5">
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div>
        <h2 class="text-base font-black tracking-tight text-[var(--text-primary)] flex items-center gap-2">
          <span>Радар Скидок & Еженедельная Халява</span>
          <span class="rounded bg-emerald-500/20 text-emerald-400 font-mono text-[10px] px-2 py-0.5 font-bold">
            LIVE SYNC
          </span>
        </h2>
        <p class="font-mono text-xs text-[var(--text-secondary)] mt-0.5">
          100% бесплатные раздачи Epic Games Store и исторические минимумы цен Steam / GOG
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-3">
        <!-- Search Input -->
        <input
          type="search"
          bind:value={searchQuery}
          placeholder="Поиск по скидкам…"
          class="w-52 rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-canvas)]/60 px-3.5 py-1.5 text-xs text-[var(--text-primary)] placeholder-zinc-500 outline-none transition-colors focus:border-[var(--accent-primary)]"
        />

        <!-- Store Open Mode Switcher -->
        <div class="flex items-center rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-canvas)]/60 p-1 shadow-inner">
          <button
            type="button"
            onclick={() => storeLinkManager.setMode('client')}
            class="flex items-center gap-1 rounded-lg px-2.5 py-1 font-mono text-[11px] font-bold transition-all {storeLinkManager.mode === 'client'
              ? 'bg-[var(--accent-primary)] text-white shadow-md'
              : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
            title="Открывать в клиенте Steam / Epic Games"
          >
            🎮 Клиент
          </button>
          <button
            type="button"
            onclick={() => storeLinkManager.setMode('launcher')}
            class="flex items-center gap-1 rounded-lg px-2.5 py-1 font-mono text-[11px] font-bold transition-all {storeLinkManager.mode === 'launcher'
              ? 'bg-[var(--accent-primary)] text-white shadow-md'
              : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
            title="Открывать во встроенном окне лаунчера"
          >
            ❐ Окно
          </button>
          <button
            type="button"
            onclick={() => storeLinkManager.setMode('browser')}
            class="flex items-center gap-1 rounded-lg px-2.5 py-1 font-mono text-[11px] font-bold transition-all {storeLinkManager.mode === 'browser'
              ? 'bg-[var(--accent-primary)] text-white shadow-md'
              : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
            title="Открывать в браузере Windows"
          >
            🌐 Браузер
          </button>
        </div>

        <!-- Currency Switcher ($ USD / ₸ KZT) -->
        <div class="flex items-center rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-canvas)]/60 p-1 shadow-inner">
          <button
            type="button"
            onclick={() => currencyStore.setCurrency('USD')}
            class="flex items-center gap-1 rounded-lg px-2 py-1 font-mono text-xs font-bold transition-all {currencyStore.code === 'USD'
              ? 'bg-[var(--accent-primary)] text-white shadow-md'
              : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
          >
            $ USD
          </button>
          <button
            type="button"
            onclick={() => currencyStore.setCurrency('KZT')}
            class="flex items-center gap-1 rounded-lg px-2 py-1 font-mono text-xs font-bold transition-all {currencyStore.code === 'KZT'
              ? 'bg-[var(--accent-primary)] text-white shadow-md'
              : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
          >
            ₸ KZT
          </button>
        </div>

        <!-- Refresh Button -->
        <button
          type="button"
          onclick={() => loadDeals()}
          disabled={loading}
          class="btn-game px-3.5 py-1.5 text-xs font-bold"
        >
          {loading ? 'Синхронизация…' : '↻ Обновить'}
        </button>
      </div>
    </div>
  </div>

  <!-- 100% Free Giveaways (Steam + Epic Games) -->
  <section class="space-y-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <span class="rounded bg-emerald-500/20 border border-emerald-500/30 px-2.5 py-0.5 font-mono text-xs font-bold text-emerald-400">
          100% FREE
        </span>
        <h3 class="text-sm font-black uppercase tracking-wider text-[var(--text-primary)]">
          Бесплатные раздачи недели ({filteredFreebies.length})
        </h3>
      </div>
      <span class="text-xs font-mono text-[var(--text-secondary)]">Epic Games & Steam</span>
    </div>

    <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
      {#each filteredFreebies as offer (offer.id)}
        {@const dealImg = getDealImage(offer)}
        <div
          class="stat-box-game group flex flex-col justify-between overflow-hidden rounded-2xl border border-[var(--border-subtle)] bg-[var(--bg-surface-base)] transition-all duration-200 hover:-translate-y-1 hover:border-[var(--border-active)] hover:shadow-2xl"
        >
          <!-- Artwork / Poster -->
          <div class="relative aspect-video w-full overflow-hidden bg-black/60">
            {#if dealImg}
              <img
                src={dealImg}
                alt={offer.title}
                loading="lazy"
                onerror={() => {
                  imageAttempts[offer.id] = (imageAttempts[offer.id] || 0) + 1;
                }}
                class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
              />
            {:else}
              <div class="flex h-full w-full flex-col items-center justify-center bg-gradient-to-br from-[var(--bg-surface-elevated)] to-[var(--bg-canvas)] p-4 text-center">
                <span class="text-xs font-mono font-bold text-emerald-400 mb-1">{offer.store.toUpperCase()}</span>
                <span class="text-xs font-bold text-[var(--text-primary)] line-clamp-2">{offer.title}</span>
              </div>
            {/if}

            <div class="absolute top-2.5 left-2.5 flex items-center gap-1.5">
              <span class="rounded bg-black/75 backdrop-blur-md px-2 py-0.5 font-mono text-[10px] font-bold text-white border border-white/10">
                {offer.store}
              </span>
              <span class="rounded bg-emerald-500 px-2 py-0.5 font-mono text-[10px] font-black text-black shadow-sm">
                БЕСПЛАТНО (-100%)
              </span>
            </div>
          </div>

          <!-- Body -->
          <div class="p-4 flex-1 flex flex-col justify-between space-y-3">
            <div>
              <h4 class="text-sm font-bold text-[var(--text-primary)] group-hover:text-emerald-400 transition-colors line-clamp-2">
                {offer.title}
              </h4>
              {#if offer.expiresAt}
                <p class="font-mono text-[10px] text-[var(--text-secondary)] mt-1">
                  До: {new Date(offer.expiresAt).toLocaleDateString()}
                </p>
              {/if}
            </div>

            <!-- Price & Action Trigger -->
            <div class="mt-4 flex items-center justify-between pt-3 border-t border-[var(--border-hairline)]">
              <div class="flex flex-col">
                <span class="font-mono text-xs text-[var(--text-muted)] line-through">
                  {currencyStore.format(offer.normalPriceUsd)}
                </span>
                <span class="font-mono text-xs font-black text-emerald-400">
                  Бесплатно
                </span>
              </div>

              <div class="flex items-center gap-1.5">
                <button
                  type="button"
                  onclick={() => handleOpenDeal(offer, 'client')}
                  class="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] p-1.5 text-[var(--text-secondary)] hover:text-emerald-400 hover:border-emerald-500/50 transition-colors"
                  title="Забрать прямо в клиенте Steam / Epic"
                >
                  <span class="font-bold text-xs">🎮</span>
                </button>
                <button
                  type="button"
                  onclick={() => handleOpenDeal(offer)}
                  class="btn-game btn-game-green px-3.5 py-1.5 text-xs font-bold"
                >
                  Забрать ↗
                </button>
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  </section>

  <!-- Deep Discounts & Historical Lows -->
  <section class="space-y-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <span class="rounded bg-amber-500/20 border border-amber-500/30 px-2.5 py-0.5 font-mono text-xs font-bold text-amber-400">
          DISCOUNTS
        </span>
        <h3 class="text-sm font-black uppercase tracking-wider text-[var(--text-primary)]">
          Крупные скидки и исторические минимумы ({filteredPriceDrops.length})
        </h3>
      </div>
      <span class="text-xs font-mono text-[var(--text-secondary)]">Steam & GOG</span>
    </div>

    <div class="grid grid-cols-1 gap-3.5 sm:grid-cols-2 lg:grid-cols-4">
      {#each filteredPriceDrops as deal (deal.id)}
        {@const dealImg = getDealImage(deal)}
        <div
          class="stat-box-game flex flex-col justify-between overflow-hidden rounded-2xl border border-[var(--border-subtle)] bg-[var(--bg-surface-base)] transition-all duration-200 hover:-translate-y-0.5 hover:border-[var(--border-active)]"
        >
          <!-- Artwork / Poster -->
          <div class="relative aspect-video w-full overflow-hidden bg-black/60">
            {#if dealImg}
              <img
                src={dealImg}
                alt={deal.title}
                loading="lazy"
                onerror={() => {
                  imageAttempts[deal.id] = (imageAttempts[deal.id] || 0) + 1;
                }}
                class="h-full w-full object-cover transition-transform duration-500 hover:scale-105"
              />
            {:else}
              <div class="flex h-full w-full flex-col items-center justify-center bg-gradient-to-br from-[var(--bg-surface-elevated)] to-[var(--bg-canvas)] p-3 text-center">
                <span class="text-xs font-mono font-bold text-amber-400 mb-0.5">{deal.store}</span>
                <span class="text-xs font-bold text-[var(--text-primary)] line-clamp-2">{deal.title}</span>
              </div>
            {/if}

            <div class="absolute top-2 left-2 flex items-center gap-1">
              <span class="rounded bg-black/75 backdrop-blur-md px-1.5 py-0.5 font-mono text-[9px] font-bold text-zinc-300 border border-white/10">
                {deal.store}
              </span>
              <span class="rounded bg-amber-400 px-1.5 py-0.5 font-mono text-[9px] font-black text-black">
                {deal.badge}
              </span>
            </div>
          </div>

          <!-- Body -->
          <div class="p-3.5 flex-1 flex flex-col justify-between space-y-2">
            <h4 class="line-clamp-1 text-xs font-bold text-[var(--text-primary)]" title={deal.title}>
              {deal.title}
            </h4>

            <div class="flex items-center justify-between pt-2 border-t border-[var(--border-hairline)]">
              <div class="font-mono text-xs flex flex-col">
                <div class="flex items-center gap-1.5">
                  <span class="text-[var(--text-muted)] line-through text-[11px]">{currencyStore.format(deal.normalPriceUsd)}</span>
                  <span class="text-xs font-bold text-[var(--text-primary)]">{currencyStore.format(deal.salePriceUsd)}</span>
                </div>
                <span class="text-[9px] text-emerald-400 mt-0.5">
                  {currencyStore.formatSavings(deal.normalPriceUsd, deal.salePriceUsd)}
                </span>
              </div>

              <div class="flex items-center gap-1">
                <button
                  type="button"
                  onclick={() => handleOpenDeal(deal, 'client')}
                  class="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] p-1 text-[var(--text-secondary)] hover:text-amber-400 hover:border-amber-400/50 transition-colors"
                  title="Купить в клиенте Steam / Epic"
                >
                  <span class="text-xs">🎮</span>
                </button>
                <button
                  type="button"
                  onclick={() => handleOpenDeal(deal)}
                  class="btn-game btn-game-dark px-2.5 py-1 text-xs"
                >
                  Купить ↗
                </button>
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  </section>
</div>
