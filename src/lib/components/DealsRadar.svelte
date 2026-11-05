<!--
  DealsRadar.svelte — Real-time 100% Free Giveaways (Epic, Steam, GOG) & Historical Low Price Radar.
  Styled with 3D Casual Game GUI and in-app store opening.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { openInAppStore } from '../stores/library';
  import { currencyStore } from '$lib/stores/store';

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

  onMount(async () => {
    try {
      report = await invoke<DealsRadarReport>('get_deals_radar');
    } catch (err) {
      console.warn('Deals radar fallback active:', err);
    } finally {
      loading = false;
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
</script>

<div class="space-y-8">
  <!-- Top Header Panel -->
  <div class="panel-game p-5">
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div>
        <h2 class="text-base font-black tracking-tight text-white">Радар Скидок & Еженедельная Халява</h2>
        <p class="font-mono text-xs text-zinc-400 mt-0.5">
          100% бесплатные раздачи Epic Games Store и исторические минимумы цен Steam / GOG
        </p>
      </div>

      <div class="flex items-center gap-3">
        <input
          type="search"
          bind:value={searchQuery}
          placeholder="Поиск по скидкам…"
          class="w-56 rounded-xl border border-white/10 bg-[#18181b] px-3.5 py-1.5 text-xs text-white placeholder-zinc-500 outline-none transition-colors focus:border-white/40"
        />

        <button
          type="button"
          onclick={async () => {
            loading = true;
            try {
              report = await invoke<DealsRadarReport>('get_deals_radar');
            } finally {
              loading = false;
            }
          }}
          disabled={loading}
          class="btn-game btn-game-white px-4 py-1.5 text-xs"
        >
          {loading ? 'Синхронизация…' : '↻ Обновить'}
        </button>
      </div>
    </div>
  </div>

  <!-- 100% Free Giveaways -->
  <section class="space-y-4">
    <div class="flex items-center gap-2">
      <span class="rounded bg-emerald-500/20 border border-emerald-500/30 px-2.5 py-0.5 font-mono text-xs font-bold text-emerald-300">
        100% FREE
      </span>
      <h3 class="text-sm font-black uppercase tracking-wider text-white">
        Бесплатные раздачи недели ({filteredFreebies.length})
      </h3>
    </div>

    <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
      {#each filteredFreebies as offer (offer.id)}
        <div
          class="stat-box-game group flex flex-col justify-between p-5 transition-all duration-200 hover:-translate-y-1 hover:border-white/30 hover:shadow-2xl"
        >
          <div>
            <div class="flex items-center justify-between">
              <span class="font-mono text-xs font-bold text-zinc-300">{offer.store}</span>
              <span class="rounded bg-emerald-400 px-2 py-0.5 font-mono text-[10px] font-black text-black shadow-sm">
                {offer.badge}
              </span>
            </div>
            <h4 class="mt-3 text-base font-bold text-white group-hover:text-emerald-300 transition-colors">
              {offer.title}
            </h4>
          </div>

          <div class="mt-6 flex items-center justify-between pt-3 border-t border-white/5">
            <span class="font-mono text-xs text-zinc-500 line-through">${offer.normalPriceUsd.toFixed(2)}</span>
            <button
              type="button"
              onclick={() => openInAppStore(offer.url, `${offer.title} • Раздача`)}
              class="btn-game btn-game-green px-4 py-1.5 text-xs font-bold"
            >
              Забрать бесплатно ↗
            </button>
          </div>
        </div>
      {/each}
    </div>
  </section>

  <!-- Deep Discounts & Historical Lows -->
  <section class="space-y-4">
    <div class="flex items-center gap-2">
      <span class="rounded bg-amber-500/20 border border-amber-500/30 px-2.5 py-0.5 font-mono text-xs font-bold text-amber-300">
        DISCOUNTS
      </span>
      <h3 class="text-sm font-black uppercase tracking-wider text-white">
        Крупные скидки и исторические минимумы ({filteredPriceDrops.length})
      </h3>
    </div>

    <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
      {#each filteredPriceDrops as deal (deal.id)}
        <div
          class="stat-box-game flex flex-col justify-between p-4 transition-all duration-200 hover:-translate-y-0.5 hover:border-white/20"
        >
          <div>
            <div class="flex items-center justify-between">
              <span class="font-mono text-[10px] font-bold text-zinc-400">{deal.store}</span>
              <span class="rounded bg-amber-400 px-1.5 py-0.5 font-mono text-[9px] font-black text-black">
                {deal.badge}
              </span>
            </div>
            <h4 class="mt-2 line-clamp-1 text-sm font-bold text-white">
              {deal.title}
            </h4>
          </div>

          <div class="mt-4 flex items-center justify-between pt-2 border-t border-white/5">
            <div class="font-mono text-xs flex flex-col">
              <div class="flex items-center gap-1.5">
                <span class="text-zinc-500 line-through text-[11px]">{currencyStore.format(deal.normalPriceUsd)}</span>
                <span class="text-sm font-bold text-white">{currencyStore.format(deal.salePriceUsd)}</span>
              </div>
              <span class="text-[9px] text-emerald-400 mt-0.5">
                {currencyStore.formatSavings(deal.normalPriceUsd, deal.salePriceUsd)}
              </span>
            </div>

            <button
              type="button"
              onclick={() => openInAppStore(deal.url, `${deal.title} • Скидка`)}
              class="btn-game btn-game-dark px-3 py-1 text-xs"
            >
              Купить ↗
            </button>
          </div>
        </div>
      {/each}
    </div>
  </section>
</div>
