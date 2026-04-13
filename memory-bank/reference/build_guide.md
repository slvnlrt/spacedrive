# Build Guide — Spacedrive Windows Fork

## Environnement requis

```powershell
# PATH à configurer avant toute compilation (mettre dans le profil PowerShell)
$env:PATH = "C:\Program Files\LLVM\bin;" +
            "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.43.34604\bin\Hostx64\x64;" +
            "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64;" +
            $env:PATH
```

- Linker : `lld-link.exe` (LLVM), configuré dans `.cargo/config.toml`
- Package manager frontend : `bun` (pas npm, pas yarn)

## Disque de compilation dédié

`target/` est énorme (~74 Go en debug seul). Dédier un disque :

1. Formater en NTFS, taille d'allocation 64K
2. Désactiver l'indexation Windows sur ce disque (propriétés du lecteur)
3. **Ajouter le dossier aux exclusions de Windows Defender** (le plus impactant)
4. Dans `.cargo/config.toml`, ajouter :
   ```toml
   [build]
   target-dir = "F:/spacedrive-target"
   ```

## Commandes de build / test

### Tests unitaires (rapide, ~1s)
```powershell
cargo test -p sd-core --lib
# 321 tests, ~1s. Suffit pour valider les fixes core.
```

### Vérification de compilation (sans exécuter)
```powershell
cargo check -p sd-core
cargo check -p spacedrive
```

### Build debug (rapide, non optimisé)
```powershell
cargo build
# Exécutables dans target/debug/
#   spacedrive.exe, sd-daemon.exe, sd-cli.exe, sd-server.exe
```

### Build release (lent — LTO + codegen-units=1)
```powershell
cargo build --release
# ~27 minutes. Exécutables dans target/release/
# Lent par design : LTO=true, codegen-units=1, opt-level="s"
```

### App Tauri complète (avec frontend + installer)

```powershell
# Build complet → NSIS installer dans target/release/bundle/nsis/
cargo tauri build --bundles nsis
```

Note : `cargo tauri build` lance automatiquement le daemon build et le frontend build via `beforeBuildCommand`. Si ça échoue (problème bash/PATH sur certaines configurations Windows), contournement manuel en 3 étapes :

```powershell
# Étape 1 : build daemon release
cargo build --release --bin sd-daemon

# Étape 2 : build frontend (depuis apps/tauri)
cd apps/tauri && bun run build && cd ../..

# Étape 3 : build Tauri en désactivant temporairement beforeBuildCommand
cd apps/tauri/src-tauri
(Get-Content tauri.conf.json) -replace '"beforeBuildCommand": "bun run build:daemon:release && bun run build"', '"beforeBuildCommand": ""' | Set-Content tauri.conf.json
cargo tauri build
git checkout tauri.conf.json   # restaurer
```

### App en mode dev (avec hot-reload frontend)
```powershell
cd apps/tauri
bun run tauri:dev
# Même problème de bash pour le beforeDevCommand → même contournement si nécessaire
```

## Tests à ne PAS lancer

```powershell
cargo test          # compile ffmpeg → erreur E0080 sur Windows (struct size mismatch)
cargo test -p sd-core  # compile les tests d'intégration → erreurs pre-existantes upstream
```

Ces erreurs sont dans le code upstream, pas dans nos fixes.

## Nettoyage de l'espace disque

```powershell
# Voir la taille
du -sh target/debug target/release

# Nettoyer debug uniquement
cargo clean --profile dev

# Tout nettoyer
cargo clean
```

## Notes connues

- `warning: Swift client directory not found` → normal, on n'a pas Swift sur Windows
- `warning: sd-core generated 1 warning (Accès refusé os error 5)` → bénin, artefacts incremental en cours d'utilisation
- Warnings `unused_import`, `dead_code`, `unreachable_code` dans `apps/tauri/src-tauri/` → code upstream, pas nos fichiers
