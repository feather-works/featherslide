#!/usr/bin/env bash
# Gera os instaladores do FeatherSlide em pacote/saida/<versão>/:
#   featherslide_<v>_amd64.deb         Linux (Debian, Ubuntu, Mint…)
#   featherslide-<v>-linux-x86_64.tar.gz   Linux portátil (qualquer distribuição)
#   FeatherSlide-<v>-windows-x64.exe    Windows 10/11 (por usuário, sem administrador)
#   SHA256SUMS
#
# Uso: pacote/gerar.sh [linux|windows ...]   (sem argumentos = todos)
# Precisa de: cargo, dpkg-deb; para Windows, o alvo x86_64-pc-windows-gnu
# (rustup target add x86_64-pc-windows-gnu), mingw-w64 e makensis.
set -euo pipefail

pacote="$(cd "$(dirname "$0")" && pwd)"
raiz="$(dirname "$pacote")"
gui="$raiz/crates/featherslide-gui"
versao="$(sed -n 's/^version = "\(.*\)"/\1/p' "$gui/Cargo.toml" | head -1)"
saida="$pacote/saida/$versao"
alvos=("$@")
[[ ${#alvos[@]} -eq 0 ]] && alvos=(linux windows)
mkdir -p "$saida"
echo "FeatherSlide $versao → $saida"

linux() {
  cargo build --release --manifest-path "$gui/Cargo.toml"
  local bin="$gui/target/release/featherslide"
  strip "$bin" 2>/dev/null || true

  # .deb
  local d; d="$(mktemp -d)"
  install -Dm755 "$bin" "$d/usr/bin/featherslide"
  install -Dm644 "$pacote/featherslide.desktop" "$d/usr/share/applications/featherslide.desktop"
  install -Dm644 "$pacote/featherslide-mime.xml" "$d/usr/share/mime/packages/featherslide.xml"
  for t in 16 24 32 48 64 128 256 512; do
    install -Dm644 "$pacote/icones/featherslide-$t.png" "$d/usr/share/icons/hicolor/${t}x${t}/apps/featherslide.png"
    install -Dm644 "$pacote/icones/featherslide-$t.png" "$d/usr/share/icons/hicolor/${t}x${t}/mimetypes/application-x-featherslide.png"
  done
  install -Dm644 "$pacote/icones/featherslide.svg" "$d/usr/share/icons/hicolor/scalable/apps/featherslide.svg"
  install -Dm644 "$raiz/LICENSE" "$d/usr/share/doc/featherslide/copyright"
  local tamanho; tamanho="$(du -sk "$d/usr" | cut -f1)"
  mkdir -p "$d/DEBIAN"
  cat > "$d/DEBIAN/control" <<EOF
Package: featherslide
Version: $versao
Section: graphics
Priority: optional
Architecture: amd64
Installed-Size: $tamanho
Depends: libc6 (>= 2.43), libxkbcommon0, libwayland-client0 | libx11-6, libfontconfig1
Recommends: libvulkan1 | libgl1
Maintainer: FeatherWorks <https://github.com/FeatherWorks>
Homepage: https://github.com/feather-works/featherslide
Description: Apresentações da suíte FeatherWorks
 Editor de apresentações em Rust com interface nativa: abre .pptx e
 .odp e exporta PDF. Roda inteiro na
 máquina: sem conta, sem login, sem telemetria.
EOF
  cat > "$d/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
command -v update-mime-database >/dev/null && update-mime-database /usr/share/mime || true
command -v update-desktop-database >/dev/null && update-desktop-database -q /usr/share/applications || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q -t /usr/share/icons/hicolor || true
EOF
  cp "$d/DEBIAN/postinst" "$d/DEBIAN/postrm"
  chmod 755 "$d/DEBIAN/postinst" "$d/DEBIAN/postrm"
  dpkg-deb --root-owner-group --build "$d" "$saida/featherslide_${versao}_amd64.deb" >/dev/null
  rm -rf "$d"

  # .tar.gz portátil: o executável, o atalho, os ícones e um instalar.sh para a pasta do usuário.
  local t; t="$(mktemp -d)/featherslide-$versao"
  mkdir -p "$t"
  cp "$bin" "$t/featherslide"
  cp "$pacote/featherslide.desktop" "$pacote/featherslide-mime.xml" "$raiz/LICENSE" "$t/"
  cp -r "$pacote/icones" "$t/"
  cat > "$t/instalar.sh" <<'EOF'
#!/bin/sh
# Instala o FeatherSlide só para este usuário (~/.local), sem administrador.
set -e
cd "$(dirname "$0")"
mkdir -p ~/.local/bin ~/.local/share/applications ~/.local/share/mime/packages
install -m755 featherslide ~/.local/bin/featherslide
sed "s#^Exec=featherslide#Exec=$HOME/.local/bin/featherslide#" featherslide.desktop > ~/.local/share/applications/featherslide.desktop
cp featherslide-mime.xml ~/.local/share/mime/packages/featherslide.xml
for t in 16 24 32 48 64 128 256 512; do
  mkdir -p ~/.local/share/icons/hicolor/${t}x${t}/apps
  cp icones/featherslide-$t.png ~/.local/share/icons/hicolor/${t}x${t}/apps/featherslide.png
done
update-mime-database ~/.local/share/mime 2>/dev/null || true
update-desktop-database ~/.local/share/applications 2>/dev/null || true
echo "FeatherSlide instalado. Procure por \"FeatherSlide\" no menu de aplicativos."
EOF
  chmod 755 "$t/instalar.sh"
  tar -C "$(dirname "$t")" -czf "$saida/featherslide-$versao-linux-x86_64.tar.gz" "featherslide-$versao"
  rm -rf "$(dirname "$t")"
}

windows() {
  cargo build --release --target x86_64-pc-windows-gnu --manifest-path "$gui/Cargo.toml"
  local exe="$gui/target/x86_64-pc-windows-gnu/release/featherslide.exe"
  x86_64-w64-mingw32-strip "$exe" 2>/dev/null || true
  (cd "$pacote" && makensis -V2 -DVERSAO="$versao" -DEXE="$exe" -DSAIDA="$saida/FeatherSlide-$versao-windows-x64.exe" featherslide.nsi)
}

for a in "${alvos[@]}"; do
  echo "== $a"
  "$a"
done
(cd "$saida" && sha256sum -- * | grep -v SHA256SUMS > SHA256SUMS)
ls -lh "$saida"
