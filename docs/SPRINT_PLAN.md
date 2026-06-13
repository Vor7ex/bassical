# Bassical - Plan de 6 Sprints para v1.0

## Visión General

Plan de desarrollo para Bassical v1.0, una aplicación desktop para práctica de bajo eléctrico. El proyecto parte de la inicialización de Tauri y avanza hacia una aplicación completa en 6 sprints, cada uno con un entregable tangible.

---

## Sprint 1: Fundamentos y Estructura *(~2 semanas)*

**Objetivo**: Base sólida del proyecto con comunicación IPC funcional

| Tarea | Detalle |
|-------|---------|
| Configurar frontend | Elegir framework (React o Svelte), Tailwind CSS, Zustand |
| Estructura de directorios | Implementar estructura según README (`src/`, `src-tauri/src/commands/`, etc.) |
| Modelo de datos | Definir esquema JSON versionado (`schema_version`, `library.json`, `config.json`) |
| IPC de prueba | Crear primer comando `#[tauri::command]` y llamarlo desde frontend con `invoke()` |
| Persistencia base | Módulo de lectura/escritura en `%APPDATA%\Bassical\` |

**Entregable**: App Tauri que abre, muestra UI básica, y guarda/lee un archivo JSON en AppData.

### Criterios de Aceptación
- [ ] Frontend configurado con framework elegido + Tailwind + Zustand
- [ ] Estructura de directorios implementada según documentación
- [ ] Esquema JSON definido con `schema_version`
- [ ] Comando IPC de prueba funcional (frontend → backend → frontend)
- [ ] Persistencia en `%APPDATA%\Bassical\` funcionando

---

## Sprint 2: Biblioteca de Canciones *(~2 semanas)*

**Objetivo**: CRUD completo de biblioteca (RF-01, RF-06)

| Tarea | Detalle |
|-------|---------|
| CRUD backend | Comandos: registrar, listar, eliminar, buscar canciones |
| UI de biblioteca | Lista con búsqueda/filtros, botón nueva canción |
| Validación de archivos | Detectar si el audio fue movido, permitir reasignar ruta (RF-01.3) |
| Guardado automático | Persistencia sin botón "Guardar" (RF-06.2) |

**Entregable**: El usuario puede agregar canciones, ver su biblioteca, buscar, y los datos persisten entre sesiones.

### Criterios de Aceptación
- [ ] RF-01.1: Registrar canción desde archivo de audio local
- [ ] RF-01.2: Almacenar título, artista, ruta, timing points, tab, metadatos de práctica
- [ ] RF-01.3: Notificar y permitir reasignar ruta de audio faltante
- [ ] RF-01.4: Eliminar entradas de la biblioteca sin borrar audio original
- [ ] RF-01.5: Buscar y filtrar por título o artista
- [ ] RF-06.2: Guardado automático entre sesiones

---

## Sprint 3: Audio Engine y Waveform *(~2.5 semanas)*

**Objetivo**: Reproducción de audio y visualización (base para calibración)

| Tarea | Detalle |
|-------|---------|
| Motor de audio | `cpal` para reproducción con baja latencia |
| Soporte de formatos | MP3, WAV, FLAC, OGG |
| Waveform view | Renderizar forma de onda con Canvas API |
| Controles básicos | Play, pause, seek |
| Pitch-shifting | Implementar `rubato` para cambio de velocidad sin cambio de tono (RF-05.2) |

**Entregable**: Reproducción de audio local con visualización de waveform y control de velocidad (25%-100%).

### Criterios de Aceptación
- [ ] Reproducir archivos MP3, WAV, FLAC, OGG
- [ ] Latencia de reproducción aceptable (< 50ms)
- [ ] Waveform renderizada correctamente en Canvas
- [ ] RF-05.2: Control de velocidad entre 25%-100% en incrementos de 5%
- [ ] RF-05.2: Sin cambio de tono al variar velocidad (pitch-shifting)
- [ ] RNF-01.2: Sin artefactos perceptibles en rango 50%-100%

---

## Sprint 4: Calibración de Tempo *(~2.5 semanas)*

**Objetivo**: Sistema de timing points completo (RF-02) - **Módulo diferenciador**

| Tarea | Detalle |
|-------|---------|
| Calibración asistida | Captura de tecla `T`, cálculo de BPM promedio y offset (RF-02.2) |
| Timing points múltiples | Agregar, editar, eliminar timing points individualmente (RF-02.5) |
| Marcadores sobre waveform | Líneas verticales arrastrables (RF-02.3) |
| Cuadrícula de beats | Visualización de beats derivada de timing points (RF-02.4) |
| Ajuste fino | Input numérico en ms + drag sobre waveform |
| Feedback visual | BPM detectado en tiempo real durante calibración |

**Entregable**: Calibración completa por pulsación de tecla con visualización y ajuste manual sobre waveform.

### Criterios de Aceptación
- [ ] RF-02.1: Definir uno o múltiples timing points (offset ms, BPM)
- [ ] RF-02.2: Calibración asistida con pulsación de tecla T
- [ ] RF-02.3: Waveform con timing points superpuestos como marcadores
- [ ] RF-02.3: Ajuste de offset por arrastre o input numérico
- [ ] RF-02.4: Cuadrícula de beats derivada de timing points
- [ ] RF-02.5: CRUD individual de timing points
- [ ] RF-02.6: Actualización en tiempo real sobre waveform
- [ ] RNF-01.1: Latencia de pulsación ≤ 10 ms
- [ ] Error de alineación ≤ 5 ms con canciones de referencia

---

## Sprint 5: Editor de Tablatura + Play-along *(~3 semanas)*

**Objetivo**: Editor completo y modo play-along sincronizado (RF-03, RF-05)

| Tarea | Detalle |
|-------|---------|
| Editor de tab | 4 cuerdas, entrada por clic + número de traste (RF-03.1, RF-03.2) |
| Valores rítmicos | Redonda, blanca, negra, corchea, semicorchea + puntillo (RF-03.3) |
| Técnicas | Hammer-on, pull-off, slide, vibrato, mute, ghost note (RF-03.5) |
| Gestión de compases | Insertar/eliminar compases (RF-03.4) |
| Undo/redo | Hasta 50 pasos (RF-03.7) |
| Play-along sincronizado | Cursor animado sobre tab sincronizado con audio (RF-05.1) |
| Loop de sección | Definir rango de compases en bucle (RF-05.5) |
| Metrónomo | Visual y audible, sincronizado con timing points (RF-05.6) |
| Navegación | Clic en compás salta a posición en audio (RF-05.4) |

**Entregable**: Editor de tablatura completo + modo play-along con cursor, loop y metrónomo.

### Criterios de Aceptación
- [ ] RF-03.1: 4 líneas horizontales (G, D, A, E) con números de traste
- [ ] RF-03.2: Entrada por clic en celda + escritura de número de traste
- [ ] RF-03.3: Valores rítmicos (redonda, blanca, negra, corchea, semicorchea + puntillo)
- [ ] RF-03.4: Insertar y eliminar compases en cualquier posición
- [ ] RF-03.5: Técnicas: hammer-on (h), pull-off (p), slide (/ \), vibrato (~), mute (x), ghost ()
- [ ] RF-03.6: Duración real derivada de BPM activo
- [ ] RF-03.7: Undo/redo hasta 50 acciones
- [ ] RF-05.1: Cursor animado sincronizado con audio
- [ ] RF-05.4: Saltar a cualquier compás por clic
- [ ] RF-05.5: Loop de sección en bucle
- [ ] RF-05.6: Metrónomo visual y audible sincronizado

---

## Sprint 6: Import/Export y Pulido Final *(~2 semanas)*

**Objetivo**: Funcionalidades de intercambio y preparación para release (RF-04)

| Tarea | Detalle |
|-------|---------|
| Importar Guitar Pro | Parser para `.gp`, `.gp5`, `.gpx` - extraer pista de bajo 4 cuerdas (RF-04.1) |
| Exportar PDF | Renderizado visual de tablatura a PDF (RF-04.2) |
| Exportar/Importar JSON | Formato `.bassical.json` versionado (RF-04.3, RF-04.4) |
| Respaldo ZIP | Exportar biblioteca completa (RF-06.3) |
| Onboarding | Guía in-app para primera vez usando calibrador (RNF-02.2) |
| Pruebas de aceptación | Validar todos los criterios del PRD sección 8 |
| Build de producción | Generar instalador `.exe` autocontenido |

**Entregable**: Bassical v1.0 completa con importación Guitar Pro, exportación, y instalador Windows.

### Criterios de Aceptación
- [ ] RF-04.1: Importar Guitar Pro (.gp, .gp5, .gpx) extrayendo pista de bajo
- [ ] RF-04.2: Exportar tab como PDF con notación visual estándar
- [ ] RF-04.3: Exportar/Importar JSON versionado
- [ ] RF-04.4: Compartir tabs entre usuarios mediante JSON
- [ ] RF-06.3: Respaldo completo de biblioteca como ZIP
- [ ] RNF-02.2: Onboarding in-app del calibrador
- [ ] RNF-03.1: Ejecutar en Windows 10/11 sin dependencias adicionales
- [ ] RNF-03.2: Instalador `.exe` autocontenido
- [ ] RNF-03.3: Funcionar en resoluciones 1366×768 hasta 4K

---

## Resumen de Cobertura

| Sprint | Requerimientos Cubiertos | Prioridad PRD |
|--------|-------------------------|---------------|
| 1 | Estructura, IPC, Persistencia base | - |
| 2 | RF-01 (Biblioteca), RF-06 (Persistencia) | Alta |
| 3 | RF-05.2 (Velocidad), base para RF-02 | Alta |
| 4 | RF-02 (Calibración) | Alta |
| 5 | RF-03 (Editor), RF-05 (Play-along) | Alta |
| 6 | RF-04 (Import/Export), pulido | Media |

---

## Stack Tecnológico Definido

### Frontend
| Tecnología | Rol |
|---|---|
| TypeScript | Lenguaje principal del frontend |
| React o Svelte | Framework de UI (a definir en Sprint 1) |
| Tailwind CSS | Estilos |
| Zustand | Gestión de estado global |
| Canvas API | Renderizado de waveform y tablatura |

### Backend
| Tecnología | Rol |
|---|---|
| Rust | Lenguaje del backend |
| Tauri 2 | Framework desktop (bridge IPC Rust ↔ frontend) |
| `cpal` | Acceso al hardware de audio (baja latencia) |
| `rubato` | Pitch-shifting independiente del tempo |
| `serde` / `serde_json` | Serialización de tabs y biblioteca |

---

## Dependencias Técnicas

```
Sprint 1 (Base)
    ↓
Sprint 2 (Biblioteca)
    ↓
Sprint 3 (Audio + Waveform)
    ↓
Sprint 4 (Calibración) ← requiere Audio + Waveform
    ↓
Sprint 5 (Editor + Play-along) ← requiere Calibración + Audio
    ↓
Sprint 6 (Import/Export + Pulido) ← requiere Editor completo
```

---

## Notas de Implementación

1. **Framework Frontend**: Definir en Sprint 1. React tiene más ecosistema; Svelte es más ligero.
2. **Parser Guitar Pro**: Puede ser el componente más complejo. Considerar usar crates Rust existentes o implementar parsing incremental.
3. **Pitch-shifting**: `rubato` es la librería recomendada. Verificar calidad en el rango 25%-50%.
4. **Latencia de calibración**: El requisito de ≤ 10ms es estricto. Evitar bloques en el thread principal de Rust.
5. **Persistencia**: Usar `serde_json` con atomic writes para evitar corrupción de datos.

---

## Criterios de Aceptación Finales (PRD Sección 8)

La aplicación se considerará completa para v1.0 cuando:

- [ ] Todos los RF de prioridad Alta (RF-01 a RF-05) pasan pruebas de aceptación
- [ ] Calibración con error ≤ 5 ms sobre canciones de referencia
- [ ] Reproducción a 50% sin artefactos perceptibles (5 archivos de prueba)
- [ ] Instalación y ejecución sin errores en Windows 10 y 11
- [ ] Sin funcionalidad que requiera internet, cuenta o pago
- [ ] Latencia de tap ≤ 10 ms en dispositivos de prueba
- [ ] RAM ≤ 300 MB durante sesión de 30 minutos
- [ ] Archivos de audio nunca modificados

---

## Fuera del Alcance (v1.0)

- Autenticación / nube
- Web / móvil
- Bajo de 5/6 cuerdas
- Otros instrumentos
- Transcripción automática
- Catálogo de tabs de terceros
- Multijugador
- Partituras convencionales
- Detección de afinación
- Gamificación

---

*Documento generado: Junio 2026*
*Autor: Juan (Vor7ex)*
*Licencia: MIT*
