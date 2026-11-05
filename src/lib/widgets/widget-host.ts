/**
 * Modular Widget System (`src/lib/widgets/widget-host.ts`).
 *
 * Implements the dual-layer widget architecture specified in the OmniLauncher blueprint:
 * 1. **Shadow DOM Web Component (`<omni-widget-card>`)**: Strict CSS style isolation
 *    while inheriting `--*` Steam Material Noir custom properties.
 * 2. **Sandboxed `<iframe>` Community Widget Host**: Runs untrusted 3rd-party HTML/JS
 *    widgets inside `sandbox="allow-scripts"` and bridges safe read-only RPC calls
 *    (`getLibraryCount`, `getTheme`, `getActiveSessions`) via `window.postMessage`.
 */

import { invoke } from '@tauri-apps/api/core';
import { themeStore } from '../stores/theme';

export interface WidgetRpcRequest {
  jsonrpc: '2.0';
  id: string | number;
  method: 'getTheme' | 'getLibraryCount' | 'getActiveSessions';
  params?: Record<string, unknown>;
}

export interface WidgetRpcResponse {
  jsonrpc: '2.0';
  id: string | number;
  result?: unknown;
  error?: { code: number; message: string };
}

const ALLOWED_METHODS = new Set(['getTheme', 'getLibraryCount', 'getActiveSessions']);

/** Handle a `postMessage` RPC call from a sandboxed community widget iframe. */
export async function handleWidgetRpcMessage(event: MessageEvent): Promise<void> {
  const data = event.data as Partial<WidgetRpcRequest> | null;
  if (!data || data.jsonrpc !== '2.0' || typeof data.method !== 'string' || data.id === undefined) {
    return;
  }

  const reply = (response: WidgetRpcResponse) => {
    (event.source as WindowProxy | null)?.postMessage(response, '*');
  };

  if (!ALLOWED_METHODS.has(data.method)) {
    reply({
      jsonrpc: '2.0',
      id: data.id,
      error: { code: -32601, message: `Method '${data.method}' is not permitted in sandbox` },
    });
    return;
  }

  try {
    let result: unknown;
    switch (data.method) {
      case 'getTheme':
        result = themeStore.theme;
        break;
      case 'getLibraryCount': {
        const games = await invoke<unknown[]>('get_library');
        result = games.length;
        break;
      }
      case 'getActiveSessions':
        result = await invoke<unknown[]>('get_active_sessions');
        break;
    }
    reply({ jsonrpc: '2.0', id: data.id, result });
  } catch (err: any) {
    reply({
      jsonrpc: '2.0',
      id: data.id,
      error: { code: -32000, message: err?.message ?? 'RPC execution failed' },
    });
  }
}

/** Custom Element `<omni-widget-card>` with Shadow DOM style isolation. */
export class OmniWidgetCardElement extends HTMLElement {
  private shadow: ShadowRoot;

  constructor() {
    super();
    this.shadow = this.attachShadow({ mode: 'open' });
  }

  connectedCallback(): void {
    const title = this.getAttribute('title') ?? 'Community Widget';
    const srcdoc = this.getAttribute('srcdoc');

    this.shadow.innerHTML = `
      <style>
        :host {
          display: block;
          border-radius: var(--card-radius, 14px);
          border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
          background: var(--bg-surface-1, #121215);
          color: var(--text-primary, #fafafa);
          overflow: hidden;
        }
        header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 10px 14px;
          border-bottom: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
          font: 600 11px/1 ui-monospace, SFMono-Regular, Menlo, monospace;
          text-transform: uppercase;
          letter-spacing: 0.08em;
          color: var(--text-secondary, #a1a1aa);
        }
        iframe {
          width: 100%;
          height: 160px;
          border: 0;
          background: transparent;
        }
        .slot-body {
          padding: 14px;
        }
      </style>
      <header>
        <span>${title}</span>
        <span>SANDBOX</span>
      </header>
      ${
        srcdoc
          ? `<iframe sandbox="allow-scripts" referrerpolicy="no-referrer"></iframe>`
          : `<div class="slot-body"><slot></slot></div>`
      }
    `;

    if (srcdoc) {
      const iframe = this.shadow.querySelector('iframe');
      if (iframe) iframe.srcdoc = srcdoc;
    }
  }
}

export function registerOmniWidgetElements(): void {
  if (typeof window === 'undefined' || !('customElements' in window)) return;
  if (!customElements.get('omni-widget-card')) {
    customElements.define('omni-widget-card', OmniWidgetCardElement);
    window.addEventListener('message', handleWidgetRpcMessage);
  }
}
