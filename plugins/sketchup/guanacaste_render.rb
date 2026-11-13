# encoding: UTF-8
# Guanacaste Render - extensión de SketchUp (registro).
#
# Este es el único archivo que va en la carpeta Plugins junto con la carpeta guanacaste_render/.
# Compatible con SketchUp 2017 o posterior (Ruby 2.2+).

require 'sketchup.rb'
require 'extensions.rb'

module GuanacasteRenderExport
  PLUGIN_NAME = 'Guanacaste Render'.freeze
  PLUGIN_VERSION = '0.1.0'.freeze

  unless file_loaded?(__FILE__)
    extension = SketchupExtension.new(PLUGIN_NAME, 'guanacaste_render/main')
    extension.description = 'Exporta el modelo de SketchUp de una vez a un proyecto .gr de ' \
                            'Guanacaste Render (mallas, materiales, jerarquía y etiquetas).'
    extension.version = PLUGIN_VERSION
    extension.creator = 'Guanacaste Render'
    extension.copyright = '2026 Guanacaste Render'
    Sketchup.register_extension(extension, true)
    file_loaded(__FILE__)
  end
end
