#!/bin/bash

OUTPUT_DIR="$1"

if [ -z "$OUTPUT_DIR" ]; then
    echo "Usage: ./download_books.sh <output-directory>"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

# 100 Gutenberg Book IDs
BOOK_IDS=(
    1342 84 11 1661 2701 98 16328 43 74 844 2701 1952 36 135 1063 120 45 55 2641 1260
    8800 19551 6130 2542 84 1513 932 5827 67979 2701 2554 158 100 219 2814 65700
    1399 145 4217 205 34901 46 514 1998 408 8492 4099 4363 6133 1400 17192 72 2500
    1232 30368 289 244 174 3600 19337 23700 5140 2591 148 783 6150 3296 3800 4300
    545 61 62 63 64 65 66 67 68 69 70 2542 14091 28900 26654 17605 24442 39728 5000
    4301 4610 5160 32032 4039 267
)

echo "Starting download of 100 Gutenberg books..."

for ID in "${BOOK_IDS[@]}"; do
    URL="https://www.gutenberg.org/files/$ID/$ID-0.txt"

    echo "Downloading Book $ID..."
    wget -q -O "$OUTPUT_DIR/$ID.txt" "$URL"

    # Fallback if "-0" version doesn't exist
    if [ $? -ne 0 ]; then
        URL2="https://www.gutenberg.org/files/$ID/$ID.txt"
        wget -q -O "$OUTPUT_DIR/$ID.txt" "$URL2"
    fi
done

echo "Download complete!"
