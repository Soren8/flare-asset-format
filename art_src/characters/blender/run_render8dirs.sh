#!/usr/bin/env bash
# Render a Mixamo FBX walk cycle into eight FLARE isometric directions.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

normalize_path() {
  local path="$1"
  case "$path" in
    "~/"*) path="${HOME}/${path#\~/}" ;;
    "~") path="$HOME" ;;
  esac
  printf '%s\n' "$path"
}

resolve_blender() {
  if [[ -n "${BLENDER:-}" ]]; then
    normalize_path "$BLENDER"
    return
  fi
  if command -v blender >/dev/null 2>&1; then
    command -v blender
    return
  fi
  cat >&2 <<'EOF'
blender not found.

Install Blender or add it to PATH, for example:
  ln -s /path/to/blender-*/blender ~/.local/bin/blender

Or point at a specific binary for this run:
  BLENDER=/path/to/blender ./run_render8dirs.sh ...
EOF
  exit 1
}

BLENDER_BIN="$(resolve_blender)"

if [[ ! -e "$BLENDER_BIN" ]]; then
  echo "Blender not found: $BLENDER_BIN" >&2
  if [[ "${BLENDER:-}" == "~"* ]]; then
    echo "Note: expand ~ in BLENDER yourself, or rely on this script's tilde expansion." >&2
  fi
  exit 1
fi

if [[ ! -x "$BLENDER_BIN" ]]; then
  echo "Blender is not executable: $BLENDER_BIN" >&2
  exit 1
fi

if [[ $# -lt 1 ]]; then
  SHOW_USAGE=1
elif [[ "$1" == "--build-only" ]]; then
  if [[ $# -lt 2 ]]; then
    SHOW_USAGE=1
  else
    BUILD_ONLY=1
    OUTPUT_DIR="$2"
    shift 2
  fi
elif [[ $# -lt 2 ]]; then
  SHOW_USAGE=1
else
  BUILD_ONLY=0
  FBX="$1"
  OUTPUT_DIR="$2"
  shift 2
fi

if [[ "${SHOW_USAGE:-0}" -eq 1 ]]; then
  cat <<EOF
Usage:
  run_render8dirs.sh <mixamo.fbx> <output_dir> [pipeline args...]
  run_render8dirs.sh --build-only <output_dir> [pipeline args...]

Uses: $BLENDER_BIN

Environment:
  BLENDER   Override Blender executable (otherwise resolved from PATH)

Examples:
  ./run_render8dirs.sh ~/Downloads/YBot_Walk.fbx /tmp/walk_out \\
    --prefix walk --frames 35 --build-sheet /tmp/walk_out/walk.png \\
    --build-txt /tmp/walk_out/walk.txt

  ./run_render8dirs.sh --build-only /tmp/walk_out \\
    --prefix walk --build-sheet /tmp/walk_out/walk.png \\
    --build-txt /tmp/walk_out/walk.txt

  BLENDER=/opt/blender/blender ./run_render8dirs.sh character.fbx ./render_out --frames 35
EOF
  exit 1
fi

mkdir -p "$OUTPUT_DIR"

PIPELINE_ARGS=(--output-dir "$OUTPUT_DIR" "$@")
if [[ "${BUILD_ONLY:-0}" -eq 1 ]]; then
  PIPELINE_ARGS=(--skip-render "${PIPELINE_ARGS[@]}")
else
  PIPELINE_ARGS=(--fbx "$FBX" "${PIPELINE_ARGS[@]}")
fi

"$BLENDER_BIN" --background --python "$SCRIPT_DIR/flare_render_pipeline.py" -- \
  "${PIPELINE_ARGS[@]}"
status=$?

if [[ $status -ne 0 ]]; then
  echo "Blender failed with exit code $status" >&2
  exit "$status"
fi

echo "Render complete: $OUTPUT_DIR"
