"""JEI in a running modded game: Options > Mods shows the mod and its settings page, the panel beside a
crafting table lists every item, the search box filters it, R opens the recipes of the item under the
pointer, the station tabs and the "+" work, and Back walks the history."""
import json
import time
from game import Game, difference

DATA = "horizon-mods/.horizon-mod-data/jei"
SLOT = 18


def item_widget(g, name):
    """The panel's item `name`: its centre in GUI pixels."""
    ui = g.run("ui")
    found = next(
        (w for w in ui["widgets"] if w.get("label") == name and w.get("kind") == "button"),
        None,
    )
    assert found, f"no {name!r} in the panel: {g.labels()}"
    x, y, w, h = found["bounds"]
    return x + w / 2, y + h / 2


def search_box(g, stone):
    """The panel's search box: under the item grid, near its left edge."""
    height = g.run("ui")["gui_size"][1]
    # The grid's left edge is the first item's centre minus half a slot and
    # its 1 px inset; the search box starts there.
    return stone[0] - 9 + 20, height - 16


def type_text(g, text, previous=""):
    """Type into the focused field; `horizon-ctl type` clears one character per `\\b`."""
    for _ in range(len(previous)):
        g.run("type", "\\b")
    if text:
        g.run("type", text)


def table(g):
    """Opens a crafting table placed beside the player."""
    x, y, z = (int(c // 1) for c in g.run("status")["player"]["position"])
    if not getattr(g, "table", None):
        g.run("command", f"setblock {x + 2} {y} {z} minecraft:crafting_table")
        g.table = (x + 2, y, z)
        time.sleep(1)
    g.run("interact", *map(str, g.table))
    time.sleep(1)
    assert g.run("screen").get("screen") not in ("playing", "creative"), g.run("screen")


def run(ctl, resources, out):
    with Game(ctl, resources, out) as g:
        # 1. The row and its settings page: "Click gives items" off by default,
        #    and the switch persists.
        g.run("options", "mods")
        assert "JEI 0.1.0: ON" in g.labels(), g.labels()
        g.run("click", g.ids()["Settings"])
        page = g.labels()
        assert "Click gives items: OFF" in page, page
        assert "Language: English" in page, page
        g.run("click", g.ids()["Click gives items: OFF"])
        time.sleep(0.3)
        stored = (g.home / DATA / "give").read_text()
        assert stored == "true", stored
        g.run("click", g.ids()["Click gives items: ON"])
        time.sleep(0.3)
        assert (g.home / DATA / "give").read_text() == "false"
        # The language setting: Spanish labels, and back.
        g.run("click", g.ids()["Language: English"])
        time.sleep(0.3)
        assert (g.home / DATA / "language").read_text() == "es"
        assert "Clic da objetos: OFF" in g.labels(), g.labels()
        g.run("click", g.ids()["Language: Español"])
        time.sleep(0.3)
        assert (g.home / DATA / "language").read_text() == "en"

        # 2. The item list beside a crafting table.
        g.play()
        g.run("command", "clear @s")
        # Unlock every recipe in the book and give the one input the transfer
        # test places, so the "+" button is active.
        g.run("command", "recipe give @s *")
        g.run("give", "minecraft:diamond_block", "1")
        time.sleep(0.5)
        table(g)
        stone = item_widget(g, "Stone")
        items = g.shot("1-items")
        width = g.run("ui")["gui_size"][0]
        panel = ((stone[0] - 2 * SLOT) / width, 0.0, 1.0, 1.0)

        # 3. The search box filters the list.
        g.run("click-at", *map(str, search_box(g, stone)))
        type_text(g, "diamond")
        time.sleep(0.4)
        filtered = g.labels()
        assert "Diamond" in filtered, filtered
        assert "Stone" not in filtered, filtered
        searched = g.shot("2-search")
        assert difference(items, searched, panel) > 4, "the panel did not change with the search"

        # 4. R over the item opens its recipes, with a tab per station.
        diamond = item_widget(g, "Diamond")
        g.run("click-at", *map(str, diamond))
        g.run("key", "r", "tap")
        time.sleep(0.5)
        labels = g.labels()
        assert "Back" in labels, labels
        assert "Crafting Table tab" in labels, labels
        assert "Furnace tab" in labels, labels
        recipes = g.shot("3-recipes")
        assert difference(searched, recipes, panel) > 4, "the recipe page did not draw"

        # 5. "+" places the recipe's ingredients in the open grid, as the book does.
        plus = next((l for l in labels if l.startswith("+ ")), None)
        assert plus, labels
        g.run("click", g.ids()[plus])
        time.sleep(0.5)
        placed = [slot.get("item") for slot in g.run("screen").get("slots", [])]
        assert "minecraft:diamond_block" in placed, placed

        # 6. A recipe slot opens that stack's recipes; Back walks the history.
        slot = next(
            (l for l in labels if l.startswith("Block of Diamond ") and l.endswith(":0")),
            None,
        )
        assert slot, labels
        g.run("click", g.ids()[slot])
        time.sleep(0.5)
        assert "Furnace tab" not in g.labels(), "not the Block of Diamond page"
        g.run("click", g.ids()["Back"])
        time.sleep(0.4)
        assert "Furnace tab" in g.labels(), g.labels()

        # 7. The furnace tab shows the smelting recipe.
        g.run("click", g.ids()["Furnace tab"])
        time.sleep(0.4)
        smelted = g.labels()
        assert any(l.startswith("Diamond Ore ") for l in smelted), smelted

        # 8. Back returns to the item list (one page per open and per R press).
        for _ in range(5):
            labels = g.labels()
            if "Back" not in labels:
                break
            g.run("click", g.ids()["Back"])
            time.sleep(0.3)
        assert "Back" not in g.labels(), g.labels()

        # 9. A search that matches nothing keeps the panel (and the search box).
        g.run("click-at", *map(str, search_box(g, stone)))
        type_text(g, "zzzznotanitem", "diamond")
        time.sleep(0.4)
        labels = g.labels()
        assert "Stone" not in labels and "Diamond" not in labels, labels

        print(json.dumps({"proof": "jei", "ok": True, "filtered": len(filtered)}))