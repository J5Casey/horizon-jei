"""JEI in a running modded game: Options > Mods shows the mod and its settings page, the panel beside a
crafting table lists every item, the search box filters it, R opens the recipes of the item under the
pointer, and Back returns to the items."""
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
        # 1. The row and its settings page.
        g.run("options", "mods")
        assert "JEI 0.0.1: ON" in g.labels(), g.labels()
        g.run("click", g.ids()["Settings"])
        page = g.labels()
        assert "Debug: ON" in page, page
        g.run("click", g.ids()["Debug: ON"])
        time.sleep(0.3)
        stored = (g.home / DATA / "debug").read_text()
        assert stored == "false", stored
        g.run("click", g.ids()["Debug: OFF"])
        time.sleep(0.3)

        # 2. The item list beside a crafting table.
        g.play()
        g.run("command", "clear @s")
        table(g)
        stone = item_widget(g, "Stone")
        items = g.shot("1-items")
        panel = ((stone[0] - 2 * SLOT) / g.run("ui")["gui_size"][0], 0.0, 1.0, 1.0)

        # 3. The search box filters the list.
        g.run("click-at", *map(str, search_box(g, stone)))
        type_text(g, "diamond")
        time.sleep(0.4)
        filtered = g.labels()
        assert "Diamond" in filtered, filtered
        assert "Stone" not in filtered, filtered
        searched = g.shot("2-search")
        assert difference(items, searched, panel) > 4, "the panel did not change with the search"

        # 4. R over the item opens its recipes; Back returns to the items.
        diamond = item_widget(g, "Diamond")
        g.run("click-at", *map(str, diamond))
        g.run("key", "r", "tap")
        time.sleep(0.5)
        assert "Back" in g.labels(), g.labels()
        recipes = g.shot("3-recipes")
        assert difference(searched, recipes, panel) > 4, "the recipe page did not draw"
        g.run("click", g.ids()["Back"])
        time.sleep(0.3)
        assert "Diamond" in g.labels(), g.labels()

        # 5. A search that matches nothing empties the list (no panic).
        g.run("click-at", *map(str, search_box(g, stone)))
        type_text(g, "zzzznotanitem", "diamond")
        time.sleep(0.4)
        labels = g.labels()
        assert "Stone" not in labels and "Diamond" not in labels, labels

        print(json.dumps({"proof": "jei", "ok": True, "filtered": len(filtered)}))