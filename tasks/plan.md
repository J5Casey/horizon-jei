# Plan de implementación: JEI para Horizon (Minecraft 26.3 en Rust)

## Resumen

Portar **JustEnoughItems** (JEI 31.17.0, rama `26.3`, licencia MIT) al ecosistema de **Horizon**
(Minecraft 26.3 reimplementado en Rust) como **mod nativo** (`.hnmod`, bundle de
[horizon-mixins](https://github.com/horizon-rust/horizon-mixins)).

El puerto **no** es una traducción línea a línea de las ~117.500 líneas Java de JEI: se reimplementa el
comportamiento (UX, atajos, gramática de búsqueda, modelo de categorías) en Rust sobre las APIs que Horizon
expone, reutilizando JEI como especificación y como fuente de tests y assets (MIT, con atribución).

Entregable por fases:

- **v0.1 "JEI Core"** (publicada el 30-oct-2026): lista de ingredientes en el panel de Horizon, búsqueda tipo
  JEI, R/U sobre cualquier stack, páginas de receta por estación, transferencia vía libro de recetas y modo
  creativo. Solo hook points estables.
- **v0.2 "Bookmarks y tooltips"**: barra de bookmarks persistente, tooltips enriquecidos, subtipos (pociones,
  libros encantados, fuegos artificiales) y configuración real. Usa los hooks de overlay que Horizon ya
  expone, sin reescribir el panel del juego.
- **v0.3 "Overlay real"**: clic y hover en slots, ingrediente fantasma, transferencia a estaciones en SP/LAN,
  modo cheat en supervivencia, yunque y piedra de afilar, búsqueda `$`/`%` y panel en creativo.
- **v1.0 "JEI Parity"**: paridad auditada, ecosistema para content mods, rendimiento y localización completa.

## Estado actual (07-nov-2026)

- **v0.1.0 "JEI Core" está publicada** en el registro de Horizon (`horizon-mods/mods/jei.toml`, autor
  `enderliker`, 30-oct-2026, `.hnmod` firmado). El código vive en `horizon-jei/`: panel con todos los items,
  búsqueda con la gramática de JEI, R/U sobre el panel y los slots, páginas de receta con tab por estación,
  historial, transferencia con `+`, give en creativo, ajustes (give e idioma en/es) y `/jei`.
- Verificado con `cargo test` (31), `native.py check` (Nightly `1da354b`) y `proof.py` end-to-end (fila y
  ajustes, búsqueda, tabs, transferencia, historial, búsqueda vacía).
- **v0.2.0 está implementada (07-nov-2026), sin publicar.** Fase 5 hecha en `horizon-jei` contra el Nightly
  `6560641` (API 411; el bundle pide 313): tooltips con `R: Recetas (n)`/`U: Usos (n)` y marca de bookmark,
  subtipos dirigidos por datos (pociones y flechas tipped, 8 por item), barra de bookmarks de 2×12 en la
  esquina superior izquierda con clic (recetas/usos), Mayús o botón central para quitar y arrastre para
  reordenar, persistencia en `config/jei/bookmarks.txt` y opciones en `config/jei/jei.cfg` (orden, hints,
  barra), tecla A, `/jei sort|bookmarks clear|config`, y ADR 0002.
  Verificado con `cargo test` (41), `cargo clippy -D warnings`, `cargo fmt --check` y
  `native.py check` contra `6560641` (diez inyecciones, las nuevas por nombre de hook). Quedan fuera, por lo
  que el plan no puede hacer todavía: 5.5 (libros encantados y fuegos artificiales, necesitan
  `native_stack_name`), los modos de búsqueda de 5.11, `proof.py` (5.13, necesita juego en marcha) y
  publicar 0.2.0 (5.14, clave de firma y PR al registro). La tabla de estado de la Fase 5 en `tasks/todo.md`
  dice qué criterio se cumple de cada tarea.
- **El roadmap cambió el 07-nov-2026.** Horizon añadió en tres semanas casi todo lo que la Fase 3 daba por
  bloqueado (ver «Revisión del 07-nov-2026»): el overlay ya no depende de un RFC upstream.

### JEI (`JustEnoughItems/`, rama 26.3, HEAD `9f285707e`)

| Dato | Valor |
|---|---|
| Versión | 31.17.0 (`specificationVersion`) |
| Minecraft | 26.3 · Java 25 · NeoForge 26.3.0.38-beta · Fabric loader 0.19.5 |
| Tamaño | 1.233 archivos Java, ~117.541 líneas |
| Módulos | `Common` (API + impl, 32.7k), `Library` (impl + plugins vanilla, 27k), `Gui` (29.3k), `Fabric` (10k), `NeoForge` (16.4k), `Debug` (2k) |
| API pública | `Common/src/api/java/mezz/jei/api/` — 187 archivos, ~12.4k líneas |
| Assets | ~52 sprites GUI, 31 idiomas (`en_us.json` ~630 claves) |
| Tests | ~18k líneas JUnit (tokenizer, filtro, índice, layout) + gametests de loader |

Puntos de entrada clave: `IModPlugin` (ciclo de vida), `IRecipeCategory`, `IRecipeManager`/`IRecipeLookup`,
`IIngredientHelper`/`IIngredientRenderer`, `ISubtypeManager`, `IRecipeTransferHandler`, `IJeiRuntime`,
`PluginLoader`/`JeiStarter` (orden de arranque), `RecipeManagerInternal` + `RecipeIngredientRoleMap` (índice
ingrediente→recetas), `SearchTokenizer`/`ElementPrefixParser`/`IngredientFilter` (búsqueda), `RecipesGui`,
`IngredientListOverlay`, `BookmarkOverlay`.

Categorías vanilla que JEI implementa: crafting (shaped/shapeless), smelting, blasting, smoking, campfire,
stonecutting, smithing (transform/trim), anvil, grindstone, compostable, fuel (por horno) y páginas de
información.

### Horizon (`horizon/` @ Nightly `6560641`, `horizon-mods/native/recipes` como referencia)

| Dato | Valor |
|---|---|
| Juego | 26.3 |
| Hook points | `NATIVE_HOOKS_API = 411` (Nightly 2026-11-07, `6560641`): **432 hook points y 174 helpers** |
| Panel v2 | `native_panel_items`, `native_panel_recipes`, `native_panel_event`, `native_recipes`, `native_known_recipes`, `native_place_recipe`, `native_give` |
| Datos de receta | 9 tipos: crafting shaped/shapeless, smelting, blasting, smoking, campfire, stonecutting, smithing, brewing |
| Límites del panel | 8.192 items · 256 recetas · 16 tabs · 9 slots/receta · 64 stacks/slot · 12 columnas · layouts fijos: `Grid`, `Cooking`, `Single`, `Smithing`, `Brewing` |
| Render del panel | `Screen::Inventory` y `Screen::Creative`, pero en creativo **solo** el mod oficial `recipes` (`native_panel.rs:201`) |
| Overlay | `native_screen_render_post` (78) dibuja sobre cualquier pantalla, creativo incluido; `native_screen_mouse_button_pressed_pre` (80, con modificadores), `mouse_scrolled_pre` (93), `key_pressed_pre`/`char_typed_pre` (104), `mouse_dragged_pre` (123) |
| Dibujo | `native_item` (20), `native_item_model`/`native_sprite`/`native_quad` (111/19), `native_fill`, `native_text`, `native_text_width` |
| Interacción | clics en items/recetas del panel; `Key { key, stack }` para **cualquier** tecla sobre el panel o un slot de la pantalla (`native_key_event` → `native_panel_key`); búsqueda propia |
| Tooltips | `native_tooltip` (12) recibe las líneas de **todo** tooltip del juego, incluido el del panel; `native_tooltip_color` (368) |
| Transferencia | `native_place_recipe(book_id, all)`; además `native_insert_item` (20) y `native_move_items` (48) en el servidor integrado (SP/LAN) |
| Give | `native_give` (solo creativo) y `native_give_items` (servidor, sin restricción de modo) |
| Datos | `native_data_map_types` (143), `native_recipes_of_type` (36), `native_component_types` |
| Config | `native_config_dir` (26), `native_register_configs` (397, demasiado nuevo para 0.2.0) |
| Mods nativos | bundles compilados dentro del juego; single-player/LAN sí; servidores dedicados **no**; sin sandbox |
| Dependencias entre mods | solo en la API tipada (`[dependencies]`); los bundles de mixins siguen sin `requires` |
| API tipada | `horizon_client::mod_api::Hooks`, desde el Nightly 2026-11-03 (launcher ≥ 0.2.37), **no** empaquetable con `native.py` todavía |
| Pruebas | `cargo test`, `native.py check` (compila el juego entero), `proof.py` con `horizon-ctl --offscreen`, `horizon-launcher native-build` |

### Revisión del 07-nov-2026

Desde la publicación de 0.1.0 (30-oct) hasta hoy, Horizon pasó de `NATIVE_HOOKS_API = 7` a **411**. Casi
todos los huecos que la Fase 3 daba por bloqueados están cubiertos; los pocos que siguen abiertos están en
«Huecos restantes». El plan se reescribe así:

| Hueco antiguo (Fase 3) | Hook real hoy | Fase nueva |
|---|---|---|
| Dibujar sobre la pantalla de contenedor | `native_screen_render_post` (78) | 5 |
| Icono de item fuera del panel | `native_item` (20), `native_item_model` (111) | 5 |
| Clic/hover con modificadores en la pantalla | `native_screen_mouse_button_pressed_pre` (80), `mouse_dragged_pre` (123) | 5 / 6 |
| Teclas y texto en pantalla | `native_screen_key_pressed_pre`, `char_typed_pre` (104) | 5 / 6 |
| Tooltip con líneas propias | `native_tooltip` (12) | 5 |
| Stack en el cursor / ghost | dibujar en `render_post` (78) | 6 |
| Dar items fuera de creativo | `native_give_items` (servidor) | 6 |
| Transferencia a estaciones | `native_insert_item` (20), `native_move_items` (48) | 6 |
| Categorías de información | `native_data_map_types` (143), `compost_item`, `enchant_item` | 6 |
| Panel en creativo | sigue vetado a `recipes` (`native_panel.rs:201`) | 6 (upstream) |
| Panel: lado, ancho, scroll | sin cambios (API 2) | 6 (upstream o overlay) |
| Textos de UI en el idioma del juego | sin `native_translate` | 1.0 (upstream) |

Lo que **no** cambió: el panel sigue siendo API 2, fijo a la derecha, de 12 columnas y paginado. La decisión
de 0.2.0 es **no reescribirlo**: se conserva el panel del juego (lista, búsqueda, páginas de receta, tabs,
historial y `+` ya funcionan y los dibuja Horizon) y se construye encima solo lo que falta.

Referencia más cercana: **Recipes** (`horizon-mods/native/recipes`, 1.002 líneas) ya implementa lista de
items, búsqueda `@`/`#`, páginas por estación, historial, transferencia y give. Es el punto de partida
funcional, no el techo: JEI añade bookmarks, subtipos, tooltips, clic en slots, categorías extra y
configuración.

## Alcance

**Dentro**

- Contenido vanilla 26.3 (items, componentes, tags, las 9 clases de receta del juego).
- Recetas de content mods que usen tipos vanilla (aparecen solas en `native_recipes`).
- Single-player y LAN; cliente y servidor integrado.
- UI en el panel de Horizon (lista, búsqueda y páginas de receta) más overlay propio encima de la pantalla
  para bookmarks, tooltips e interacción (0.2 y 0.3).
- Subtipos: pociones, libros encantados, fuegos artificiales y estrellas, leídos de `NativeStack.components`.
- i18n de la UI del mod (en/es inicialmente), nombres de item desde el juego.
- Configuración persistente en `horizon-mods/.horizon-mod-data/jei/` y `config/jei.json`.
- Transferencia a estaciones (horno, ahumador, alto horno, cortapiedras, herrería, brewing) **solo en SP/LAN**.
- Distribución en el registro de Horizon (`.hnmod` firmado, revisado).

**Fuera (explícito)**

- Cargar jars de Fabric/NeoForge o plugins Java de JEI.
- Servidores dedicados (Horizon aún no compila mods nativos para ellos).
- Fluidos/energía como ingredientes (el panel solo acepta items; requiere API nueva).
- Transferencia a estaciones en servidores remotos (necesita protocolo; no se intenta).
- API de plugins en tiempo de ejecución para otros mods (Horizon solo tiene dependencias entre mods tipados;
  para mixins se propone un modelo orientado a datos, ver AD-3).
- Paridad 1:1 de textura/pixel con JEI; se prioriza comportamiento y legibilidad.

## Decisiones de arquitectura

- **AD-1 — Reimplementar, no transliterar.** El código Java está acoplado a `ItemStack`, `SlotDisplay`,
  `Screen` y `GuiGraphicsExtractor`; traducirlo costaría más que reescribirlo. JEI se usa como especificación
  de comportamiento, sus tests puros se portan como red de seguridad y sus assets/idiomas se reutilizan con
  `NOTICE` (MIT).
- **AD-2 — v0.1 solo con hook points estables (`hooks: 2`).** Sin fijar commit: el launcher compila el mod
  sobre el juego que juega el jugador. En 0.1 el panel era la única superficie de dibujo interactiva sobre
  contenedores; desde API 78 existe `native_screen_render_post`, así que 0.2 sube a `hooks: 123` y 0.3 usará
  la misma vía. El HUD sigue quedando detrás de las pantallas: no sirve de overlay.
- **AD-3 — El juego es dueño de los datos; JEI es dueño de la presentación.** Las recetas vienen de
  `native_recipes`; las categorías nuevas (yunque, compost, fuel, info) deben venir de Horizon (hook o
  registro de tipos de receta, ya previsto en `docs/modpacks.md` para content mods 0.4). No se inventa un ABI
  de plugins que Horizon no puede cargar.
- **AD-4 — Núcleo puro y glue delgado.** `runtime.rs` es lógica sin tipos del juego (tests con `cargo test`);
  `glue.rs` convierte `NativeItem`/`NativeRecipe`/`NativePanelEvent` ↔ modelo propio; cada hook tiene un
  helper de una función.
- **AD-5 — Nada de campos privados de `Game`.** Los helpers viven en `game::native_hooks` y *podrían* leer
  campos privados (hijo del módulo `game`), pero eso es un pin encubierto: los nombres de campos no forman
  parte del contrato. Solo helpers documentados; lo que falte se pide upstream.
- **AD-6 — Configuración por archivos + comando.** La página de ajustes de Horizon solo ofrece botones de
  etiqueta; la config rica vive en `native_data_dir("jei")` y se edita con `/jei config ...`, reflejando JEI.
- **AD-7 — Repo propio, tooling oficial.** El mod vive en un repo `horizon-jei/` separado (no se toca
  `horizon-mods`); las comprobaciones usan el `native.py` de un clone de `horizon-mods`.
- **AD-8 — Estrategia de compatibilidad.** `compatibility.json` apunta al Nightly más reciente y declara la
  versión mínima de hooks que el mod usa; CI compila contra `--horizon latest`. Ya **no hace falta shim**:
  los hooks de overlay son oficiales y aditivos. Si un hook concreto cambiara, se pide upstream antes que
  pinear un commit.
- **AD-11 — 0.2 no reescribe el panel.** El panel del juego ya da lista, búsqueda, tabs, páginas de receta,
  historial y `+`; reimplementarlo costaría semanas y perdería el render de recetas. Solo se construye encima
  lo que el panel no puede hacer (bookmarks, tooltips, hover, creativo).
- **AD-12 — Inyecciones por nombre de hook.** Los nuevos helpers usan `hook = "screen_render_post"` y
  `hook_version = 1` (horizon-mixins ≥ 0.6.0) en lugar de `target = "..."`, para que el mod no se rompa si
  Horizon mueve la función en su código. Las inyecciones de 0.1 se migran cuando se toquen.
- **AD-13 — API tipada aplazada.** `native.py` todavía no empaqueta bundles con `[api]`, así que 0.2 y 0.3
  siguen con helpers de mixin. Se reevalúa para 1.0.
- **AD-14 — Los bookmarks son del mod, no del juego.** Lista propia (item + componentes) persistida en
  `native_data_dir("jei")`, dibujada en el overlay y reordenable con arrastre. No se escribe nada fuera de esa
  carpeta ni se envía nada al servidor.
- **AD-9 — Pirámide de pruebas.** (1) unit tests del núcleo, incluidos los portados de JEI; (2) `native.py
  check` como puerta de compilación del juego completo; (3) `proof.py` end-to-end con `horizon-ctl`
  (capturas + widgets + inventario); (4) checklist manual (F1/F3, servidor, reinicio, mod off).
- **AD-10 — Rendimiento medido, no supuesto.** El panel reconvierte todos los stacks cuando cambia la lista;
  con 8.192 items y búsqueda por tecla hay que medir y, si hace falta, debounce/limitación.

## Diseño

### Estructura del bundle

```
horizon-jei/
├── Cargo.toml                      # lib = mixins/jei/runtime.rs (tests sin juego)
├── compatibility.json              # game 26.3, hooks 2, revisión del Nightly
├── README.md  LICENSE  NOTICE
├── proof.py                        # escenarios end-to-end (patrón de native/recipes)
├── docs/
│   ├── jei-parity.md               # matriz feature→estado (tarea 0.4)
│   └── adr/0001-arquitectura.md
└── mixins/jei/
    ├── horizon-mixin.toml          # [mod], [compatibility], [[module]] ×2, [[injection]] ×N
    ├── runtime.rs                  # núcleo: modelo, índice, búsqueda, estado, config, i18n
    ├── glue.rs                     # adaptadores Horizon ↔ núcleo
    ├── list.rs  keys.rs  panel.rs  settings.rs  frame.rs   # un helper por hook
    └── data/                       # (fase 3) categorías JSON, si aplica
```

### Flujo de datos (v0.1)

```
mundo abierto (native_tick / native_player)
   └─ native_recipes() ──► glue ──► catálogo (items+variantes, recetas, índice, búsqueda)
                                        │
inventario/cofre (Screen::Inventory) ──► native_panel_items(lista filtrada)
   ├─ Search(text) ──► viewer.set_search → nueva lista → native_panel_items
   ├─ Key{R|U, stack} ──► viewer.open(stack, uses) → native_panel_recipes(title,tabs,views)
   ├─ Tab/Page/Back ──► viewer.nav → native_panel_recipes
   ├─ SlotClicked ──► viewer.open(stack)
   └─ Action(i) ──► native_place_recipe(book_id, all) | native_give(item, count)
```

### Mapeo de categorías JEI → panel v2

| Categoría JEI | Dato en Horizon | Layout v1 | Fase |
|---|---|---|---|
| Crafting shaped | `CraftingShaped` | `Grid` (w×h) | 2 |
| Crafting shapeless | `CraftingShapeless` | `Grid` reubicado | 2 |
| Smelting / Blasting / Smoking / Campfire | 4 tipos | `Cooking` + nota (s, XP) | 2 |
| Stonecutting | `Stonecutting` | `Single` | 2 |
| Smithing (transform/trim) | `Smithing` | `Smithing` | 2 |
| Brewing | `Brewing` | `Brewing` | 2 |
| Anvil | receta calculada (no existe) | página info | 3 |
| Grindstone | receta calculada | página info | 3 |
| Compostable | tabla interna del juego (no datapack) | página info | 3 (necesita datos) |
| Fuel | propiedad interna del item | página info | 3 (necesita datos) |
| Información (texto) | — | página info | 3 |

### Huecos restantes (entrada para el RFC upstream, 07-nov-2026)

Solo queda esto; el RFC de la Fase 3 se recorta a estas cinco peticiones.

| Necesidad de JEI | Hoy en Horizon | Impacto | Fase |
|---|---|---|---|
| Posición/tamaño/scroll del panel | fijo derecha, 12 columnas, páginas (API 2) | paridad de layout | 6 (upstream o overlay) |
| Panel en inventario creativo | vetado a `recipes` (`native_panel.rs:201`) | paridad creativo | 6 (upstream) |
| `ratón → slot` y rects de slot | solo internos (`Game::slot_at` privado) | clic sin geometría propia, drag a slot | 6 |
| Nombre localizado de stack/efecto/encantamiento | `NativeItem.name` es por item, no por stack; no hay `native_translate` | subtipos legibles | 5 (petición pequeña) |
| Tablas de combustible y compost | internas del juego, sin helper | categorías fuel/compost | 6 |
| Texto de tooltips en bloque | `native_tooltip` solo al dibujar | búsqueda `$tooltip` completa | 6 |
| >16 tabs / >256 recetas / layouts propios | límites del panel | mods con muchas categorías | 6 |
| Fluidos/energía como ingredientes | items solo | paridad modded | 1.0 |
| Idioma del juego para la UI del mod | no expuesto | i18n del mod | 1.0 |

## Fases y tareas (índice)

El detalle con criterios de aceptación y verificación está en `tasks/todo.md`.

### Fase 0 — Fundaciones (≈2-3 días)
- 0.1 Repo y esqueleto del bundle
- 0.2 Prueba de humo del toolchain (`native.py check`)
- 0.3 Spike end-to-end en el juego
- 0.4 Matriz de paridad `docs/jei-parity.md`
- 0.5 CI y comandos de desarrollo

### Fase 1 — Núcleo puro (≈2 semanas)
- 1.1 Modelo de ingrediente y variantes (subtipos)
- 1.2 Modelo de receta y categoría
- 1.3 Índice ingrediente→recetas (recipes/uses)
- 1.4 Búsqueda: tokenizer + índice
- 1.5 Filtro y ordenación
- 1.6 Máquina de estados del viewer
- 1.7 Configuración y persistencia
- 1.8 Tests portados de JEI

### Fase 2 — Panel y v0.1 "JEI Core" (≈3-4 semanas)
- 2.1 Catálogo al abrir/cerrar mundo
- 2.2 Sincronización de items y búsqueda
- 2.3 R/U, historial, tabs y páginas
- 2.4 Vistas de receta por layout
- 2.5 Transferencia (+ / shift)
- 2.6 Give y modo creativo
- 2.7 Fila del mod, teclas y ajustes
- 2.8 Casos límite y mensajes
- 2.9 i18n en/es
- 2.10 Suite de pruebas end-to-end
- 2.11 Empaquetar y publicar 0.1.0

### Fase 3 — Paridad de overlay (superada el 07-nov-2026)

> Se conserva como referencia histórica. Sus tareas se reparten entre las Fases 5 y 6; el RFC upstream se
> recorta a los «huecos restantes» de arriba y deja de ser puerta de entrada.

- 3.1 RFC upstream de hooks → **5.1** (versión recortada)
- 3.2 Implementar contra hooks aceptados (o shim) → **5.2** (sin shim: hooks oficiales)
- 3.3 Barra de bookmarks → **5.6-5.9**
- 3.4 Interacción de slots (tooltips, cursor, ghost) → **5.3** (tooltips) y **6.2-6.3** (slots, ghost)
- 3.5 Categorías restantes (yunque, afilar, compost, fuel) → **6.6-6.7**
- 3.6 Búsqueda avanzada (`$`, `%`, autocompletado) → **6.9**
- 3.7 Configuración completa → **5.10**
- 3.8 Panel en creativo y opciones de panel → **6.8** y **6.11**
- 3.9 Árbol de recetas y navegación → **6.10**
- 3.10 Transferencia fuera de crafting → **6.4**

### Fase 5 — 0.2.0 «Bookmarks y tooltips» (≈3 semanas)

- 5.1 Subir el bundle al Nightly actual: `hooks: 123`, revisión `6560641`, inyecciones por nombre
- 5.2 Capa de overlay propia sobre la pantalla (`native_screen_render_post`)
- 5.3 Tooltips enriquecidos (`native_tooltip`): atajos R/U, id, tags, namespace
- 5.4 Subtipos: pociones y flechas (desde componentes)
- 5.5 Subtipos: libros encantados y fuegos artificiales
- 5.6 Núcleo y persistencia de bookmarks (tests sin juego)
- 5.7 Barra de bookmarks (dibujo, hover, tooltip propio)
- 5.8 Interacción de bookmarks: tecla A, clic izquierdo/derecho, borrar
- 5.9 Arrastrar para reordenar
- 5.10 Configuración real: `config/jei.json` + `/jei config` (ver AD-6)
- 5.11 Modos de búsqueda y ordenación
- 5.12 Documentación, paridad y RFC recortado
- 5.13 `proof.py`: escenarios de 0.2.0
- 5.14 Empaquetar, firmar y publicar 0.2.0

### Fase 6 — 0.3.0 «Overlay real» (≈4-6 semanas)

- 6.1 Tabla de geometría de slots vanilla (menú → rects), validada con proof
- 6.2 Hover y clic en slots: recetas/usos sin tecla
- 6.3 Ingrediente fantasma en el cursor (dibujo; sin tocar el protocolo)
- 6.4 Transferencia a estaciones en SP/LAN (`native_insert_item`, `native_move_items`)
- 6.5 Modo cheat en supervivencia (`native_give_items`) — spike de viabilidad primero
- 6.6 Yunque y piedra de afilar calculados desde componentes
- 6.7 Combustible y compost (upstream o tabla del mod con tarea de actualización)
- 6.8 Panel en creativo (upstream; si no, overlay solo para la lista)
- 6.9 Búsqueda `$tooltip` y `%creative tab` + autocompletado
- 6.10 Árbol de recetas y navegación adelante/atrás completa
- 6.11 Opciones de panel: lado, ancho y scroll (upstream o overlay)
- 6.12 Presupuesto de rendimiento (adelanta parte de 4.2)
- 6.13 Documentación y publicar 0.3.0

### Fase 4 — Ecosistema, rendimiento y 1.0 (≈2 semanas; **después de 5 y 6**)
- 4.1 Categorías data-driven para content mods
- 4.2 Presupuesto de rendimiento y benchmarks
- 4.3 Localización completa
- 4.4 Documentación y atribución
- 4.5 Publicar 1.0.0 y playbook de mantenimiento

### Checkpoints

- **A (tras Fase 0):** el bundle compila, se empaqueta y muestra items en el juego.
- **B (tras Fase 1):** `cargo test` cubre índice, búsqueda, orden y navegación; sin código de juego.
- **C (tras Fase 2):** v0.1 jugable y probada end-to-end en el Nightly; publicada en el registro.
- **0.2.0 (tras Fase 5):** bookmarks, tooltips y subtipos publicados; paridad actualizada; solo el helper de
  nombres queda pedido upstream.
- **0.3.0 (tras Fase 6):** clic en slots, fantasma, transferencia SP/LAN y categorías de información;
  auditoría de paridad contra JEI.
- **E (tras Fase 4):** 1.0 publicada, CI verde contra `--horizon latest`, documentación completa.

El antiguo **Checkpoint D (tras Fase 3)** queda superado: sus criterios se reparten entre 0.2.0 y 0.3.0.

## Fase 5 — 0.2.0 «Bookmarks y tooltips»

**Objetivo:** que un usuario de JEI reconozca el mod: bookmarks, tooltips con atajos y la lista completa
(incluidos subtipos), sin tocar el panel del juego.

**Diseño**

- **Capa de overlay propia.** Dos módulos nuevos: `overlay.rs` (puro: rects, layout de la barra, hit-test,
  escalado — testeable con `cargo test`) y `screen.rs` (el helper de `native_screen_render_post`, que solo
  delega en el núcleo). El resto de helpers (`mouse.rs`, `tooltip.rs`) son envoltorios de una función, como
  en 0.1.
- **Sin geometría de slots.** La tecla A no la necesita: `native_key_event` reenvía **cualquier** tecla a
  `native_panel_key`, que resuelve el stack apuntado (item del panel o slot de la pantalla) y lo entrega como
  `Key { key, stack }`. Basta con registrar `bookmark` en las claves del mod y comparar con `native_mod_key`.
  Limitación: requiere que nuestro panel esté visible (en creativo no lo estará hasta 0.3).
- **La barra va a la izquierda del contenedor**, calculada desde `NativeScreenView.rect`, para no solaparse
  con el panel del juego (que vive a la derecha). Si no cabe, se coloca arriba/abajo de la pantalla.
- **Clics de la barra:** `native_screen_mouse_button_pressed_pre`; si el punto cae en la barra se pone
  `canceled = true` y el juego no ve el clic. El `released_pre` solo se cancela si se canceló su press.
- **Bookmarks del mod:** `Vec<Bookmark { item, components }>` con dedupe, límite configurable y orden propio.
  Persistencia JSON en `native_data_dir("jei")/bookmarks.json`; fichero corrupto → lista vacía sin perder el
  anterior (`.bak`).
- **Tooltips:** `native_tooltip` añade líneas al tooltip existente (nunca lo sustituye): `R: recetas`,
  `U: usos`, id y tags. Se apaga con config; si el stack oculta su tooltip, no se llama.
- **Subtipos:** el índice de 0.1 pasa a expandir `NativeStack.components` (pociones, flechas, libros
  encantados, cohetes y estrellas). El tope de variantes es configurable para no reventar las 8.192 entradas
  del panel.
- **Config:** `config/jei.json` vía `native_config_dir()` (API 26), validado al cargar, más
  `/jei config get|set|reset` y los botones actuales. No se usa `native_register_configs` (API 397, demasiado
  nueva).

**Entregable:** 0.2.0 firmada en el registro con bookmarks, tooltips y subtipos, `docs/` actualizado y el RFC
upstream recortado.

**No entra:** clic en slots, fantasma, transferencia a estaciones, creativo, categorías de información.

**Depende de:** 5.1 (bundle sobre el Nightly actual). El resto es independiente entre sí salvo 5.7-5.9.

## Fase 6 — 0.3.0 «Overlay real»

**Objetivo:** comportamiento de JEI de verdad: clicar slots, arrastrar ingredientes, rellenar estaciones y
cubrir las categorías que el panel no puede dibujar.

**Diseño**

- **Geometría de slots (6.1-6.2).** `NativeScreenView` da el tipo de menú (`screen.menu`), el rect del panel y
  el contador de slots, pero no sus rects. Se implementa una tabla por menú vanilla (`generic_9x3`, `crafting`,
  `furnace`, `smithing`, `stonecutter`, `brewing`...) validada con proof. Menú desconocido → se conserva R/U.
  En paralelo se pide upstream un helper `mouse → slot` para eliminar la tabla.
- **Fantasma (6.3).** Dibujar el icono del ingrediente bajo el cursor en `render_post`. Visual: no cambia el
  carrito ni envía paquetes.
- **Transferencia (6.4).** Solo SP/LAN, sobre el servidor integrado: `native_insert_item` y
  `native_move_items`, con la posición del contenedor abierto (`native_container_opened`) y transacción
  comprobada (si algo no cabe, no se mueve nada). Detrás de un flag de config; en servidor remoto no aparece
  el botón.
- **Cheat (6.5).** Spike primero: verificar en qué lado corre el evento del panel y si se puede alcanzar el
  servidor integrado con `native_give_items`. Si no hay vía limpia, se descarta y se documenta; no se inventa
  un protocolo.
- **Categorías (6.6-6.7).** Yunque y piedra de afilar se calculan desde componentes. Combustible y compost
  necesitan datos: primero se piden a upstream (`native_burn_time`, `native_compost_chance`); si no, tabla
  del mod con tarea de actualización por versión.
- **Creativo y opciones de panel (6.8, 6.11).** Petición upstream para quitar el veto a `recipes` y para
  lado/ancho/scroll. Si no llega, 6.8 puede dibujar solo la lista en creativo (sin páginas de receta).

**Entregable:** 0.3.0 firmada, con paridad JEI auditada y los límites documentados.

**No entra:** fluidos, servidores dedicados, plugin ABI.

**Depende de:** 0.2.0; 6.2, 6.3 y 6.4 dependen de 6.1; 6.5 empieza por un spike de viabilidad.

## Estimación

| Fase | Esfuerzo (1 persona) | Riesgo |
|---|---|---|
| 0 Fundaciones | 2-3 días | Bajo |
| 1 Núcleo | 8-10 días | Bajo |
| 2 Panel / v0.1 | 15-20 días | Medio |
| 5 Bookmarks y tooltips / 0.2.0 | 12-18 días | Medio |
| 6 Overlay real / 0.3.0 | 25-35 días | Medio-alto (geometría de slots y transferencia) |
| 4 Ecosistema / 1.0 | 8-12 días | Medio |
| **Total restante** | **~2-3 meses** a tiempo completo (0.2 + 0.3 + 1.0) | |

La antigua Fase 3 ya no es un esfuerzo aparte: su alcance se ha repartido entre las Fases 5 y 6, y el shim
pinneado que preveía ha dejado de ser necesario.

## Riesgos y mitigación

| Riesgo | Impacto | Mitigación |
|---|---|---|
| El panel no permite overlay ni bookmarks | **Resuelto** | `native_screen_render_post` y los `native_screen_*` existen desde API 78-123 |
| El panel sigue vetado en creativo | Medio | 0.3 lo pide upstream; mientras, la lista no aparece en creativo y se documenta |
| Subtipos sin nombre localizado de efecto/encantamiento | Medio | Pedir `native_stack_name`/`native_translate` (petición pequeña); si no llega, etiqueta con el id y aviso |
| Geometría de slots propia se rompe con menús de mods | Medio | Solo menús vanilla conocidos (`screen.menu`); el resto conserva R/U; pedir helper upstream |
| Cambios diarios en Horizon rompen helpers | Medio | APIs documentadas, inyección por nombre (AD-12), CI contra `--horizon latest`, sin campos privados (AD-5) |
| El overlay tapa el panel o el tooltip | Medio | Layout con márgenes fijos y proof por captura; el panel siempre arriba en z |
| Transferencia server-side duplica o pierde items | Alto | Solo SP/LAN, transaccional, desactivada por defecto hasta auditoría; nunca en servidores remotos |
| Modo cheat en supervivencia se percibe como trampa | Medio | Config explícita + confirmación en chat; desactivado por defecto |
| Búsqueda lenta al reconvertir 8.192 stacks por tecla | Medio | Medir en 5.13/6.11; debounce y límite de resultados; índice propio |
| Sin recursos de MC 26.3 preparados no hay proof in-game | Medio | Documentar `horizon-prepare`; CI opcional con caché de recursos; `native.py check` como puerta mínima |
| Revisión del registro (código nativo, confianza) | Medio | Código pequeño y legible, sin red ni escrituras fuera de `native_data_dir`/`config/`; README con alcance exacto |
| Expectativa de "JEI completo" | Alto | Este plan y `docs/jei-parity.md` hacen explícito qué llega en cada versión |
| Nombre/marca "JEI" | Bajo | Atribución MIT, "no afiliado a mezz ni a Horizon", id `jei`, aviso en README |
| Servidores dedicados no soportados | Medio | Documentarlo; el mod funciona en SP/LAN; transferencia server-side solo donde Horizon la permite |

## Decisiones tomadas (30-oct-2026)

1. **Primera entrega en el panel**: se publica v0.1 "JEI Core" con los hooks estables; la paridad de overlay
   llega en v1.0. *(Revisada el 07-nov: el overlay llega en 0.2/0.3.)*
2. **Upstream sí**: se enviarán PRs a Horizon para los hooks que falten, con el mismo estándar de calidad y
   tests que el resto del proyecto (cada hook con ejemplo mínimo compilable y prueba).
3. **Entorno de pruebas**: el MCP de Horizon está conectado (`opencode mcp add horizon -- .../horizon-launcher.exe mcp`).
   Nightly instalado y preparado: `nightly-2026-10-30-1da354b` (`1da354b67a509812c4141b67de2affd6de65c342`),
   recursos en `.../versions/nightly-2026-10-30-1da354b/.local/horizon-client/26.3`. Hay WSLg
   (`DISPLAY=:0`), así que `proof.py --offscreen` es viable en Linux.
4. **Repo**: `horizon-jei/` propio en este workspace; las comprobaciones usan un clone de `horizon-mods` en
   `/home/ender/horizon-tools/horizon-mods`.
5. **Idiomas**: inglés + español en v0.1; el resto en la Fase 4.

## Decisiones tomadas (07-nov-2026)

1. **0.2.0 no reescribe el panel** (AD-11): se conserva el panel del juego y se le añade overlay.
2. **0.2.0 sube a `hooks: 123`**: es la versión que necesitan `screen_render_post` (78),
   `mouse_button_pressed_pre` (80) y `mouse_dragged_pre` (123). Cualquier Nightly reciente la tiene (el del
   04-nov ya iba por 359).
3. **El shim pinneado desaparece**: los hooks de overlay son oficiales y aditivos.
4. **La API tipada se aplaza a 1.0**: `native.py` no empaqueta bundles con `[api]`.
5. **Clic en slots se queda en 0.3** (6.1), salvo que sobre tiempo en 0.2: es la única pieza que aún necesita
   geometría propia o un helper upstream.
6. **El RFC upstream se recorta** a los «huecos restantes»: nombres de stack, `ratón → slot`, panel en
   creativo/config, tablas de fuel/compost y `$tooltip`. Nada de lo ya existente se vuelve a pedir.

## Preguntas abiertas (menores)

- **Nombres de subtipos**: ¿se pide `native_stack_name`/`native_translate` upstream en 0.2.0 y se espera, o se
  publica con el id de efecto/encantamiento como etiqueta y se mejora en 0.3? (Recomendación: pedirlo ya y
  publicar con id si no llega a tiempo.)
- **Bookmarks de recetas** además de ingredientes: ¿entran en 0.2.0 o en 0.3.0? (Recomendación: 0.3.0; la
  barra de 0.2.0 es solo de ingredientes.)
- **Modo cheat**: ¿visible por defecto o detrás de `/jei config cheat true` con confirmación? (Recomendación:
  detrás de config, y nunca enviar nada al servidor.)
- **¿Se mantiene `id = "jei"`?** Ya está publicado con ese id en el registro, así que se mantiene.

## Referencias

- JEI: `JustEnoughItems/` (rama `26.3`, MIT), wiki de desarrolladores, `Common/src/api/...`.
- Horizon: `docs/native-mods.md` (catálogo por versión de API), `native-hooks.json` (432 hooks y 174 helpers
  con su firma), `crates/horizon-client/src/game/native_hooks.rs`, `crates/horizon-client/src/mod_api.rs`
  (`NativeScreenView`, `NativePanelEvent`, `NativeStack`), `native_panel.rs` (veto del creativo, línea 201),
  `native_panel_recipes.rs`.
- Nightly de referencia: **2026-11-07, `6560641`**, `NATIVE_HOOKS_API = 411`. Launcher 0.2.49; la API tipada
  necesita ≥ 0.2.37 y horizon-mixins ≥ 0.6.0 para `hook = "..."`.
- Documentación pública: <https://playhorizon.dev/docs/make-a-mod>, `/docs/mod-code`, `/docs/mod-manifest`,
  `/docs/content-mods`, `/docs/test-a-mod`, `/docs/publish-a-mod`.
- Registro: `horizon-mods/mods/jei.toml` (0.1.0 publicada), `MOD-POLICY.md` (licencias revisables).
- Referencia funcional: `horizon-mods/native/recipes` (bundle de ~900 líneas, hooks 2).
- Tooling: `horizon-mods/native/tools/native.py`, `native/tools/game.py`, `horizon-ctl`,
  `horizon-launcher native-build`.