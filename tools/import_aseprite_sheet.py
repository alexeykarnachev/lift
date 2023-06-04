from dataclasses import dataclass
from collections import defaultdict
import math
from typing import Tuple
import re
import json
from pathlib import Path

import cv2
import numpy as np


@dataclass
class Sprite:
    name: str
    frame_idx: int
    image: np.ndarray
    tl: Tuple[int, int]

    @property
    def h(self):
        return self.image.shape[0]

    @property
    def w(self):
        return self.image.shape[1]

    @property
    def tr(self):
        return (self.tl[0] + self.w, self.tl[1])

    @property
    def bl(self):
        return (self.tl[0], self.tl[1] + self.h)

    def get_neighbor_tls(self):
        bl = self.bl
        tr = self.tr
        return [(bl[0], bl[1] + 1), (tr[0] + 1, tr[1])]

    def to_meta(self, sheet_h):
        # NOTE:
        # When convertic sprite to the meta (coordinates on the sprite sheet)
        # we shrink its size by 1 pixel on each size, because the sprite
        # has been extruded and we need to un-extrude its size back when
        # saving the sheet meta
        x, y = self.tl
        x += 1
        y += 1
        
        y = sheet_h - y - 1
        w, h = self.w, self.h
        w -= 2
        h -= 2

        return {
            "name": self.name,
            "x": int(x),
            "y": int(y),
            "w": int(w),
            "h": int(h),
            "frame_idx": int(self.frame_idx),
        }


@dataclass
class Collider:
    name: str
    frame_idx: int
    tl: Tuple[int, int]
    w: int
    h: int

    def to_meta(self, sprite_h):
        x, y = self.tl
        y = sprite_h - y - 1

        return {
            "name": self.name,
            "x": int(x),
            "y": int(y),
            "w": int(self.w),
            "h": int(self.h),
            "frame_idx": int(self.frame_idx),
        }


_THIS_DIR = Path(__file__).parent
_ROOT_DIR = _THIS_DIR / ".."
_ASEPRITE_DIR = _ROOT_DIR / "aseprite"
_ASSETS_DIR = _ROOT_DIR / "assets/sprites"

_SPRITE_LAYER = "sprite"
_RIGID_COLLIDER_LAYER = "rigid_collider"
_ATTACK_COLLIDER_LAYER = "attack_collider"
_COLLIDER_LAYERS = (_RIGID_COLLIDER_LAYER, _ATTACK_COLLIDER_LAYER)

_ALLOWED_LAYERS = (
    _SPRITE_LAYER,
    _RIGID_COLLIDER_LAYER,
    _ATTACK_COLLIDER_LAYER,
)


if __name__ == "__main__":
    out_dir = _ASSETS_DIR

    # --------------------------------------------------------------------
    # Parse aseprite files and extract sprite images from png images
    sprites = []
    rigid_colliders = []
    attack_colliders = []
    for file_path in _ASEPRITE_DIR.iterdir():
        if not str(file_path).endswith(".json"):
            continue

        dir_ = file_path.parent
        sprite_name = re.sub(r"\.json$", "", file_path.name)
        meta_fp = dir_ / (sprite_name + ".json")
        with open(meta_fp) as f:
            meta = json.load(f)
        frames = meta["frames"]
        meta = meta["meta"]

        sheet_fp = dir_ / meta["image"]
        sheet = cv2.imread(str(sheet_fp), cv2.IMREAD_UNCHANGED)
        layer_names = [layer["name"] for layer in meta["layers"]]
        if _SPRITE_LAYER not in layer_names:
            raise ValueError(f"{meta_fp} is missing the `sprite` layer")

        for frame in frames:
            name = frame["filename"]
            sprite_name, layer_name, frame_idx = name.split(".")
            frame_idx = int(frame_idx)
            if layer_name not in _ALLOWED_LAYERS:
                raise ValueError(
                    f"{name} frame contains unexpected layer: "
                    "{layer_name}. Only {_ALLOWED_LAYERS} "
                    "layers are allowed"
                )

            x, y, w, h = (frame["frame"][k] for k in ("x", "y", "w", "h"))

            if _RIGID_COLLIDER_LAYER not in layer_names:
                rigid_colliders.append(None)

            if _ATTACK_COLLIDER_LAYER not in layer_names:
                attack_colliders.append(None)

            if layer_name == _SPRITE_LAYER:
                # Expand the sprite frame on 1 pixel on both sides, because
                # the sheet contains extruded sprites
                x -= 1
                y -= 1
                w += 2
                h += 2

                image = sheet[y : y + h, x : x + w, :]
                sprite = Sprite(
                    name=sprite_name,
                    frame_idx=frame_idx,
                    image=image,
                    tl=(0, 0),
                )
                sprites.append(sprite)
            elif layer_name in _COLLIDER_LAYERS:
                image = sheet[y : y + h, x : x + w, :].max(-1)
                collider = None
                if image.max() > 0:
                    row = image.max(0)
                    col = image.max(1)
                    left_x = row.argmax()
                    top_y = col.argmax()
                    right_x = len(row) - row[::-1].argmax() - 1
                    bot_y = len(col) - col[::-1].argmax() - 1

                    # Shift the collider on 1 pixel because the corresponding
                    # sprite has been expanded
                    # left_x += 1
                    bot_y += 1
                    # right_x += 1
                    top_y += 1

                    w=right_x - left_x + 1
                    h=bot_y - top_y + 1

                    bot_left = (left_x, bot_y)
                    top_right = (right_x, top_y)
                    collider = Collider(
                        name=sprite_name,
                        frame_idx=frame_idx,
                        tl=(left_x, top_y),
                        w=w,
                        h=h,
                    )
                if layer_name == _RIGID_COLLIDER_LAYER:
                    rigid_colliders.append(collider)
                elif layer_name == _ATTACK_COLLIDER_LAYER:
                    attack_colliders.append(collider)
                else:
                    assert False, f"Unhandled layer: {layer_name}"
            else:
                assert False, f"Unhandled layer: {layer_name}"

    # --------------------------------------------------------------------
    # Pack sprites on the sheet
    inds = sorted(
        range(len(sprites)), key=lambda i: -sprites[i].image.size
    )

    sheet = np.zeros((0, 0, 5), dtype=np.uint8)

    tls_to_try = [(0, 0)]

    for sprite in sprites:
        best_tl_idx = None
        while best_tl_idx is None:
            for i, tl in enumerate(tls_to_try):
                min_x, min_y = tl
                max_x = min_x + sprite.w
                max_y = min_y + sprite.h
                if (
                    sheet.shape[1] > max_x
                    and sheet.shape[0] > max_y
                    and sheet[min_y:max_y, min_x:max_x, -1].max() == 0
                ):
                    best_tl_idx = i
                    break

            if best_tl_idx is None:
                new_sheet = np.zeros(
                    (
                        sheet.shape[0] + sprite.h,
                        sheet.shape[1] + sprite.w,
                        5,
                    ),
                    dtype=np.uint8,
                )
                new_sheet[: sheet.shape[0], : sheet.shape[1]] = sheet
                sheet = new_sheet

        tl = tls_to_try.pop(best_tl_idx)
        sprite.tl = tl

        min_x, min_y = tl
        max_x = min_x + sprite.w
        max_y = min_y + sprite.h
        sheet[min_y:max_y, min_x:max_x, :-1] = sprite.image
        sheet[min_y:max_y, min_x:max_x, -1] = 1
        tls_to_try.extend(sprite.get_neighbor_tls())

    sheet = sheet[..., :-1]
    # --------------------------------------------------------------------
    # Prepare sheet meta file (sprites and colliders coordinates)
    # NOTE: aseprite assumes that the min y is at the top of the sheet,
    # but I flip it and the 0th y coordinate is at the bottom.
    # Such the format is more convenient, because it corresponds to the
    # OpenGL textures coordinates
    sheet_h, sheet_w = sheet.shape[:2]
    meta = {
        "size": [sheet_w, sheet_h],
        "frames": defaultdict(list),
    }
    for i in range(len(sprites)):
        sprite = sprites[i]
        rigid_collider = rigid_colliders[i]
        attack_collider = attack_colliders[i]

        sprite_meta = sprite.to_meta(sheet_h)
        rigid_collider_meta = (
            rigid_collider.to_meta(sprite.h) if rigid_collider else None
        )
        attack_collider_meta = (
            attack_collider.to_meta(sprite.h) if attack_collider else None
        )

        if rigid_collider_meta:
            assert sprite_meta["name"] == rigid_collider_meta["name"]
            assert (
                sprite_meta["frame_idx"]
                == rigid_collider_meta["frame_idx"]
            )
        if attack_collider_meta:
            assert sprite_meta["name"] == attack_collider_meta["name"]
            assert (
                sprite_meta["frame_idx"]
                == attack_collider_meta["frame_idx"]
            )

        frame_meta = {
            _SPRITE_LAYER: sprite_meta,
            _RIGID_COLLIDER_LAYER: rigid_collider_meta,
            _ATTACK_COLLIDER_LAYER: attack_collider_meta,
        }
        meta["frames"][sprite.name].append(frame_meta)

    # Sort frames and delete `name` and `frame_idx` keys from the meta
    frames_meta = meta["frames"]
    for name in frames_meta:
        frames_meta[name] = sorted(
            frames_meta[name], key=lambda d: d[_SPRITE_LAYER]["frame_idx"]
        )
        for frame_meta in frames_meta[name]:
            del frame_meta[_SPRITE_LAYER]["frame_idx"]
            del frame_meta[_SPRITE_LAYER]["name"]

            if frame_meta[_RIGID_COLLIDER_LAYER]:
                del frame_meta[_RIGID_COLLIDER_LAYER]["frame_idx"]
                del frame_meta[_RIGID_COLLIDER_LAYER]["name"]

            if frame_meta[_ATTACK_COLLIDER_LAYER]:
                del frame_meta[_ATTACK_COLLIDER_LAYER]["frame_idx"]
                del frame_meta[_ATTACK_COLLIDER_LAYER]["name"]

    # Save the final sheet and meta json
    cv2.imwrite(str(out_dir / "atlas.png"), sheet)
    with open(out_dir / "atlas.json", "w") as f:
        json.dump(meta, f, indent=4)
