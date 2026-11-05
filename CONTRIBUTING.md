# Contributing to OmniLauncher

Thank you for your interest in contributing to OmniLauncher! We welcome community contributions, bug reports, and theme creations.

---

## 🚀 Getting Started

1. **Fork & Clone**: Fork the repository on GitHub and clone your fork locally:
   ```bash
   git clone https://github.com/<your-username>/TMlauncher.git
   cd TMlauncher
   ```
2. **Prerequisites**:
   - [Node.js](https://nodejs.org/) v20+
   - [Rust & Cargo](https://rustup.rs/) (Stable 1.80+)
   - Visual Studio 2022 C++ Build Tools
3. **Install Dependencies**:
   ```bash
   npm install
   ```

---

## 📦 Commit Conventions

We follow [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` New features or integrations
- `fix:` Bug fixes
- `perf:` Performance optimizations
- `style:` Formatting, UI styles, or theme manifests
- `refactor:` Code improvements without changing behavior
- `docs:` Documentation improvements
- `chore:` Dependency bumps, build scripts

---

## 🎨 Adding Custom Themes

To contribute a new theme:
1. Create a JSON manifest in `src/lib/themes/<theme-id>.json` and `themes/<theme-id>.json`.
2. Ensure high contrast and adherence to WCAG AA guidelines.
3. Test window vibrancy settings (`"acrylic"`, `"mica"`, or `"none"`).
4. Register the theme in `src/lib/themes/theme-engine.ts`.

---

## 🧪 Testing Guidelines

Before opening a pull request, verify that:
1. The automated test suite passes:
   ```bash
   node tests/runner.cjs
   ```
2. The frontend builds cleanly:
   ```bash
   npm run build
   ```
3. Rust backend compiles without warnings or errors:
   ```bash
   cargo check --manifest-path src-tauri/Cargo.toml
   ```
