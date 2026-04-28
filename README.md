# dm-log — Bitácora Sword & Wizardry para Emacs

Paquete Emacs para gestionar bitácoras de campañas **Sword & Wizardry** (o cualquier OSR).

## Características

- **Buffer read-only** `*SW-Bitacora*` con vista limpia y formateada
- **Navegación via Transient** (estilo Magit) con `SPC`
- **Tracking automático de tiempo ficticio** con avances según tipo de turno
- **Cálculo automático de consumibles** (antorchas, raciones, agua, etc.)
- **Saltos de tiempo arbitrarios** (descansos, viajes, etc.)
- **Multi-campaña**: selector inicial de campañas
- **Integración org-mode**: datos almacenados en archivos `.org` estructurados
- **Compatible org-roam**: IDs generados para campañas, sesiones y jugadores

## Instalación

### Requisitos

- Emacs 30+
- `transient` (viene con Emacs 30+, o instalar desde ELPA)

### Instalación manual

```bash
git clone <repo> ~/.emacs.d/lisp/dm-log
```

En tu `init.el`:

```elisp
(add-to-list 'load-path "~/.emacs.d/lisp/dm-log/lisp")
(require 'dm-log)
```

### Con use-package (straight)

```elisp
(use-package dm-log
  :straight (:host github :repo "tu-usuario/dm-log")
  :custom
  (dm-log-campaigns-directory "~/campaigns"))
```

## Uso

### Iniciar

```
M-x dm-log
```

Se mostrará un **menú transient** con las campañas disponibles en `~/campaigns/`.
Selecciona una para cargarla.

### En el buffer *SW-Bitacora*

| Tecla | Acción |
|-------|--------|
| `SPC` | Abrir menú transient principal |
| `q` | Cerrar bitácora |

### Menú principal (SPC)

| Tecla | Acción |
|-------|--------|
| `r` | Refrescar bitácora |
| `a` | Agregar entrada (abre sub-menú) |
| `t` | Salto de tiempo arbitrario |
| `c` | Cambiar campaña |
| `q` | Cerrar menú |

### Agregar entrada (`a`)

| Tecla | Tipo | Avance de tiempo |
|-------|------|------------------|
| `d` | Calabozo | +10 minutos (configurable) |
| `e` | Exteriores | +1 hora (configurable) |
| `c` | Combate | +1 ronda/minuto (configurable) |
| `s` | Salto arbitrario | Personalizado |
| `q` | Cancelar | — |

Tras seleccionar tipo:
1. Se abre buffer temporal `*dm-log-editar-turno*` con:
   - Sección **Memo** (edita libremente)
   - Tabla **Consumibles** pre-calculada (edita valores si es necesario)
2. `C-c C-c` → Guarda, avanza tiempo, actualiza org, refresca bitácora
3. `C-c C-k` → Cancela

## Estructura de archivos por campaña

Cada campaña es un directorio bajo `~/campaigns/<nombre>/`:

```
mycampaign/
├── bitacora.org       ← Entradas de turnos y tiempo de juego
├── consumibles.org    ← Tabla de tasas de consumo
└── jugadores.org      ← Inventario base y metadatos
```

### bitacora.org

```org
:PROPERTIES:
:ID:       <uuid>
:FORMATO_TIEMPO: %B %d, %E %Y %H:%M
:TIEMPO_ACTUAL: [2024-01-10 13:45]
:END:
#+TITLE: Mi Campaña

* Metadatos
** Jugadores
* Bitácora
** Sesión 3
:PROPERTIES:
:ID: <uuid>
:NUMERO: 3
:FECHA_REAL: [2025-04-23 Wed]
:END:

*** Turno 28 [Calabozo] (10m)
:PROPERTIES:
:TURNO_NUMERO: 28
:TURNO_TIPO: calabozo
:TIEMPO_INICIO: [2024-01-10 13:45]
:TIEMPO_FIN: [2024-01-10 13:55]
:AVANCE: 10m
:END:

**** Memo
Exploraron el pasillo norte.

**** Consumibles
| Item      | Jugador A | Jugador B |
|-----------+-----------+-----------|
| Antorchas |      4.83 |      2.00 |
| Raciones  |     10.00 |      8.00 |
```

### consumibles.org

Tabla de tasas: **por cada período de tiempo que pase, se consume X cantidad**.

```org
| Item      | Periodo | Cantidad |
|-----------+---------+----------|
| Antorchas | 1h      |      1.0 |
| Raciones  | 24h     |      3.0 |
| Aceite    | 30m     |      0.5 |
| Agua      | 8h      |      1.0 |
| Flechas   | --      |      0.0 |
```

Una antorcha se consume **1.0 unidad cada 1 hora**. En un turno de calabozo (10 min) se consume `10/60 * 1.0 = 0.17` unidades.

### jugadores.org

```org
#+TITLE: Jugadores
* Jugador A
:PROPERTIES:
:ID: <uuid>
:ANTORCHAS: 5
:RACIONES: 10
:ACEITE: 2
:AGUA: 3
:ANTORCHA_ENCENDIDA: t
:END:
```

La propiedad `:ANTORCHA_ENCENDIDA:` controla si el jugador consume antorchas automáticamente.

## Personalización

| Variable | Valor por defecto | Descripción |
|----------|-------------------|-------------|
| `dm-log-campaigns-directory` | `~/campaigns` | Directorio base de campañas |
| `dm-log-bitacora-filename` | `bitacora.org` | Nombre archivo de bitácora |
| `dm-log-consumibles-filename` | `consumibles.org` | Nombre archivo de consumibles |
| `dm-log-jugadores-filename` | `jugadores.org` | Nombre archivo de jugadores |
| `dm-log-turno-calabozo-avance` | `10m` | Tiempo por turno de calabozo |
| `dm-log-turno-exteriores-avance` | `1h` | Tiempo por turno exteriores |
| `dm-log-turno-combate-avance` | `1m` | Tiempo por turno de combate |

## Formato de tiempo

El formato de tiempo ficticio usa la sintaxis de `format-time-string`:

- `%B` → Nombre del mes
- `%d` → Día
- `%E` → Era (personalizado, requiere ajuste)
- `%Y` → Año
- `%H:%M` → Hora:Minuto

Ejemplo: `"%B %d, %E %Y %H:%M"` → `Enero 10, 1 Era Inventada 13:45`

**Nota:** `%E` no es estándar en `format-time-string`. Si necesitas eras personalizadas, edita la propiedad `:FORMATO_TIEMPO:` para usar texto literal o ajusta la función `dm-log-time--format-game-timestamp`.

## Integración org-roam

Todos los headings principales (campaña, sesiones, jugadores) generan propiedades `:ID:` con `org-id-new`. Esto permite enlazarlos desde cualquier nota de org-roam.

## Licencia

GPL v3
