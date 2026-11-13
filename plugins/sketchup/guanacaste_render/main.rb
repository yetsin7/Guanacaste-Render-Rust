# encoding: UTF-8
# Menú Extensiones > Guanacaste Render > Exportar..., barra de herramientas y diálogos.

require 'sketchup.rb'
require 'guanacaste_render/exporter'

module GuanacasteRenderExport
  SETTINGS = 'GuanacasteRenderExport'.freeze
  ICON = File.join(File.dirname(__FILE__), 'export.svg')

  module Ui
    module_function

    def status(text)
      Sketchup.status_text = text
    end

    def ask_options
      current = Sketchup.read_default(SETTINGS, 'skip_hidden', true)
      choice = UI.inputbox(['Omitir objetos ocultos y etiquetas apagadas'],
                           [current ? 'Sí' : 'No'], ['Sí|No'], 'Exportar a Guanacaste Render')
      return nil unless choice
      skip = choice[0] == 'Sí'
      Sketchup.write_default(SETTINGS, 'skip_hidden', skip)
      { skip_hidden: skip }
    end

    def suggested_name(model)
      base = model.title.to_s.empty? ? 'modelo' : model.title.to_s
      "#{base}.gr"
    end

    def ask_path(model)
      folder = model.path.to_s.empty? ? nil : File.dirname(model.path)
      path = UI.savepanel('Exportar a Guanacaste Render', folder, suggested_name(model))
      return nil unless path
      path.end_with?('.gr') ? path : "#{path}.gr"
    end

    def run
      model = Sketchup.active_model
      options = ask_options
      return unless options
      path = ask_path(model)
      return unless path
      status('Guanacaste Render: preparando la exportación...')
      result = Exporter.export(model, path, options, method(:status))
      status('Guanacaste Render: exportación terminada.')
      UI.messagebox(done_text(result))
    rescue ExportError => error
      status('')
      UI.messagebox("No se pudo exportar a Guanacaste Render.\n\n#{error.message}")
    rescue StandardError => error
      status('')
      UI.messagebox("Error inesperado al exportar a Guanacaste Render.\n\n" \
                    "#{error.class}: #{error.message}")
    end

    def done_text(result)
      text = "Exportación terminada.\n\n#{result[:path]}\n" \
             "#{result[:faces]} caras, #{result[:triangles]} triángulos, " \
             "#{result[:materials]} materiales, #{result[:nodes]} nodos."
      text += "\n#{result[:skipped_hidden]} objetos ocultos omitidos." if result[:skipped_hidden] > 0
      if result[:textured] > 0
        text += "\n#{result[:textured]} materiales con textura: el formato .gr aún no guarda " \
                'texturas, se exportó solo el color.'
      end
      text
    end
  end

  unless file_loaded?(__FILE__)
    command = UI::Command.new('Exportar a Guanacaste Render...') { Ui.run }
    command.tooltip = 'Exportar a Guanacaste Render'
    command.status_bar_text = 'Exporta el modelo a un proyecto .gr de Guanacaste Render.'
    command.small_icon = ICON
    command.large_icon = ICON

    menu = UI.menu(Sketchup.version.to_i >= 17 ? 'Extensions' : 'Plugins')
    menu.add_submenu('Guanacaste Render').add_item(command)

    toolbar = UI::Toolbar.new('Guanacaste Render')
    toolbar.add_item(command)
    toolbar.restore

    file_loaded(__FILE__)
  end
end
