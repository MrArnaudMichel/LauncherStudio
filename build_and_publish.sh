#!/bin/bash
# Script d'automatisation du build, packaging .deb et génération du repo APT pour GitHub Pages
set -e

# 1. Build Rust en release
cargo build --release

# 2. Génère le paquet .deb
cargo deb

# 3. Copie le .deb dans le dossier debian/
DEB_FILE=$(ls target/debian/*.deb | head -n1)
cp "$DEB_FILE" debian/

# 4. Nettoie les anciens .deb dans debian/ (garde le dernier)
find debian/ -name '*.deb' ! -newer "$DEB_FILE" -delete

# 5. Génère Packages.gz
cd debian
rm -f Packages.gz
# Ignore le warning override file
 dpkg-scanpackages . /dev/null | gzip -9c > Packages.gz
cd ..

# 6. Instructions pour publier sur GitHub Pages
cat <<EOF

Build et packaging terminés !
Pour publier sur GitHub Pages :
- Commitez et poussez le dossier debian/ sur la branche gh-pages ou main selon votre configuration.
- Vérifiez que GitHub Pages est activé sur le dossier debian/.
EOF

