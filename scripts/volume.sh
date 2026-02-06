#!/bin/bash
# Volume control script for Ajazz N1 dial
# Usage: ./volume.sh <rotation_value>
#   rotation_value: -1 for counter-clockwise (volume down)
#                   +1 for clockwise (volume up)

STEP=5  # Volume change percent per tick

if [ "$1" -lt 0 ]; then
    # Counter-clockwise - decrease volume
    pactl set-sink-volume @DEFAULT_SINK@ "-${STEP}%"
else
    # Clockwise - increase volume
    pactl set-sink-volume @DEFAULT_SINK@ "+${STEP}%"
fi
