#!/bin/bash
# Renders the chain 4 testnet social cards into docs/marketing/.
# Requires ImageMagick 7 (magick) and the DejaVu fonts.
#
#   bash docs/marketing/make-cards.sh
set -eu

OUT="$(cd "$(dirname "$0")" && pwd)"

BG='#09090b'
PANEL='#121214'
FG='#f2f1ee'
MUTED='#8a8a92'
GOLD='#F2A900'
TEAL='#2fbaa4'
BLUE='#8aa0b4'

FONT_B=/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf
FONT_R=/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf
FONT_M=/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf

DOT=$(printf '\xc2\xb7')
EM=$(printf '\xe2\x80\x94')

# ---------- A. 1200x630 - Facebook / LinkedIn / X link card ----------
magick -size 1200x630 "xc:$BG" \
  -fill "$GOLD" -draw "rectangle 0,0 1200,8" \
  -font "$FONT_M" -pointsize 19 -kerning 7 -fill "$GOLD" -annotate +80+104 "KOVANICA  $DOT  CHAIN 4 TESTNET" \
  -kerning 0 \
  -font "$FONT_B" -pointsize 74 -fill "$FG" -annotate +80+238 "Chain 4 testnet" \
  -annotate +80+326 "is now the default." \
  -font "$FONT_R" -pointsize 27 -fill "$MUTED" -annotate +80+396 "Wallet  $DOT  Lightnode  $DOT  Comms  $DOT  Desktop console" \
  -fill "$GOLD" -draw "rectangle 80,446 250,449" \
  -font "$FONT_M" -pointsize 23 -fill "$FG" -annotate +80+530 "kovanica.online" \
  -gravity NorthEast -font "$FONT_M" -pointsize 17 -fill "$MUTED" -annotate +80+526 "chain id 4  $DOT  4s rounds  $DOT  12s committed leader" \
  -gravity NorthWest \
  "$OUT/chain4-launch-1200x630.png"

# ---------- B. 1600x900 - X single post image ----------
magick -size 1600x900 "xc:$BG" \
  -fill "$GOLD" -draw "rectangle 0,0 1600,10" \
  -font "$FONT_M" -pointsize 24 -kerning 8 -fill "$GOLD" -annotate +110+150 "KOVANICA  $DOT  CHAIN 4 TESTNET" \
  -kerning 0 \
  -font "$FONT_B" -pointsize 100 -fill "$FG" -annotate +110+330 "Chain 4 testnet" \
  -annotate +110+446 "is now the default." \
  -font "$FONT_R" -pointsize 34 -fill "$MUTED" -annotate +110+530 "No RPC config. Install, open, you're on chain 4." \
  -fill "$TEAL" -draw "rectangle 110,600 130,603" \
  -font "$FONT_M" -pointsize 30 -fill "$FG" -annotate +160+622 "Wallet 0.3.3" \
  -font "$FONT_M" -pointsize 22 -fill "$MUTED" -annotate +160+660 "PRO-21 addresses, faucet button" \
  -fill "$GOLD" -draw "rectangle 110,700 130,703" \
  -font "$FONT_M" -pointsize 30 -fill "$FG" -annotate +160+722 "Lightnode 0.8.4" \
  -font "$FONT_M" -pointsize 22 -fill "$MUTED" -annotate +160+760 "4s DAG sync, chain and epoch view" \
  -fill "$BLUE" -draw "rectangle 110,800 130,803" \
  -font "$FONT_M" -pointsize 30 -fill "$FG" -annotate +160+822 "Comms 0.2.0" \
  -font "$FONT_M" -pointsize 22 -fill "$MUTED" -annotate +160+860 "encrypted P2P messaging, network switch" \
  -gravity NorthEast -font "$FONT_M" -pointsize 24 -fill "$GOLD" -annotate +110+620 "kovanica.online" \
  -gravity NorthWest \
  "$OUT/chain4-x-1600x900.png"

# ---------- C. 1200x400 - Discord banner ----------
magick -size 1200x400 "xc:$BG" \
  -fill "$GOLD" -draw "rectangle 0,0 1200,6" \
  -font "$FONT_M" -pointsize 17 -kerning 6 -fill "$TEAL" -annotate +72+96 "LIVE NOW" \
  -kerning 0 \
  -font "$FONT_B" -pointsize 56 -fill "$FG" -annotate +72+190 "Testnet (chain 4)" \
  -font "$FONT_R" -pointsize 24 -fill "$MUTED" -annotate +72+240 "Wallet  $DOT  Lightnode  $DOT  Comms  $DOT  default on fresh install" \
  -fill "$PANEL" -draw "rectangle 72,290 1128,344" \
  -font "$FONT_M" -pointsize 19 -fill "$GOLD" -annotate +92+323 "https://rpc-testnet.kovanica.online/rpc" \
  -font "$FONT_M" -pointsize 17 -fill "$MUTED" -annotate +700+323 "faucet: faucet.kovanica.online" \
  "$OUT/chain4-discord-1200x400.png"

# ---------- D. 1080x1080 - square (Instagram / Discord embed) ----------
magick -size 1080x1080 "xc:$BG" \
  -fill "$GOLD" -draw "rectangle 0,0 1080,8" \
  -font "$FONT_M" -pointsize 18 -kerning 6 -fill "$GOLD" -annotate +80+110 "KOVANICA  $DOT  CHAIN 4 TESTNET" \
  -kerning 0 \
  -font "$FONT_B" -pointsize 86 -fill "$FG" -annotate +80+300 "Chain 4" \
  -annotate +80+400 "testnet is" \
  -annotate +80+500 "the default." \
  -font "$FONT_R" -pointsize 28 -fill "$MUTED" -annotate +80+600 "Every Kovanica app opens on testnet." \
  -annotate +80+642 "No endpoints to paste, no config files." \
  -fill "$TEAL" -draw "rectangle 80,720 100,723" \
  -font "$FONT_M" -pointsize 26 -fill "$FG" -annotate +130+742 "Wallet $DOT Lightnode $DOT Comms" \
  -fill "$GOLD" -draw "rectangle 80,800 100,803" \
  -font "$FONT_M" -pointsize 26 -fill "$FG" -annotate +130+822 "4s rounds $DOT 12s committed leader" \
  -fill "$BLUE" -draw "rectangle 80,880 100,883" \
  -font "$FONT_M" -pointsize 26 -fill "$FG" -annotate +130+902 "KUNA, 9 decimals $DOT 15${EM}21 validators" \
  -font "$FONT_M" -pointsize 26 -fill "$GOLD" -annotate +80+1010 "kovanica.online" \
  "$OUT/chain4-square-1080x1080.png"

for f in "$OUT"/chain4-*.png; do
  identify -format "%f %wx%h %b\n" "$f"
done
