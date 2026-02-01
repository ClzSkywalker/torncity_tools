。、#!/usr/bin/env bash
# 从 app.png 生成 app.ico（保持比例、不拉长）
# 原因：ICO 每个尺寸都是正方形；若直接用 -resize 256x256 且源图不是正方形，
#       ImageMagick 会强制成正方形导致拉长。正确做法：先按比例缩放到 256 内，再居中垫成 256x256。

set -e
root="$(cd "$(dirname "$0")/.." && pwd)"
png="$root/godot/assets/appicon/app.png"
ico="$root/godot/assets/appicon/app.ico"

if [ ! -f "$png" ]; then
  echo "找不到 $png" >&2
  exit 1
fi

# 1) 按比例缩放到 256x256 内（不拉伸）
# 2) 透明底、居中，扩展到 256x256（成正方形）
# 3) 生成多尺寸 ICO
magick "$png" \
  -resize 256x256 \
  -background none -gravity center -extent 256x256 \
  -define icon:auto-resize=256,128,96,64,48,32,16 \
  "$ico"

echo "已生成: $ico"
