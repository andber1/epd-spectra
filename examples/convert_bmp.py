#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "numpy~=2.3",
#     "pillow~=11.3",
# ]
# ///
import sys
import numpy as np
from pathlib import Path
from PIL import Image

def main():
    bmp_name = Path("ferris.bmp")
    if len(sys.argv) > 1:
        bmp_name = Path(sys.argv[1])

    print(f"Converting bmp image {bmp_name}")
    img = Image.open(bmp_name)
    img = np.array(img)
    height = img.shape[0]
    width = img.shape[1]
    # round to next multiple of 4 (one pixel encoded into 2 bits and each resulting row needs to contain a whole number of bytes.)
    width_c = ((width - 1) // 4 + 1) * 4
    assert img.shape[2] == 3

    raw_u2 = []
    for v in range(0, height):
        for u in range(0, width_c):
            if u < width:
                # same algorithm as in graphics.rs, search for `impl From<Rgb888> for TriColor`
                color = img[v][u]
                brightness = max(color)
                chroma = max(color) - min(color)
                red = 0
                black = 0
                if chroma > 85 and color[0] > color[1] and color[0] > color[2]:
                    red = 1 
                elif brightness < 128:
                    black = 1
                raw_u2.append(black + (red << 1))
            else:
                # last byte is padded with zeros
                raw_u2.append(0)

    print("Generated code:")
    print(f'const {bmp_name.stem.upper()}_WIDTH: u32 = {width};')
    print(f'const {bmp_name.stem.upper()}_IMG: &[u8] = &[\n    ', end='')
    byte_list = [(raw_u2[i] << 6) + (raw_u2[i+1] << 4) + (raw_u2[i+2] << 2) + raw_u2[i+3] for i in range(0, len(raw_u2), 4)]
    byte_cnt = 0
    for byte in byte_list:
        print(f"{byte}, ", end='')
        byte_cnt += 1
        if byte_cnt > 30:
            print('\n    ', end='')
            byte_cnt = 0
    print("\n];")


if __name__ == "__main__":
    main()