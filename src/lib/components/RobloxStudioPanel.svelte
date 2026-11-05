<!--
  RobloxStudioPanel.svelte — Roblox Experiences / Modes Hub, Search by Name,
  Direct Place Launcher, FastFlags Editor & Multi-Instance Controller.
  Styled with 3D Casual Game GUI.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    fetchPopularRobloxExperiences,
    launchRobloxPlace,
    type RobloxExperiencePreset,
  } from '../stores/library';

  type FastFlagMap = Record<string, unknown>;

  interface RobloxPlaceInfo {
    placeId: number;
    universeId: number | null;
    name: string;
    builder: string | null;
    description: string | null;
  }

  let flags = $state<FastFlagMap>({});
  let multiInstance = $state(false);
  let targetFps = $state(240);
  let graphicsApi = $state<'default' | 'direct3d11' | 'vulkan' | 'opengl'>('vulkan');
  let lighting = $state<'default' | 'voxel' | 'shadowmap' | 'future'>('future');
  let disableTelemetry = $state(true);
  let statusBanner = $state<string | null>(null);

  // Experiences & Direct Mode Launching
  let experiences = $state<RobloxExperiencePreset[]>([]);
  let searchQuery = $state('');
  let searchResults = $state<RobloxExperiencePreset[]>([]);
  let searching = $state(false);
  let customPlaceInput = $state('');
  let recentPlace = $state<RobloxPlaceInfo | null>(null);
  let launchingPlaceId = $state<number | null>(null);

  let searchTimer: ReturnType<typeof setTimeout> | null = null;

  // Roblox Multi-Account Profiles for Bootstrapper
  interface RobloxAccount {
    id: string;
    username: string;
    displayName: string;
    isActive: boolean;
  }

  let accounts = $state<RobloxAccount[]>([]);
  let newAccountUsername = $state('');
  let isAddingAccount = $state(false);

  function loadAccounts() {
    try {
      const saved = localStorage.getItem('omni_roblox_accounts');
      if (saved) {
        accounts = JSON.parse(saved);
      } else {
        accounts = [
          { id: 'acc_default', username: 'Current User', displayName: 'Текущий профиль Windows', isActive: true },
        ];
      }
    } catch {
      accounts = [
        { id: 'acc_default', username: 'Current User', displayName: 'Текущий профиль Windows', isActive: true },
      ];
    }
  }

  function saveAccounts() {
    try {
      localStorage.setItem('omni_roblox_accounts', JSON.stringify(accounts));
    } catch {}
  }

  function selectAccount(id: string) {
    accounts = accounts.map((a) => ({ ...a, isActive: a.id === id }));
    saveAccounts();
    const active = accounts.find((a) => a.isActive);
    statusBanner = `Аккаунт переключен: ${active?.displayName || active?.username}`;
  }

  function addAccount() {
    const name = newAccountUsername.trim();
    if (!name) return;
    const newAcc: RobloxAccount = {
      id: `acc_${Date.now()}`,
      username: name,
      displayName: name,
      isActive: false,
    };
    accounts = [...accounts, newAcc];
    saveAccounts();
    newAccountUsername = '';
    isAddingAccount = false;
    selectAccount(newAcc.id);
  }

  function removeAccount(id: string) {
    if (accounts.length <= 1) return;
    accounts = accounts.filter((a) => a.id !== id);
    if (!accounts.some((a) => a.isActive)) {
      accounts[0].isActive = true;
    }
    saveAccounts();
  }

  function proxyRobloxImg(url: string | null | undefined): string {
    if (!url) {
      return 'https://images.weserv.nl/?url=tr.rbxcdn.com/180DAY-b4d24a9e525166299b9ba89953ad7ff3/150/150/Image/Webp/noFilter';
    }
    if (url.includes('tr.rbxcdn.com') || url.includes('images.rbxcdn.com')) {
      const stripped = url.replace(/^https?:\/\//, '');
      return `https://images.weserv.nl/?url=${encodeURIComponent(stripped)}&w=300&h=300&fit=cover`;
    }
    return url;
  }
  const FALLBACK_MODES: RobloxExperiencePreset[] = [
    {
      placeId: 2753915549,
      title: 'Blox Fruits',
      category: 'Anime RPG',
      description: 'Исследуйте моря, побеждайте рейд-боссов и открывайте мощнейшие дьявольские фрукты.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-b4d24a9e525166299b9ba89953ad7ff3/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 4924922222,
      title: 'Brookhaven 🏡 RP',
      category: 'Roleplay & Life',
      description: 'Огромный город для ролевой игры, покупка недвижимости, суперкары и общение.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-39d5622345eebfe97bfabdf0fb2cbf93/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 17625359962,
      title: 'RIVALS ⚔️',
      category: 'Competitive FPS',
      description: 'Киберспортивный шутер 1v1 и 2v2 с высоким TTK, кастомизацией скинов и снайперскими дуэлями.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-923cb9f6cb7beaa4ff499c7ea0e92750/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 920587237,
      title: 'Adopt Me!',
      category: 'Pets & Simulation',
      description: 'Выращивайте редких питомцев, торгуйте на площади и обустраивайте особняк.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-5ae32b262a67e54c75ca78e826b5c00e/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 16732694052,
      title: 'Fisch 🎣',
      category: 'Open Ocean RPG',
      description: 'Ловите мифических морских чудовищ на глубинах, улучшайте лодки и зачаровывайте снасти.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-5ca5989eef1ff51e51b1406e23635a96/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 142823291,
      title: 'Murder Mystery 2',
      category: 'Survival Mystery',
      description: 'Шериф с револьвером, хитрый убийца и невиновные, пытающиеся пережить раунд.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-f0e74f1b5394ef338f97bbcb17ca2c68/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 13772394625,
      title: 'Blade Ball ⚔️',
      category: 'Arcade Combat',
      description: 'Отражайте сверхскоростной самонаводящийся мяч мечом и комбо-способностями.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-fdbdcf8362678da47f8ba7febeba74c2/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 10449761463,
      title: 'The Strongest Battlegrounds',
      category: 'Anime Fighting',
      description: 'Разрушение зданий, эпические комбо Сайтамы и Гароу, бескомпромиссные дуэли.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-218228591f4861b53bb26fe0b6667ff4/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 6516141723,
      title: 'DOORS 👁️',
      category: 'Horror Roguelite',
      description: 'Пройдите 100 номеров зловещего отеля, прячьтесь в шкафах от Rush и решайте загадки.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-27f91a27e7ca4a243c49e772274d817f/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 1962086868,
      title: 'Tower of Hell',
      category: 'Hardcore Obby',
      description: 'Шесть случайно сгенерированных секций паркура без чекпоинтов. Поднимитесь на вершину.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-79b8f15858cf3e271eb1b4b7c126ec6e/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 8737899170,
      title: 'Pet Simulator 99 🐱',
      category: 'Tycoon & Pets',
      description: 'Массовое открытие яиц, золотые и радужные питомцы, гигантские зоны фермы монет.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-5ae32b262a67e54c75ca78e826b5c00e/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 286090829,
      title: 'Arsenal',
      category: 'Arcade Gun Game',
      description: 'Каждое убийство меняет оружие. Сделайте решающий удар золотым ножом для победы.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-923cb9f6cb7beaa4ff499c7ea0e92750/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 6872265039,
      title: 'BedWars 🛏️',
      category: 'Team Strategy PvP',
      description: 'Защищайте кровать своей команды, собирайте алмазы и уничтожайте базы соперников.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-fdbdcf8362678da47f8ba7febeba74c2/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 15101393044,
      title: 'Dress To Impress 👗',
      category: 'Fashion & Runway',
      description: 'Подбирайте образы под тему показа, голосуйте за моделей и побеждайте на подиуме.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-39d5622345eebfe97bfabdf0fb2cbf93/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 12411473842,
      title: 'Pressure ⚓',
      category: 'Deep Sea Horror',
      description: 'Подводный комплекс Hadal Blacksite. Спуститесь в бездну и доставьте секретные кристаллы.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-27f91a27e7ca4a243c49e772274d817f/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 6403373529,
      title: 'Slap Battles 🖐️',
      category: 'Chaotic Arena',
      description: 'Сотни уникальных перчаток со сверхспособностями. Сбросьте всех противников с арены.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-b4d24a9e525166299b9ba89953ad7ff3/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 189707,
      title: 'Natural Disaster Survival',
      category: 'Classic Survival',
      description: 'Выживайте во время метеоритных дождей, цунами, торнадо и извержений вулканов.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-5ca5989eef1ff51e51b1406e23635a96/150/150/Image/Webp/noFilter',
    },
    {
      placeId: 6068496244,
      title: 'Jailbreak 🚔',
      category: 'Open World Heist',
      description: 'Ограбление банков, погони на вертолетах, полиция против криминального синдиката.',
      iconUrl: 'https://tr.rbxcdn.com/180DAY-923cb9f6cb7beaa4ff499c7ea0e92750/150/150/Image/Webp/noFilter',
    },
  ];

  onMount(async () => {
    loadAccounts();
    try {
      const [f, m, exp, activePlace] = await Promise.all([
        invoke<FastFlagMap>('roblox_get_fast_flags').catch(() => ({})),
        invoke<boolean>('roblox_multi_instance_status').catch(() => false),
        fetchPopularRobloxExperiences().catch(() => []),
        invoke<RobloxPlaceInfo | null>('roblox_detect_place').catch(() => null),
      ]);
      flags = f;
      multiInstance = m;
      experiences = exp.length > 0 ? exp : FALLBACK_MODES;
      recentPlace = activePlace;
    } catch {
      experiences = FALLBACK_MODES;
    }
  });

  $effect(() => {
    const q = searchQuery.trim();
    if (searchTimer) clearTimeout(searchTimer);
    if (!q) {
      searchResults = [];
      searching = false;
      return;
    }
    searching = true;
    searchTimer = setTimeout(async () => {
      try {
        const res = await invoke<RobloxExperiencePreset[]>('roblox_search_experiences', { query: q });
        searchResults = res;
      } catch (err) {
        console.error('Failed to search Roblox platform:', err);
      } finally {
        searching = false;
      }
    }, 350);
  });

  const filteredExperiences = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    if (q) {
      return searchResults.length > 0
        ? searchResults
        : experiences.filter(
            (e) =>
              e.title.toLowerCase().includes(q) ||
              e.category.toLowerCase().includes(q) ||
              String(e.placeId).includes(q)
          );
    }
    return experiences;
  });

  async function applyPreset() {
    try {
      flags = await invoke<FastFlagMap>('roblox_apply_preset', {
        preset: {
          targetFps,
          graphicsApi,
          lighting,
          disableTelemetry,
        },
      });
      statusBanner = 'Параметры FastFlags сохранены в ClientAppSettings.json';
    } catch (err: any) {
      statusBanner = err?.message ?? 'Ошибка сохранения FastFlags';
    }
  }

  async function toggleMutex() {
    try {
      multiInstance = await invoke<boolean>('roblox_set_multi_instance', {
        enabled: !multiInstance,
      });
      statusBanner = multiInstance
        ? 'ROBLOX_singletonMutex захвачен — Мульти-клиент АКТИВЕН'
        : 'Мульти-клиент отключен';
    } catch (err: any) {
      statusBanner = err?.message ?? 'Ошибка переключения mutex';
    }
  }

  function parsePlaceId(input: string): number | null {
    const trimmed = input.trim();
    if (!trimmed) return null;
    const match = trimmed.match(/games\/(\d+)/i);
    if (match && match[1]) {
      return parseInt(match[1], 10);
    }
    const num = parseInt(trimmed, 10);
    return isNaN(num) ? null : num;
  }

  async function handleLaunchMode(placeId: number, title?: string) {
    if (launchingPlaceId === placeId) return;
    launchingPlaceId = placeId;
    statusBanner = `Запуск режима ${title ? `"${title}"` : `#${placeId}`}…`;
    try {
      await launchRobloxPlace(placeId);
    } catch (err: any) {
      statusBanner = `Ошибка запуска: ${err?.message ?? err}`;
    } finally {
      setTimeout(() => {
        launchingPlaceId = null;
      }, 1500);
      setTimeout(() => {
        statusBanner = null;
      }, 4000);
    }
  }

  function handleCustomLaunch() {
    const pid = parsePlaceId(customPlaceInput);
    if (!pid) {
      statusBanner = 'Введите корректный Place ID (число) или ссылку на игру';
      return;
    }
    handleLaunchMode(pid, `Place #${pid}`);
  }
</script>

<div class="space-y-6">
  <!-- Top Banner & Notification -->
  {#if statusBanner}
    <div
      class="flex items-center justify-between rounded-xl border border-sky-500/30 bg-sky-950/60 px-4 py-2.5 font-mono text-xs text-sky-200 backdrop-blur-md shadow-lg"
    >
      <span>{statusBanner}</span>
      <button type="button" onclick={() => (statusBanner = null)} class="text-zinc-400 hover:text-white">✕</button>
    </div>
  {/if}

  <!-- Roblox Bootstrapper Account Selector -->
  <div class="panel-game p-5">
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2">
          <span class="rounded bg-red-500/20 px-2 py-0.5 font-mono text-[10px] font-bold text-red-400">
            BOOTSTRAPPER PROFILES
          </span>
          <h3 class="text-sm font-black text-white">Выбор аккаунта Roblox</h3>
        </div>
        <p class="text-xs text-zinc-400 mt-0.5">
          Переключайтесь между профилями для запуска клиента и мультиинстанса.
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        {#each accounts as acc}
          <div
            class="flex items-center gap-1.5 rounded-xl px-2.5 py-1 text-xs transition-all {acc.isActive
              ? 'border-2 border-red-500 bg-red-500/20 text-white font-bold shadow-md shadow-red-500/10'
              : 'border border-white/10 bg-white/5 text-zinc-400 hover:border-white/25 hover:text-white'}"
          >
            <button
              type="button"
              onclick={() => selectAccount(acc.id)}
              class="flex items-center gap-2"
            >
              <span class="h-2 w-2 rounded-full {acc.isActive ? 'bg-red-400 shadow-[0_0_8px_#f87171]' : 'bg-zinc-600'}"></span>
              <span>{acc.displayName}</span>
            </button>
            {#if accounts.length > 1 && !acc.isActive}
              <button
                type="button"
                onclick={() => removeAccount(acc.id)}
                class="ml-1 text-zinc-500 hover:text-red-400 text-[10px]"
                title="Удалить профиль"
              >✕</button>
            {/if}
          </div>
        {/each}

        {#if !isAddingAccount}
          <button
            type="button"
            onclick={() => (isAddingAccount = true)}
            class="rounded-xl border border-dashed border-white/20 bg-white/5 px-3 py-1.5 text-xs text-zinc-400 hover:border-white/40 hover:text-white"
          >
            + Аккаунт
          </button>
        {:else}
          <div class="flex items-center gap-1.5">
            <input
              type="text"
              bind:value={newAccountUsername}
              placeholder="Имя профиля..."
              class="w-32 rounded-lg border border-red-400 bg-black/60 px-2.5 py-1 text-xs text-white outline-none"
            />
            <button
              type="button"
              onclick={addAccount}
              class="rounded-lg bg-red-500 px-2.5 py-1 text-xs font-bold text-white hover:bg-red-600"
            >
              OK
            </button>
            <button
              type="button"
              onclick={() => (isAddingAccount = false)}
              class="text-zinc-500 hover:text-white px-1"
            >
              ✕
            </button>
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Direct Place Launcher (Custom URL or Place ID) -->
  <div class="panel-game p-5">
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div class="flex items-center gap-4">
        <div
          class="flex h-12 w-12 items-center justify-center rounded-2xl border-2 border-red-500/30 bg-red-500/20 text-red-400 font-black text-sm shadow-inner"
        >
          RBX
        </div>
        <div>
          <h3 class="text-base font-black text-white tracking-tight">Быстрый запуск в режим Roblox</h3>
          <p class="text-xs text-zinc-400 mt-0.5">
            Вставьте Place ID или ссылку на плейс roblox.com, чтобы запуститься напрямую без браузера.
          </p>
        </div>
      </div>

      <div class="flex flex-1 max-w-md items-center gap-2">
        <input
          type="text"
          bind:value={customPlaceInput}
          placeholder="Place ID или ссылка на плейс…"
          class="flex-1 rounded-xl border border-white/10 bg-[#18181b] px-3.5 py-2 text-xs text-white placeholder-zinc-500 outline-none transition-colors focus:border-red-400"
        />
        <button
          type="button"
          onclick={handleCustomLaunch}
          class="btn-game btn-game-red px-5 py-2 text-xs font-bold"
        >
          <svg class="h-3.5 w-3.5 fill-current" viewBox="0 0 24 24">
            <path d="M8 5v14l11-7z" />
          </svg>
          <span>Войти</span>
        </button>
      </div>
    </div>
  </div>

  <!-- Search By Mode Name Bar -->
  <div class="flex flex-wrap items-center justify-between gap-3 border-b border-white/10 pb-4">
    <div class="flex items-center gap-3">
      <h3 class="text-sm font-black uppercase tracking-wider text-white">
        Выбор режима Roblox ({filteredExperiences.length})
      </h3>
      <span class="rounded bg-red-500/20 border border-red-500/30 px-2 py-0.5 font-mono text-[10px] text-red-300">
        1-CLICK LAUNCH
      </span>
    </div>

    <!-- Search input for mode name across ENTIRE Roblox -->
    <div class="relative w-80">
      <input
        type="search"
        bind:value={searchQuery}
        placeholder="Поиск по всему каталогу Roblox…"
        class="w-full rounded-xl border border-white/15 bg-[#121215] px-3.5 py-2 text-xs text-white placeholder-zinc-500 outline-none transition-colors focus:border-red-400"
      />
      {#if searching}
        <div class="absolute right-3 top-2.5 h-3.5 w-3.5 animate-spin rounded-full border border-red-400 border-t-transparent"></div>
      {/if}
    </div>
  </div>

  <!-- Curated Popular Experiences Grid -->
  {#if filteredExperiences.length === 0}
    <div class="panel-game flex flex-col items-center justify-center py-16 text-center text-zinc-400">
      <p class="text-sm font-bold text-zinc-300">Режим не найден</p>
      <p class="mt-1 text-xs text-zinc-500">Попробуйте ввести другой запрос или запустите по прямому Place ID выше.</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
      {#each filteredExperiences as exp (exp.placeId)}
        <div
          class="stat-box-game group relative flex flex-col justify-between overflow-hidden p-3.5 transition-all duration-200 hover:-translate-y-1 hover:border-white/30 hover:shadow-xl"
        >
          <div>
            <div class="flex items-center gap-3">
              <img
                src={proxyRobloxImg(exp.iconUrl)}
                alt={exp.title}
                loading="lazy"
                class="h-14 w-14 rounded-xl object-cover border-2 border-white/10 shadow-md group-hover:scale-105 transition-transform"
              />
              <div class="min-w-0 flex-1">
                <span class="rounded bg-white/10 border border-white/10 px-1.5 py-0.5 font-mono text-[9px] uppercase font-bold text-zinc-300">
                  {exp.category}
                </span>
                <h4 class="truncate text-sm font-bold text-white mt-1 group-hover:text-red-400 transition-colors">
                  {exp.title}
                </h4>
              </div>
            </div>

            <p class="mt-2.5 line-clamp-2 text-xs text-zinc-400 leading-snug">
              {exp.description}
            </p>
          </div>

          <div class="mt-4 pt-2.5 border-t border-white/5 flex items-center justify-between">
            <span class="font-mono text-[10px] text-zinc-500">#{exp.placeId}</span>
            <button
              type="button"
              onclick={() => handleLaunchMode(exp.placeId, exp.title)}
              disabled={launchingPlaceId === exp.placeId}
              class="btn-game btn-game-green px-3.5 py-1.5 text-xs font-bold"
            >
              <svg class="h-3 w-3 fill-current" viewBox="0 0 24 24">
                <path d="M8 5v14l11-7z" />
              </svg>
              <span>{launchingPlaceId === exp.placeId ? 'Запуск…' : 'В режим'}</span>
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  <!-- FastFlags & Multi-Instance Engine -->
  <div class="panel-game p-5">
    <div class="flex flex-wrap items-center justify-between gap-4 border-b border-white/10 pb-4">
      <div>
        <h3 class="text-sm font-bold text-white">Roblox Bootstrapper • FastFlags & Multi-Instance</h3>
        <p class="font-mono text-[11px] text-zinc-400">
          Снятие ограничения FPS, выбор рендера и одновременный запуск нескольких окон Roblox
        </p>
      </div>

      <button
        type="button"
        onclick={toggleMutex}
        class="btn-game {multiInstance ? 'btn-game-green text-white' : 'btn-game-dark text-zinc-300'} px-4 py-2 text-xs font-mono"
      >
        Мульти-клиент: {multiInstance ? 'АКТИВЕН (UNLOCKED)' : 'ОТКЛЮЧЕН'}
      </button>
    </div>

    <div class="mt-4 grid grid-cols-1 gap-4 md:grid-cols-4">
      <label class="block space-y-1">
        <span class="font-mono text-[10px] uppercase text-zinc-400">FPS Лимит (Target FPS)</span>
        <input
          type="number"
          bind:value={targetFps}
          min="30"
          max="999"
          class="w-full rounded-xl border border-white/10 bg-[#18181b] px-3 py-2 font-mono text-xs text-white"
        />
      </label>

      <label class="block space-y-1">
        <span class="font-mono text-[10px] uppercase text-zinc-400">Графический API</span>
        <select
          bind:value={graphicsApi}
          class="w-full rounded-xl border border-white/10 bg-[#18181b] px-3 py-2 font-mono text-xs text-white"
        >
          <option value="vulkan">Vulkan (Низкая задержка)</option>
          <option value="direct3d11">Direct3D 11</option>
          <option value="opengl">OpenGL</option>
          <option value="default">По умолчанию</option>
        </select>
      </label>

      <label class="block space-y-1">
        <span class="font-mono text-[10px] uppercase text-zinc-400">Движок освещения</span>
        <select
          bind:value={lighting}
          class="w-full rounded-xl border border-white/10 bg-[#18181b] px-3 py-2 font-mono text-xs text-white"
        >
          <option value="future">Future Is Bright (Phase 3)</option>
          <option value="shadowmap">ShadowMap (Phase 2)</option>
          <option value="voxel">Voxel</option>
          <option value="default">Игровой</option>
        </select>
      </label>

      <div class="flex items-end">
        <button
          type="button"
          onclick={applyPreset}
          class="btn-game btn-game-white w-full py-2 text-xs font-bold"
        >
          Применить FastFlags
        </button>
      </div>
    </div>
  </div>
</div>
