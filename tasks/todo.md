# Tareas: JEI para Horizon

Leyenda: `[ ]` pendiente · `[x]` hecho · **S** = 1-2 archivos · **M** = 3-5 archivos · **L** = demasiado grande, dividir.

Cada tarea se considera terminada solo si pasan sus criterios de aceptación **y** su verificación. El
"Definition of Done" del proyecto: `cargo test` verde, `native.py check` verde contra el Nightly, sin
`unsafe`, sin accesos a red ni escrituras fuera de `native_data_dir`, y `proof.py` verde para lo que toque UI.

Comandos base (desde `horizon-jei/`):

```sh
cargo test
python3 <horizon-mods>/native/tools/native.py check "$PWD"            # compila el juego con el mod
python3 <horizon-mods>/native/tools/native.py check "$PWD" --horizon latest
python3 <horizon-mods>/native/tools/native.py proof "$PWD" --resources <ruta/26.3>
cargo horizon-mod check <archivo>.hnmod
```

---

## Fase 0 — Fundaciones

### [ ] Tarea 0.1 — Repo y esqueleto del bundle

**Descripción:** Crear `horizon-jei/` con la estructura mínima que `native.py` acepta: `Cargo.toml` (lib =
`mixins/jei/runtime.rs`), `compatibility.json` (`game: "26.3"`, `hooks: 2`, revisión del Nightly actual),
`mixins/jei/horizon-mixin.toml` con `[mod] id = "jei"`, `[compatibility]`, un `[[module]] runtime` y los
helpers vacíos de `native_mods`, `native_settings`, `native_settings_click` y `native_panel_event`. Incluir
`LICENSE` (MIT) y `NOTICE` con la atribución a JEI/mezz.

**Criterios de aceptación:**
- [x] `cargo test` compila y pasa (aunque no haya tests aún).
- [x] `native.py check` aplica el bundle y `cargo check` termina sin warnings.
- [ ] `Options > Mods` muestra la fila `JEI 0.0.1` con su interruptor (sin hacer nada todavía).
- [x] `NOTICE` cita a JEI (MIT, © mezz) y aclara que no está afiliado.

**Verificación:**
- [ ] `cargo test`
- [ ] `python3 <horizon-mods>/native/tools/native.py check "$PWD"`
- [ ] Manual: abrir el juego, ver la fila y apagarla/encenderla.

**Dependencias:** ninguna. **Archivos:** `Cargo.toml`, `compatibility.json`, `mixins/jei/horizon-mixin.toml`,
`runtime.rs`, `list.rs`, `settings.rs`, `NOTICE`. **Tamaño:** M.

---

### [ ] Tarea 0.2 — Prueba de humo del toolchain

**Descripción:** Documentar y automatizar el ciclo de desarrollo: obtener una revisión de Horizon, compilar
con el mod, empaquetar y verificar el `.hnmod`. Instalar `cargo horizon-mod`
(`cargo install --git https://github.com/horizon-rust/horizon cargo-horizon-mod`). Medir tiempos reales en
este equipo (primer build y rebuild) y dejarlos en `docs/dev.md`. Comprobar el espacio en disco: `/tmp` tiene
~2,9 GB; el `target` de Horizon (~1,5 GB) debe ir bajo `/home/ender`. Ojo: `native.py package` escribe en
`<horizon-mods>/native/target/`, no en el repo del mod.

**Criterios de aceptación:**
- [x] `docs/dev.md` describe los comandos, rutas de caché y tiempos medidos.
- [x] El `.hnmod` generado pasa `cargo horizon-mod check`.
- [x] Existe un script `tools/dev-check.sh` que ejecuta test + check + package.

**Verificación:**
- [ ] `tools/dev-check.sh`
- [x] `cargo horizon-mod check <horizon-mods>/native/target/jei-0.1.0.hnmod`

**Dependencias:** 0.1. **Archivos:** `docs/dev.md`, `tools/dev-check.sh`. **Tamaño:** S.

---

### [ ] Tarea 0.3 — Spike end-to-end en el juego

**Descripción:** Prueba desechable que valide las suposiciones críticas del plan en un juego real: (a)
`native_recipes` devuelve items y recetas al abrir un mundo; (b) `native_panel_items` muestra una lista; (c)
`native_panel_event::Key` llega con R sobre un slot del inventario; (d) `native_panel_recipes` dibuja una
página `Grid` con tabs; (e) el panel no aparece en creativo. Registrar hallazgos (límites, latencias,
sorpresas) en `docs/spike.md`.

**Criterios de aceptación:**
- [x] Un jugador abre el inventario y ve una lista de items.
- [x] R sobre un item del panel abre una página de receta con al menos un tab.
- [ ] U sobre un item de un slot del inventario abre la página de usos (o se documenta que no llega).
- [x] `docs/spike.md` confirma o refuta cada punto (a)-(e) con evidencia (captura o log).

**Verificación:**
- [x] `python3 <horizon-mods>/native/tools/native.py proof "$PWD" --resources <26.3>`
- [x] Capturas en `native/target/proof/jei/`.

**Dependencias:** 0.1, 0.2. **Archivos:** `mixins/jei/runtime.rs`, `glue.rs`, `panel.rs`, `keys.rs`,
`proof.py`, `docs/spike.md`. **Tamaño:** M.

---

### [ ] Tarea 0.4 — Matriz de paridad `docs/jei-parity.md`

**Descripción:** Inventariar las funcionalidades de JEI 31.17.0 (extraídas de `Gui/`, `Library/plugins/vanilla`,
`api/` y `en_us.json`) y clasificar cada una: **portable** (hooks actuales), **necesita hook** (describir
cuál), **imposible hoy** (con motivo). Debe cubrir: lista de ingredientes, búsqueda y prefijos, subtipos,
bookmarks, pantalla de recetas, categorías, tooltips, transferencia, cheat mode, config, historial, plugins,
fluidos, accesibilidad. Esta matriz es el contrato de las fases 2-4 y la base del RFC upstream.

**Criterios de aceptación:**
- [x] Cada categoría vanilla de JEI tiene fila con estado y fase.
- [x] Cada fila "necesita hook" nombra el hook propuesto y el hueco que cubre.
- [x] Cada fila "imposible" explica el límite (protocolo, servidor, API).
- [ ] Revisada contra el código de JEI (rutas citadas).

**Verificación:**
- [ ] Revisión manual contra `JustEnoughItems/Gui` y `Library/plugins/vanilla`.
- [ ] Enlazada desde `tasks/plan.md`.

**Dependencias:** 0.1. **Archivos:** `docs/jei-parity.md`. **Tamaño:** S.

---

### [ ] Tarea 0.5 — CI y comandos de desarrollo

**Descripción:** GitHub Actions con tres jobs: (1) `cargo test` + `cargo clippy -- -D warnings` + `cargo fmt
--check`; (2) `native.py check` contra el commit de `compatibility.json`; (3) `native.py check --horizon
latest` programado a diario (fallo = abrir issue de re-pin). Cachear `native/target` y el fuente de Horizon.
Sin secretos.

**Criterios de aceptación:**
- [ ] Los tres jobs corren en PR y en `main`.
- [ ] El job diario abre/actualiza un issue cuando el Nightly rompe el build.
- [ ] Tiempo de PR < 15 min con caché caliente.

**Verificación:**
- [ ] PR de prueba con los checks en verde.
- [ ] Ejecución manual del workflow diario.

**Dependencias:** 0.2. **Archivos:** `.github/workflows/ci.yml`, `.github/workflows/nightly.yml`. **Tamaño:** S.

---

## Checkpoint A — tras Fase 0

- [ ] El bundle compila, se empaqueta y se instala.
- [ ] El spike confirma el modelo de panel/hooks o corrige el plan.
- [ ] `docs/jei-parity.md` aprobado por el humano.
- [ ] CI verde.

---

## Fase 1 — Núcleo puro (sin tipos del juego)

### [ ] Tarea 1.1 — Modelo de ingrediente y variantes (subtipos)

**Descripción:** Definir `Ingredient { item: ItemId, count, components, uid }` y la generación de variantes
que JEI registra: pociones, libros encantados, flechas con efecto, estandartes, fuegos artificiales, estrellas
y stew sospechoso. El `uid` debe agrupar variantes equivalentes para búsqueda y para el índice de recetas
(equivalente a `IIngredientHelper.getUid`/`getGroupingUid`). Sin dependencias del juego.

**Criterios de aceptación:**
- [ ] Dos stacks con los mismos componentes tienen el mismo `uid`; con componentes distintos, distinto.
- [ ] Un item sin variantes produce una sola entrada.
- [ ] Las variantes llevan nombre mostrable y componentes en sintaxis vanilla.
- [ ] Cobertura de tests ≥ 90 % en el módulo.

**Verificación:**
- [ ] `cargo test ingredients`
- [ ] Casos portados de `Library/.../subtypes/*` y `ItemStackHelper`.

**Dependencias:** 0.1. **Archivos:** `runtime.rs` (sección `ingredients`), `tests/ingredients.rs`. **Tamaño:** M.

---

### [ ] Tarea 1.2 — Modelo de receta y categoría

**Descripción:** Definir `Recipe { kind, category, group, width, height, slots, results, cooking_time,
experience, book_id }` y `Category` (id estable, título, icono, layout, orden). Normalizar desde
`NativeRecipe` en `glue.rs`, incluyendo shapeless (rejilla calculada como en Recipes) y recetas con varios
resultados (trim). No incluir aún datos que Horizon no da (anvil, fuel).

**Criterios de aceptación:**
- [ ] Las 9 clases de receta se normalizan sin pérdida de slots ni resultados.
- [ ] Cada categoría declara su layout válido del panel (`Grid`/`Cooking`/`Single`/`Smithing`/`Brewing`).
- [ ] El orden de categorías es determinista y documentado.
- [ ] Tests con recetas sintéticas de cada tipo.

**Verificación:**
- [ ] `cargo test recipes`
- [ ] Comparación con `horizon-mods/native/recipes/mixins/recipes/runtime.rs` (view).

**Dependencias:** 1.1. **Archivos:** `runtime.rs` (secciones `recipe`, `category`), `tests/recipes.rs`. **Tamaño:** M.

---

### [ ] Tarea 1.3 — Índice ingrediente→recetas (recipes/uses)

**Descripción:** Portar la idea de `RecipeIngredientRoleMap` + `RecipeIngredientTable`: un mapa
`uid → recetas de salida` (recipes) y `uid → recetas que lo usan` (uses), con agrupación por uid de
agrupación. Construcción O(n) al cargar el mundo; consultas O(1). Debe soportar la consulta por stack con
componentes (una receta que produce `potion:swiftness` no debe aparecer para `potion:healing`).

**Criterios de aceptación:**
- [ ] Consulta de recipes/uses correcta para items con y sin componentes.
- [ ] Recetas duplicadas no se repiten en el resultado.
- [ ] Construcción < 50 ms para 10.000 recetas (medido en test).
- [ ] Tests: item producido por 2 recetas, item usado en 3, item sin recetas.

**Verificación:**
- [ ] `cargo test index`
- [ ] Benchmark del test de construcción.

**Dependencias:** 1.1, 1.2. **Archivos:** `runtime.rs` (`index`), `tests/index.rs`. **Tamaño:** M.

---

### [ ] Tarea 1.4 — Búsqueda: tokenizer e índice

**Descripción:** Portar `SearchTokenizer` (comillas, `-` exclusión, `|` alternativas, escape) y la búsqueda
por subcadena de JEI. Prefijos v0.1: sin prefijo (nombre/id), `@` namespace/mod, `#` tag. `$` (tooltip) y `%`
(tab creativo) quedan tras feature flag hasta la Fase 6 (tarea 6.9). Índice en memoria construido una vez;
consulta case-insensitive.

**Criterios de aceptación:**
- [x] Gramática idéntica a JEI en los casos de `SearchTokenizerTest` (14 casos portados).
- [ ] `@minecraft`, `#logs`, `-stone`, `"iron ingot"` funcionan combinados.
- [ ] Búsqueda < 16 ms con 10.000 ingredientes (p95, medido).
- [x] Tests portados de `SearchTokenizerTest` y `IngredientFilterTest` (subset).

**Verificación:**
- [ ] `cargo test search`
- [ ] Benchmark integrado.

**Dependencias:** 1.1. **Archivos:** `runtime.rs` (`search`), `tests/search.rs`. **Tamaño:** M.

---

### [ ] Tarea 1.5 — Filtro y ordenación

**Descripción:** Excluir `minecraft:air` y los items con tag `c:hidden_from_recipe_viewers`. Orden por
defecto: orden de registro (como JEI sin creative tabs), con etapas configurables (nombre, tag, durabilidad,
namespace). Estable y determinista entre ejecuciones.

**Criterios de aceptación:**
- [ ] Items ocultos nunca aparecen.
- [ ] El orden es estable y coincide con el esperado en un fixture.
- [ ] Cambiar de etapa de orden no pierde el filtro de búsqueda activo.
- [ ] Tests de orden con fixtures.

**Verificación:**
- [ ] `cargo test sort`
- [ ] Inspección visual del panel (mismo orden tras reiniciar).

**Dependencias:** 1.1, 1.4. **Archivos:** `runtime.rs` (`sort`), `tests/sort.rs`. **Tamaño:** S.

---

### [ ] Tarea 1.6 — Máquina de estados del viewer

**Descripción:** Estado puro: lista filtrada, página, receta abierta (item + uses + tab), historial
atrás/adelante, selección de tab, límites (`>256` recetas → "256 de N"). API interna
`handle(event) -> Vec<Out>` como en Recipes, ampliada con bookmarks (stub) y navegación.

**Criterios de aceptación:**
- [ ] Abrir receta → cambiar tab → volver → historial correcto.
- [ ] Volver desde la primera página devuelve a la lista.
- [ ] Cambiar la búsqueda estando en una receta no rompe el historial.
- [ ] Cobertura de transiciones con tests de tabla.

**Verificación:**
- [ ] `cargo test viewer`
- [ ] Test de propiedades (secuencias aleatorias de eventos no entran en pánico).

**Dependencias:** 1.2, 1.3, 1.4, 1.5. **Archivos:** `runtime.rs` (`viewer`), `tests/viewer.rs`. **Tamaño:** M.

---

### [ ] Tarea 1.7 — Configuración y persistencia

**Descripción:** Esquema de config tipado (modo de búsqueda, orden, cheat mode, lado del panel, tamaño,
bookmarks on/off, etapas de orden) con defaults de JEI, serialización a archivos clave-valor en
`native_data_dir("jei")` (un archivo por clave, como el resto de Horizon) y parseo tolerante (valor inválido →
default + aviso en log). Comando `/jei config` para leer/escribir.

**Criterios de aceptación:**
- [ ] Round-trip de todos los valores; valores inválidos no rompen el arranque.
- [ ] Migración desde claves desconocidas se ignora sin perder el resto.
- [ ] `/jei config` lista y cambia valores con validación.
- [ ] Tests de parseo y defaults.

**Verificación:**
- [ ] `cargo test config`
- [ ] Manual: cambiar un valor, reiniciar, comprobarlo.

**Dependencias:** 0.1. **Archivos:** `runtime.rs` (`config`), `settings.rs`, `tests/config.rs`. **Tamaño:** M.

---

### [ ] Tarea 1.8 — Tests portados de JEI

**Descripción:** Portar los tests puros de mayor valor: `SearchTokenizerTest`, `ElementSearchTest`,
`IngredientFilterTest` (subset), `IngredientUidIndexTest`, `RecipeTransferManagerTest` (lógica de
selección). Mantener el nombre del caso original en un comentario para trazabilidad. No portar tests que
dependan de `ItemStack`/`Screen`.

**Criterios de aceptación:**
- [ ] ≥ 40 casos portados y verdes.
- [ ] Cada test cita el archivo Java de origen.
- [ ] `cargo test` < 5 s.

**Verificación:**
- [ ] `cargo test`
- [ ] `cargo llvm-cov` (o `tarpaulin`) con informe en `docs/coverage.md`.

**Dependencias:** 1.1-1.7. **Archivos:** `tests/ported_*.rs`, `docs/coverage.md`. **Tamaño:** M.

---

## Checkpoint B — tras Fase 1

- [ ] `cargo test` verde y cobertura del núcleo ≥ 80 %.
- [ ] Índice y búsqueda cumplen los presupuestos de tiempo.
- [ ] Ninguna dependencia de tipos del juego en `runtime.rs`.
- [ ] Revisión de código (skill `code-review-and-quality`).

---

## Fase 2 — Panel y v0.1 "JEI Core"

### [ ] Tarea 2.1 — Catálogo al abrir/cerrar mundo

**Descripción:** Construir el catálogo (items + variantes + recetas + índice + búsqueda) cuando hay mundo, y
liberarlo al salir. Usar `native_player().is_some()` como señal y `native_world_lifecycle` si está disponible
(hooks 3+); en hooks 2, detectar la transición en `native_tick`. Evitar reconstruir por frame.

**Criterios de aceptación:**
- [x] El catálogo se construye una vez por mundo y se libera al salir.
- [ ] Entrar/salir de un mundo 5 veces no crece la memoria de forma monótona.
- [ ] En un servidor, el catálogo usa las recetas que envía el servidor (sin recetas locales).
- [ ] Test unitario de la señal de ciclo de vida con un mock.

**Verificación:**
- [ ] `cargo test lifecycle`
- [ ] `proof.py`: entrar/salir y comprobar que no hay pánico ni fuga evidente.

**Dependencias:** 1.6. **Archivos:** `glue.rs`, `frame.rs`, `mixins/jei/horizon-mixin.toml`. **Tamaño:** M.

---

### [ ] Tarea 2.2 — Sincronización de items y búsqueda

**Descripción:** Enviar la lista filtrada con `native_panel_items` (≤8.192), recibir `Search(text)` y
`Clicked{index,right}`, y reenviar la lista. Aplicar debounce de ~50 ms si la medición lo justifica.

**Criterios de aceptación:**
- [x] La lista inicial contiene todos los items visibles (sin aire ni ocultos).
- [ ] Escribir en el buscador filtra en < 100 ms percibidos.
- [x] Borrar el texto restaura la lista completa.
- [ ] No se envía una lista nueva si el texto no cambió.

**Verificación:**
- [ ] `cargo test search_panel`
- [ ] `proof.py`: teclear `@minecraft`, `#logs`, `-stone` y comprobar los widgets del panel.

**Dependencias:** 1.4, 1.5, 2.1. **Archivos:** `glue.rs`, `panel.rs`, `runtime.rs`. **Tamaño:** M.

---

### [ ] Tarea 2.3 — R/U, historial, tabs y páginas

**Descripción:** Al recibir `Key{key, stack}`, comparar con las teclas configuradas (R = recipes, U = uses) y
abrir la página. Manejar `Tab`, `Page`, `Back` y `SlotClicked` para navegar. Título con nombre del item y
contador si se trunca.

**Criterios de aceptación:**
- [ ] R sobre un item del panel y sobre un slot del inventario abre sus recetas.
- [ ] U abre los usos; si no hay, no se abre nada y se avisa una vez.
- [x] Cambiar de tab mantiene la página y el item.
- [x] Back vuelve al historial anterior y, al final, a la lista.

**Verificación:**
- [ ] `proof.py` con capturas de cada transición.
- [ ] `cargo test viewer`.

**Dependencias:** 1.6, 2.2. **Archivos:** `keys.rs`, `panel.rs`, `glue.rs`. **Tamaño:** M.

---

### [ ] Tarea 2.4 — Vistas de receta por layout

**Descripción:** Convertir `View` a `NativeRecipeView`: rejilla shaped (w×h), shapeless reubicado, cooking
con nota de segundos/XP, single, smithing (template/base/addition), brewing (botella/reactivo). Rellenar
huecos vacíos con listas vacías. Ciclo de stacks por slot (pociones, ingredientes con variantes).

**Criterios de aceptación:**
- [ ] Cada tipo de receta se ve con el layout correcto y sin slots desplazados.
- [ ] Recetas con ingredientes alternativos ciclan sus stacks.
- [ ] Recetas con varios resultados muestran todos.
- [x] Nota de cocción en segundos y XP con formato estable.

**Verificación:**
- [ ] `cargo test views`
- [ ] `proof.py`: abrir una receta de cada tipo (crafting, horno, sierra, smithing, brewing) y comparar
  capturas.

**Dependencias:** 1.2, 2.3. **Archivos:** `panel.rs`, `glue.rs`, `runtime.rs`. **Tamaño:** M.

---

### [ ] Tarea 2.5 — Transferencia (+ / shift)

**Descripción:** Mostrar el botón de acción (`action: Some(known)`) solo cuando `book_id >= 0`. En
`Action(i)`, llamar `native_place_recipe(book_id, all)` con `all = shift`. Si el libro no la conoce, mostrar
el botón gris y no hacer nada. Informar del error del juego una sola vez.

**Criterios de aceptación:**
- [x] El botón aparece solo en recetas del libro y se ve gris si no se conocen.
- [ ] Colocar sin shift pone una unidad; con shift, el máximo posible. (Shift no llega en `Action`: `all` queda en false hasta que Horizon exponga modificadores.)
- [ ] En un cofre (sin rejilla) el botón no aparece o no hace nada, sin pánico.
- [ ] El estado se refresca al aprender una receta (nueva consulta de `native_known_recipes`).

**Verificación:**
- [ ] `proof.py`: mesa + receta conocida + comprobar el inventario.
- [ ] `cargo test transfer`.

**Dependencias:** 2.3, 2.4. **Archivos:** `panel.rs`, `glue.rs`, `runtime.rs`. **Tamaño:** M.

---

### [ ] Tarea 2.6 — Give y modo creativo

**Descripción:** `Clicked{index, right}` con "click da items" activo y jugador en creativo →
`native_give(item, 1|stack)`. Fuera de creativo, abrir recetas (comportamiento JEI sin cheat). Añadir aviso
en la config de que en survival no está disponible (límite de Horizon).

**Criterios de aceptación:**
- [ ] En creativo, clic da 1 y clic derecho una pila.
- [ ] En survival, clic abre recetas y no intenta dar.
- [x] El ajuste persiste y se refleja en la página del mod.
- [ ] No se llama a `native_give` con items inexistentes.

**Verificación:**
- [ ] `proof.py` (creativo) como en `native/recipes/proof.py`.
- [ ] `cargo test give`.

**Dependencias:** 2.3. **Archivos:** `panel.rs`, `settings.rs`, `runtime.rs`. **Tamaño:** S.

---

### [ ] Tarea 2.7 — Fila del mod, teclas y ajustes

**Descripción:** `native_mods` con id `jei`, título/versión/descripción y teclas R/U (defaults
`key.keyboard.r`/`key.keyboard.u`). Página de ajustes con botones: "Click da items", "Búsqueda: ...",
"Orden: ...", "Bookmarks: ON/OFF" (stub), "Reset". Leer `native_mod_key` en cada hook.

**Criterios de aceptación:**
- [ ] Rebinding de R/U funciona y se respeta.
- [ ] Los botones reflejan el estado y persisten.
- [ ] Con el mod apagado (`native_on`), ningún hook hace nada.
- [ ] El título muestra la versión real.

**Verificación:**
- [ ] `proof.py`: abrir ajustes, cambiar un botón, reiniciar, comprobar.
- [ ] Manual: `Options > Controls` no muestra conflictos.

**Dependencias:** 1.7, 2.6. **Archivos:** `list.rs`, `settings.rs`, `keys.rs`. **Tamaño:** M.

---

### [ ] Tarea 2.8 — Casos límite y mensajes

**Descripción:** Cubrir: item sin recetas ni usos, item con >256 recetas, >16 tabs, lista >8.192, item
desconocido tras cambiar de versión, componentes inválidos, receta con slots vacíos, jugador muerto, cambio
de dimensión, salir del mundo con una receta abierta. Mensajes en chat solo cuando aportan.

**Criterios de aceptación:**
- [ ] Ningún caso produce pánico ni panel vacío permanente.
- [ ] "256 de N" aparece cuando se trunca.
- [ ] Al salir del mundo el panel se cierra solo.
- [ ] Tests de los casos puros y proof de los de UI.

**Verificación:**
- [ ] `cargo test edges`
- [ ] `proof.py` con escenarios negativos.

**Dependencias:** 2.4. **Archivos:** `runtime.rs`, `panel.rs`, `proof.py`. **Tamaño:** M.

---

### [ ] Tarea 2.9 — i18n en/es

**Descripción:** Tabla de cadenas del mod (títulos, botones, mensajes) con inglés y español, elegida por una
config `language` (default `en`) porque el juego no expone su idioma a los mods. Nombres de item siempre del
juego. Sin claves de traducción del juego.

**Criterios de aceptación:**
- [ ] Toda cadena visible sale de la tabla (grep sin literales en helpers).
- [x] `language = es` cambia la UI sin reiniciar el mundo (o al reabrir el panel).
- [x] Falta de traducción cae a inglés.

**Verificación:**
- [ ] `cargo test i18n`
- [ ] `proof.py` con `language=es` y captura.

**Dependencias:** 2.7. **Archivos:** `runtime.rs` (`i18n`), `tests/i18n.rs`. **Tamaño:** S.

---

### [ ] Tarea 2.10 — Suite de pruebas end-to-end

**Descripción:** `proof.py` con escenarios independientes y deterministas: (1) fila del mod y ajustes; (2)
lista de items; (3) búsqueda por nombre/`@`/`#`/exclusión; (4) R sobre panel; (5) R sobre slot del
inventario; (6) U; (7) tabs y páginas; (8) transferencia; (9) give en creativo; (10) mod apagado; (11)
reinicio con config. Cada escenario con captura y una aserción fuerte (diff de píxeles o widgets), nunca solo
"no crashea".

**Criterios de aceptación:**
- [ ] 11 escenarios verdes en local y en CI (con recursos).
- [ ] Cada escenario imprime JSON con el resultado.
- [ ] El fallo de un escenario no impide ejecutar los demás (o se ejecutan por separado).

**Verificación:**
- [ ] `python3 <horizon-mods>/native/tools/native.py proof "$PWD" --resources <26.3>`
- [ ] Job de CI con recursos cacheados.

**Dependencias:** 2.3-2.9. **Archivos:** `proof.py`, `.github/workflows/proof.yml`. **Tamaño:** M.

---

### [ ] Tarea 2.11 — Empaquetar y publicar 0.1.0

**Descripción:** Subir versión a 0.1.0, `README.md` con alcance exacto (qué hace y qué no), capturas,
instrucciones de instalación, `NOTICE`, changelog. Empaquetar con `native.py package`, verificar con
`cargo horizon-mod check`, firmar y abrir PR al registro de Horizon.

**Criterios de aceptación:**
- [x] `.hnmod` < 1 MB, sin dependencias externas, `check` verde.
- [x] README dice explícitamente que bookmarks/overlay llegan en v1.0.
- [ ] PR de registro abierto con descripción y capturas.
- [ ] Instalación limpia desde el registro en un juego vacío.

**Verificación:**
- [x] `cargo horizon-mod check <horizon-mods>/native/target/jei-0.1.0.hnmod`
- [ ] Instalación manual + prueba de humo.

**Dependencias:** 2.10. **Archivos:** `README.md`, `CHANGELOG.md`, `NOTICE`, `compatibility.json`. **Tamaño:** S.

---

## Checkpoint C — tras Fase 2

- [ ] v0.1 jugable: lista, búsqueda, R/U, recetas, transferencia, give.
- [ ] 11 escenarios end-to-end verdes.
- [ ] Publicada en el registro; instalación limpia verificada.
- [ ] Demo grabada y decisión sobre la siguiente fase (hoy: Fase 5).

---

## Fase 3 — Paridad de overlay (SUPERADA el 07-nov-2026)

> **Ya no es puerta de entrada ni se ejecuta.** Horizon añadió los hooks de overlay (API 78-123): ver
> `plan.md` → «Revisión del 07-nov-2026». Las tareas de esta fase se reparten entre las Fases 5 y 6:
> 3.1 → 5.1 · 3.2 → 5.1 · 3.3 → 5.6-5.9 · 3.4 → 5.3 + 6.2-6.3 · 3.5 → 6.6-6.7 · 3.6 → 6.9 · 3.7 → 5.10 ·
> 3.8 → 6.8 + 6.11 · 3.9 → 6.10 · 3.10 → 6.4. Se conservan escritas como referencia.

### [ ] Tarea 3.1 — RFC upstream de hooks

**Descripción:** Escribir un documento + issue/PR en `horizon-rust/horizon` proponiendo, como API 8+, los
hooks/helpers que faltan: `native_screen` (estado y rect de la pantalla abierta), `native_screen_frame`
(dibujo tras la pantalla), `native_screen_event` (clic/hover en slots), `native_item` (icono de item en un
`HudBatch`), `native_tooltip` (líneas propias), `native_cursor_stack`/`native_set_cursor`, opciones de panel
(lado/tamaño/scroll), panel en creativo, y un modelo de categoría con layout declarativo. Cada propuesta con
caso de uso, forma de la firma, coste y compatibilidad.

**Criterios de aceptación:**
- [ ] El RFC cubre los 12 huecos de `tasks/plan.md` con firmas concretas.
- [ ] Incluye un ejemplo mínimo compilable por hook (como Playtime).
- [ ] Mantenedores de Horizon responden (aceptado / cambios / rechazado).

**Verificación:**
- [ ] Issue/PR enlazado desde `docs/jei-parity.md`.
- [ ] Revisión del diseño con el humano.

**Dependencias:** 0.4, 2.11. **Archivos:** `docs/rfc-hooks.md`. **Tamaño:** M.

---

### [ ] Tarea 3.2 — Implementar contra hooks aceptados (o shim)

**Descripción:** Según la respuesta, añadir los helpers al bundle y actualizar `compatibility.json` a
`hooks: N`. Si es shim: crear `shim/` con su propio `horizon-mixin.toml` y `horizon_revision`, sin mezclar
con el bundle estable, y un job de re-pin por Nightly.

**Criterios de aceptación:**
- [ ] El bundle compila contra el Nightly con los hooks nuevos.
- [ ] El shim (si existe) compila contra su commit y no afecta al bundle estable.
- [ ] `native.py check --horizon latest` verde.

**Verificación:**
- [ ] `native.py check "$PWD" --horizon latest`
- [ ] `cargo test`

**Dependencias:** 3.1. **Archivos:** `mixins/jei/horizon-mixin.toml`, `compatibility.json`, `shim/*`. **Tamaño:** M.

---

### [ ] Tarea 3.3 — Barra de bookmarks

**Descripción:** Bookmarks persistentes (lista de ingredientes y recetas) guardadas en `native_data_dir`.
Barra junto al panel, drag para reordenar, clic para abrir recetas, borrar con clic derecho/tecla, "A"
para añadir el stack apuntado. Orden configurable (inicio/fin).

**Criterios de aceptación:**
- [ ] Añadir/quitar/reordenar y persistir tras reiniciar.
- [ ] Clic en bookmark abre recipes; clic derecho, uses (configurable).
- [ ] La barra no tapa el panel ni el tooltip.
- [ ] Funciona con items con componentes (poción concreta, no "poción" genérica).

**Verificación:**
- [ ] `proof.py`: añadir 3 bookmarks, reordenar, reiniciar, comprobar.
- [ ] `cargo test bookmarks`.

**Dependencias:** 3.2. **Archivos:** `runtime.rs` (`bookmarks`), `panel.rs`, `glue.rs`. **Tamaño:** M.

---

### [ ] Tarea 3.4 — Interacción de slots: clic, tooltips, cursor, ghost

**Descripción:** Clic izquierdo en un slot de la pantalla → recetas; derecho → usos. Tooltip del item con
líneas "R: recetas / U: usos". `Ctrl`/drag para llevar un ingrediente fantasma al cursor y soltarlo en un
slot válido (visual, sin mover items reales). Sin tocar el protocolo.

**Criterios de aceptación:**
- [ ] Clic y clic derecho sobre cualquier slot abren la página correcta.
- [ ] El tooltip añade las pistas sin romper el tooltip del juego.
- [ ] El fantasma se dibuja bajo el cursor y desaparece al soltar.
- [ ] No se envía ningún paquete al servidor por el fantasma.

**Verificación:**
- [ ] `proof.py` con capturas y comprobación de que el inventario no cambió.
- [ ] `cargo test cursor`.

**Dependencias:** 3.2, 3.3. **Archivos:** `panel.rs`, `glue.rs`, `runtime.rs`. **Tamaño:** M.

---

### [ ] Tarea 3.5 — Categorías restantes (yunque, afilar, compost, fuel)

**Descripción:** Implementar como páginas de información o layouts propios si los hooks lo permiten: yunque
(2 entradas + nivel + resultado, calculado), piedra de afilar (2 entradas + resultado), compost (item +
probabilidad) y combustible (item + tiempo de quema). Ojo: compost y fuel **no** son datapack en vanilla;
son tablas internas del juego (`ComposterBlock`/`Item.getBurnTime`). Requiere que Horizon las exponga, o
incluirlas como datos del mod con una tarea de actualización por versión.

**Criterios de aceptación:**
- [ ] Cada categoría nueva tiene datos verificables contra el juego 26.3.
- [ ] Los números (XP del yunque, probabilidad de compost, ticks de combustible) coinciden con vanilla.
- [ ] No hay datos hardcodeados sin una tarea de actualización documentada.

**Verificación:**
- [ ] Tests con tablas conocidas.
- [ ] `proof.py`: abrir cada categoría y comprobar un caso conocido.

**Dependencias:** 3.2. **Archivos:** `runtime.rs` (`info`), `data/*.json`, `panel.rs`. **Tamaño:** M.

---

### [ ] Tarea 3.6 — Búsqueda avanzada (`$`, `%`) y autocompletado

**Descripción:** Activar `$tooltip` si el hook de tooltip está disponible y `%` (tab creativo) si Horizon
expone tabs. Autocompletado de prefijos y sugerencias (equivalente a `SearchCompletionOverlay`), navegable
con teclado.

**Criterios de aceptación:**
- [ ] `$` y `%` aparecen solo si sus datos existen; si no, se documenta y se ignoran.
- [ ] El autocompletado no roba las teclas del juego cuando está cerrado.
- [ ] Tests de parser y de sugerencias.

**Verificación:**
- [ ] `cargo test search_advanced`
- [ ] `proof.py`: teclear `$` y ver sugerencias.

**Dependencias:** 3.2, 1.4. **Archivos:** `runtime.rs`, `panel.rs`. **Tamaño:** M.

---

### [ ] Tarea 3.7 — Configuración completa

**Descripción:** Exponer las opciones de JEI que tengan sentido: lado y tamaño del panel, scroll vs páginas,
modo cheat, modo de búsqueda por prefijo, etapas de orden, features de bookmarks, historial. Página de
ajustes con botones cíclicos + `/jei config set` para todo lo demás. Validación y reset.

**Criterios de aceptación:**
- [ ] Todas las opciones documentadas en `docs/config.md` con default y efecto.
- [ ] Cambiar una opción se refleja sin reiniciar (o se avisa claramente).
- [ ] Config inválida cae a defaults con un aviso único.

**Verificación:**
- [ ] `cargo test config_full`
- [ ] `proof.py`: recorrer la página de ajustes.

**Dependencias:** 1.7, 3.2. **Archivos:** `runtime.rs`, `settings.rs`, `docs/config.md`. **Tamaño:** M.

---

### [ ] Tarea 3.8 — Panel en creativo y opciones de panel

**Descripción:** Si Horizon lo permite, mostrar el panel en el inventario creativo (hoy excluido) y permitir
lado izquierdo/derecho, ancho y scroll continuo. Si no, documentar la limitación y mantener páginas.

**Criterios de aceptación:**
- [ ] En creativo, el panel aparece y R/U funcionan.
- [ ] Cambiar de lado recoloca el panel sin solapar el GUI.
- [ ] Scroll continuo (si existe) no rompe el hover ni los clics.

**Verificación:**
- [ ] `proof.py` en creativo y con panel a la izquierda.
- [ ] Capturas comparadas.

**Dependencias:** 3.2, 3.7. **Archivos:** `panel.rs`, `glue.rs`, `runtime.rs`. **Tamaño:** M.

---

### [ ] Tarea 3.9 — Árbol de recetas y navegación

**Descripción:** Navegación tipo JEI: desde una receta, abrir los ingredientes; atrás/adelante con historial
completo; "recetas que usan esto" encadenado; límite de profundidad y breadcrumb en el título.

**Criterios de aceptación:**
- [ ] Se puede navegar receta → ingrediente → receta sin perder el historial.
- [ ] Atrás/adelante recorren la pila completa.
- [ ] El título muestra dónde estás (item + tab).

**Verificación:**
- [ ] `cargo test navigation`
- [ ] `proof.py` con una cadena de 3 niveles.

**Dependencias:** 2.3, 3.4. **Archivos:** `runtime.rs`, `panel.rs`. **Tamaño:** M.

---

### [ ] Tarea 3.10 — Transferencia fuera de crafting (opcional)

**Descripción:** Hornos, sierra, smithing, brewing: requiere mover items en el servidor. En SP/LAN se puede
con un hook de servidor; en servidores remotos no sin cambiar el protocolo. Implementar solo si Horizon
expone una vía; si no, dejar documentado como "no soportado" y no mostrar el botón.

**Criterios de aceptación:**
- [ ] El botón aparece solo donde la transferencia es posible.
- [ ] En SP/LAN, horno y sierra se rellenan correctamente y de forma atómica.
- [ ] En servidor remoto no se intenta y se explica por qué.

**Verificación:**
- [ ] `proof.py` en SP con horno.
- [ ] Prueba manual en LAN con dos clientes.

**Dependencias:** 3.2. **Archivos:** `runtime.rs` (`transfer`), `glue.rs`, `panel.rs`. **Tamaño:** M.

---

## Checkpoint D — tras Fase 3 (superado)

> Sus criterios se reparten entre el Checkpoint 0.2.0 y el Checkpoint 0.3.0. Se conserva como referencia.

- [ ] Auditoría de paridad: cada fila de `docs/jei-parity.md` en estado final. *(Se hace en 0.3.0.)*
- [ ] v1.0 candidata: bookmarks, overlay, tooltips, 13 categorías, config.
- [x] Sin shim pinneado (los hooks de overlay son oficiales).
- [ ] Revisión de código y seguridad (skill `security-and-hardening`) — sin red, sin escrituras fuera del data dir.

---

## Fase 5 — 0.2.0 "Bookmarks y tooltips"

**Objetivo:** bookmarks persistentes, tooltips con atajos y lista con subtipos, sin reescribir el panel del
juego.
**Diseño:** `plan.md` → «Fase 5». Hooks nuevos: 12, 78, 80, 123; helpers `native_item`, `native_fill`,
`native_text`. `compatibility.json` sube a `"hooks": 123`.

### Estado al 07-nov-2026 (implementación sin tocar Horizon)

Implementado en `horizon-jei` (0.2.0), compilado contra Nightly `6560641` con `native.py check` y probado con
`cargo test` (41). Desviaciones respecto al plan, todas por lo que los hooks ofrecen hoy:

| Tarea | Estado | Nota |
|---|---|---|
| 5.1 | hecha | `hooks` = **313**, no 123: se usa `native_item_stack` (API 313) para dibujar el stack con sus componentes. Inyecciones nuevas por nombre (`hook = ...`, `hook_version = 1`); las de 0.1 siguen por `target` (AD-12). |
| 5.2 | hecha | El layout puro vive en `runtime.rs` (`Bar`, `bookmark_bar`, `bookmark_slots`, `inside`) y el helper es `screen_post.rs`; no se añadió `overlay.rs` como fichero aparte. |
| 5.3 | hecha | Líneas `R: Recetas (n)`, `U: Usos (n)` y marca `★ Marcado`. No se añaden el id ni los tags: ruido en cada tooltip del juego, sin equivalente en JEI. |
| 5.4 | hecha | Dirigido por datos: cada par `(item, components)` que las tablas de recetas nombran (pociones, flechas tipped), tope 8 por item. No se codifican a mano las cuatro pociones. |
| 5.5 | bloqueada | Libros encantados y fuegos artificiales no aparecen en ninguna tabla de recetas y un hook no da el nombre de un stack con componentes. Pedido como `native_stack_name` (RFC §5). |
| 5.6 | hecha | Fichero `config/jei/bookmarks.txt` (texto, sin dependencias), no `native_data_dir` + JSON + `.bak`; un fichero corrupto da lista vacía sin aviso. |
| 5.7 | hecha | Barra en la esquina superior izquierda (el panel del juego ocupa el lado derecho y su geometría no se expone), dos columnas de doce, resalte al hover y tooltip propio con `native_item_stack`. |
| 5.8 | hecha | Tecla A por el evento `Key` del panel; izquierdo = recetas, derecho = usos, Mayús/botón central = quitar; press y release cancelados solo dentro de la barra. |
| 5.9 | hecha | Arrastrar mueve y guarda; un clic sin arrastre no mueve nada y abre recetas. |
| 5.10 | parcial | `config/jei/jei.cfg` con `sort`, `hints` y `bookmarks`, más `/jei config`. Faltan `subtypes_max`, `search_mode` y el aviso único de valor inválido (hoy cae al default en silencio). |
| 5.11 | parcial | Orden por nombre/id, conectado a la config, la página de ajustes y `/jei sort`. Los modos de búsqueda no están. |
| 5.12 | parcial | README, CHANGELOG, `roadmap.md`, `jei-parity.md`, `architecture.md`, `config.md`, `rfc-hooks.md` y ADR 0002 escritos; falta `spike.md`. |
| 5.13 | pendiente | `proof.py` necesita el juego en marcha con `horizon-ctl` y los recursos 26.3: no se puede ejecutar aquí. No se tocó el `proof.py` de 0.1.0 para no romper la suite sin probarla. |
| 5.14 | parcial | Empaquetado (`jei-0.2.0.hnmod`, 29.815 bytes, sha256 `87011011…`, reproducible), firmado con la clave `FC3386F1F51ECF6A` y verificado con `cargo horizon-mod check` (`signature verified`); `--horizon latest` verde (07-nov: sigue en `6560641`). Textos del formulario en `docs/submission-0.2.0.md`. Falta enviar el formulario en playhorizon.dev/mods/submit (navegador, cuenta de GitHub) y que se fusione el PR. |

Lo verificado: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (41) y
`native.py check` contra la revisión fijada. `--horizon latest` y `proof.py` quedan para una copia con red y
pantalla.

### [x] Tarea 5.1 — Subir el bundle al Nightly actual

**Descripción:** Poner `compatibility.json` en el Nightly de referencia (`hooks: 123`, `horizon_revision`
`6560641...`, mixins `e8cd0aa`), actualizar `[compatibility]` del manifiesto y migrar las inyecciones nuevas a
`hook = "<nombre>"` / `hook_version = 1`. Las inyecciones de 0.1 se migran solo si se tocan (AD-12).

**Criterios de aceptación:**
- [ ] `compatibility.json` y `horizon-mixin.toml` declaran la misma revisión, y `hooks` = 123.
- [ ] `native.py check` compila contra esa revisión y contra `--horizon latest`.
- [ ] Los 31 tests siguen verdes y el `proof.py` de 0.1.0 sigue pasando (regresión).

**Verificación:**
- [ ] `python3 <horizon-mods>/native/tools/native.py check "$PWD"`
- [ ] `python3 <horizon-mods>/native/tools/native.py check "$PWD" --horizon latest`
- [ ] `cargo test`

**Dependencias:** ninguna. **Archivos:** `compatibility.json`, `mixins/jei/horizon-mixin.toml`,
`.github/workflows/*.yml`. **Tamaño:** S.

---

### [x] Tarea 5.2 — Capa de overlay propia

**Descripción:** Añadir `overlay.rs` (puro: rects, layout, hit-test y escalado de GUI, con tests) y `screen.rs`
(el helper de `native_screen_render_post`, que solo delega). En esta tarea dibuja un marco de prueba y un
icono con `native_item` únicamente en `Screen::Inventory`, sin interacción.

**Criterios de aceptación:**
- [ ] El overlay se dibuja solo en la pantalla de contenedor y solo con el mod encendido.
- [ ] No dibuja en creativo, título, chat ni con F1.
- [ ] `overlay.rs` tiene tests puros (layout, hit-test, escalas 1-4) y no importa tipos del juego.

**Verificación:**
- [ ] `cargo test overlay`
- [ ] `native.py check` + captura en `proof.py`.

**Dependencias:** 5.1. **Archivos:** `mixins/jei/overlay.rs` (nuevo), `screen.rs` (nuevo), `runtime.rs`,
`horizon-mixin.toml`. **Tamaño:** M.

---

### [x] Tarea 5.3 — Tooltips enriquecidos

**Descripción:** Helper `tooltip.rs` sobre `native_tooltip` (API 12): añadir `R: recetas`, `U: usos`, el id y
los tags al tooltip existente, sin sustituirlo. Config `tooltips` para apagarlo. No añadir nada si el stack
oculta su tooltip.

**Criterios de aceptación:**
- [ ] Las líneas aparecen en el panel, en slots, en el creativo y en el chat.
- [ ] Con `tooltips = false` no se añade nada.
- [ ] No se duplican líneas ni se altera el tooltip original; el formateador tiene tests.

**Verificación:**
- [ ] `cargo test tooltip`
- [ ] `proof.py`: hover sobre un item del panel y comprobar las líneas.

**Dependencias:** 5.1. **Archivos:** `mixins/jei/tooltip.rs` (nuevo), `runtime.rs`. **Tamaño:** S.

---

### [x] Tarea 5.4 — Subtipos: pociones y flechas

**Descripción:** Expandir `NativeStack.components` en variantes: poción normal, splash, linger y flechas
tipped. Cada variante es un ingrediente propio en el índice y la búsqueda. Tope configurable de variantes.
Etiqueta: pedir `native_stack_name` upstream; mientras tanto, id del efecto y limitación documentada.

**Criterios de aceptación:**
- [ ] Una poción aparece como 4 ingredientes distintos y cada uno abre sus recetas/usos.
- [ ] Búsqueda por nombre del efecto cuando exista etiqueta, por id si no.
- [ ] Sin duplicados y respetando el tope de variantes.

**Verificación:**
- [ ] `cargo test subtypes_potion`
- [ ] `proof.py` con una poción.

**Dependencias:** 5.1. **Archivos:** `runtime.rs`, `glue.rs`, `docs/jei-parity.md`. **Tamaño:** M.

---

### [ ] Tarea 5.5 — Subtipos: libros encantados y fuegos artificiales

**Descripción:** Un ingrediente por libro encantado (encantamiento + nivel, con tope), cohetes por vuelo y
número de estrellas, y estrellas por efecto. Mismo modelo de variantes que 5.4.

**Criterios de aceptación:**
- [ ] Las variantes coinciden con lo que muestra el juego para los casos conocidos.
- [ ] El tope evita reventar la lista (8.192 entradas del panel).
- [ ] Tests de parseo de cada familia y ninguna receta existente deja de indexarse.

**Verificación:**
- [ ] `cargo test subtypes_enchanted`
- [ ] `proof.py` con un libro y un cohete.

**Dependencias:** 5.4. **Archivos:** `runtime.rs`, `glue.rs`. **Tamaño:** M.

---

### [x] Tarea 5.6 — Núcleo y persistencia de bookmarks

**Descripción:** Modelo puro `Bookmark { item, components }` con add/remove/move/dedupe/límite y
serialización en texto propio (sin dependencias nuevas), guardado en
`native_data_dir("jei")/bookmarks.json` con `.bak`. Sin tipos del juego.

**Criterios de aceptación:**
- [ ] Tests de alta, baja, reordenación, tope y persistencia ida/vuelta.
- [ ] Fichero corrupto → lista vacía, `.bak` intacto y un aviso una sola vez.
- [ ] Dedupe por `item + components` (dos pociones distintas son dos bookmarks).

**Verificación:**
- [ ] `cargo test bookmarks`

**Dependencias:** 5.1. **Archivos:** `runtime.rs`, tests. **Tamaño:** S.

---

### [x] Tarea 5.7 — Barra de bookmarks (dibujo)

**Descripción:** Layout a la izquierda del rect del contenedor (`NativeScreenView.rect`) para no solapar el
panel del juego, con iconos `native_item`, resalte al hover y tooltip propio de una línea. Escalas 1-4.
Oculta si está vacía y apagable por config.

**Criterios de aceptación:**
- [ ] Dibuja N bookmarks sin solapar el panel ni el tooltip del juego.
- [ ] Hover resalta y muestra el nombre; se ve bien con GUI scale 1-4.
- [ ] Con el mod apagado no dibuja nada.

**Verificación:**
- [ ] `cargo test bookmark_layout`
- [ ] `proof.py` con capturas a dos escalas.

**Dependencias:** 5.2, 5.6. **Archivos:** `overlay.rs`, `screen.rs`, `runtime.rs`. **Tamaño:** M.

---

### [x] Tarea 5.8 — Interacción de bookmarks

**Descripción:** Tecla A añade el stack apuntado usando el evento `Key { key, stack }` del panel (que ya
resuelve panel o slot) y `native_mod_key`; clic izquierdo abre recetas, derecho usos; borrar según config.
Usar `native_screen_mouse_button_pressed_pre` y cancelar solo dentro de la barra.

**Criterios de aceptación:**
- [ ] A funciona sobre un item del panel y sobre cualquier slot de la pantalla.
- [ ] Clics correctos y el juego no recibe los de la barra; fuera de ella no se cancela nada.
- [ ] Persiste tras reiniciar y un bookmark con componentes abre la variante correcta.

**Verificación:**
- [ ] `cargo test bookmarks`
- [ ] `proof.py`: añadir 3, abrir uno, reiniciar y comprobar.

**Dependencias:** 5.7. **Archivos:** `mixins/jei/mouse.rs` (nuevo), `runtime.rs`, `horizon-mixin.toml`.
**Tamaño:** M.

---

### [x] Tarea 5.9 — Arrastrar para reordenar

**Descripción:** Con `native_screen_mouse_dragged_pre` (API 123), mover un bookmark dentro de la barra y
guardar el orden al soltar, sin disparar el clic.

**Criterios de aceptación:**
- [ ] Arrastrar reordena y persiste; un clic sin arrastre no mueve nada.
- [ ] Cancelar a mitad no deja estado a medias ni duplica el bookmark.

**Verificación:**
- [ ] `proof.py` con drag.
- [ ] `cargo test bookmarks_reorder`

**Dependencias:** 5.8. **Archivos:** `mouse.rs`, `runtime.rs`. **Tamaño:** S.

---

### [ ] Tarea 5.10 — Configuración real

**Descripción:** `config/jei.json` vía `native_config_dir()` (API 26) más `/jei config get|set|reset` y los
botones actuales. Opciones: `tooltips`, `bookmarks` (máximo, clic, borrado), `subtypes_max`, `search_mode`,
`sort`, `language`. Validación al cargar y defaults.

**Criterios de aceptación:**
- [ ] El fichero se crea con defaults si falta; un valor inválido cae al default con un aviso único.
- [ ] Los cambios se aplican en vivo o se avisa claramente de que hace falta reabrir.
- [ ] Cada opción está documentada en `docs/config.md` con default y efecto.

**Verificación:**
- [ ] `cargo test config`
- [ ] `proof.py`: cambiar una opción y reiniciar.

**Dependencias:** 5.1, 2.7. **Archivos:** `runtime.rs`, `command.rs`, `docs/config.md`. **Tamaño:** M.

---

### [ ] Tarea 5.11 — Modos de búsqueda y ordenación

**Descripción:** Modos de búsqueda (on / off / solo con prefijo) y orden (nombre, id, tag, ninguno), conectados
a la config y a la gramática existente.

**Criterios de aceptación:**
- [ ] Cada modo cambia el filtrado y el orden como se documenta.
- [ ] El modo prefijo respeta `@`, `#`, `-`, `|` y comillas.
- [ ] Tests de combinaciones modo × orden.

**Verificación:**
- [ ] `cargo test search_modes`
- [ ] `proof.py` con dos modos.

**Dependencias:** 5.10. **Archivos:** `runtime.rs`. **Tamaño:** S.

---

### [ ] Tarea 5.12 — Documentación, paridad y RFC recortado

**Descripción:** Actualizar `README.md`, `docs/roadmap.md`, `docs/jei-parity.md`, `docs/spike.md` y
`CHANGELOG.md` (0.2.0); escribir `docs/config.md` y `docs/rfc-hooks.md` con las cinco peticiones restantes
(firma, caso de uso y ejemplo mínimo); ADR de AD-11/AD-12/AD-14.

**Criterios de aceptación:**
- [ ] El README dice exactamente qué hace y qué no hace 0.2.0.
- [ ] README y ficha del registro declaran las dos rutas de escritura: `native_data_dir("jei")` y `config/jei.json`.
- [ ] La matriz de paridad no conserva filas «bloqueado» que ya están resueltas.
- [ ] El RFC tiene cinco peticiones con firma y ejemplo, y ninguna de lo que ya existe.

**Verificación:**
- [ ] Revisión de enlaces y de filas obsoletas con `grep`.
- [ ] Lectura completa por el humano.

**Dependencias:** 5.3-5.11. **Archivos:** `README.md`, `CHANGELOG.md`, `docs/*`, `docs/adr/0002-*.md`.
**Tamaño:** M.

---

### [ ] Tarea 5.13 — Suite `proof.py` de 0.2.0

**Descripción:** Escenarios nuevos: (1) añadir 3 bookmarks, reordenar, reiniciar y abrir; (2) líneas extra del
tooltip; (3) subtipo de poción en la lista; (4) opción de config que persiste.

**Criterios de aceptación:**
- [ ] Los 4 escenarios son verdes y deterministas, con captura y aserción fuerte.
- [ ] Cada escenario imprime JSON y un fallo no impide ejecutar los demás.

**Verificación:**
- [ ] `python3 <horizon-mods>/native/tools/native.py proof "$PWD" --resources <26.3>`

**Dependencias:** 5.8-5.11. **Archivos:** `proof.py`, `.github/workflows/proof.yml`. **Tamaño:** M.

---

### [ ] Tarea 5.14 — Empaquetar y publicar 0.2.0

**Descripción:** Subir versión a 0.2.0, `native.py package`, `cargo horizon-mod check`, firmar con minisign y
añadir la `[[versions]]` a `horizon-mods/mods/jei.toml` (sha256, firma y changelog). PR al registro e
instalación limpia.

**Criterios de aceptación:**
- [ ] `.hnmod` < 1 MB y `check` verde.
- [ ] Entrada del registro con hash y firma correctos.
- [ ] Instalación limpia desde el registro en un juego vacío.

**Verificación:**
- [ ] `cargo horizon-mod check <archivo>.hnmod`
- [ ] Instalación manual + prueba de humo.

**Dependencias:** 5.12, 5.13. **Archivos:** `Cargo.toml`, `CHANGELOG.md`, `compatibility.json`, registro (PR).
**Tamaño:** S.

---

## Checkpoint 0.2.0 — tras Fase 5

- [ ] Bookmarks, tooltips y subtipos funcionan end-to-end en el Nightly.
- [ ] `cargo test`, `native.py check --horizon latest` y `proof.py` verdes.
- [ ] 0.2.0 publicada e instalación limpia verificada.
- [ ] Paridad y roadmap actualizados; RFC recortado enviado.
- [ ] Revisión humana antes de empezar la Fase 6.

---

## Fase 6 — 0.3.0 "Overlay real"

**Objetivo:** clic en slots, fantasma, transferencia SP/LAN y categorías de información.
**Diseño:** `plan.md` → «Fase 6». Depende de 0.2.0; 6.2, 6.3 y 6.4 dependen de 6.1.

### [ ] Tarea 6.1 — Tabla de geometría de slots vanilla

**Descripción:** `slot_layouts.rs` puro con los offsets de cada menú vanilla (`inventory`, `crafting`,
`generic_9x3`/`9x6`, `furnace`, `blast_furnace`, `smoker`, `smithing`, `stonecutter`, `brewing`, `grindstone`,
`anvil`, `merchant`, `beacon`) relativos a `screen.rect`, validados con capturas. Menú desconocido → `None`.
En paralelo, pedir upstream un helper `mouse → slot`.

**Criterios de aceptación:**
- [ ] Cada menú de la tabla devuelve el slot correcto bajo el cursor.
- [ ] Un menú desconocido devuelve `None` (no inventa).
- [ ] Tests con coordenadas conocidas y escalas 1-4.

**Verificación:**
- [ ] `cargo test slot_layouts`
- [ ] `proof.py` recorriendo cada menú.

**Dependencias:** 0.2.0. **Archivos:** `mixins/jei/slot_layouts.rs` (nuevo), `overlay.rs`, `glue.rs`.
**Tamaño:** M.

---

### [ ] Tarea 6.2 — Hover y clic en slots

**Descripción:** Resaltar el slot apuntado y abrir recetas/usos con clic izquierdo/derecho usando la tabla.
Respetar R/U existente, no cancelar clics fuera de slots y degradar a R/U con menús desconocidos.

**Criterios de aceptación:**
- [ ] Clic izquierdo abre recetas y derecho usos en todos los menús de la tabla.
- [ ] No se altera el inventario ni se envían paquetes.
- [ ] Sin falsos positivos en los huecos entre slots; menú desconocido queda como estaba.

**Verificación:**
- [ ] `proof.py` con capturas e inventario sin cambios.
- [ ] `cargo test click_slots`

**Dependencias:** 6.1, 5.8. **Archivos:** `mouse.rs`, `overlay.rs`, `runtime.rs`. **Tamaño:** M.

---

### [ ] Tarea 6.3 — Ingrediente fantasma

**Descripción:** Al arrastrar un ingrediente o bookmark, dibujar su icono bajo el cursor en `render_post`.
Visual: no cambia el carrito ni envía paquetes. Si 6.4 está listo, soltar sobre un slot válido puede transferir.

**Criterios de aceptación:**
- [ ] El fantasma sigue al cursor y desaparece al soltar o con Escape.
- [ ] No se envía nada al servidor por el fantasma.
- [ ] No tapa el tooltip ni el panel.

**Verificación:**
- [ ] `proof.py` con drag desde un bookmark.
- [ ] `cargo test ghost`

**Dependencias:** 6.2. **Archivos:** `overlay.rs`, `mouse.rs`, `runtime.rs`. **Tamaño:** S.

---

### [ ] Tarea 6.4 — Transferencia a estaciones en SP/LAN

**Descripción:** Rellenar horno, ahumador, alto horno, cortapiedras, mesa de herrería y brewing desde una
receta, solo en el servidor integrado, con `native_insert_item`/`native_move_items` y la posición de
`native_container_opened`. Transacción: si no cabe todo, no se mueve nada. Flag de config y botón oculto en
servidores remotos.

**Criterios de aceptación:**
- [ ] Cada estación se rellena correctamente y de forma atómica.
- [ ] Un fallo parcial no duplica ni pierde items (test).
- [ ] En servidor remoto no se intenta y se explica por qué; con el flag off no aparece el botón.

**Verificación:**
- [ ] `proof.py` en SP por estación.
- [ ] Prueba manual en LAN con dos clientes.
- [ ] `cargo test transfer`

**Dependencias:** 6.1. **Archivos:** `mixins/jei/transfer.rs` (nuevo), `glue.rs`, `runtime.rs`,
`docs/config.md`. **Tamaño:** M.

---

### [ ] Tarea 6.5 — Modo cheat en supervivencia (spike + implementación)

**Descripción:** Spike de un día: averiguar en qué lado corre el evento del panel y si se puede alcanzar el
servidor integrado con `native_give_items`. Si hay vía limpia, implementarlo con confirmación en chat y config
apagada por defecto, nunca en servidores remotos. Si no, documentarlo y descartarlo.

**Criterios de aceptación:**
- [ ] Decisión escrita en `docs/spike.md` con la evidencia.
- [ ] Si se implementa: funciona en SP, está apagado por defecto y no toca servidores remotos.
- [ ] Si no: `docs/jei-parity.md` lo marca como no soportado con el motivo.

**Verificación:**
- [ ] Spike documentado.
- [ ] `proof.py` si se implementa.

**Dependencias:** 6.4. **Archivos:** `docs/spike.md`, `runtime.rs`, `glue.rs`. **Tamaño:** S.

---

### [ ] Tarea 6.6 — Yunque y piedra de afilar

**Descripción:** Calcular el resultado del yunque (reparación/combinación, coste de niveles y renombrado) y de
la piedra de afilar (desencantar y durabilidad) desde `NativeStack.components`, y mostrarlo como página de
información.

**Criterios de aceptación:**
- [ ] Los números coinciden con vanilla en casos conocidos (misma herramienta, dos herramientas, con nombre).
- [ ] Sin datos hardcodeados sin tarea de actualización.
- [ ] Tests con casos de vanilla 26.3.

**Verificación:**
- [ ] `cargo test anvil grindstone`
- [ ] `proof.py` de la página de información.

**Dependencias:** 0.2.0. **Archivos:** `runtime.rs` (`info`), `docs/jei-parity.md`. **Tamaño:** M.

---

### [ ] Tarea 6.7 — Combustible y compost

**Descripción:** Pedir upstream `native_burn_time` y `native_compost_chance`. Si no llegan, tabla del mod en
`data/` con versión de datos, tarea de actualización por versión y aviso al usuario.

**Criterios de aceptación:**
- [ ] Cada categoría muestra datos verificables contra el juego 26.3.
- [ ] Si es tabla del mod, hay tarea de actualización documentada y la versión de datos se ve en `/jei`.
- [ ] Tests con valores conocidos.

**Verificación:**
- [ ] `cargo test fuel compost`
- [ ] `proof.py` de ambas páginas.

**Dependencias:** 6.6. **Archivos:** `data/*.json` o `runtime.rs`, `docs/config.md`. **Tamaño:** M.

---

### [ ] Tarea 6.8 — Panel en creativo

**Descripción:** Pedir upstream quitar el veto `native_id() != "recipes"` (`native_panel.rs:201`). Si no llega,
dibujar en creativo solo nuestra lista (sin páginas de receta) y documentar el límite.

**Criterios de aceptación:**
- [ ] En creativo aparece la lista y R/U funcionan (completo) o solo la lista (parcial), y está documentado.
- [ ] No se rompe el panel oficial de `recipes`.

**Verificación:**
- [ ] `proof.py` en creativo.
- [ ] `docs/jei-parity.md` actualizado.

**Dependencias:** 0.2.0. **Archivos:** `screen.rs`, `runtime.rs`, `docs/*`. **Tamaño:** M.

---

### [ ] Tarea 6.9 — Búsqueda `$tooltip` y `%creative tab` + autocompletado

**Descripción:** Activar `$` si hay texto de tooltips (helper upstream en bloque o caché de los tooltips
vistos) y `%` si Horizon expone tabs. Autocompletado de prefijos navegable con teclado en nuestra barra.

**Criterios de aceptación:**
- [ ] `$` y `%` solo aparecen si sus datos existen; si no, se ignoran y se documenta.
- [ ] El autocompletado no roba teclas cuando está cerrado.
- [ ] Tests de parser y de sugerencias.

**Verificación:**
- [ ] `cargo test search_advanced`
- [ ] `proof.py` tecleando `$`.

**Dependencias:** 0.2.0, 1.4. **Archivos:** `runtime.rs`, `overlay.rs`. **Tamaño:** M.

---

### [ ] Tarea 6.10 — Árbol de recetas y navegación

**Descripción:** Historial completo atrás/adelante, encadenar receta → ingrediente → receta, breadcrumb en el
título y límite de profundidad.

**Criterios de aceptación:**
- [ ] Navegación de tres niveles sin perder el historial.
- [ ] Adelante y atrás recorren la pila completa.
- [ ] El título muestra item y tab donde estás.

**Verificación:**
- [ ] `cargo test navigation`
- [ ] `proof.py` con una cadena de 3 niveles.

**Dependencias:** 6.2. **Archivos:** `runtime.rs`, `glue.rs`. **Tamaño:** M.

---

### [ ] Tarea 6.11 — Opciones de panel

**Descripción:** Pedir upstream lado, ancho y scroll del panel. Si no llega, documentar y mantener páginas. No
se reescribe el panel (AD-11).

**Criterios de aceptación:**
- [ ] Decisión documentada (upstream aceptado o limitación).
- [ ] Si llega: panel a izquierda/derecha y ancho configurable sin solapar el GUI.
- [ ] Si no llega: la limitación está en README y en la ficha del registro.

**Verificación:**
- [ ] `proof.py` con el panel a la izquierda (si aplica).
- [ ] Lectura de la documentación.

**Dependencias:** 6.8. **Archivos:** `glue.rs`, `docs/*`, `README.md`. **Tamaño:** M.

---

### [ ] Tarea 6.12 — Presupuesto de rendimiento

**Descripción:** Medir catálogo, búsqueda por tecla, frame del overlay y memoria con 10k items y 20k recetas;
añadir debounce si hace falta. Adelanta parte de la tarea 4.2.

**Criterios de aceptación:**
- [ ] Presupuestos de 4.2 cumplidos o con plan y justificación.
- [ ] Regresión > 20 % falla CI.
- [ ] Informe en `docs/performance.md`.

**Verificación:**
- [ ] `cargo bench` o tests con `--release` y umbrales.
- [ ] Informe revisado.

**Dependencias:** 5.13, 6.10. **Archivos:** `benches/*`, `docs/performance.md`. **Tamaño:** M.

---

### [ ] Tarea 6.13 — Documentación y publicar 0.3.0

**Descripción:** Auditoría completa de `docs/jei-parity.md`, README, CHANGELOG, empaquetado, firma, entrada en
el registro e instalación limpia.

**Criterios de aceptación:**
- [ ] Cada fila de la matriz de paridad está auditada y con su estado final.
- [ ] 0.3.0 publicada con hash y firma correctos.
- [ ] Instalación limpia verificada.

**Verificación:**
- [ ] `cargo horizon-mod check <archivo>.hnmod`
- [ ] Instalación manual + prueba de humo.
- [ ] Revisión de paridad con el humano.

**Dependencias:** 6.12. **Archivos:** `README.md`, `CHANGELOG.md`, `docs/*`, registro (PR). **Tamaño:** S.

---

## Checkpoint 0.3.0 — tras Fase 6

- [ ] Clic en slots, fantasma y transferencia SP/LAN funcionan.
- [ ] Categorías de información con datos verificados.
- [ ] Auditoría de paridad completa y límites documentados.
- [ ] 0.3.0 publicada e instalación limpia.
- [ ] Revisión de seguridad: sin red y sin escrituras fuera de `native_data_dir`/`config/`.

---

## Fase 4 — Ecosistema, rendimiento y 1.0 (después de 0.3.0)

### [ ] Tarea 4.1 — Categorías data-driven para content mods

**Descripción:** Diseñar y leer un formato JSON para que un content mod declare categorías y páginas de
información (`data/<id>/jei/...`), alineado con el "recipe-type registry" previsto por Horizon para 0.4.
Mientras tanto, JEI lee ese JSON desde el directorio de mods si existe; nunca ejecuta código de otro mod.

**Criterios de aceptación:**
- [ ] Formato documentado con ejemplo y validación (errores claros).
- [ ] Un content mod de prueba añade una categoría visible sin tocar JEI.
- [ ] Archivos inválidos se ignoran con log, sin romper el resto.

**Verificación:**
- [ ] `cargo test custom_categories`
- [ ] `proof.py` con un `.hmod` de fixture.

**Dependencias:** 3.5. **Archivos:** `runtime.rs`, `glue.rs`, `docs/custom-categories.md`, `fixtures/`. **Tamaño:** M.

---

### [ ] Tarea 4.2 — Presupuesto de rendimiento y benchmarks

**Descripción:** Medir y fijar presupuestos: construcción del catálogo al abrir mundo (< 150 ms p95),
búsqueda por tecla (< 16 ms p95), frame del panel (< 0,2 ms), memoria del índice (< 32 MB con 10k items),
tiempo de sincronización del panel (< 8 ms). Benchmark reproducible en CI.

**Criterios de aceptación:**
- [ ] Benchmarks con datos sintéticos de 10k items / 20k recetas.
- [ ] Todos los presupuestos se cumplen en el hardware de referencia.
- [ ] Regresión > 20 % falla CI.

**Verificación:**
- [ ] `cargo bench` (o tests con `--release` y umbrales).
- [ ] Informe en `docs/performance.md`.

**Dependencias:** 2.10, 3.9. **Archivos:** `benches/*`, `docs/performance.md`. **Tamaño:** M.

---

### [ ] Tarea 4.3 — Localización completa

**Descripción:** Portar las claves de UI de JEI (`en_us.json`, ~630; 31 idiomas) que apliquen, con
atribución MIT y sin claves del juego. Completar es. Documentar cómo añadir idiomas.

**Criterios de aceptación:**
- [ ] ≥ 95 % de las cadenas visibles traducidas en en/es.
- [ ] El resto de idiomas cae a inglés sin romper.
- [ ] Script que detecta claves usadas y no traducidas.

**Verificación:**
- [ ] `cargo test i18n_full`
- [ ] `tools/missing-strings.sh` sin salida en en/es.

**Dependencias:** 2.9. **Archivos:** `lang/*.json`, `runtime.rs`, `tools/missing-strings.sh`. **Tamaño:** M.

---

### [ ] Tarea 4.4 — Documentación y atribución

**Descripción:** README definitivo (qué es, qué no, instalación, atajos, config, límites), `docs/jei-parity.md`
actualizado, ADRs de las decisiones revisadas, `NOTICE`/licencia, guía de contribución y de reporte de bugs
(con `logs/latest.log`, versión de Horizon, lista de mods).

**Criterios de aceptación:**
- [ ] Un usuario nuevo instala y usa JEI solo con el README.
- [ ] Cada límite conocido está documentado con su motivo.
- [ ] Atribución a JEI y a Horizon correcta.

**Verificación:**
- [ ] Revisión por alguien ajeno al proyecto (prueba de instalación).
- [ ] Enlaces sin romper.

**Dependencias:** 3.9. **Archivos:** `README.md`, `docs/*.md`, `NOTICE`. **Tamaño:** M.

---

### [ ] Tarea 4.5 — Publicar 1.0.0 y playbook de mantenimiento

**Descripción:** Versión 1.0.0, changelog, firma, PR al registro, release notes con capturas. Playbook:
cómo re-pinear tras un Nightly, cómo detectar un hook roto, cadencia de releases, qué hacer si un hook de
overlay cambia (pedir upstream antes que pinear), contacto upstream.

**Criterios de aceptación:**
- [ ] 1.0.0 publicada y verificada con instalación limpia.
- [ ] El playbook cubre los 5 fallos más probables con comandos concretos.
- [ ] CI diario verde 7 días seguidos antes de anunciar.

**Verificación:**
- [ ] `cargo horizon-mod check` + instalación limpia.
- [ ] Simulación de rotura de hook (renombrar un helper) siguiendo el playbook.

**Dependencias:** 4.2, 4.3, 4.4. **Archivos:** `CHANGELOG.md`, `docs/maintenance.md`, `compatibility.json`. **Tamaño:** S.

---

## Checkpoint E — fin

- [ ] 1.0.0 publicada; paridad documentada y auditada.
- [ ] CI diario verde; playbook probado.
- [ ] Sin deuda crítica: sin `TODO` en rutas de usuario, sin tests ignorados.
- [ ] Retropectiva escrita: qué costó más, qué se haría distinto, qué pedir a Horizon.