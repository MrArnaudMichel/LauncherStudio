#!/bin/bash
set -e

if [ ! -d "debian" ]; then
  echo "Le dossier debian/ n'existe pas."
  exit 1
fi

# S'assure d'avoir la dernière version distante de gh-pages
git fetch origin gh-pages || true

# Ajoute/actualise le worktree
if git ls-remote --exit-code --heads origin gh-pages >/dev/null 2>&1; then
  # -B crée ou réinitialise la branche locale gh-pages sur origin/gh-pages
  git worktree add -B gh-pages /tmp/gh-pages origin/gh-pages
else
  # Si la branche distante n'existe pas, on crée une nouvelle branche locale
  git worktree add -b gh-pages /tmp/gh-pages
fi

# Copie le contenu sans écraser .git
rsync -av --delete --exclude '.git' debian/ /tmp/gh-pages/

# Commit et push
cd /tmp/gh-pages
# Met à jour l'index avec les changements
git add .
# Crée un commit s'il y a des modifications
git commit -m "Publish debian repo for APT updates" || echo "Aucun changement à publier."
# Pousse en évitant l'erreur non-fast-forward si quelqu'un a poussé entre-temps
git push --force-with-lease origin gh-pages

# Nettoyage du worktree
cd -
                  git worktree remove /tmp/gh-pages
