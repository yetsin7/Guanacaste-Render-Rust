# Guanacaste Render para SketchUp

Extensión de SketchUp (Ruby, 2017 o posterior) que exporta el modelo de una vez a un proyecto `.gr` de Guanacaste
Render (firma `GRRPROJ1`), sin pasar por DAE/OBJ. El formato está descrito byte a byte en
`docs/plugin-export-format.md`.

## Instalación

* Con el `.rbz`: ejecute `powershell -File plugins\sketchup\build_rbz.ps1` (crea
  `plugins\sketchup\artifacts\guanacaste_render-<versión>.rbz`) y en SketchUp use
  Ventana > Administrador de extensiones > Instalar extensión.
* Copia directa: `powershell -File plugins\sketchup\install.ps1 -Version 2024` copia solo
  `guanacaste_render.rb` y `guanacaste_render\` a la carpeta Plugins de esa versión. Solo se ejecuta cuando usted
  lo lanza. Reinicie SketchUp después.

## Uso

Extensiones > Guanacaste Render > Exportar a Guanacaste Render... (o el botón de la barra de herramientas).
Elija si omitir objetos ocultos y etiquetas apagadas, y el archivo `.gr` de destino. El progreso se muestra en la
barra de estado; al terminar se resume lo exportado. En Guanacaste Render use "Abrir" sobre el `.gr`.

## Qué se exporta

* Unidades: pulgadas a metros (0,0254). SketchUp ya es Z arriba: no se giran ejes.
* Grupos y componentes, de forma recursiva y con sus transformaciones; cada uno es un nodo de la jerarquía
  (nombre, definición del componente en `type_name`, etiqueta en `class`). La geometría de instancias se copia
  expandida (el formato no tiene instancias). Las instancias reflejadas conservan el sentido correcto de las caras.
* Caras con huecos: se triangulan con `Face#mesh`.
* Cara frontal y posterior: se usa el material frontal (o el heredado del grupo); si solo la cara posterior está
  pintada, se invierte el sentido de la cara y se usa ese material.
* Materiales: nombre, color y transparencia (alfa).
* Etiquetas (capas): la etiqueta efectiva queda en `class`; caras con otra etiqueta que su contenedor van en un
  nodo hijo.
* Ocultos: opción para omitir objetos ocultos y etiquetas apagadas (activada por defecto).
* `origin_shift`: centro XY del modelo y mínimo Z, para no perder precisión en las mallas de 32 bits.

## Limitaciones

* Las texturas no se exportan: el formato `.gr` actual no las guarda. Se exporta el color del material y la
  extensión avisa de cuántos materiales tenían textura.
* No se exportan aristas sueltas, curvas, líneas guía, texto, cotas ni secciones.
* El origen se rotula "SketchUp"; la aplicación lo muestra como origen genérico.

## Pruebas

`ruby tests/test_export.rb` (desde esta carpeta) usa una API de SketchUp simulada (`tests/stub_sketchup.rb`) y un
lector independiente del contenedor. El lado Rust se comprueba con
`cargo test plugin_fixture`, que abre un `.gr` armado byte a byte con el mismo diseño.

Nota honesta: al escribir esta extensión no había Ruby ni SketchUp instalados, así que ni las pruebas de Ruby ni la
extensión dentro de SketchUp se han ejecutado todavía. Sí se probó el empaquetado `.rbz` y la lectura en Rust.
