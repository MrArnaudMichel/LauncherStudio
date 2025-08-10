#!/bin/bash
set -e

if [ ! -d "debian" ]; then
  echo "Le dossier debian/ n'existe pas."
  exit 1
fi

# Ajoute le worktree (créé s’il n’existe pas déjà)
if ! git show-ref --verify --quiet refs/heads/gh-pages; then
  git worktree add -b gh-pages /tmp/gh-pages
else
  git worktree add /tmp/gh-pages gh-pages
fi

# Copie le contenu sans écraser .git
rsync -av --delete --exclude '.git' debian/ /tmp/gh-pages/

# Commit et push
cd /tmp/gh-pages
git add .
git commit -m "Publish debian repo for APT updates" || echo "Aucun changement à publier."
git push origin gh-pages

# Nettoyage du worktree
cd -
git worktree remove /tmp/gh-pages
