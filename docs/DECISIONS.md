# Decisiones de Arquitectura — Bassical

Registro de Decisiones Arquitectónicas (ADR) para Bassical. Cada entrada documenta una decisión tomada, su contexto, justificación y consecuencias.

*Documento generado: Junio 2026 — Sprint 4 (Calibración de Tempo).*

---

## ADR-001 — Time signature por timing point (default 4/4)

**Fecha**: Sprint 4 · **Estado**: Aceptada

### Contexto
El PRD (RF-02) define timing points como pares `(offset_ms, BPM)` sin incluir el compás/time signature. El usuario requiere poder ajustar los beats por barra y la longitud del compás según el estándar de teoría musical (3/4, 4/4, 6/8, etc.) para que la cuadrícula de beats y el metrónomo respeten la métrica real de la sección.

### Decisión
Cada `TimingPoint` lleva un campo opcional `timeSignature: { numerator, denominator }`. Cuando está ausente, se aplica el default **4/4**. Esto permite cambios de compás mid-canción (estilo osu! meter), coherente con el modelo de timing points variables por sección.

### Consecuencias
- El modelo `TimingPoint` pasa de `{ offsetMs, bpm }` a `{ offsetMs, bpm, timeSignature? }`.
- El cálculo de la duración de un compás: `barMs = numerator * 60000 / bpm` (el BPM se interpreta como *notas-del-denominador por minuto*: 4/4 → negras/min, 6/8 → corcheas/min).
- La cuadrícula de beats (Sprint 4) y el metrónomo (Sprint 5) leen `timeSignature` de cada timing point activo.
- El formato `.bassical.json` (ver ADR-003) incluye `timeSignature` solo cuando difiere del default, vía `skip_serializing_if`.

### Alternativas descartadas
- **Time signature global por canción**: no soporta cambios de compás, limita el modelo variable.
- **Default global + override por TP**: máxima flexibilidad pero duplica estado y complica la UI/persistencia.

---

## ADR-002 — Metrónomo audible diferido a Sprint 5

**Fecha**: Sprint 4 · **Estado**: Aceptada

### Contexto
El SPRINT_PLAN original sitúa el metrónomo visual+audible (RF-05.6) en Sprint 5, no en Sprint 4. La cuadrícula de beats (RF-02.4) sí es de Sprint 4. Implementar el metrónomo audible requiere mezclar clicks en el callback de `cpal` con conocimiento de los timing points (cambios de tempo), lo que añade riesgo y scope al sprint de calibración.

### Decisión
El **metrónomo audible se diferencia a Sprint 5**, como prevé el SPRINT_PLAN. La **cuadrícula visual de beats/barras** (RF-02.4) sí se implementa en Sprint 4 — cumple el espíritu de validación visual del alineamiento. Sprint 4 deja la matemática de beats en un módulo reutilizable (TS primero; se porta a Rust en Sprint 5 para el click audible sample-accurate).

### Consecuencias
- Sprint 4 no emite clicks de audio.
- Sprint 4 entrega la función `computeBeatGrid(timingPoints, viewport)` reusable por el metrónomo de Sprint 5.
- El atajo `M` (activar/desactivar metrónomo) queda reservado; no se implementa en Sprint 4.

---

## ADR-003 — Archivo único `.bassical.json` con tab/practice opcionales

**Fecha**: Sprint 4 · **Estado**: Aceptada

### Contexto
El README prevé `songs/<uuid>.bassical.json` conteniendo `tab` + `timing_points` + `practice`. La tablatura se implementa en Sprint 5, no en Sprint 4. Separar la calibración en `calibration.json` generaría fragmentación y deuda técnica: al llegar Sprint 5 habría que fusionar dos archivos o mantener lógica dual.

### Decisión
Se usa **un único archivo `songs/<uuid>.bassical.json` desde Sprint 4**, con el esquema versionado (`schema_version: 1`). Los campos `tab` y `practice` son **opcionales** y se **omiten por completo del JSON** cuando no existen, usando `#[serde(skip_serializing_if = "Option::is_none")]` en Rust y `undefined`→omitido en TS. Esto evita escribir campos nulos/vacíos y mantiene el esquema limpio.

### Consecuencias
- Estructura persistida en Sprint 4:
  ```json
  {
    "schemaVersion": 1,
    "id": "<uuid>",
    "title": "...",
    "artist": "..." | omitido,
    "audioPath": "...",
    "timingPoints": [ { "offsetMs": 1240, "bpm": 132.0, "timeSignature": { "numerator": 4, "denominator": 4 } } ]
  }
  ```
  Cuando el timing point use el default 4/4, `timeSignature` también se omite.
- Sprint 5 añadirá `tab` y `practice` al mismo archivo sin migración.
- `Song.hasCalibration` y `Song.bpm` se actualizan en `library.json` al guardar calibración (índice para la biblioteca).

### Alternativas descartadas
- **`calibration.json` dedicado**: fragmentación + deuda de fusión en Sprint 5.
- **`bassical.json` con `tab: null` forzado**: ensucia el esquema y acopla al modelo de tab antes de definirlo.

---

## ADR-004 — Tap: compensación dinámica de latencia, dominio del reloj de audio

**Fecha**: Sprint 4 · **Estado**: Aceptada

### Contexto
RNF-01.1 exige latencia de tap ≤ 10 ms y error de alineación ≤ 5 ms. El motor actual:
- Polea la posición de audio a 100 ms desde el frontend (`usePracticePlayback.ts:143`) — inviable para tap.
- En modo full-buffer, la posición reportada **adelanta el audio audible** por el prefill de SoundTouch (`PREFILL_CHUNKS=4` × 4096 frames ≈ 340 ms a 48 kHz).
- El `AudioEngine` es un singleton con un stream `cpal`; los comandos actuales bloquean `Mutex<AudioEngine>` (contención con el hilo de UI).

### Decisión
El tap se registra en el **dominio del reloj de audio**, no del reloj del sistema (SystemTime). Se implementa un `CalibrationState` de **solo variables atómicas** (sin mutex), actualizado por el callback de audio cada bloque, con dos rutas de cálculo de la posición audible real:

**Ruta directa (speed = 1.0, sin SoundTouch)**:
```
posicionReal = posicionAtomica
              - (output_buffer_frames / sample_rate * 1000)
              - (ring_buffer_samples / sample_rate * 1000)
              - latencia_os_ms
```
A 1.0× se **bypassa el pipeline de SoundTouch** (lectura directa del buffer), eliminando su latencia.

**Ruta DSP (speed ≠ 1.0, con SoundTouch)**:
```
retrasoTotal = (samples_en_soundtouch + samples_en_ring_buffer + output_buffer_frames)
               / sample_rate * 1000
             + latencia_os_ms
posicionReal = posicionAtomica - retrasoTotal
```
Se consultan dinámicamente los samples atrapados en los buffers internos de SoundTouch (`numUnprocessedSamples`) y en el ring buffer de salida, no una constante mágica.

El comando `record_calibration_tap()` lee **solo atómicos** — sin `Mutex<AudioEngine>`, sin contención con el hilo de UI — y devuelve la posición de audio calibrada. El frontend no usa SystemTime para el tap; el timestamp del tap es el estado atómico del hilo de audio en ese instante.

### Consecuencias
- Nuevo `CalibrationState` (atomics) gestionado por separado del `PlaybackState` pesado.
- El callback de audio actualiza `soundtouch_unprocessed`, `ring_buffer_samples`, `output_buffer_frames`, `sample_rate`, `speed`, `is_full_buffer`, `read_position` cada bloque.
- Se añade un bypass de SoundTouch a speed = 1.0 en full-buffer (o se mide/corrije su prefill).
- `latencia_os_ms` se estima del `buffer_size` de cpal (configurable; cpal no expone la latencia del DAC de forma portable en Windows).
- El mapeo tap→posición es robusto a drift entre relojes (no hay dos relojes; solo el de audio).

### Alternativas descartadas
- **Compensación con constante mágica (340 ms)**: válida solo a sample rate fijo y prefill fijo; se rompe al cambiar buffer size o sample rate.
- **Calibrar solo en modo streaming a 1×**: exacto pero impide calibrar a velocidad reducida (caso de uso real para pasajes rápidos).
- **Timestamp de SystemTime para el tap**: deriva frente al reloj del DAC; viola el requisito de ≤5 ms de error.
