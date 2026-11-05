<!--
  FriendsList.svelte — Unified Steam & Epic Games Friends Presence + Discord RPC 2.0.
  Styled with 3D Casual Game GUI.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    fetchDiscordRpcStatus,
    fetchUnifiedFriends,
    setDiscordRpcEnabled,
    type DiscordRpcStatus,
    type UnifiedFriend,
  } from '../stores/friends';

  let friends = $state<UnifiedFriend[]>([]);
  let rpc = $state<DiscordRpcStatus | null>(null);
  let loading = $state(true);
  let searchQuery = $state('');

  // Steam Credentials Drawer
  let showCreds = $state(false);
  let apiKeyInput = $state('');
  let steamIdInput = $state('');
  let saveStatus = $state<string | null>(null);

  async function loadData() {
    loading = true;
    try {
      const [f, d, creds] = await Promise.all([
        fetchUnifiedFriends().catch(() => []),
        fetchDiscordRpcStatus().catch(() => null),
        invoke<any>('get_steam_credentials').catch(() => null),
      ]);
      friends = f;
      rpc = d;
      if (creds) {
        if (creds.apiKey) apiKeyInput = creds.apiKey;
        if (creds.steamId64) steamIdInput = creds.steamId64;
      }
    } catch (err) {
      console.warn('Social sync fallback:', err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadData();
  });

  async function toggleRpc() {
    if (!rpc) return;
    rpc = await setDiscordRpcEnabled(!rpc.enabled);
  }

  async function saveCredentials() {
    saveStatus = 'Сохранение настроек Steam…';
    try {
      await invoke('set_steam_credentials', {
        credentials: {
          apiKey: apiKeyInput.trim() || null,
          steamId64: steamIdInput.trim() || null,
          steamgriddbApiKey: null,
        },
      });
      saveStatus = 'Steam подключен! Обновление списка друзей…';
      await loadData();
      setTimeout(() => {
        showCreds = false;
        saveStatus = null;
      }, 1200);
    } catch (err: any) {
      saveStatus = `Ошибка: ${err?.message ?? err}`;
    }
  }

  const filteredFriends = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return friends;
    return friends.filter(
      (f) =>
        f.displayName.toLowerCase().includes(q) ||
        (f.currentGame && f.currentGame.toLowerCase().includes(q))
    );
  });
</script>

<div class="space-y-6">
  <!-- Top Control Bar -->
  <div class="panel-game p-5">
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div>
        <h2 class="text-base font-black tracking-tight text-white">Список друзей • Steam & Epic Games</h2>
        <p class="font-mono text-xs text-zinc-400 mt-0.5">
          Живой статус активности, статус в игре и быстрое присоединение к серверу (Universal Discord RPC 2.0)
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <button
          type="button"
          onclick={() => (showCreds = !showCreds)}
          class="btn-game btn-game-white px-3.5 py-2 text-xs"
        >
          ⚙ Настройки Steam API
        </button>

        {#if rpc}
          <button
            type="button"
            onclick={toggleRpc}
            class="btn-game {rpc.enabled ? 'btn-game-green' : 'btn-game-dark'} px-4 py-2 text-xs"
          >
            Discord RPC: {rpc.enabled ? 'АКТИВЕН' : 'ПАУЗА'}
          </button>
        {/if}

        <button
          type="button"
          onclick={loadData}
          class="btn-game btn-game-dark px-3 py-2 text-xs"
          title="Синхронизировать список друзей"
        >
          ↻ Обновить
        </button>
      </div>
    </div>

    <!-- Credentials Drawer (if open) -->
    {#if showCreds}
      <div class="mt-4 rounded-xl border border-white/10 bg-black/40 p-4 space-y-3">
        <h4 class="text-xs font-bold text-white uppercase tracking-wider">Подключение Steam Web API</h4>
        <p class="text-xs text-zinc-400">
          Для синхронизации онлайн-друзей Steam введите SteamID64 и Web API Key (получить можно на
          <a href="https://steamcommunity.com/dev/apikey" target="_blank" class="text-sky-400 underline">steamcommunity.com/dev/apikey</a>).
        </p>

        <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <input
            type="text"
            bind:value={steamIdInput}
            placeholder="SteamID64 (например, 76561199308848804)"
            class="rounded-xl border border-white/10 bg-[#18181b] px-3.5 py-2 text-xs text-white outline-none focus:border-white/40"
          />
          <input
            type="password"
            bind:value={apiKeyInput}
            placeholder="Steam Web API Key"
            class="rounded-xl border border-white/10 bg-[#18181b] px-3.5 py-2 text-xs text-white outline-none focus:border-white/40"
          />
        </div>

        <div class="flex items-center justify-between pt-1">
          <span class="font-mono text-xs text-emerald-300">{saveStatus ?? ''}</span>
          <button
            type="button"
            onclick={saveCredentials}
            class="btn-game btn-game-green px-4 py-1.5 text-xs font-bold"
          >
            Сохранить и подключить
          </button>
        </div>
      </div>
    {/if}

    <!-- Friends Search Bar -->
    <div class="mt-4 flex items-center justify-between gap-4 pt-3 border-t border-white/5">
      <div class="flex items-center gap-2">
        <span class="font-mono text-[10px] uppercase text-zinc-400">Друзей в сети:</span>
        <span class="rounded bg-emerald-500/20 px-2 py-0.5 font-mono text-[10px] text-emerald-300 border border-emerald-500/30">
          {friends.filter((f) => f.state === 'ingame' || f.state === 'online').length} онлайн
        </span>
      </div>

      <input
        type="search"
        bind:value={searchQuery}
        placeholder="Поиск по друзьям…"
        class="w-60 rounded-xl border border-white/10 bg-[#18181b] px-3.5 py-1.5 text-xs text-white placeholder-zinc-500 outline-none transition-colors focus:border-white/40"
      />
    </div>
  </div>

  <!-- Friends Cards Grid -->
  {#if loading}
    <div class="flex flex-col items-center justify-center py-20 text-zinc-400">
      <div class="h-8 w-8 animate-spin rounded-full border-2 border-white/20 border-t-white"></div>
      <p class="mt-3 font-mono text-xs">Запрос статусов Steam и Epic Games…</p>
    </div>
  {:else if filteredFriends.length === 0}
    <div
      class="panel-game flex flex-col items-center justify-center py-16 text-center text-zinc-400"
    >
      <p class="text-sm font-bold text-zinc-200">Друзья не синхронизированы</p>
      <p class="mt-1 text-xs text-zinc-500 max-w-md">
        Нажмите «Настройки Steam API» выше и укажите ваш SteamID64 / API ключ для загрузки списка друзей Steam в реальном времени.
      </p>
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
      {#each filteredFriends as friend (friend.id)}
        <div
          class="stat-box-game flex items-center justify-between p-4 transition-all duration-200 hover:-translate-y-0.5 hover:border-white/20"
        >
          <div class="flex items-center gap-3 min-w-0">
            {#if friend.avatarUrl}
              <img
                src={friend.avatarUrl}
                alt=""
                class="h-10 w-10 rounded-xl object-cover border border-white/10"
              />
            {:else}
              <div
                class="flex h-10 w-10 items-center justify-center rounded-xl bg-white/5 border border-white/10 font-bold text-xs text-zinc-300"
              >
                {friend.displayName.slice(0, 2).toUpperCase()}
              </div>
            {/if}

            <div class="min-w-0 space-y-0.5">
              <div class="flex items-center gap-2">
                <span
                  class="h-2 w-2 rounded-full {friend.state === 'ingame'
                    ? 'bg-emerald-400 shadow-[0_0_8px_#34d399]'
                    : friend.state === 'online'
                      ? 'bg-sky-400'
                      : 'bg-zinc-600'}"
                ></span>
                <span class="truncate text-xs font-bold text-white">{friend.displayName}</span>
                <span
                  class="rounded bg-white/5 border border-white/10 px-1 font-mono text-[9px] uppercase text-zinc-400"
                >
                  {friend.platform}
                </span>
              </div>
              <p class="truncate font-mono text-[11px] {friend.state === 'ingame' ? 'text-emerald-300' : 'text-zinc-400'}">
                {friend.currentGame ?? (friend.state === 'online' ? 'В сети' : 'Не в сети')}
              </p>
            </div>
          </div>

          {#if friend.joinUri}
            <a
              href={friend.joinUri}
              class="btn-game btn-game-green px-3.5 py-1.5 text-xs font-bold"
            >
              Войти ▶
            </a>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>
