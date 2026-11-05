/**
 * Forensic Output Verification Script: Contract Schema Validation
 * Validates actual Rust struct serialization outputs against PROJECT.md & contracts.cjs
 */
const { validateStorePage, validateEmulatorEntry, validateNetworkSettings } = require('../../../tests/helpers/contracts.cjs');

console.log('=== FORENSIC OUTPUT VERIFICATION ===');

// 1. Rust store.rs serialized output simulation (using camelCase as declared in #[serde(rename_all = "camelCase")])
const actualRustStorePageOutput = {
  page: 0,
  currency: 'USD',
  items: [
    {
      appId: 1091500,
      title: 'Cyberpunk 2077',
      priceRaw: 29.99,
      priceFormatted: '$29.99',
      originalPriceRaw: 59.99,
      originalPriceFormatted: '$59.99',
      discountPercent: 50,
      thumbnailUrl: 'https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1091500/capsule_231x87.jpg',
      headerImage: 'https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1091500/header.jpg',
      bannerUrl: 'https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1091500/library_hero.jpg',
      tags: ['Action', 'RPG', 'FPS'],
      reviewScore: 'Very Positive (91%)',
      media: {
        screenshots: [],
        webmVideo: 'https://example.com/video.webm',
        mp4Video: 'https://example.com/video.mp4',
      },
      shortDescription: 'Open world RPG',
      releaseDate: '10 Dec, 2020',
      storeUrl: 'https://store.steampowered.com/app/1091500',
      installUri: 'steam://install/1091500',
    },
  ],
  hasMore: true,
  totalPages: 10,
  endMessage: null,
};

let storePagePassed = false;
let storePageError = null;
try {
  validateStorePage(actualRustStorePageOutput);
  storePagePassed = true;
} catch (err) {
  storePageError = err.message;
}

console.log('1. StorePage contract validation on actual store.rs camelCase output:');
console.log('   Result:', storePagePassed ? 'PASS' : 'FAIL');
console.log('   Error:', storePageError);

// 2. Emulator catalog output (from emulators.rs)
const actualRustEmulatorOutput = {
  id: 'dolphin',
  name: 'Dolphin',
  repo: 'dolphin-emu/dolphin',
  version: '2407',
  installed: false,
  executable_name: 'Dolphin.exe',
  executableName: 'Dolphin.exe',
  arch: 'x86_64',
  system: 'Nintendo GameCube / Wii',
  short_system: 'GAMECUBE/WII',
  author: 'Dolphin Team',
  description: 'Open-source Nintendo GameCube and Wii emulator',
  supported_extensions: ['.iso', '.gcm'],
  install_path: null,
  executable_path: null,
  is_rate_limited: false,
  website: 'https://dolphin-emu.org',
};

let emuPassed = false;
let emuError = null;
try {
  validateEmulatorEntry(actualRustEmulatorOutput);
  emuPassed = true;
} catch (err) {
  emuError = err.message;
}

console.log('\n2. EmulatorEntry contract validation on emulators.rs output:');
console.log('   Result:', emuPassed ? 'PASS' : 'FAIL');
console.log('   Error:', emuError);

// 3. Network settings output (from network.rs)
const actualRustNetworkSettingsOutput = {
  dns_provider: 'cloudflare',
  doh_enabled: true,
  custom_doh_url: null,
  proxy_enabled: false,
  proxy_url: null,
  cloudflare_media_enabled: true,
  cloudflare_media_endpoint: 'https://media.omnilauncher.app/cdn-cgi/image',
  download_directory: null,
};

let netPassed = false;
let netError = null;
try {
  validateNetworkSettings(actualRustNetworkSettingsOutput);
  netPassed = true;
} catch (err) {
  netError = err.message;
}

console.log('\n3. NetworkSettings contract validation on network.rs output:');
console.log('   Result:', netPassed ? 'PASS' : 'FAIL');
console.log('   Error:', netError);
