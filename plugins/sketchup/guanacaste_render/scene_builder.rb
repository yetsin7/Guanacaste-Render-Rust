# encoding: UTF-8
# Recorre el modelo de SketchUp (grupos y componentes de forma recursiva, con sus transformaciones)
# y produce nodos, materiales y mallas triangulares en metros, Z arriba.

module GuanacasteRenderExport
  class SceneBuilder
    INCH_TO_METER = 0.0254
    MAX_DEPTH = 64
    UNTAGGED = ['Layer0', 'Untagged', 'Sin etiqueta'].freeze
    DEFAULT_MATERIAL = 'Sin material'.freeze
    ROOT_LOOSE_NAME = 'Geometría suelta'.freeze

    # nodes: Array<Hash> con las claves del documento (parent es índice); meshes: Array<Hash>
    # { material:, positions: [Float metros], indices: [Integer] }.
    attr_reader :nodes, :materials, :meshes, :stats, :min, :max

    # options: skip_hidden (true/false). progress: objeto con #call(texto) opcional.
    def initialize(model, options = {}, progress = nil)
      @model = model
      @skip_hidden = options.fetch(:skip_hidden, true)
      @progress = progress
      @nodes = []
      @materials = []
      @material_index = {}
      @meshes = []
      @min = [Float::INFINITY] * 3
      @max = [-Float::INFINITY] * 3
      @stats = { faces: 0, skipped_hidden: 0, textured: 0, instances: 0, degenerate: 0 }
      @total_faces = model.respond_to?(:number_faces) ? model.number_faces : 0
    end

    def build(root_name)
      root = add_node(root_name, '', 'project', nil)
      walk(@model.entities, identity, root, ROOT_LOOSE_NAME, :root, nil, 0)
      self
    end

    def empty?
      @meshes.empty?
    end

    private

    def identity
      Geom::Transformation.new
    end

    def untagged?(name)
      name.nil? || name.empty? || UNTAGGED.include?(name)
    end

    def tag_of(entity, inherited)
      layer = entity.respond_to?(:layer) ? entity.layer : nil
      return inherited if layer.nil? || untagged?(layer.name)
      layer.name
    end

    def hidden?(entity)
      return false unless @skip_hidden
      return true if entity.respond_to?(:hidden?) && entity.hidden?
      layer = entity.respond_to?(:layer) ? entity.layer : nil
      !layer.nil? && layer.respond_to?(:visible?) && !layer.visible?
    end

    def text(value)
      value.to_s.encode('UTF-8', invalid: :replace, undef: :replace)
    end

    def add_node(name, klass, kind, parent, type_name = nil)
      @nodes << { 'name' => text(name), 'class' => text(klass), 'kind' => kind,
                  'type_name' => type_name && text(type_name), 'parent' => parent,
                  'meshes' => [], 'level' => nil }
      @nodes.length - 1
    end

    def material_for(material)
      key = material.nil? ? :none : material
      return @material_index[key] if @material_index.key?(key)
      if material.nil?
        entry = { 'name' => DEFAULT_MATERIAL, 'color' => [0.8, 0.8, 0.8, 1.0] }
      else
        color = material.color
        @stats[:textured] += 1 if material.respond_to?(:texture) && material.texture
        alpha = material.respond_to?(:alpha) ? material.alpha : 1.0
        name = material.respond_to?(:display_name) ? material.display_name : material.name
        entry = { 'name' => text(name),
                  'color' => [color.red / 255.0, color.green / 255.0, color.blue / 255.0, alpha.to_f] }
      end
      @materials << entry
      @material_index[key] = @materials.length - 1
    end

    def definition_of(entity)
      entity.respond_to?(:definition) ? entity.definition : entity
    end

    def mirrored?(transform)
      transform.xaxis.cross(transform.yaxis).dot(transform.zaxis) < 0
    end

    def report
      return unless @progress && (@stats[:faces] % 250).zero?
      total = @total_faces > 0 ? " de #{@total_faces}" : ''
      @progress.call("Guanacaste Render: exportando caras #{@stats[:faces]}#{total}...")
    end

    # Recorre las entidades de un contenedor y reparte su geometría directa por (etiqueta, material).
    def walk(entities, transform, node, name, container_tag, inherited_material, depth)
      raise 'El modelo tiene anidación de grupos demasiado profunda.' if depth > MAX_DEPTH
      buckets = {}
      flip_mirror = mirrored?(transform)
      entities.each do |entity|
        if hidden?(entity)
          @stats[:skipped_hidden] += 1
          next
        end
        if entity.is_a?(Sketchup::Face)
          add_face(entity, transform, flip_mirror, container_tag, inherited_material, buckets)
        elsif entity.is_a?(Sketchup::Group) || entity.is_a?(Sketchup::ComponentInstance)
          enter(entity, transform, node, container_tag, inherited_material, depth)
        end
      end
      flush(buckets, node, name, container_tag)
    end

    def enter(entity, transform, parent, container_tag, inherited_material, depth)
      definition = definition_of(entity)
      component = entity.is_a?(Sketchup::ComponentInstance)
      label = entity.name.to_s.empty? ? definition.name.to_s : entity.name.to_s
      label = component ? 'Componente' : 'Grupo' if label.empty?
      tag = tag_of(entity, container_tag == :root ? nil : container_tag)
      klass = tag || (component ? 'Componente' : 'Grupo')
      node = add_node(label, klass, 'group', parent, component ? definition.name : nil)
      @stats[:instances] += 1
      material = entity.material || inherited_material
      walk(definition.entities, transform * entity.transformation, node, label, tag, material, depth + 1)
    end

    def add_face(face, transform, mirror, container_tag, inherited_material, buckets)
      tag = tag_of(face, container_tag == :root ? nil : container_tag)
      flip = mirror
      material = face.material || inherited_material
      if material.nil? && face.back_material
        material = face.back_material
        flip = !flip
      end
      bucket = (buckets[tag] ||= {})
      (bucket[material_for(material)] ||= { positions: [], indices: [] })
      store_triangles(face, transform, flip, bucket[material_for(material)])
      @stats[:faces] += 1
      report
    end

    def store_triangles(face, transform, flip, target)
      mesh = face.mesh(0)
      base = target[:positions].length / 3
      mesh.points.each do |point|
        world = transform * point
        xyz = [world.x.to_f * INCH_TO_METER, world.y.to_f * INCH_TO_METER, world.z.to_f * INCH_TO_METER]
        3.times do |axis|
          @min[axis] = xyz[axis] if xyz[axis] < @min[axis]
          @max[axis] = xyz[axis] if xyz[axis] > @max[axis]
        end
        target[:positions].concat(xyz)
      end
      mesh.polygons.each do |polygon|
        ids = polygon.map { |i| i.abs - 1 + base }
        if ids.length < 3
          @stats[:degenerate] += 1
          next
        end
        (1...(ids.length - 1)).each do |k|
          triangle = flip ? [ids[0], ids[k + 1], ids[k]] : [ids[0], ids[k], ids[k + 1]]
          target[:indices].concat(triangle)
        end
      end
    end

    # Convierte los cubos acumulados en mallas: la etiqueta del contenedor va en su propio nodo;
    # las demás etiquetas crean un nodo hijo.
    def flush(buckets, node, name, container_tag)
      buckets.each do |tag, by_material|
        target = node
        if tag != container_tag || container_tag == :root
          suffix = tag ? " [#{tag}]" : ''
          target = add_node(name + suffix, tag || '', 'element', node)
        end
        @nodes[target]['kind'] = 'element'
        by_material.each do |material, data|
          next if data[:indices].empty?
          @meshes << { material: material, positions: data[:positions], indices: data[:indices] }
          @nodes[target]['meshes'] << @meshes.length - 1
        end
      end
    end
  end
end
