from __future__ import annotations

import math
import subprocess
import tempfile
import wave
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "media" / "yomiflow-demo.mp4"
W, H = 1280, 720
BG = "#09090b"
PANEL = "#18181b"
PANEL_DARK = "#0c0c0f"
LINE = "#29292f"
TEXT = "#f4f4f5"
MUTED = "#a1a1aa"
INDIGO = "#6258f5"
GREEN = "#34d399"

FONT_DIR = Path("C:/Windows/Fonts")
REGULAR = FONT_DIR / "segoeui.ttf"
BOLD = FONT_DIR / "segoeuib.ttf"
JP = FONT_DIR / "YuGothM.ttc"


def font(size: int, bold: bool = False, japanese: bool = False) -> ImageFont.FreeTypeFont:
    choices = [JP, BOLD if bold else REGULAR, FONT_DIR / "arial.ttf"] if japanese else [BOLD if bold else REGULAR, FONT_DIR / "arial.ttf"]
    for path in choices:
        if path.exists():
            try:
                return ImageFont.truetype(str(path), size)
            except OSError:
                continue
    return ImageFont.load_default()


def rounded(draw: ImageDraw.ImageDraw, box: tuple[int, int, int, int], fill: str, outline: str = LINE, radius: int = 14) -> None:
    draw.rounded_rectangle(box, radius=radius, fill=fill, outline=outline, width=1)


def text(draw: ImageDraw.ImageDraw, xy: tuple[int, int], value: str, size: int = 16, color: str = TEXT, bold: bool = False, japanese: bool = False) -> None:
    draw.text(xy, value, font=font(size, bold, japanese), fill=color)


def pill(draw: ImageDraw.ImageDraw, x: int, y: int, label: str, color: str = INDIGO) -> None:
    f = font(12, True)
    box = draw.textbbox((0, 0), label, font=f)
    width = box[2] - box[0] + 24
    draw.rounded_rectangle((x, y, x + width, y + 28), radius=14, fill=color)
    draw.text((x + 12, y + 6), label, font=f, fill="#ffffff")


def chrome(title: str, active: str, subtitle: str, step: int) -> tuple[Image.Image, ImageDraw.ImageDraw]:
    im = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 248, H), fill="#08080a")
    d.line((248, 0, 248, H), fill=LINE, width=1)
    rounded(d, (16, 17, 49, 50), INDIGO, INDIGO, 10)
    text(d, (25, 22), "Y", 19, "#ffffff", True)
    text(d, (61, 20), "YomiFlow", 18, TEXT, True)
    text(d, (24, 76), "CONSUMPTION", 10, "#71717a", True)
    nav = [("Dashboard", "D"), ("Manga", "M"), ("Video", "V"), ("Audio Vault", "A"), ("Web", "W"), ("Light Novel", "N")]
    nav += [("ACTIVE STUDY", ""), ("Decks", "D"), ("SRS Study", "S"), ("Image Occlusion", "I"), ("Kana Dojo", "K"), ("Notebook", "N"), ("JLPT Analytics", "J")]
    nav += [("TOOLS", ""), ("Dictionary", "D"), ("Reading Level", "R"), ("Calendar", "C")]
    y = 98
    for name, icon in nav:
        if name in ("ACTIVE STUDY", "TOOLS"):
            text(d, (24, y + 5), name, 10, "#71717a", True)
            y += 28
            continue
        if name == active:
            d.rounded_rectangle((12, y - 4, 230, y + 32), radius=10, fill=INDIGO)
        color = "#ffffff" if name == active else "#a1a1aa"
        text(d, (26, y + 4), icon, 12, color, True)
        text(d, (54, y + 5), name, 13, color, name == active)
        y += 39
    rounded(d, (12, 456, 236, 626), "#141417", "#303036", 16)
    text(d, (27, 472), "GLOBAL POMODORO", 11, MUTED, True)
    text(d, (75, 495), "25:00", 31, TEXT, True)
    text(d, (28, 542), "Focus (min)     Break (min)", 10, MUTED)
    rounded(d, (27, 559, 119, 589), PANEL_DARK, LINE, 7)
    rounded(d, (128, 559, 220, 589), PANEL_DARK, LINE, 7)
    text(d, (39, 566), "25", 11)
    text(d, (141, 566), "5", 11)
    rounded(d, (27, 597, 220, 618), INDIGO, INDIGO, 7)
    text(d, (105, 600), "Start", 11, "#ffffff", True)
    text(d, (26, 652), "Focus Mode (Zen)", 12, MUTED)
    text(d, (26, 686), "Settings", 12, MUTED)

    # App header and persistent player mirror the real desktop shell.
    text(d, (285, 28), title, 26, TEXT, True)
    text(d, (285, 66), subtitle, 13, MUTED)
    d.line((285, 96, 1240, 96), fill=LINE, width=1)
    d.rectangle((249, 650, W, H), fill="#111114")
    d.line((249, 650, W, 650), fill=LINE)
    text(d, (274, 671), "A", 13, MUTED, True)
    text(d, (309, 665), "Audio Vault", 13, TEXT, True)
    text(d, (309, 687), "Passive immersion", 10, MUTED)
    text(d, (715, 670), "<       PLAY       >", 11, TEXT, True)
    rounded(d, (851, 681, 1137, 686), "#3f3f46", "#3f3f46", 3)
    d.ellipse((891, 677, 903, 689), fill=GREEN)
    text(d, (1157, 674), "● Local tracking active", 10, MUTED)
    # Keep the screen focused on the interface; chapter captions identify the section.
    d.rounded_rectangle((1046, 24, 1241, 49), radius=12, fill="#211f3e", outline="#38346c")
    return im, d


def scene_dashboard() -> Image.Image:
    im, d = chrome("Welcome back", "Dashboard", "A visão diária para organizar o próximo bloco de estudo.", 1)
    text(d, (285, 118), "Your daily study progress", 15, MUTED)
    cards = [(285, "DUE REVIEWS", "12", "◷", "#9b8cff"), (591, "NEW CARDS", "8", "✧", GREEN), (897, "TOTAL CARDS", "246", "▤", TEXT)]
    for x, label, value, icon, color in cards:
        rounded(d, (x, 151, x + 285, 254), PANEL)
        text(d, (x + 22, 171), label, 11, MUTED, True)
        text(d, (x + 22, 196), value, 35, color, True)
        text(d, (x + 243, 172), icon, 18, color, True)
    rounded(d, (285, 278, 1200, 425), PANEL, LINE, 18)
    text(d, (310, 299), "Alphabet Proficiency (Kana Dojo)", 16, TEXT, True)
    text(d, (310, 324), "Mastery and speed across both Japanese syllabaries", 11, MUTED)
    for x, label, char, pct, c in [(310, "Hiragana", "あ", 64, INDIGO), (759, "Katakana", "ア", 48, "#10b981")]:
        rounded(d, (x, 355, x + 410, 407), PANEL_DARK, LINE, 12)
        text(d, (x + 15, 366), char, 19, c, True, True)
        text(d, (x + 49, 369), label, 12, TEXT, True)
        text(d, (x + 356, 369), f"{pct}%", 12, c, True)
        rounded(d, (x + 15, 393, x + 395, 399), "#303036", "#303036", 3)
        rounded(d, (x + 15, 393, x + 15 + int(380 * pct / 100), 399), c, c, 3)
    text(d, (285, 454), "Study Decks", 17, TEXT, True)
    for i, (name, desc, due, new) in enumerate([
        ("Everyday Japanese", "Words and phrases from daily immersion", 9, 4),
        ("N5 Foundations", "Core vocabulary and essential kanji", 3, 4),
    ]):
        x = 285 + i * 465
        rounded(d, (x, 486, x + 435, 620), PANEL, LINE, 14)
        text(d, (x + 20, 506), name, 15, TEXT, True)
        text(d, (x + 20, 533), desc, 11, MUTED)
        text(d, (x + 20, 574), f"{new} new     {due} due", 12, GREEN)
        rounded(d, (x + 326, 577, x + 411, 608), INDIGO, INDIGO, 8)
        text(d, (x + 342, 585), "Study", 11, "#ffffff", True)
    d.ellipse((x + 305, 560, x + 333, 588), outline="#ffffff", width=2)
    d.ellipse((x + 315, 570, x + 323, 578), fill="#ffffff")
    return im


def scene_reader() -> Image.Image:
    im, d = chrome("Web Reader", "Web", "Leia conteúdo japonês com áudio, contexto e análise morfológica.", 2)
    rounded(d, (285, 121, 870, 618), PANEL, LINE, 17)
    pill(d, 312, 144, "NHK EASY • ARTICLE")
    text(d, (312, 193), "日本の春、桜の季節が", 29, TEXT, True, True)
    text(d, (312, 235), "やってきました。各地で桜が咲き始め、", 22, TEXT, False, True)
    text(d, (312, 271), "多くの人が公園を訪れています。", 22, TEXT, False, True)
    d.line((312, 317, 832, 317), fill=LINE)
    text(d, (312, 338), "Cherry blossom season has arrived in Japan.", 14, "#d4d4d8")
    text(d, (312, 363), "People are visiting parks as the flowers begin to bloom.", 12, MUTED)
    rounded(d, (312, 406, 829, 466), PANEL_DARK, LINE, 10)
    text(d, (331, 419), "Listen to article audio", 13, TEXT, True)
    text(d, (331, 443), "Saved locally for offline study", 10, MUTED)
    # Word lookup tooltip reflects the integrated dictionary interaction.
    rounded(d, (474, 276, 710, 393), "#242233", "#56507f", 12)
    text(d, (493, 289), "桜   さくら", 19, TEXT, True, True)
    text(d, (493, 319), "sakura • cherry blossom", 12, "#d4d4d8")
    text(d, (493, 348), "noun · JLPT N2", 11, "#b9b3ff")
    text(d, (493, 370), "+ Add to vocabulary cards", 10, GREEN, True)
    d.ellipse((458, 264, 486, 292), outline="#ffffff", width=2)
    d.ellipse((468, 274, 476, 282), fill="#ffffff")
    rounded(d, (895, 121, 1200, 618), "#101013", LINE, 17)
    text(d, (920, 146), "READING SESSION", 11, MUTED, True)
    text(d, (920, 184), "Comprehension", 13, TEXT, True)
    text(d, (920, 208), "78%", 30, GREEN, True)
    rounded(d, (920, 252, 1174, 260), "#303036", "#303036", 4)
    rounded(d, (920, 252, 1118, 260), GREEN, GREEN, 4)
    text(d, (920, 286), "Vocabulary in passage", 12, TEXT, True)
    for i, (word, reading, meaning, c) in enumerate([
        ("桜", "さくら", "cherry blossom", GREEN),
        ("季節", "きせつ", "season", "#c4b5fd"),
        ("訪れる", "おとずれる", "to visit", "#fda4af"),
    ]):
        y = 321 + i * 77
        rounded(d, (916, y, 1181, y + 61), PANEL, LINE, 10)
        text(d, (930, y + 9), f"{word}  {reading}", 15, c, True, True)
        text(d, (930, y + 35), meaning, 10, MUTED)
    return im


def scene_notebook() -> Image.Image:
    im, d = chrome("Linguistic Notebook", "Notebook", "Salve uma observação e veja o texto japonês separado por função.", 3)
    rounded(d, (285, 122, 530, 617), PANEL, LINE, 15)
    text(d, (308, 145), "My Notes", 16, TEXT, True)
    rounded(d, (470, 138, 508, 170), INDIGO, INDIGO, 8)
    text(d, (483, 142), "+", 23, TEXT, True)
    rounded(d, (305, 187, 510, 221), PANEL_DARK, LINE, 8)
    text(d, (319, 197), "⌕  Search notes", 11, "#71717a")
    rounded(d, (304, 237, 510, 306), "#26243a", "#47416b", 10)
    text(d, (320, 248), "Ordering at a café", 12, TEXT, True)
    text(d, (320, 272), "コーヒーを一つください。", 10, MUTED, japanese=True)
    text(d, (308, 338), "Grammar patterns", 12, MUTED)
    text(d, (308, 378), "Travel phrases", 12, MUTED)
    text(d, (308, 418), "Reading notes", 12, MUTED)
    rounded(d, (553, 122, 1200, 617), PANEL, LINE, 15)
    text(d, (590, 153), "Ordering at a café", 20, TEXT, True)
    text(d, (590, 184), "Autosaved just now", 10, GREEN)
    d.line((589, 211, 1160, 211), fill=LINE)
    text(d, (591, 249), "店員さんに、こう言えます：", 23, TEXT, False, True)
    text(d, (591, 294), "コーヒーを一つください。", 27, TEXT, True, True)
    text(d, (591, 357), "コーヒー", 23, "#fca5a5", True, True)
    text(d, (711, 357), "を", 23, "#93c5fd", True, True)
    text(d, (743, 357), "一つ", 23, "#c4b5fd", True, True)
    text(d, (799, 357), "ください", 23, "#fca5a5", True, True)
    d.line((591, 391, 688, 391), fill="#f87171", width=3)
    d.line((711, 391, 736, 391), fill="#60a5fa", width=3)
    d.line((743, 391, 790, 391), fill="#c4b5fd", width=3)
    d.line((799, 391, 896, 391), fill="#f87171", width=3)
    rounded(d, (590, 436, 1156, 532), PANEL_DARK, LINE, 10)
    text(d, (612, 451), "Noun     Particle     Counter     Request", 11, MUTED)
    text(d, (612, 480), "A simple and polite way to order one coffee.", 13, "#d4d4d8")
    pill(d, 591, 555, "OFFLINE READY", "#176344")
    return im


def scene_kana() -> Image.Image:
    im, d = chrome("Kana Dojo", "Kana Dojo", "Treine hiragana e katakana com prática rápida e domínio progressivo.", 4)
    for x, title, value, color, pct in [(285, "Hiragana mastery", "64%", "#9b8cff", 64), (591, "Katakana mastery", "48%", GREEN, 48), (897, "Overall proficiency", "56%", "#fbbf24", 56)]:
        rounded(d, (x, 121, x + 285, 199), PANEL)
        text(d, (x + 18, 138), title, 11, MUTED)
        text(d, (x + 18, 157), value, 23, color, True)
        rounded(d, (x + 157, 180, x + 265, 186), "#303036", "#303036", 3)
        rounded(d, (x + 157, 180, x + 157 + int(108 * pct / 100), 186), color, color, 3)
    rounded(d, (285, 220, 1200, 617), PANEL, LINE, 18)
    text(d, (323, 250), "Hiragana • Quick Recall", 16, TEXT, True)
    text(d, (323, 279), "Question 8 of 20", 11, MUTED)
    text(d, (506, 306), "ね", 119, TEXT, True, True)
    rounded(d, (486, 472, 784, 523), PANEL_DARK, LINE, 11)
    text(d, (513, 488), "Type the reading…", 14, "#71717a")
    rounded(d, (800, 472, 932, 523), INDIGO, INDIGO, 11)
    text(d, (832, 489), "Check", 13, TEXT, True)
    text(d, (504, 561), "Gojūon   •   Dakuon   •   Yōon", 12, MUTED)
    d.ellipse((795, 478, 823, 506), outline="#ffffff", width=2)
    d.ellipse((805, 488, 813, 496), fill="#ffffff")
    return im


def scene_srs() -> Image.Image:
    im, d = chrome("SRS Study", "SRS Study", "Revise palavras no ritmo certo com repetição espaçada e áudio.", 5)
    text(d, (285, 119), "12 cards remaining", 13, MUTED)
    rounded(d, (285, 164, 1200, 552), PANEL, LINE, 22)
    pill(d, 323, 192, "VOCABULARY")
    text(d, (631, 249), "見つける", 66, TEXT, True, True)
    d.ellipse((861, 260, 907, 306), fill="#24242b", outline=LINE, width=1)
    text(d, (871, 271), "AUDIO", 9, MUTED, True)
    rounded(d, (490, 357, 995, 410), PANEL_DARK, LINE, 12)
    text(d, (669, 373), "Show answer   ·   Space", 14, TEXT, True)
    rounded(d, (319, 447, 1165, 507), "#141417", LINE, 12)
    text(d, (348, 468), "A palavra apareceu no contexto da leitura.", 13, MUTED)
    # Answer reveal and FSRS scheduling choices are shown together as a short demo beat.
    rounded(d, (405, 532, 570, 586), "#3b1c22", "#71323c", 10)
    text(d, (451, 550), "1 · Again", 13, "#fda4af", True)
    rounded(d, (583, 532, 748, 586), "#342b1d", "#725224", 10)
    text(d, (631, 550), "2 · Hard", 13, "#fcd34d", True)
    rounded(d, (761, 532, 926, 586), "#123127", "#21634a", 10)
    text(d, (809, 550), "3 · Good", 13, "#6ee7b7", True)
    rounded(d, (939, 532, 1104, 586), "#1d2542", "#343f74", 10)
    text(d, (984, 550), "4 · Easy", 13, "#c4b5fd", True)
    d.ellipse((925, 539, 953, 567), outline="#ffffff", width=2)
    d.ellipse((935, 549, 943, 557), fill="#ffffff")
    return im


def make_music(path: Path, seconds: int = 27, sample_rate: int = 24000) -> None:
    # A quiet, locally synthesized ambient bed keeps the demo self-contained.
    notes = [110.0, 164.81, 220.0, 261.63]
    with wave.open(str(path), "wb") as wav:
        wav.setnchannels(1)
        wav.setsampwidth(2)
        wav.setframerate(sample_rate)
        chunk = bytearray()
        for i in range(seconds * sample_rate):
            t = i / sample_rate
            fade = min(1.0, t / 1.8, (seconds - t) / 2.0)
            value = sum(math.sin(2 * math.pi * hz * t + j * 0.7) for j, hz in enumerate(notes)) / len(notes)
            value = max(-1.0, min(1.0, value * 0.075 * max(0.0, fade)))
            chunk += int(value * 32767).to_bytes(2, "little", signed=True)
        wav.writeframes(chunk)


def main() -> None:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    scenes = [scene_dashboard(), scene_reader(), scene_notebook(), scene_kana(), scene_srs()]
    captions = [
        ("Tudo começa no painel: progresso, metas e baralhos em um só lugar.", "From the dashboard"),
        ("Leia japonês, consulte palavras e guarde vocabulário com contexto.", "Read and look up words"),
        ("Transforme frases em anotações com análise linguística integrada.", "Understand the language"),
        ("Pratique kana em sessões curtas e acompanhe sua evolução.", "Practice at your pace"),
        ("Revise com lembrança ativa. O YomiFlow organiza o próximo passo.", "Review with spaced repetition"),
    ]
    # Caption cards are embedded into each screen to keep the MP4 usable without narration.
    captioned: list[Image.Image] = []
    for image, (caption, _) in zip(scenes, captions):
        frame = image.copy()
        draw = ImageDraw.Draw(frame, "RGBA")
        draw.rectangle((249, 610, W, 650), fill=(5, 5, 8, 225))
        f = font(16, True)
        bbox = draw.textbbox((0, 0), caption, font=f)
        draw.text(((W + 249 - (bbox[2] - bbox[0])) // 2, 620), caption, font=f, fill="#f4f4f5")
        captioned.append(frame)

    with tempfile.TemporaryDirectory(prefix="yomiflow-demo-") as temp_name:
        temp = Path(temp_name)
        for index, frame in enumerate(captioned):
            frame.save(temp / f"scene-{index}.png")
        make_music(temp / "ambient.wav")
        inputs: list[str] = []
        for index in range(len(captioned)):
            inputs.extend(["-loop", "1", "-t", "6", "-i", str(temp / f"scene-{index}.png")])
        inputs.extend(["-i", str(temp / "ambient.wav")])
        filter_graph = "[0:v]format=yuv420p[v0];"
        for index in range(1, len(captioned)):
            filter_graph += f"[{index}:v]format=yuv420p[v{index}];"
        last = "v0"
        for index in range(1, len(captioned)):
            out = f"x{index}"
            filter_graph += f"[{last}][v{index}]xfade=transition=fade:duration=0.8:offset={5.2 * index}[{out}];"
            last = out
        filter_graph += f"[{len(captioned)}:a]volume=0.18,afade=t=in:st=0:d=1.5,afade=t=out:st=24:d=2.5[a]"
        ffmpeg = "ffmpeg"
        command = [ffmpeg, "-y", *inputs, "-filter_complex", filter_graph, "-map", f"[{last}]", "-map", "[a]", "-t", "26.8", "-r", "24", "-c:v", "libx264", "-preset", "medium", "-crf", "21", "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "128k", "-movflags", "+faststart", str(OUT)]
        subprocess.run(command, check=True, stdout=subprocess.DEVNULL)
    print(f"Created {OUT} ({OUT.stat().st_size:,} bytes)")


if __name__ == "__main__":
    main()
