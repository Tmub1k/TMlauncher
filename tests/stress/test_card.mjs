import * as $ from 'svelte/internal/server';

import {
	formatBytes,
	formatPlaytime,
	installGameById,
	launchGameById,
	resolveArtUrl
} from '../stores/library';

export default function _unknown_($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		let { game, onSelect, onLaunched } = $$props;
		let imageFailCount = 0;
		let busy = false;
		let statusMessage = null;

		const posterUrl = $.derived(() => {
			if (imageFailCount >= 3) return null;

			const base = resolveArtUrl(game.art.poster);

			if (imageFailCount === 0) return base;

			if (game.platform === 'steam') {
				if (imageFailCount === 1) {
					return `https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/${game.nativeId}/header.jpg`;
				}

				if (imageFailCount === 2) {
					return `https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/${game.nativeId}/capsule_616x353.jpg`;
				}
			}

			return null;
		});

		const PLATFORM_LABELS = {
			steam: 'STEAM',
			epic: 'EPIC',
			roblox: 'ROBLOX',
			itch: 'ITCH',
			emulator: 'RETRO'
		};

		async function handleAction(event) {
			event.stopPropagation();

			if (busy) return;

			busy = true;

			if (game.installed) {
				statusMessage = 'Launching…';

				try {
					await launchGameById(game.id);
					onLaunched?.(game);
				} catch(err) {
					console.error('Failed to launch game:', err);
				} finally {
					setTimeout(
						() => {
							busy = false;
							statusMessage = null;
						},
						1200
					);
				}
			} else {
				statusMessage = 'Installing…';

				try {
					await installGameById(game.id);
				} catch(err) {
					console.error('Failed to start install:', err);
				} finally {
					setTimeout(
						() => {
							busy = false;
							statusMessage = null;
						},
						1500
					);
				}
			}
		}

		$$renderer.push(`<div class="group relative aspect-[2/3] w-full max-w-[240px] cursor-pointer select-none overflow-hidden rounded-[var(--radius-lg,16px)] border border-[var(--border-subtle,rgba(255,255,255,0.08))] bg-[var(--bg-surface-base,#121215)] transition-all duration-200 ease-out hover:-translate-y-1.5 hover:border-[var(--border-active,rgba(255,255,255,0.25))] hover:shadow-[var(--shadow-card-hover,0_20px_40px_-12px_rgba(0,0,0,0.95))] svelte-ujccyr" role="button" tabindex="0">`);

		if (posterUrl()) {
			$$renderer.push(`<!--[0--><img${$.attr('src', posterUrl())} alt="" aria-hidden="true" class="pointer-events-none absolute inset-0 h-full w-full scale-110 object-cover opacity-20 blur-lg"/> <img${$.attr('src', posterUrl())}${$.attr('alt', game.title)} loading="lazy" decoding="async" class="tura-poster relative h-full w-full object-cover object-top transition-all duration-300 ease-out group-hover:scale-[1.03] svelte-ujccyr" onerror="this.__e=event"/>`);
		} else {
			$$renderer.push(`<!--[-1--><div class="flex h-full w-full flex-col items-center justify-center bg-gradient-to-b from-[var(--bg-surface-elevated,#18181c)] to-[var(--bg-canvas,#09090b)] p-5 text-center"><span class="mb-2 rounded-full border border-white/10 bg-white/5 px-2.5 py-0.5 font-mono text-[9px] tracking-widest text-zinc-300">${$.escape(PLATFORM_LABELS[game.platform])}</span> <p class="line-clamp-3 text-xs font-bold tracking-tight text-zinc-100">${$.escape(game.title)}</p></div>`);
		}

		$$renderer.push(`<!--]--> <div class="pointer-events-none absolute inset-x-3 top-3 z-10 flex items-center justify-between"><span class="rounded-md border border-white/15 bg-black/70 px-2 py-0.5 font-mono text-[9px] font-bold tracking-wider text-zinc-200 backdrop-blur-md">${$.escape(PLATFORM_LABELS[game.platform])}</span> `);

		if (game.installed) {
			$$renderer.push(`<!--[0--><span class="flex items-center gap-1.5 rounded-full border border-emerald-500/30 bg-black/60 px-2 py-0.5 font-mono text-[9px] font-semibold text-emerald-400 backdrop-blur-md" title="Installed"><span class="h-1.5 w-1.5 rounded-full bg-emerald-400 shadow-[0_0_6px_#34d399]"></span> <span>READY</span></span>`);
		} else {
			$$renderer.push(`<!--[-1--><span class="flex items-center gap-1 rounded-full border border-white/10 bg-black/60 px-2 py-0.5 font-mono text-[9px] text-zinc-400 backdrop-blur-md" title="In Cloud / Uninstalled"><svg class="h-2.5 w-2.5 fill-current opacity-70" viewBox="0 0 24 24"><path d="M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96z"></path></svg> <span>CLOUD</span></span>`);
		}

		$$renderer.push(`<!--]--></div> <div class="absolute inset-x-0 bottom-0 z-10 flex flex-col justify-end bg-gradient-to-t from-black via-black/85 to-transparent p-3 pt-12"><h3 class="truncate text-xs font-bold tracking-tight text-white drop-shadow-sm">${$.escape(game.title)}</h3> <div class="mt-1 flex items-center justify-between font-mono text-[10px] text-zinc-400"><span>${$.escape(game.installed ? formatPlaytime(game.playtimeMinutes) : 'LIBRARY')}</span> `);

		if (game.sizeBytes) {
			$$renderer.push(`<!--[0--><span>${$.escape(formatBytes(game.sizeBytes))}</span>`);
		} else {
			$$renderer.push('<!--[-1-->');
		}

		$$renderer.push(`<!--]--></div> <button type="button"${$.attr('disabled', busy, true)}${$.attr_class(`mt-2 flex w-full translate-y-2 items-center justify-center gap-1.5 py-1 text-xs font-bold opacity-0 transition-all duration-200 group-hover:translate-y-0 group-hover:opacity-100 disabled:opacity-50 ${game.installed ? 'btn-chrome' : 'btn-minimal'}`)}>`);

		if (busy) {
			$$renderer.push(`<!--[0--><span>${$.escape(statusMessage)}</span>`);
		} else if (game.installed) {
			$$renderer.push(`<!--[1--><svg class="h-3 w-3 fill-current" viewBox="0 0 24 24" aria-hidden="true"><path d="M8 5v14l11-7z"></path></svg> <span>PLAY</span>`);
		} else {
			$$renderer.push(`<!--[-1--><svg class="h-3 w-3 fill-current" viewBox="0 0 24 24" aria-hidden="true"><path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"></path></svg> <span>INSTALL</span>`);
		}

		$$renderer.push(`<!--]--></button></div></div>`);
	});
}