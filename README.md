# Desktop Entry Creator

A GTK4 desktop application for creating and managing `.desktop` files on Linux.

## Features
- GUI for creating `.desktop` launchers
- Supports all major desktop entry fields
- Localized fields and advanced options
- Saves to `~/.local/share/applications`

## Getting Started

### Prerequisites
- Rust (latest stable)
- GTK4 development libraries

### Build and Run
```bash
git clone https://github.com/yourusername/desktop_app.git
cd desktop_app
cargo build --release
./target/release/desktop_app
```

## Build, Package, and Publish

### 1. Générer le paquet et le repo APT
```bash
./build_and_publish.sh
```
Ce script :
- Compile l'application en release
- Génère le .deb
- Met à jour le dossier debian/ et Packages.gz

### 2. Publier sur GitHub Pages
```bash
./publish_to_ghpages.sh
```
Ce script :
- Publie le contenu du dossier debian/ sur la branche gh-pages
- Met à jour le dépôt distant

### 3. Configuration GitHub Pages
- Dans les paramètres du dépôt, configurez GitHub Pages pour pointer sur la branche gh-pages et le dossier racine.

### 4. Mise à jour pour les utilisateurs
- Les utilisateurs ayant ajouté votre dépôt APT verront les mises à jour dans leur gestionnaire d'applications.

## Dépendances
- Rust
- cargo-deb
- dpkg-scanpackages
- git
- rsync
