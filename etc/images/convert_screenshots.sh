#!/bin/bash

# App Store screenshot dimensions (landscape)
DIMENSIONS=(
    "1280x800"
    "1440x900" 
    "2560x1600"
    "2880x1800"
)

# Create output directory
mkdir -p appstore_screenshots

# Function to center image on white background with shadow
process_screenshot() {
    local input_file="$1"
    local output_file="$2"
    local target_width="$3"
    local target_height="$4"
    
    # Calculate scale factor to fit image with padding (40px on each side for shadow space)
    local scale_width=$((target_width - 80))
    local scale_height=$((target_height - 80))
    
    # Create temp file for rounded corners
    local rounded_image=$(mktemp).png
    
    # First resize and add rounded corners
    magick "$input_file" \
        -colorspace sRGB \
        -resize "${scale_width}x${scale_height}>" \
        -alpha set \
        \( +clone -alpha extract \
           -draw 'fill black polygon 0,0 0,10 10,0 fill white circle 10,10 10,0' \
           \( +clone -flip \) -compose Multiply -composite \
           \( +clone -flop \) -compose Multiply -composite \
        \) -alpha off -compose CopyOpacity -composite \
        "$rounded_image"
    
    # Then add shadow and center on background
    magick "$rounded_image" \
        \( +clone -background black -shadow 60x6+8+8 \) \
        +swap \
        -background white \
        -layers merge \
        -gravity center \
        -extent "${target_width}x${target_height}" \
        "$output_file"
    
    # Cleanup
    rm -f "$rounded_image"
    
    echo "Created: $output_file (${target_width}x${target_height})"
}

# Process each screenshot file
for screenshot in *.png; do
    if [[ -f "$screenshot" ]]; then
        # Get base filename without extension
        base_name=$(basename "$screenshot" .png)
        
        # Process for each App Store dimension
        for dim in "${DIMENSIONS[@]}"; do
            width=$(echo "$dim" | cut -d'x' -f1)
            height=$(echo "$dim" | cut -d'x' -f2)
            
            output_file="appstore_screenshots/${base_name}_${dim}.png"
            
            echo "Processing $screenshot for ${dim}..."
            process_screenshot "$screenshot" "$output_file" "$width" "$height"
        done
    fi
done

echo "All screenshots processed and saved to appstore_screenshots/"
echo "App Store compatible dimensions created: ${DIMENSIONS[*]}"