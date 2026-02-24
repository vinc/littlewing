#!/bin/sh

N="$1"
T="$2"
S="$3"
F="$(date +"$T-%Y%m%d-%H%M%S.pgn")"
C="$(($(getconf _NPROCESSORS_ONLN) - 1))"

case "$T" in
  "short") TC="10+0.1"; H="16" ;;
  "long")  TC="60+0.6"; H="128" ;;
  *) exit ;;
esac

case "$S" in
  "non-regression") E0="-10"; E1="0" ;;
  "gainer")         E0="0"; E1="10" ;;
  *)                E0="0"; E1="10" ;;
esac

if [ "$N" = "sprt" ]; then
  # Usage: sh tournament.sh sprt {short|long}
  fastchess -tournament gauntlet -rounds "10000" -concurrency "$C" -repeat \
    -sprt elo0="$E0" elo1="$E1" alpha=0.05 beta=0.05 \
    -resign movecount=3 score=400 -draw movenumber=40 movecount=8 score=10 \
    -openings file=8moves_v3.pgn format=pgn order=random \
    -ratinginterval 20 -pgnout file="$F" -recover \
    -engine cmd="littlewing-new" proto=uci name="Little Wing 0.7.0-new" \
    -engine cmd="littlewing-old" proto=uci name="Little Wing 0.7.0-old" \
    -each tc="$TC" option.Hash="$H"
else
  # Usage: sh tournament.sh <number> {short|long}
  cutechess-cli -tournament gauntlet -rounds "$N" -concurrency "$C" -repeat \
    -resign movecount=3 score=400 -draw movenumber=40 movecount=8 score=20 \
    -openings file=8moves_v3.pgn format=pgn order=random \
    -ratinginterval 20 -pgnout "$F" -recover \
    -engine cmd="littlewing-0.7.0-34" proto=xboard name="Little Wing v0.7.0-34-g8e6afdd XB" \
    -engine cmd="sungorus" proto=uci name="Sungorus 1.4" \
    -engine cmd="foxsee" proto=uci name="FoxSEE 8.2" \
    -engine cmd="mora" proto=uci name="MORA 1.1.0" \
    -engine cmd="spacedog" proto=uci name="Spacedog 0.97.7" \
    -engine cmd="robocide" proto=uci name="Robocide 0.1" \
    -engine cmd="cadabra" proto=uci name="Cadabra 2.0.1" \
    -engine cmd="odonata" proto=uci name="Odonata 0.4.0" \
    -engine cmd="akimbo" proto=uci name="Akimbo 0.3.0" \
    -engine cmd="achillees" proto=uci name="Achillees 1.0" \
    -each tc="$TC" option.Hash="$H"
fi
