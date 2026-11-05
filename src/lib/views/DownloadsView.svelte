<script lang="ts">
  import { onMount } from 'svelte';
  import { emulatorStore } from '$lib/stores/emulators';
  import { directoryPicker, fetchSystemDrives, type DriveInfo } from '$lib/stores/filesystem';
  import { networkStore } from '$lib/stores/network';
  import { installGameById } from '$lib/stores/library';

  let { games = [] }: { games?: any[] } = $props();

  let drives = $state<DriveInfo[]>([]);
  let busyId = $state<string | null>(null);

  const active = $derived(
    Object.values(emulatorStore.progressMap).filter((p) => p.phase !== 'completed' && p.phase !== 'error')
  );
  const finished = $derived(Object.values(emulatorStore.progressMap).filter((p) => p.phase === 'completed'));
  const queue = $derived(games.filter((g) => !g.installed).slice(0, 60));

  onMount(async () => {
    emulatorStore.init();
    networkStore.init?.();
    drives = await fetchSystemDrives();
  });

  async function pickDir() {
    const sel = await directoryPicker.open('Папка загрузок', networkStore.settings.download_directory || undefined);
    if (sel) await networkStore.saveSettings({ download_directory: sel });
  }

  async function install(g: any) {
    busyId = g.id;
    try {
      await installGameById(g.id);
    } finally {
      setTimeout(() => (busyId = null), 1500);
    }
  }
</script>

<div class="space-y-6 pb-12">
  <div class="flex flex-wrap items-center justify-between gap-4 rounded-2xl border border-white/10 bg-[#121215]/80 p-5">
    <div>
      <h2 class="text-lg font-black tracking-tight text-white">DOWNLOADS</h2>
      <p class="font-mono text-xs text-zinc-400">{networkStore.settings.download_directory || 'Папка по умолчанию'}</p>
    </div>
    <button type="button" onclick={pickDir} class="btn-minimal px-4 py-2 text-xs">Выбрать папку</button>
  </div>

  <div class="grid grid-cols-2 gap-3 md:grid-cols-4">
    {#each drives as d}
      <div class="rounded-xl border border-white/10 bg-[#121215]/90 p-3">
        <div class="flex justify-between font-mono text-xs text-zinc-300"><span>{d.drive}</span><span>{d.freeGb.toFixed(0)} GB free</span></div>
        <div class="mt-2 h-1.5 overflow-hidden rounded-full bg-white/10">
          <div class="h-full bg-white/70" style="width:{d.usedPercentage}%"></div>
        </div>
      </div>
    {/each}
  </div>

  <section class="space-y-2">
    <h3 class="text-sm font-bold text-white">Активные ({active.length})</h3>
    {#each active as p (p.emulatorId)}
      <div class="rounded-xl border border-white/10 bg-[#121215]/90 p-3">
        <div class="flex justify-between font-mono text-xs text-zinc-300">
          <span>{p.emulatorId} · {p.phase}</span>
          <span>{Math.round(p.percentage)}% · {p.speedMbS.toFixed(1)} MB/s</span>
        </div>
        <div class="mt-2 h-1.5 overflow-hidden rounded-full bg-white/10">
          <div class="h-full bg-amber-400" style="width:{p.percentage}%"></div>
        </div>
      </div>
    {:else}
      <p class="text-xs text-zinc-500">Нет активных загрузок</p>
    {/each}
    {#each finished as p (p.emulatorId)}
      <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-3 font-mono text-xs text-emerald-300">✓ {p.emulatorId}</div>
    {/each}
  </section>

  <section class="space-y-2">
    <h3 class="text-sm font-bold text-white">Не установлены ({queue.length})</h3>
    <div class="grid grid-cols-1 gap-2 md:grid-cols-2">
      {#each queue as g (g.id)}
        <div class="flex items-center justify-between rounded-xl border border-white/10 bg-[#121215]/90 px-3 py-2">
          <span class="truncate text-xs text-white">{g.title}</span>
          <button type="button" disabled={busyId === g.id} onclick={() => install(g)} class="btn-chrome px-3 py-1 text-xs">
            {busyId === g.id ? '…' : 'Скачать'}
          </button>
        </div>
      {/each}
    </div>
  </section>
</div>
