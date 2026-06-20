# Bassical 🎸

**Aplicación desktop para aprender a tocar bajo eléctrico.**  
Carga tus canciones, calibra el tempo, crea tus propias tabs y practica con play-along — sin anuncios, sin premium, sin internet.

---

## Tabla de contenidos

- [Descripción](#descripción)
- [Características](#características)
- [Stack tecnológico](#stack-tecnológico)
- [Arquitectura](#arquitectura)
- [Requisitos del sistema](#requisitos-del-sistema)
- [Instalación](#instalación)
- [Uso](#uso)
- [Estructura del proyecto](#estructura-del-proyecto)
- [Formatos soportados](#formatos-soportados)
- [Atajos de teclado](#atajos-de-teclado)
- [Persistencia de datos](#persistencia-de-datos)
- [Hoja de ruta](#hoja-de-ruta)
- [Fuera del alcance (v1.0)](#fuera-del-alcance-v10)
- [Documentación técnica](#documentación-técnica)
- [Licencia](#licencia)

---

## Descripción

Bassical es una herramienta de práctica para bajistas de nivel intermedio y avanzado. A diferencia de plataformas como Songsterr, Bassical **no depende de un catálogo externo**: el usuario crea y gestiona su propia biblioteca de tabs. Toda la información se almacena localmente; la aplicación no requiere conexión a internet en ningún momento.

El diferenciador técnico central es el **sistema de calibración de tempo variable por sección**, inspirado en el editor de beatmaps de osu!, que permite definir múltiples timing points a lo largo de una canción con precisión de milisegundos.

---

## Características

- **Biblioteca local** — registra canciones desde archivos de audio en tu equipo (MP3, WAV, FLAC, OGG).
- **Calibración de tempo asistida** — presiona una tecla al ritmo de la canción para detectar BPM y offset automáticamente. Soporta BPM variable por sección (timing points múltiples).
- **Editor de tablatura** — notación estándar de bajo de 4 cuerdas con soporte para técnicas (hammer-on, pull-off, slide, vibrato, mute, ghost note). Undo/redo hasta 50 pasos.
- **Play-along sincronizado** — cursor animado que avanza por la tab en sincronía con el audio, derivado de los timing points activos.
- **Control de velocidad sin cambio de tono** — reduce el tempo entre 25 % y 100 % en incrementos de 5 % usando pitch-shifting independiente.
- **Loop de sección** — define un rango de compases y practica ese fragmento en bucle.
- **Metrónomo visual y audible** — sincronizado con los timing points, activable de forma independiente.
- **Importación de Guitar Pro** — carga tabs en formato `.gp`, `.gp5` y `.gpx`; extrae automáticamente la pista de bajo de 4 cuerdas.
- **Exportación a PDF y JSON** — comparte tus tabs o guárdalas como respaldo.
- **100 % gratuito y offline** — sin cuentas, sin publicidad, sin telemetría.

---

## Stack tecnológico

### Frontend
| Tecnología | Rol |
|---|---|
| TypeScript | Lenguaje principal del frontend |
| React o Svelte | Framework de UI (a definir en diseño) |
| Tailwind CSS | Estilos |
| Zustand | Gestión de estado global |
| Canvas API | Renderizado de waveform y tablatura |

### Backend
| Tecnología | Rol |
|---|---|
| Rust | Lenguaje del backend |
| Tauri 2 | Framework desktop (bridge IPC Rust ↔ frontend) |
| `cpal` | Acceso al hardware de audio (baja latencia) |
| `soundtouch` | Pitch-shifting independiente del tempo (via FFI) |
| `serde` / `serde_json` | Serialización de tabs y biblioteca |

### Distribución
| Artefacto | Descripción |
|---|---|
| `.exe` autocontenido | Instalador para Windows 10/11 de 64 bits, sin dependencias externas |

> **¿Por qué Rust + Tauri?**  
> El sistema de calibración por pulsación de tecla requiere latencia de input ≤ 10 ms — garantía que los navegadores web no pueden ofrecer de forma confiable. Rust brinda esa precisión en el backend, mientras que Tauri permite construir la interfaz con las mismas tecnologías web que en cualquier aplicación moderna.

---

## Arquitectura

Bassical sigue una arquitectura de dos capas desacopladas, comunicadas por el IPC de Tauri:

```
┌──────────────────────────────────────────────────────┐
│                  Capa de presentación                │
│          HTML · CSS · TypeScript · React             │
│                                                      │
│  ┌──────────────┐  ┌───────────────┐  ┌──────────┐   │
│  │  Tab editor  │  │ Waveform view │  │  Player  │   │
│  └──────────────┘  └───────────────┘  └──────────┘   │
└────────────────────────┬─────────────────────────────┘
                         │  invoke() / eventos
┌────────────────────────▼─────────────────────────────┐
│                 Tauri IPC bridge                     │
│          Comandos registrados · eventos              │
└────────────────────────┬─────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────┐
│                   Capa de lógica (Rust)              │
│                                                      │
│  ┌──────────────┐  ┌────────────┐  ┌──────────────┐  │
│  │ Audio engine │  │ Calibrador │  │ Persistencia │  │
│  │ cpal · soundtouch│  │ BPM·offset │  │ JSON·AppData │  │
│  └──────────────┘  └────────────┘  └──────────────┘  │
│                                                      │
│  ┌──────────────────────────────────────────────┐    │
│  │           Tab parser (.gp · .gpx · JSON)     │    │
│  └──────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────┘
         │
         ▼
  Sistema operativo Windows · hardware de audio
```

### Flujo del IPC

El frontend llama al backend mediante `invoke()`:

```typescript
// frontend/src/lib/audio.ts
import { invoke } from '@tauri-apps/api/core';

// Registrar un tap de calibración
const bpm = await invoke<number>('tap_calibration', {
  timestamp_ms: performance.now(),
});

// Iniciar reproducción a velocidad reducida
await invoke('play_audio', { speed: 0.75 });
```

El backend expone comandos declarados con `#[tauri::command]`:

```rust
// src-tauri/src/commands/calibration.rs
#[tauri::command]
fn tap_calibration(timestamp_ms: f64) -> Result<f64, String> {
    calibrador::registrar_tap(timestamp_ms)
        .map_err(|e| e.to_string())
}
```

---

## Requisitos del sistema

| Componente | Mínimo |
|---|---|
| Sistema operativo | Windows 10 de 64 bits (build 1903+) |
| RAM | 4 GB |
| Almacenamiento | 100 MB (instalador) + espacio para archivos de audio del usuario |
| Audio | Dispositivo de salida compatible con WASAPI o ASIO |
| Entrada | Teclado físico (requerido para calibración por pulsación) |

> Los archivos de audio permanecen en su ubicación original. Bassical solo almacena la ruta de referencia, no copia ni modifica los archivos del usuario.

---

## Instalación

### Usuario final

1. Descarga el instalador `Bassical_vX.X.X_setup.exe` desde la sección [Releases](../../releases).
2. Ejecuta el instalador. No se requieren permisos de administrador.
3. Abre Bassical desde el menú de inicio o el acceso directo en el escritorio.

### Desarrolladores

**Requisitos previos:**

```bash
# Rust (toolchain estable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# Node.js >= 18
# (recomendado: instalar con nvm o fnm)

# Tauri CLI
cargo install tauri-cli
```

**Clonar y ejecutar en modo desarrollo:**

```bash
git clone https://github.com/tu-usuario/bassical.git
cd bassical

# Instalar dependencias del frontend
npm install

# Iniciar en modo desarrollo (hot-reload en ambas capas)
cargo tauri dev
```

**Compilar el instalador de producción:**

```bash
cargo tauri build
# El instalador .exe queda en: src-tauri/target/release/bundle/
```

---

## Uso

### Flujo básico

```
1. Agregar canción
   └── Archivo > Nueva canción → seleccionar archivo de audio

2. Calibrar tempo
   └── Pestaña "Tempo" → reproducir audio → presionar T al ritmo
       → ajustar timing points manualmente sobre la waveform si es necesario

3. Crear o importar tab
   └── Crear: pestaña "Editor" → clic en celda → escribir número de traste
   └── Importar: Archivo > Importar tab → seleccionar .gp / .gp5 / .gpx / .json

4. Practicar
   └── Pestaña "Play-along" → Barra espaciadora para reproducir
       → Ctrl+← / Ctrl+→ para ajustar velocidad
       └── L para activar loop de sección
```

### Calibración de tempo (detalle)

El calibrador funciona igual que el editor de beatmaps de osu!:

1. Se reproduce el audio de la canción.
2. El usuario presiona `T` al ritmo de cada beat durante al menos 8 compases.
3. Bassical calcula el BPM promedio del segmento y el offset del primer beat.
4. El resultado se visualiza como un marcador vertical sobre la waveform.
5. Para canciones con cambios de tempo, se repite el proceso en cada sección (múltiples timing points).
6. El ajuste fino del offset se realiza arrastrando el marcador sobre la waveform o ingresando el valor en milisegundos directamente.

---

## Estructura del proyecto

```
bassical/
├── src/                        # Frontend (TypeScript)
│   ├── components/
│   │   ├── Audio/              # PlaybackControls, WaveformView
│   │   ├── Layout/             # Shared layout components
│   │   └── Library/            # Biblioteca de canciones
│   ├── views/
│   │   ├── LibraryView.tsx     # Vista de biblioteca
│   │   └── AudioView.tsx       # Vista de reproducción + waveform
│   ├── lib/
│   │   ├── audio.ts            # Llamadas invoke() al backend de audio
│   │   ├── usePracticePlayback.ts # Hook de reproducción (decode, speed, position)
│   │   ├── useAudioPlayback.ts # Hook de reproducción global (PlayerBar)
│   │   └── store/              # Zustand slices
│   │       └── sessionStore.ts
│   ├── App.tsx
│   └── main.tsx
│
├── src-tauri/                  # Backend (Rust)
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs              # Tauri builder + command registration
│   │   ├── commands/
│   │   │   ├── audio.rs        # play, pause, seek, speed, decode, full-buffer
│   │   │   └── library.rs      # CRUD de canciones
│   │   ├── audio/
│   │   │   ├── engine.rs       # Motor de reproducción (cpal, dual-mode callback)
│   │   │   ├── decoder.rs      # Streaming decoder (symphonia)
│   │   │   └── buffer_playback.rs # Full-buffer + SoundTouch time-stretching
│   │   ├── models/
│   │   │   ├── song.rs
│   │   │   └── tab.rs
│   │   ├── persistence/
│   │   │   └── storage.rs      # JSON read/write en AppData
│   │   ├── calibration/        # Timing point logic (stub)
│   │   └── parser/             # Guitar Pro parser (stub)
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── docs/
│   ├── PRD.md                  # SRS v1.1
│   ├── SPRINT_PLAN.md          # Plan de 6 sprints
│   └── DESIGN.md
├── package.json
├── tsconfig.json
├── AGENTS.md                   # Instructions for AI agents
└── README.md
```

---

## Formatos soportados

### Audio de entrada
| Formato | Extensiones |
|---|---|
| MP3 | `.mp3` |
| WAV (PCM) | `.wav` |
| FLAC | `.flac` |
| OGG Vorbis | `.ogg` |

### Tabs de entrada
| Formato | Extensiones | Notas |
|---|---|---|
| Guitar Pro 4/5 | `.gp`, `.gp5` | Se extrae la pista de bajo de 4 cuerdas |
| Guitar Pro X | `.gpx` | Se extrae la pista de bajo de 4 cuerdas |
| Bassical JSON | `.bassical.json` | Formato interno, esquema versionado |

### Salidas
| Formato | Uso |
|---|---|
| PDF | Exportación visual de la tablatura |
| `.bassical.json` | Respaldo y transferencia entre equipos |
| ZIP | Respaldo completo de la biblioteca (tabs + metadatos) |

---

## Atajos de teclado

| Acción | Atajo |
|---|---|
| Play / Pause | `Espacio` |
| Pulsación de calibración | `T` |
| Reducir velocidad 5 % | `Ctrl` + `←` |
| Aumentar velocidad 5 % | `Ctrl` + `→` |
| Compás anterior | `←` |
| Compás siguiente | `→` |
| Activar / desactivar loop | `L` |
| Activar / desactivar metrónomo | `M` |
| Deshacer | `Ctrl` + `Z` |
| Rehacer | `Ctrl` + `Shift` + `Z` |
| Respaldo manual | `Ctrl` + `S` |

---

## Persistencia de datos

Bassical almacena todos sus datos en `%APPDATA%\Bassical\`:

```
%APPDATA%\Bassical\
├── library.json          # Índice de canciones registradas
├── songs\
│   ├── <uuid>.bassical.json   # Tab + timing points de cada canción
│   └── ...
└── config.json           # Preferencias de audio y atajos
```

**Principios de persistencia:**
- Guardado automático después de cada cambio — no hay botón "Guardar".
- Los archivos de audio **nunca** se copian ni modifican. Solo se almacena la ruta de referencia.
- Si un archivo de audio es movido, Bassical lo notifica y permite reasignar la ruta.
- El usuario puede exportar toda la biblioteca como un archivo ZIP para respaldo o migración.

### Esquema del formato `.bassical.json`

```json
{
  "schema_version": 1,
  "id": "a3f2e1b0-...",
  "title": "My generation",
  "artist": "The Who",
  "audio_path": "C:\\Users\\...\\music\\my_generation.mp3",
  "timing_points": [
    { "offset_ms": 1240, "bpm": 132.0 },
    { "offset_ms": 48300, "bpm": 134.5 }
  ],
  "tab": {
    "strings": 4,
    "tuning": ["E", "A", "D", "G"],
    "measures": [
      {
        "time_signature": [4, 4],
        "beats": [
          {
            "duration": "quarter",
            "notes": [{ "string": 3, "fret": 3, "technique": null }]
          }
        ]
      }
    ]
  },
  "practice": {
    "preferred_speed": 0.85,
    "last_position_ms": 42000
  }
}
```

---

## Hoja de ruta

### v1.0 — Core (en desarrollo)
- [x] Definición de requerimientos (SRS v1.1)
- [x] Diseño arquitectónico
- [x] Prototipo: reproducción de audio local vía Rust + Tauri
- [ ] Calibrador de timing points (tap + waveform)
- [ ] Editor de tablatura básico
- [ ] Play-along sincronizado con cursor
- [x] Control de velocidad con pitch-shifting (SoundTouch)
- [ ] Loop de sección
- [x] Persistencia local en AppData
- [ ] Importación de Guitar Pro (.gp / .gp5 / .gpx)
- [ ] Exportación a PDF y JSON
- [ ] Instalador `.exe` para Windows

### v1.1 — Pulido
- [ ] Metrónomo visual y audible
- [ ] Undo/redo completo (50 pasos)
- [ ] Respaldo completo como ZIP
- [ ] Onboarding del calibrador (primera vez)
- [ ] Soporte ASIO para menor latencia

### v2.0 — Futuro (sin fecha)
- [ ] Bajo de 5 cuerdas
- [ ] Detección de afinación vía micrófono
- [ ] Port a macOS y Linux

---

## Fuera del alcance (v1.0)

Los siguientes elementos están explícitamente excluidos de la versión 1.0:

- Autenticación de usuarios o sincronización en la nube
- Versión web o móvil (Android / iOS)
- Bajo de 5 o 6 cuerdas
- Soporte para otros instrumentos (guitarra, ukulele, etc.)
- Transcripción automática de audio a tablatura
- Catálogo público de tabs de terceros
- Modo multijugador o jam session en red
- Editor de partituras en pentagrama convencional
- Gamificación o sistema de lecciones guiadas

---

## Documentación técnica

| Documento | Descripción |
|---|---|
| [`docs/SRS_v1.1.docx`](docs/SRS_v1.1.docx) | Especificación de Requerimientos de Software |
| `docs/ARCHITECTURE.md` | Diseño arquitectónico detallado *(próximamente)* |
| `docs/SCHEMA.md` | Especificación del formato `.bassical.json` *(próximamente)* |

---

## Licencia

MIT © 2026 Vor7ex

---

> Bassical no está afiliado con Songsterr, Guitar Pro ni ningún otro producto mencionado en esta documentación. Todos los formatos de terceros se soportan mediante ingeniería inversa o especificaciones públicas.
