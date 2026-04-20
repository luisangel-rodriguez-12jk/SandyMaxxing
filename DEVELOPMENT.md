# SandyMaxxing v1.0.0 — Guía completa

App de escritorio (Tauri + React + SQLite) para planeación nutricional familiar.
Interfaz 100 % en español, funciona offline, usa OpenAI sólo cuando explícitamente generas un plan o analizas un licuado.

---

## 1 · Arrancar la app en tu PC de desarrollo

### 1.1 Requisitos (una sola vez)

| Sistema | Requisitos |
| --- | --- |
| **Windows 10/11** | WebView2 (ya viene con Windows 11 y con Windows 10 actualizado) |
| **macOS** | Xcode Command Line Tools (`xcode-select --install`) |
| **Linux** | `webkit2gtk-4.1` y `libssl-dev` |

Y además, en cualquier sistema:

- [**Rust**](https://rustup.rs) 1.77 o superior (`rustup update`)
- [**Node.js**](https://nodejs.org) 20 o superior + npm

### 1.2 Primer arranque

```bash
cd SandyMaxxing
npm install
npm run tauri dev
```

El primer `cargo` tarda 5–10 minutos; los siguientes arranques son casi instantáneos.

---

## 2 · Generar el instalador para compartir con familia / amigos

```bash
npm run tauri build
```

Al terminar encontrarás dos instaladores listos:

```
src-tauri/target/release/bundle/
  ├── msi/SandyMaxxing_1.0.0_x64_en-US.msi          ← silencioso (doble clic y listo)
  └── nsis/SandyMaxxing_1.0.0_x64-setup.exe         ← con asistente gráfico
```

Ambos son totalmente independientes: no hace falta instalar Rust ni Node en la PC
destino. El ejecutable suelto (sin instalador) está en
`src-tauri/target/release/sandymaxxing.exe` — sirve para probar, pero lo recomendado
es pasar el `.exe` de NSIS porque crea acceso directo en el menú Inicio.

### 2.1 Instalar en otra PC (la de mamá, papá, abuela, etc.)

1. Copia `SandyMaxxing_1.0.0_x64-setup.exe` a la otra PC (USB, Drive, WhatsApp).
2. Doble clic → *Siguiente* → *Siguiente* → *Finalizar*.
3. Windows SmartScreen puede avisar "aplicación no reconocida" porque el instalador
   no está firmado. Clic en **Más información → Ejecutar de todos modos**.
4. Aparece en el menú Inicio como **SandyMaxxing**.
5. La primera vez hay que ir a **Ajustes → Clave IA** y pegar una clave de OpenAI
   (`sk-…`) — sin ella no funcionan los licuados ni los planes, pero sí todo lo demás.

Los datos se guardan en `%APPDATA%\SandyMaxxing\sandymaxxing.sqlite`. Respalda ese
archivo para no perder el historial.

---

## 3 · Qué hay en la app (5 secciones)

| Sección | Pestañas / contenido |
| --- | --- |
| 👪 **Usuarios** | Alta, edición y selección del usuario activo |
| 🥗 **Mi plan** | **Porciones** (grid por comida × grupo) · **Prohibidos** (chip por alimento) · **Licuados** (IA) |
| 🍳 **Cocinar** | **Plan IA semanal** (generar/guardar/editar/PDF) · **Comida única** (3 opciones + generar más) · **Lista de compras** |
| 📈 **Progreso** | **Gráficas** (peso, cintura, abdomen, cadera) · **Historial** (porciones + mediciones de cualquier semana) |
| ⚙️ **Ajustes** | **Clave IA** (OpenAI) · **Equivalencias** (tabla del folleto editable) |

---

## 4 · Novedades de la v1.0.0

Todas estas son las respuestas a la lista de 10 cambios pedidos:

1. **Prohibidos**: cada alimento es su propia etiqueta. Marcar "Plátano" ya no marca frutas distintas.
2. **Porciones**: los grupos salen en el orden del folleto (Grasas → Verduras → … → Cereales). Los alimentos dentro de cada grupo también.
3. **Fechas**: cualquier campo de fecha (medición, plan, historial) aparece con la fecha de hoy por defecto.
4. **Porciones → Guardar**: botón "Guardar" con mensaje "✓ Porciones guardadas correctamente" al terminar. Mientras haya cambios sin guardar se muestra un aviso.
5. **Historial** (pestaña en Progreso): elige una fecha, muestra las porciones registradas para esa semana y las mediciones que caen en ese rango, además del historial completo.
6. **Plan IA semanal** ahora funciona end-to-end:
   - Botón `Generar semana` revisa que haya usuarios seleccionados y clave de OpenAI antes de pedir el plan.
   - Cada comida tiene botón **✎ editar con IA** para cambiarla con una instrucción ("cámbialo por algo con pollo") sin regenerar el resto.
   - Preparación en pasos numerados con tiempos y utensilios.
   - Botones **Guardar**, **Actualizar**, **Exportar PDF** y lista inferior de planes guardados con *Abrir* / *×*.
   - Sigue aceptando múltiples usuarios: las porciones son por persona pero la comida es la misma.
7. **Comida única**: selección multi-usuario + tiempo de comida + "Generar 3 opciones" + "Generar 3 opciones más (sin repetir)" que manda las ya vistas como `exclude`.
8. **Errores siempre explicados**: cualquier botón que no funcione muestra debajo el motivo en español ("Selecciona al menos un usuario", "Falta configurar la clave de OpenAI", "Escribe los ingredientes del licuado", etc.).
9. **Equivalencias**: los renglones que el folleto junta con comas ahora son alimentos individuales en toda la app (p. ej. "Helado crema" y "Frutas en almíbar" son dos items separados).
10. **Esta guía** resume cómo ejecutar, empaquetar y compartir.

> **Nota de upgrade**: si vienes de un prototipo anterior, puede que tengas
> que volver a pegar la clave de OpenAI en Ajustes → Clave IA una sola vez. Los
> datos de usuarios, mediciones, alimentos, etc., no se pierden.

---

## 5 · Árbol de código

```
src-tauri/            Backend Rust
  src/db/             Esquema SQLite + semilla del folleto (sort_order por grupo/alimento)
  src/repo/           SQL puro por agregado (foods, users, diets, smoothies, saved_plans…)
  src/services/       Lógica (family_compat, shopping_list, pdf_export)
  src/ai/             Cliente OpenAI + prompts (smoothie_parser, plan_generator, meal_options, tweak_meal)
  src/commands/       #[tauri::command] — superficie IPC
  src/crypto.rs       AES-GCM para la clave de OpenAI en disco
src/
  pages/              Páginas y paneles reutilizables
  components/         Layout, Tabs, charts/LineChart, UserSwitcher
  api/invoke.ts       Wrappers tipados para los comandos Tauri
  i18n/es.ts          Todas las cadenas en español (ningún texto hardcodeado en páginas)
```

---

## 6 · Verificación end-to-end (manual)

Arranca `npm run tauri dev` y recorre:

1. **Usuarios** → crea `María` y `Pedro`.
2. **Mi plan → Porciones** → cambia a María, asigna 2 porciones de Verduras en Comida → *Guardar* → debe salir "✓ Porciones guardadas correctamente".
3. **Mi plan → Prohibidos** → marca sólo `Plátano` para María (debe quedar rojo únicamente ese). Marca `Chorizo` para Pedro.
4. **Ajustes → Clave IA** → pega `sk-…`.
5. **Mi plan → Licuados** → `"Licuar: 1 taza leche de almendra, 1 plátano, 2 cucharadas avena, 1 scoop proteína"` → *Analizar con IA* → 4 ingredientes extraídos.
6. **Cocinar → Plan IA** → selecciona a María y Pedro → *Generar semana* → 7 días × 5 comidas. Los nombres no deben incluir chorizo. Pulsa *editar con IA* en alguna comida → escribe "cámbialo por algo con arroz" → la comida se reemplaza sola.
7. *Nombre del plan*: "Semana de prueba" → *Guardar*. Refresca la lista inferior → aparece. Pulsa *Nuevo plan* → la UI se limpia. *Abrir* → regresa el plan.
8. *Exportar PDF* → elige ruta → se guarda un PDF legible.
9. **Cocinar → Comida única** → selecciona a María, tipo = Comida → *Generar 3 opciones* → 3 cards. *Generar 3 opciones más* → 3 nuevos sin repetir nombres.
10. **Cocinar → Lista de compras** → selecciona a los dos usuarios → *Calcular* → tabla. *Exportar PDF* funciona.
11. **Progreso → Gráficas** → añade 4 mediciones (fechas distintas) → aparecen las 4 gráficas. *Exportar PDF* funciona.
12. **Progreso → Historial** → cambia la fecha → ves las porciones de esa semana + mediciones que caen en ese rango.

Si algún paso devuelve un error, la app lo muestra en rojo con el motivo exacto.

---

## 7 · Datos precargados

El folleto de la nutrióloga se carga al primer arranque desde `src-tauri/src/db/migrations.rs::seed_foods`. Si quieres "restaurar de fábrica" borra:

```
%APPDATA%\SandyMaxxing\sandymaxxing.sqlite
```

y vuelve a abrir la app.

---

## 8 · Seguridad

- La clave de OpenAI se guarda cifrada con **AES-GCM** (llave derivada del nombre
  de usuario del SO + sal aleatoria almacenada en la misma base).
- Toda la base vive **sólo** en `%APPDATA%\SandyMaxxing` — nada sale del equipo
  salvo las llamadas HTTPS a `api.openai.com` que tú disparas con los botones de IA.
- Si vas a respaldar tu base (`.sqlite`) y llevártela a otro PC, la clave cifrada
  dejará de descifrarse ahí (cambia el usuario del SO). Re-ingrésala en Ajustes.
