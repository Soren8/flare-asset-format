#!/usr/bin/env bash
# Publish rendered frame PNGs into fantasycore mod paths (spritesheet + animation txt).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

usage() {
  cat <<EOF
Usage: publish_character_to_mod.sh <character> <animation> [render_dir]

Publishes 8-direction render frames into fantasycore:

  mods/fantasycore/images/avatar/<character>/<animation>.png
  mods/fantasycore/animations/avatar/<character>/<animation>.txt

Defaults:
  character: prince
  animation: walk
  render_dir: art_src/characters/blender/render

Example:
  ./publish_character_to_mod.sh prince walk
EOF
}

CHARACTER="${1:-prince}"
ANIMATION="${2:-walk}"
RENDER_DIR="${3:-$REPO_ROOT/art_src/characters/blender/render}"

if [[ "$CHARACTER" == "-h" || "$CHARACTER" == "--help" ]]; then
  usage
  exit 0
fi

if [[ ! -d "$RENDER_DIR" ]]; then
  echo "Render directory not found: $RENDER_DIR" >&2
  exit 1
fi

PNG_OUT="$REPO_ROOT/mods/fantasycore/images/avatar/$CHARACTER/$ANIMATION.png"
TXT_OUT="$REPO_ROOT/mods/fantasycore/animations/avatar/$CHARACTER/$ANIMATION.txt"
IMAGE_MOD_PATH="images/avatar/$CHARACTER/$ANIMATION.png"

mkdir -p "$(dirname "$PNG_OUT")" "$(dirname "$TXT_OUT")"

"$SCRIPT_DIR/run_render8dirs.sh" --build-only "$RENDER_DIR" \
  --prefix "$ANIMATION" \
  --build-sheet "$PNG_OUT" \
  --build-txt "$TXT_OUT" \
  --image-mod-path "$IMAGE_MOD_PATH"

echo "Published $CHARACTER/$ANIMATION to fantasycore mod:"
echo "  png: $PNG_OUT"
echo "  txt: $TXT_OUT"
echo
echo "Game client:"
echo "  FLARE_CHARACTER=animations/avatar/$CHARACTER/$ANIMATION.txt \\"
echo "  FLARE_CHARACTER_SPRITE=$IMAGE_MOD_PATH \\"
echo "    cargo run -p client"
