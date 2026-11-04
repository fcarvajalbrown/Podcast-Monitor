# Registro de cambios

Todos los cambios de Podcast-Monitor, la app de escritorio que convierte episodios de podcasts chilenos en un informe PDF para leer en vez de escuchar. Formato de Keep a Changelog, versiones según SemVer 2.0.0.

## [0.2.0]

El programa prepara por sí mismo el asistente que escribe los resúmenes.

### Añadido

- Instalar y conectar instala Claude o Codex si falta y abre el navegador para iniciar sesión; si ya está instalado, no lo vuelve a instalar.

### Corregido

- El programa encuentra Claude y Codex instalados con el instalador oficial de cada uno.

## [0.1.0]

Primera versión pública, para Windows. Transcribe en el propio computador cada episodio de los programas guardados y entrega un PDF con el resumen, los temas por tiempo al aire, las personas mencionadas y citas textuales con su minuto.

### Añadido

- Ventana en español con la lista de programas guardada; un programa se agrega buscándolo por nombre en el directorio de podcasts de Apple, con su editor a la vista para confirmar, o pegando su dirección RSS.
- Generar informe: descarga el episodio elegido, lo transcribe en el computador, corrige los nombres con el glosario y lo resume con el asistente elegido en la ventana.
- Informe semanal con los episodios de los últimos 7 días de toda la lista, con una tabla de temas de la semana.
- En el PDF, cada cita se busca en la transcripción y se imprime con su minuto; la que no aparece se descarta.
- Continuar retoma un informe donde quedó si la app se cerró o el asistente llegó a su límite de uso.
- Descargar modelo trae el modelo de voz en el primer uso, se puede retomar y se verifica antes de usarlo.
- Instalador para Windows con acceso en el menú Inicio y desinstalador.
- La versión aparece en el título de la ventana.
