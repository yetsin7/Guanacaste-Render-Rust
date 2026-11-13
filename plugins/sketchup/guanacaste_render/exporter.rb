# encoding: UTF-8
# Orquesta la exportación: recorre el modelo, arma el documento y escribe el .gr.

require 'guanacaste_render/scene_builder'
require 'guanacaste_render/gr_writer'

module GuanacasteRenderExport
  class ExportError < StandardError; end

  module Exporter
    module_function

    # Exporta el modelo a `path`. Devuelve un Hash con estadísticas. Lanza ExportError con un
    # mensaje en español si no se puede.
    def export(model, path, options = {}, progress = nil)
      raise ExportError, 'No hay un modelo abierto.' if model.nil?
      builder = SceneBuilder.new(model, options, progress)
      builder.build(project_name(model))
      if builder.empty?
        raise ExportError, 'El modelo no tiene geometría visible para exportar. ' \
                           'Revise las etiquetas apagadas o desactive "omitir ocultos".'
      end
      shift = origin_shift(builder)
      document = build_document(model, builder, shift)
      meshes = builder.meshes.map { |mesh| shifted(mesh, shift) }
      write_atomically(path, document, meshes)
      summary(builder, path)
    end

    def project_name(model)
      title = model.respond_to?(:title) ? model.title.to_s : ''
      title.empty? ? 'Sin título' : title
    end

    # Centro XY del cuadro delimitador y mínimo Z (en metros), para conservar precisión en f32.
    def origin_shift(builder)
      [(builder.min[0] + builder.max[0]) / 2.0, (builder.min[1] + builder.max[1]) / 2.0, builder.min[2]]
    end

    def shifted(mesh, shift)
      positions = mesh[:positions].each_with_index.map { |value, i| value - shift[i % 3] }
      { positions: positions, indices: mesh[:indices] }
    end

    def build_document(model, builder, shift)
      now = Time.now.to_i
      file = model.respond_to?(:path) && !model.path.to_s.empty? ? File.basename(model.path) : ''
      version = Sketchup.respond_to?(:version) ? Sketchup.version.to_s : ''
      {
        'meta' => { 'name' => project_name(model), 'created' => now, 'modified' => now,
                    'app_version' => PLUGIN_VERSION },
        'origin' => { 'format' => 'SketchUp', 'version' => "SketchUp #{version}".strip,
                      'application' => 'SketchUp', 'file_name' => file, 'file_size' => 0,
                      'length_unit' => 'in', 'unit_scale' => SceneBuilder::INCH_TO_METER,
                      'imported' => now },
        'origin_shift' => shift,
        'levels' => [],
        'materials' => builder.materials,
        'nodes' => builder.nodes,
        'meshes' => builder.meshes.map do |mesh|
          { 'material' => mesh[:material], 'vertices' => mesh[:positions].length / 3,
            'indices' => mesh[:indices].length }
        end
      }
    end

    # Escribe en un temporal y lo mueve al destino, para no dejar un .gr a medias.
    def write_atomically(path, document, meshes)
      temporary = "#{path}.tmp"
      begin
        GrWriter.write(temporary, document, meshes)
        File.delete(path) if File.exist?(path)
        File.rename(temporary, path)
      rescue SystemCallError, IOError => error
        File.delete(temporary) if File.exist?(temporary)
        raise ExportError, "No se pudo escribir el archivo: #{error.message}"
      end
    end

    def summary(builder, path)
      { path: path, faces: builder.stats[:faces], meshes: builder.meshes.length,
        nodes: builder.nodes.length, materials: builder.materials.length,
        triangles: builder.meshes.inject(0) { |sum, m| sum + m[:indices].length / 3 },
        skipped_hidden: builder.stats[:skipped_hidden], textured: builder.stats[:textured],
        bytes: File.size(path) }
    end
  end
end
