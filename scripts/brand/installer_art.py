"""Genere les visuels des installeurs NSIS et WiX depuis les symboles de src-tauri/icons/source.

Usage : python scripts/brand/installer_art.py   (depuis la racine du depot, Pillow requis)
"""
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "src-tauri" / "icons" / "source"
OUT = ROOT / "src-tauri" / "icons" / "installer"
FONT = Path(r"C:\Windows\Fonts\SegUIVar.ttf")

OBSIDIAN = (14, 15, 19)
FG = (231, 232, 236)
MUTED = (132, 134, 143)
ION = (46, 196, 214)
WHITE = (255, 255, 255)
RULE = (230, 232, 236)
TAGLINE = ("Maintenance, sécurité", "et optimisation")


def render_mark(svg: Path, workdir: Path) -> Image.Image:
    out = workdir / svg.stem
    subprocess.run(
        ["bun", "x", "tauri", "icon", str(svg), "-o", str(out)],
        cwd=ROOT, check=True, capture_output=True, shell=sys.platform == "win32",
    )
    mark = Image.open(out / "icon.png").convert("RGBA")
    return mark.crop(mark.getbbox())


def fit(mark: Image.Image, size: int) -> Image.Image:
    ratio = size / max(mark.size)
    return mark.resize((round(mark.width * ratio), round(mark.height * ratio)), Image.Resampling.LANCZOS)


def font(size: int, weight: int) -> ImageFont.FreeTypeFont:
    face = ImageFont.truetype(str(FONT), size)
    face.set_variation_by_axes([weight])
    return face


def centered(draw: ImageDraw.ImageDraw, width: int, y: int, text: str, face, fill) -> None:
    draw.text(((width - draw.textlength(text, font=face)) / 2, y), text, font=face, fill=fill)


def brand_panel(canvas: Image.Image, mark: Image.Image, width: int) -> None:
    draw = ImageDraw.Draw(canvas)
    draw.rectangle([0, 0, width - 1, canvas.height], fill=OBSIDIAN)
    m = fit(mark, 84)
    canvas.paste(m, ((width - m.width) // 2, 92), m)
    centered(draw, width, 170, "Wintyx", font(26, 600), FG)
    small = font(11, 400)
    centered(draw, width, 206, TAGLINE[0], small, MUTED)
    centered(draw, width, 222, TAGLINE[1], small, MUTED)
    draw.rectangle([0, canvas.height - 4, width - 1, canvas.height], fill=ION)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        dark = render_mark(SOURCE / "mark-dark.svg", Path(tmp))
        light = render_mark(SOURCE / "mark-light.svg", Path(tmp))

    sidebar = Image.new("RGB", (164, 314), WHITE)
    brand_panel(sidebar, dark, 164)
    sidebar.save(OUT / "nsis-sidebar.bmp")

    header = Image.new("RGB", (150, 57), WHITE)
    m = fit(light, 26)
    header.paste(m, (150 - m.width - 10, (57 - m.height) // 2), m)
    header.save(OUT / "nsis-header.bmp")

    banner = Image.new("RGB", (493, 58), WHITE)
    m = fit(light, 28)
    banner.paste(m, (493 - m.width - 16, (58 - m.height) // 2), m)
    ImageDraw.Draw(banner).line([0, 57, 493, 57], fill=RULE)
    banner.save(OUT / "wix-banner.bmp")

    dialog = Image.new("RGB", (493, 312), WHITE)
    brand_panel(dialog, dark, 164)
    dialog.save(OUT / "wix-dialog.bmp")

    for name in ("nsis-sidebar", "nsis-header", "wix-banner", "wix-dialog"):
        print(OUT / f"{name}.bmp")


if __name__ == "__main__":
    main()
