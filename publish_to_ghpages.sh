#!/bin/bash
# Script pour publier le dossier debian/ sur la branche gh-pages de votre dépôt GitHub
set -e

# Vérifie que le dossier debian/ existe
if [ ! -d "debian" ]; then
  echo "Le dossier debian/ n'existe pas. Exécutez d'abord build_and_publish.sh."
  exit 1
fi

# Crée un dossier temporaire pour le commit
TMP_DIR=$(mktemp -d)
cp -r debian/* "$TMP_DIR/"

cd "$TMP_DIR"
git init

git checkout -b gh-pages


git add .
git commit -m "Publish debian repo for APT updates"
git push --force origin gh-pages

cd -
rm -rf "$TMP_DIR"

echo "Publication terminée sur la branche gh-pages !"
echo "Vérifiez les paramètres GitHub Pages pour pointer sur la branche gh-pages et le dossier racine."

