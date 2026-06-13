**BASSICAL**

*Bass Learning & Tab Studio*

**Especificaci˜n de Requerimientos de Software (SRS)**

Versi˜n 1.1 ˜ Junio 2026

*Revisi˜n: eliminados m˜dulo de autenticaci˜n y sincronizaci˜n en nube.*

*Plataforma redefinida: aplicaci˜n desktop Windows, ejecuci˜n 100% local.*

Autor:

**Juan**

*Proyecto personal ˜ Uso y distribuci˜n abierta*

# **1. Introducci˜n**

## **1.1 Prop˜sito**

Este documento define los requerimientos funcionales y no funcionales de Bassical, una aplicaci˜n de escritorio para Windows orientada al aprendizaje y pr˜ctica del bajo el˜ctrico. Sirve como referencia base para las fases de dise˜o, implementaci˜n y validaci˜n.

## **1.2 Alcance**

Bassical es una aplicaci˜n desktop local para Windows. Permite al usuario cargar canciones desde su sistema de archivos, calibrar el tempo con precisi˜n variable por secci˜n, crear o importar tablatura de bajo en notaci˜n est˜ndar de 4 cuerdas, y practicar en modo play-along con control de velocidad. Toda la informaci˜n se almacena localmente. No requiere conexi˜n a internet, cuenta de usuario ni servicios externos.

## **1.3 Producto de referencia y diferenciaci˜n**

El producto de referencia es Songsterr (songsterr.com). Bassical se diferencia en tres ejes:

* Autonom˜a de contenido: el usuario crea y gestiona su propio cat˜logo de tabs.
* Acceso completo sin pago: todas las funciones (incluyendo control de velocidad y calibraci˜n) disponibles sin restricciones.
* Calibraci˜n de tempo avanzada: sistema de timing points variables por secci˜n, similar al editor de beatmaps de osu!.

## **1.4 Decisiones de plataforma**

Bassical se distribuye como aplicaci˜n desktop nativa para Windows por las siguientes razones t˜cnicas y de producto:

* El sistema de calibraci˜n por pulsaci˜n de tecla requiere latencia de input < 10 ms, garant˜a que los navegadores web no pueden ofrecer de forma confiable debido al event loop de JavaScript.
* El acceso al sistema de archivos local (lectura de audio, escritura de datos) es m˜s natural y eficiente en una aplicaci˜n nativa que en un contexto web.
* La ejecuci˜n completamente local, sin backend, elimina la necesidad de infraestructura de servidor y garantiza privacidad total al usuario.

## **1.5 Definiciones y acr˜nimos**

| **T˜rmino** | **Definici˜n** |
| --- | --- |
| BPM | Beats Per Minute. Unidad de medida del tempo musical. |
| Tab / Tablatura | Notaci˜n para bajo que indica qu˜ traste presionar en cada cuerda, sin requerir lectura de partitura convencional. |
| Offset | Retardo en milisegundos desde el inicio del audio hasta el primer beat del comp˜s 1. |
| Timing point | Par (offset ms, BPM) que define el tempo a partir de un instante dado. Una canci˜n puede tener m˜ltiples timing points. |
| Play-along | Modo de pr˜ctica donde el usuario toca el bajo siguiendo la tab mientras el audio de referencia se reproduce simult˜neamente. |
| Pitch-shifting | Alteraci˜n de la velocidad de reproducci˜n sin modificar el tono percibido. |
| AppData | Directorio del sistema Windows (%APPDATA%) donde Bassical almacena todos sus datos locales. |

## **1.6 Referencias**

* IEEE Std 830-1998: Recommended Practice for Software Requirements Specifications.
* osu! editor de beatmaps ˜ modelo de referencia para el sistema de timing points.
* Songsterr.com ˜ referencia para la experiencia play-along.

# **2. Descripci˜n General del Sistema**

## **2.1 Perspectiva del producto**

Bassical es una aplicaci˜n desktop autocontenida. No depende de ning˜n servicio externo para su operaci˜n. Todos los datos (biblioteca, tabs, configuraci˜n) se persisten en el directorio AppData del usuario en Windows. Los archivos de audio permanecen en su ubicaci˜n original en el sistema de archivos; Bassical solo almacena la ruta de referencia.

## **2.2 Usuarios objetivo**

| **Perfil** | **Caracter˜sticas** | **Necesidad principal** |
| --- | --- | --- |
| Bajista intermedio | Conoce teor˜a b˜sica, lee tabs, practica canciones completas | Herramienta de pr˜ctica con velocidad ajustable y tabs propias |
| Bajista avanzado | Domina el instrumento, estudia canciones con cambios de tempo | Calibraci˜n precisa de timing variable y sincronizaci˜n exacta |

## **2.3 Supuestos y dependencias**

* El usuario dispone de archivos de audio en formatos est˜ndar (MP3, WAV, FLAC, OGG) almacenados localmente.
* El sistema operativo es Windows 10 de 64 bits o posterior.
* El dispositivo cuenta con teclado f˜sico para la calibraci˜n por pulsaci˜n.
* No se requiere conexi˜n a internet en ning˜n momento del uso normal de la aplicaci˜n.

## **2.4 Restricciones generales**

* Sin publicidad de ning˜n tipo.
* Sin funciones restringidas a versiones de pago.
* Sin conexi˜n a internet requerida para ninguna funcionalidad core.
* Sin cat˜logos de tabs de terceros integrados.
* El sistema no transcribe audio autom˜ticamente (fuera del alcance v1.0).
* Los archivos de audio nunca son copiados ni modificados por la aplicaci˜n.

# **3. Requerimientos Funcionales**

## **RF-01 ˜ Gesti˜n de biblioteca de canciones**

| **Atributo** | **Descripci˜n** |
| --- | --- |
| **RF-01.1** | El sistema debe permitir registrar una canci˜n seleccionando un archivo de audio local (MP3, WAV, FLAC, OGG). Solo se almacena la ruta; el archivo no se copia. |
| **RF-01.2** | Cada entrada de la biblioteca debe almacenar: t˜tulo, artista (opcional), ruta del archivo de audio, timing points, tab asociada y metadatos de pr˜ctica (velocidad preferida, ˜ltima posici˜n de reproducci˜n). |
| **RF-01.3** | Si el archivo de audio de una entrada ya no existe en la ruta registrada, el sistema debe notificarlo claramente al usuario y permitirle reasignar la ruta. |
| **RF-01.4** | El usuario debe poder eliminar entradas de la biblioteca. La eliminaci˜n nunca borra el archivo de audio original. |
| **RF-01.5** | El usuario debe poder buscar y filtrar entradas por t˜tulo o artista. |

## **RF-02 ˜ Calibraci˜n de tempo (Timing Points)**

M˜dulo diferenciador central de Bassical. Permite definir el tempo con precisi˜n de milisegundos en una o m˜ltiples secciones de la canci˜n.

| **Atributo** | **Descripci˜n** |
| --- | --- |
| **RF-02.1** | El usuario debe poder definir uno o m˜ltiples timing points. Cada timing point es un par (offset en ms, BPM) que rige el tempo a partir de ese instante. |
| **RF-02.2** | El sistema debe ofrecer modo de calibraci˜n asistida: el audio se reproduce y el usuario presiona la tecla T al ritmo de la canci˜n; el sistema registra cada pulsaci˜n, calcula el BPM promedio del segmento y el offset del primer beat. |
| **RF-02.3** | El sistema debe mostrar la forma de onda (waveform) del audio con los timing points superpuestos como marcadores verticales. El usuario puede ajustar el offset de cada timing point arrastrando su marcador o ingresando el valor num˜rico en milisegundos. |
| **RF-02.4** | El sistema debe renderizar una cuadr˜cula de beats sobre la waveform derivada de los timing points activos, para validaci˜n visual del alineamiento. |
| **RF-02.5** | El usuario debe poder agregar, editar y eliminar timing points individualmente sin afectar a los dem˜s. |
| **RF-02.6** | Cualquier cambio en un timing point debe actualizarse en tiempo real sobre la cuadr˜cula de la waveform. |

## **RF-03 ˜ Editor de tablatura**

| **Atributo** | **Descripci˜n** |
| --- | --- |
| **RF-03.1** | La tablatura se representa con cuatro l˜neas horizontales (cuerdas G, D, A, E de arriba a abajo) y n˜meros indicando el traste (0˜24). |
| **RF-03.2** | El usuario ingresa notas haciendo clic en la celda correspondiente y escribiendo el n˜mero de traste. |
| **RF-03.3** | El editor debe soportar valores r˜tmicos: redonda, blanca, negra, corchea, semicorchea y sus equivalentes con puntillo. |
| **RF-03.4** | El usuario debe poder insertar y eliminar compases en cualquier posici˜n de la tab. |
| **RF-03.5** | El editor debe soportar las siguientes t˜cnicas de bajo: hammer-on (h), pull-off (p), slide ascendente (/), slide descendente (\), vibrato (~), mute (x) y ghost note (entre par˜ntesis). |
| **RF-03.6** | La duraci˜n real en milisegundos de cada comp˜s se deriva autom˜ticamente del BPM activo seg˜n los timing points; el editor no permite editar este valor manualmente. |
| **RF-03.7** | El editor debe soportar deshacer y rehacer (undo/redo) hasta 50 acciones consecutivas. |

## **RF-04 ˜ Importaci˜n y exportaci˜n de tabs**

| **Atributo** | **Descripci˜n** |
| --- | --- |
| **RF-04.1** | El sistema debe permitir importar tabs en formato Guitar Pro (.gp, .gp5, .gpx). Solo se procesar˜ la pista de bajo de 4 cuerdas; las dem˜s pistas se ignorar˜n o ser˜n seleccionables por el usuario. |
| **RF-04.2** | El sistema debe permitir exportar la tab como PDF, conservando la notaci˜n visual est˜ndar de tablatura. |
| **RF-04.3** | El sistema debe permitir exportar e importar tabs en formato interno JSON (esquema Bassical versionado) para respaldo y transferencia entre equipos. |
| **RF-04.4** | El usuario debe poder compartir una tab entregando el archivo JSON exportado a otro usuario de Bassical, quien puede importarlo directamente. |

## **RF-05 ˜ Reproducci˜n y modo play-along**

| **Atributo** | **Descripci˜n** |
| --- | --- |
| **RF-05.1** | El sistema debe reproducir el audio local sincronizado con la tab, mostrando un cursor que avanza por la tablatura al ritmo de los timing points activos. |
| **RF-05.2** | El usuario debe poder ajustar la velocidad de reproducci˜n entre 25% y 100% del tempo original, en incrementos de 5%, sin alterar el tono percibido del audio (pitch-shifting independiente del tempo). |
| **RF-05.3** | Al modificar la velocidad, el cursor de la tab y la cuadr˜cula de beats escalan proporcionalmente para mantener la sincronizaci˜n con el audio. |
| **RF-05.4** | El usuario debe poder saltar a cualquier comp˜s haciendo clic sobre ˜l; el audio saltar˜ al instante correspondiente calculado desde los timing points. |
| **RF-05.5** | El sistema debe ofrecer modo loop de secci˜n: el usuario define un comp˜s de inicio y uno de fin; el sistema reproduce ese fragmento en bucle de forma continua. |
| **RF-05.6** | El sistema debe incluir un metr˜nomo visual y audible sincronizado con los timing points, activable y desactivable de forma independiente al audio. |

## **RF-06 ˜ Persistencia local**

Todo el estado de la aplicaci˜n se persiste localmente en el directorio %APPDATA%\Bassical\.

| **Atributo** | **Descripci˜n** |
| --- | --- |
| **RF-06.1** | La biblioteca, las tabs y la configuraci˜n de usuario se almacenan en archivos JSON estructurados dentro de %APPDATA%\Bassical\. |
| **RF-06.2** | Los datos deben persistir entre sesiones sin ninguna acci˜n expl˜cita del usuario (guardado autom˜tico). |
| **RF-06.3** | El usuario debe poder realizar un respaldo manual exportando toda su biblioteca (tabs + metadatos, sin archivos de audio) como un archivo ZIP. |
| **RF-06.4** | El sistema nunca debe modificar, mover ni eliminar los archivos de audio originales del usuario. |

# **4. Requerimientos No Funcionales**

## **RNF-01 ˜ Rendimiento**

| **ID** | **Requerimiento** | **Criterio de aceptaci˜n** |
| --- | --- | --- |
| RNF-01.1 | La latencia entre pulsaci˜n de tecla y registro en el calibrador no debe superar 10 ms. | Medido con herramientas de profiling en hardware de referencia (procesador de gama media, 2022+). |
| RNF-01.2 | El cambio de velocidad no debe generar artefactos de audio perceptibles en el rango 50%˜100%. | Evaluado subjetivamente por el autor y al menos dos bajistas de prueba. |
| RNF-01.3 | La carga de una canci˜n con tab (audio < 50 MB) debe completarse en menos de 3 segundos. | Medido en hardware de referencia con disco SSD. |
| RNF-01.4 | La aplicaci˜n no debe consumir m˜s de 300 MB de RAM en uso normal. | Medido con el monitor de recursos de Windows durante una sesi˜n de play-along de 30 minutos. |

## **RNF-02 ˜ Usabilidad**

| **ID** | **Requerimiento** | **Criterio de aceptaci˜n** |
| --- | --- | --- |
| RNF-02.1 | Un bajista intermedio sin experiencia previa con Bassical debe cargar una canci˜n, crear una tab b˜sica de 8 compases y reproducirla en play-along en menos de 15 minutos. | Prueba con al menos 3 usuarios. |
| RNF-02.2 | El calibrador debe incluir una gu˜a paso a paso in-app la primera vez que el usuario lo utiliza. | Verificado en prueba de usabilidad. |
| RNF-02.3 | Todos los controles de reproducci˜n (play, pause, velocidad, loop, metr˜nomo) deben ser accesibles sin abandonar la vista de play-along. | Inspecci˜n de UI. |

## **RNF-03 ˜ Compatibilidad de plataforma**

| **ID** | **Requerimiento** |
| --- | --- |
| RNF-03.1 | La aplicaci˜n debe ejecutarse en Windows 10 (64 bits) o posterior sin instalaci˜n de dependencias adicionales por parte del usuario. |
| RNF-03.2 | El instalador debe distribuirse como un ejecutable .exe aut˜nomo (sin requerir acceso de administrador para uso normal). |
| RNF-03.3 | La aplicaci˜n debe funcionar correctamente en resoluciones desde 1366˜768 hasta 4K, con escalado DPI autom˜tico. |

## **RNF-04 ˜ Privacidad y datos**

| **ID** | **Requerimiento** |
| --- | --- |
| RNF-04.1 | La aplicaci˜n no debe realizar ninguna conexi˜n de red durante su operaci˜n normal. |
| RNF-04.2 | No se debe integrar ning˜n SDK de telemetr˜a, anal˜tica o publicidad. |
| RNF-04.3 | Los archivos de audio del usuario nunca deben ser le˜dos fuera del proceso de reproducci˜n local. |

## **RNF-05 ˜ Mantenibilidad**

| **ID** | **Requerimiento** |
| --- | --- |
| RNF-05.1 | El m˜dulo de audio, el editor de tabs y la capa de presentaci˜n deben estar desacoplados mediante interfaces definidas, reemplazables de forma independiente. |
| RNF-05.2 | El esquema JSON de almacenamiento de tabs debe incluir un campo de versi˜n (schema\_version) para garantizar compatibilidad en actualizaciones futuras. |
| RNF-05.3 | El c˜digo fuente debe estructurarse de forma que agregar soporte para bajo de 5 cuerdas en una versi˜n futura no requiera reescribir el motor de tablatura. |

# **5. Requerimientos de Interfaz**

## **5.1 Vistas principales**

| **Vista** | **Descripci˜n** |
| --- | --- |
| Biblioteca | Lista de canciones registradas con b˜squeda y filtros. Acceso para crear nueva entrada o abrir una existente. |
| Editor de canci˜n | Vista principal dividida en tres zonas: waveform + timing points (arriba), editor de tablatura (centro), controles de reproducci˜n (abajo). |
| Calibrador de tempo | Vista de foco para la calibraci˜n: waveform a pantalla completa, bot˜n de tap (tecla T) prominente, feedback visual del BPM detectado en tiempo real. |
| Modo play-along | Tab renderizada con cursor animado, controles de velocidad, loop y metr˜nomo accesibles sin cambiar de vista. Puede coexistir con el editor en modo solo lectura. |
| Importar / Exportar | Panel modal para cargar archivos .gp o exportar PDF / JSON. |
| Ajustes | Preferencias de audio (dispositivo de salida, buffer), apariencia y atajos de teclado. |

## **5.2 Atajos de teclado**

| **Acci˜n** | **Atajo** |
| --- | --- |
| Play / Pause | Barra espaciadora |
| Pulsaci˜n de calibraci˜n | T |
| Reducir velocidad 5% | Ctrl + Flecha izquierda |
| Aumentar velocidad 5% | Ctrl + Flecha derecha |
| Comp˜s anterior | Flecha izquierda |
| Comp˜s siguiente | Flecha derecha |
| Activar / desactivar loop | L |
| Activar / desactivar metr˜nomo | M |
| Deshacer | Ctrl + Z |
| Rehacer | Ctrl + Shift + Z |
| Guardar (respaldo manual) | Ctrl + S |

## **5.3 Interfaces de datos**

* Audio de entrada: MP3 (MPEG-1/2 Layer III), WAV (PCM), FLAC, OGG Vorbis.
* Tabs de entrada: Guitar Pro 4/5 (.gp, .gp5) y Guitar Pro X (.gpx).
* Tabs de salida: JSON (formato interno Bassical, esquema versionado), PDF (exportaci˜n visual).
* Respaldo de biblioteca: archivo ZIP conteniendo todos los JSON de la biblioteca del usuario.

# **6. Casos de Uso Priorizados**

| **ID** | **Caso de uso** | **Prioridad** |
| --- | --- | --- |
| CU-01 | Registrar canci˜n desde archivo de audio local | Alta |
| CU-02 | Calibrar timing points por pulsaci˜n de tecla | Alta |
| CU-03 | Crear tab desde cero en el editor | Alta |
| CU-04 | Reproducir en modo play-along | Alta |
| CU-05 | Ajustar velocidad de reproducci˜n (pitch-shift) | Alta |
| CU-06 | Definir y usar loop de secci˜n | Alta |
| CU-07 | Ajuste manual fino de timing points sobre la waveform | Alta |
| CU-08 | Importar tab desde archivo Guitar Pro (.gp/.gp5/.gpx) | Media |
| CU-09 | Exportar tab como PDF | Media |
| CU-10 | Exportar / importar tab como JSON (compartir entre usuarios) | Media |
| CU-11 | Realizar respaldo completo de la biblioteca | Media |
| CU-12 | Reasignar ruta de audio cuando el archivo fue movido | Media |

# **7. Fuera del Alcance ˜ Versi˜n 1.0**

* Autenticaci˜n de usuarios, cuentas o sincronizaci˜n en la nube.
* Versi˜n web o m˜vil (Android / iOS).
* Bajo de 5 o 6 cuerdas.
* Soporte para otros instrumentos (guitarra, ukulele, etc.).
* Transcripci˜n autom˜tica de audio a tablatura.
* Cat˜logo p˜blico de tabs de terceros.
* Modo multijugador o jam session en red.
* Editor de partituras en pentagrama convencional.
* Detecci˜n de afinaci˜n del bajo en tiempo real v˜a micr˜fono.
* Gamificaci˜n o sistema de lecciones guiadas.

# **8. Criterios de Aceptaci˜n del Sistema**

El sistema se considerar˜ completo para la versi˜n 1.0 cuando cumpla todos los siguientes criterios:

* Todos los requerimientos de prioridad Alta (RF-01 a RF-05, CU-01 a CU-07) pasan sus pruebas de aceptaci˜n.
* La calibraci˜n de timing points produce un error de alineaci˜n <= 5 ms medido sobre canciones de referencia con tempo conocido.
* La reproducci˜n a 50% de velocidad no genera artefactos perceptibles en al menos 5 archivos de audio de prueba en formatos distintos.
* La aplicaci˜n se instala y ejecuta sin errores en Windows 10 y Windows 11 (dispositivos de prueba definidos).
* Ninguna funcionalidad requiere conexi˜n a internet, cuenta de usuario ni pago.
* La latencia del tap de calibraci˜n es <= 10 ms en los dispositivos de prueba.
* El consumo de RAM no supera 300 MB durante una sesi˜n de play-along de 30 minutos.
* Los archivos de audio originales del usuario no son modificados bajo ninguna circunstancia.
