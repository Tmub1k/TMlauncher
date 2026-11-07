/**
 * UI State Store (Svelte 5 Runes)
 * Centralizes UI states like storeOpenMode, storeCurrency, and quick actions.
 */

import { storeLinkManager, type StoreOpenMode, openStoreLink } from '../utils/storeLink';
import { currencyStore } from './store';

class UIState {
  get storeOpenMode(): StoreOpenMode {
    return storeLinkManager.mode;
  }

  setStoreOpenMode(mode: StoreOpenMode) {
    storeLinkManager.setMode(mode);
  }

  get storeCurrency(): 'USD' | 'KZT' {
    return currencyStore.code;
  }

  setStoreCurrency(code: 'USD' | 'KZT') {
    currencyStore.setCurrency(code);
  }

  openStoreLink = openStoreLink;
}

export const uiState = new UIState();
export { storeLinkManager, openStoreLink, type StoreOpenMode };
