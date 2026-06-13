# Bassical — Historias de Usuario v1.0

>Documento generado: Junio 2026
>Autor: Juan (Vor7ex)

---

## US-01: Gestión de Biblioteca y Carga de Archivos

**Historia:**
Como usuario, quiero registrar una canción seleccionando un archivo de audio local (MP3, WAV, FLAC, OGG) sin que este sea copiado o modificado, para construir mi catálogo de práctica.

**Prioridad:** Alta
**Sprint:** 2
**Requerimientos asociados:** RF-01, RF-06

### Criterios de Aceptación

| # | Criterio | Referencia PRD |
|---|----------|----------------|
| CA-01.1 | El sistema debe almacenar la ruta del archivo, título y artista | RF-01.1, RF-01.2 |
| CA-01.2 | Debe existir una lista de canciones con capacidad de búsqueda y filtrado por título o artista | RF-01.5 |
| CA-01.3 | Si el archivo original es movido o eliminado, la interfaz debe notificarlo y permitir reasignar la ruta | RF-01.3 |
| CA-01.4 | El usuario puede eliminar entradas de la biblioteca; la eliminación nunca borra el archivo de audio original | RF-01.4 |
| CA-01.5 | Los datos persisten entre sesiones sin acción explícita del usuario (guardado automático) | RF-06.2 |

### Mockup

Ver: [`mockups/01-library.html`](mockups/01-library.html)

---

## US-02: Calibrador de Tempo Asistido (Tap)

**Historia:**
Como bajista, quiero reproducir el audio y presionar la tecla 'T' al ritmo de la música, para que el sistema calcule automáticamente el BPM promedio y el offset inicial de la canción.

**Prioridad:** Alta
**Sprint:** 4
**Requerimientos asociados:** RF-02

### Criterios de Aceptación

| # | Criterio | Referencia PRD |
|---|----------|----------------|
| CA-02.1 | La vista debe mostrar la forma de onda (waveform) del audio a pantalla completa | RF-02.3 |
| CA-02.2 | Debe existir un feedback visual en tiempo real del BPM detectado tras cada pulsación | RF-02.2 |
| CA-02.3 | La latencia entre la pulsación y el registro en el calibrador debe ser inferior a 10 ms | RNF-01.1 |
| CA-02.4 | El sistema calcula el BPM promedio del segmento y el offset del primer beat | RF-02.2 |
| CA-02.5 | El resultado se visualiza como un marcador vertical sobre la waveform | RF-02.3 |

### Mockup

Ver: [`mockups/02-calibrator.html`](mockups/02-calibrator.html)

---

## US-03: Editor de Tablatura Básico

**Historia:**
Como usuario, quiero ingresar números de trastes (0-24) sobre una cuadrícula de cuatro líneas que representan las cuerdas del bajo (G, D, A, E), para transcribir secciones de bajo.

**Prioridad:** Alta
**Sprint:** 5
**Requerimientos asociados:** RF-03

### Criterios de Aceptación

| # | Criterio | Referencia PRD |
|---|----------|----------------|
| CA-03.1 | El ingreso de notas se realiza mediante clic en la celda y teclado numérico | RF-03.2 |
| CA-03.2 | El editor debe tener herramientas para asignar valores rítmicos (redonda a semicorchea) | RF-03.3 |
| CA-03.3 | El editor debe soportar técnicas de bajo: hammer-on, pull-off, slide, vibrato, mute, ghost note | RF-03.5 |
| CA-03.4 | El sistema debe permitir deshacer/rehacer hasta 50 acciones consecutivas | RF-03.7 |
| CA-03.5 | El usuario debe poder insertar y eliminar compases en cualquier posición | RF-03.4 |

### Mockup

Ver: [`mockups/03-tab-editor.html`](mockups/03-tab-editor.html)

---

## US-04: Práctica Play-Along y Control de Velocidad

**Historia:**
Como bajista en práctica, quiero reproducir el audio sincronizado con la tablatura y ajustar la velocidad entre 25% y 100%, para estudiar pasajes complejos sin alterar el tono original (pitch-shifting).

**Prioridad:** Alta
**Sprint:** 5
**Requerimientos asociados:** RF-05

### Criterios de Aceptación

| # | Criterio | Referencia PRD |
|---|----------|----------------|
| CA-04.1 | Un cursor animado debe avanzar por la tablatura al ritmo dictado por los timing points | RF-05.1 |
| CA-04.2 | Los controles de velocidad, metrónomo y loop de sección deben estar accesibles en la misma vista de lectura | RNF-02.3 |
| CA-04.3 | Al cambiar la velocidad, la sincronización visual y auditiva debe mantenerse escalarmente | RF-05.3 |
| CA-04.4 | El usuario puede saltar a cualquier compás haciendo clic sobre él | RF-05.4 |
| CA-04.5 | El metrónomo visual y audible está sincronizado con los timing points | RF-05.6 |

### Mockup

Ver: [`mockups/04-play-along.html`](mockups/04-play-along.html)

---

## US-05: Ajuste Fino de Timing Points

**Historia:**
Como usuario avanzado, quiero visualizar y editar manualmente los timing points como marcadores superpuestos en la forma de onda, para corregir fluctuaciones de tempo en la canción original.

**Prioridad:** Alta
**Sprint:** 4
**Requerimientos asociados:** RF-02

### Criterios de Aceptación

| # | Criterio | Referencia PRD |
|---|----------|----------------|
| CA-05.1 | El usuario puede arrastrar los marcadores verticales o ingresar valores exactos en milisegundos | RF-02.3 |
| CA-05.2 | El sistema debe renderizar y actualizar en tiempo real una cuadrícula de beats sobre la waveform basándose en los timing points activos | RF-02.4 |
| CA-05.3 | La modificación de un timing point no debe desalinear los timing points definidos previamente | RF-02.5 |
| CA-05.4 | El usuario puede agregar, editar y eliminar timing points individualmente | RF-02.5 |
| CA-05.5 | Cualquier cambio se actualiza en tiempo real sobre la cuadrícula | RF-02.6 |

### Mockup

Ver: [`mockups/05-timing-points.html`](mockups/05-timing-points.html)

---

## Matriz de Trazabilidad

| Historia | Sprint | RF asociados | Prioridad |
|----------|--------|--------------|-----------|
| US-01 Biblioteca | 2 | RF-01, RF-06 | Alta |
| US-02 Calibrador | 4 | RF-02 | Alta |
| US-03 Editor Tab | 5 | RF-03 | Alta |
| US-04 Play-Along | 5 | RF-05 | Alta |
| US-05 Timing Points | 4 | RF-02 | Alta |
