import argparse
import re
import json
import shutil
from collections import defaultdict
from pathlib import Path

import cv2
import numpy as np

_THIS_DIR = Path(__file__).parent
_ROOT_DIR = _THIS_DIR.parent
_ASEPRITE_DIR = _ROOT_DIR / "aseprite"
_ASSETS_DIR = _ROOT_DIR / "assets/sprites"


if __name__ == "__main__":
    out_dir = _ASSETS_DIR

    metas = []
    sheets = []
    sprite_names = []

    for file_path in _ASEPRITE_DIR.iterdir():
        if not str(file_path).endswith(".json"):
            continue

        dir_ = file_path.parent
        sprite_name = re.sub("\.json$", "", file_path.name)
        meta_fp = dir_ / (sprite_name + ".json")
        sheet_fp = dir_ / (sprite_name + ".png")

        with open(meta_fp) as f:
            meta = json.load(f)

        sheet = cv2.imread(str(sheet_fp), cv2.IMREAD_UNCHANGED)

        metas.append(meta)
        sheets.append(sheet)
        sprite_names.append(sprite_name)

    atlas_width = 0
    atlas_height = 0
    for meta in metas:
        w = meta["meta"]["size"]["w"]
        h = meta["meta"]["size"]["h"]
        atlas_width = max(w, atlas_width)
        atlas_height += h

    atlas_sheet = np.zeros((atlas_height, atlas_width, 4), dtype=np.uint8)
    atlas_meta = defaultdict(list)
    cursor = 0
    for meta, sheet, sprite_name in zip(metas, sheets, sprite_names):
        atlas_sheet[cursor : cursor + sheet.shape[0], : sheet.shape[1], :] = sheet

        frames = meta["frames"]
        for frame in frames:
            sprite_name, frame_idx = frame["filename"].split(".")
            source_size = frame["sourceSize"]
            frame = frame["frame"]

            frame["x"] += 1
            frame["y"] += 1 + cursor
            frame["w"] -= 2
            frame["h"] -= 2

            u = frame["x"]
            v = atlas_height - frame["y"]
            w = frame["w"]
            h = frame["h"]

            sprite = {
                "u": u,
                "v": v,
                "w": w,
                "h": h,
            }

            atlas_meta[sprite_name].append(sprite)

        cursor += sheet.shape[0]

    result = {
        "file_name": "atlas.png",
        "size": [atlas_width, atlas_height],
        "sprites": atlas_meta,
    }

    out_dir.mkdir(exist_ok=True, parents=True)
    with open(out_dir / "atlas.json", "w") as f:
        json.dump(result, f, indent=4)

    cv2.imwrite(str(out_dir / "atlas.png"), atlas_sheet)
