# encoding: UTF-8
# API mínima de SketchUp para probar el exportador sin SketchUp. Las unidades internas son pulgadas.

module Geom
  class Vector3d
    attr_reader :x, :y, :z
    def initialize(x, y, z)
      @x = x.to_f
      @y = y.to_f
      @z = z.to_f
    end

    def cross(o)
      Vector3d.new(y * o.z - z * o.y, z * o.x - x * o.z, x * o.y - y * o.x)
    end

    def dot(o)
      x * o.x + y * o.y + z * o.z
    end
  end

  class Point3d
    attr_reader :x, :y, :z
    def initialize(x, y, z)
      @x = x.to_f
      @y = y.to_f
      @z = z.to_f
    end
  end

  # Matriz 3x3 por columnas (ejes) más traslación.
  class Transformation
    attr_reader :xaxis, :yaxis, :zaxis, :origin
    def initialize(xaxis = nil, yaxis = nil, zaxis = nil, origin = [0, 0, 0])
      @xaxis = xaxis || Vector3d.new(1, 0, 0)
      @yaxis = yaxis || Vector3d.new(0, 1, 0)
      @zaxis = zaxis || Vector3d.new(0, 0, 1)
      @origin = origin
    end

    def self.translation(x, y, z)
      new(nil, nil, nil, [x, y, z])
    end

    def self.scaling_x(factor)
      new(Vector3d.new(factor, 0, 0))
    end

    def *(other)
      if other.is_a?(Point3d)
        Point3d.new(xaxis.x * other.x + yaxis.x * other.y + zaxis.x * other.z + origin[0],
                    xaxis.y * other.x + yaxis.y * other.y + zaxis.y * other.z + origin[1],
                    xaxis.z * other.x + yaxis.z * other.y + zaxis.z * other.z + origin[2])
      else
        moved = self * Point3d.new(*other.origin)
        Transformation.new(apply(other.xaxis), apply(other.yaxis), apply(other.zaxis),
                           [moved.x, moved.y, moved.z])
      end
    end

    def apply(v)
      Vector3d.new(xaxis.x * v.x + yaxis.x * v.y + zaxis.x * v.z,
                   xaxis.y * v.x + yaxis.y * v.y + zaxis.y * v.z,
                   xaxis.z * v.x + yaxis.z * v.y + zaxis.z * v.z)
    end
  end
end

module Sketchup
  def self.version
    '24.0.0'
  end

  class Layer
    attr_reader :name
    def initialize(name, visible = true)
      @name = name
      @visible = visible
    end

    def visible?
      @visible
    end
  end

  class Color
    attr_reader :red, :green, :blue
    def initialize(r, g, b)
      @red = r
      @green = g
      @blue = b
    end
  end

  class Material
    attr_reader :name, :color, :alpha, :texture
    def initialize(name, rgb, alpha = 1.0, texture = nil)
      @name = name
      @color = Color.new(*rgb)
      @alpha = alpha
      @texture = texture
    end
  end

  class Entity
    attr_accessor :layer, :material, :hidden
    def initialize(layer = nil)
      @layer = layer || Layer.new('Layer0')
    end

    def hidden?
      @hidden ? true : false
    end
  end

  class PolygonMesh
    attr_reader :points, :polygons
    def initialize(points, polygons)
      @points = points
      @polygons = polygons
    end
  end

  # Cara con triángulos ya dados: puntos en pulgadas y polígonos de índices base 1.
  class Face < Entity
    attr_accessor :back_material
    def initialize(points, polygons, layer = nil)
      super(layer)
      @points = points.map { |p| Geom::Point3d.new(*p) }
      @polygons = polygons
    end

    def mesh(_flags = 0)
      PolygonMesh.new(@points, @polygons)
    end
  end

  class Definition
    attr_reader :name, :entities
    def initialize(name, entities)
      @name = name
      @entities = entities
    end
  end

  class Group < Entity
    attr_accessor :name, :transformation, :definition
    def initialize(name, entities, transformation = Geom::Transformation.new, layer = nil)
      super(layer)
      @name = name
      @definition = Definition.new(name, entities)
      @transformation = transformation
    end
  end

  class ComponentInstance < Group; end

  class Model
    attr_accessor :entities, :title, :path
    def initialize(entities, title = 'Casa')
      @entities = entities
      @title = title
      @path = ''
    end
  end
end

# Cuadrado unitario de 1 pulgada de lado en z = 0 (dos triángulos).
def unit_square_face(layer = nil)
  Sketchup::Face.new([[0, 0, 0], [1, 0, 0], [1, 1, 0], [0, 1, 0]], [[1, 2, 3], [1, 3, 4]], layer)
end
