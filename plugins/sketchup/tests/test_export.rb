# encoding: UTF-8
# Ejecutar desde plugins/sketchup:  ruby tests/test_export.rb
$LOAD_PATH.unshift(File.expand_path('..', __dir__))
$LOAD_PATH.unshift(__dir__)

require 'minitest/autorun'
require 'tmpdir'
require 'stub_sketchup'

module GuanacasteRenderExport
  PLUGIN_VERSION = 'test'.freeze
end
require 'guanacaste_render/exporter'

# Lector independiente del contenedor, que sigue docs/plugin-export-format.md.
class GrReader
  attr_reader :header, :document, :blocks

  def initialize(path)
    bytes = File.binread(path)
    sig, cver, _r, schema, _id, jraw, jgz, count, _r2, jsha, index_at =
      bytes[0, 96].unpack('a8 v v V a16 Q< Q< V V a32 Q<')
    raise 'firma' unless sig == 'GRRPROJ1' && cver == 1 && schema == 1
    json = Zlib::GzipReader.new(StringIO.new(bytes[96, jgz])).read
    raise 'json' unless json.bytesize == jraw && Digest::SHA256.digest(json) == jsha
    raise 'tamaño' unless index_at + 64 * count == bytes.bytesize
    @document = JSON.parse(json)
    @blocks = (0...count).map do |i|
      kind, id, off, gz, raw, sha = bytes[index_at + 64 * i, 64].unpack('V V Q< Q< Q< a32')
      data = Zlib::GzipReader.new(StringIO.new(bytes[off, gz])).read
      raise 'bloque' unless data.bytesize == raw && Digest::SHA256.digest(data) == sha
      nv, ni = data.unpack('VV')
      raise 'longitud' unless data.bytesize == 8 + nv * 12 + ni * 4
      { kind: kind, id: id, positions: data[8, nv * 12].unpack('e*'),
        indices: data[8 + nv * 12, ni * 4].unpack('V*') }
    end
  end
end

class ExportTest < Minitest::Test
  E = GuanacasteRenderExport

  def export(model, options = {})
    path = File.join(Dir.mktmpdir, 'salida.gr')
    result = E::Exporter.export(model, path, options)
    [GrReader.new(path), result]
  end

  def test_single_face_becomes_one_mesh_in_meters
    model = Sketchup::Model.new([unit_square_face])
    gr, result = export(model)
    assert_equal 2, result[:triangles]
    assert_equal 1, gr.blocks.length
    xs = gr.blocks[0][:positions].each_slice(3).map(&:first)
    assert_in_delta 0.0254, xs.max - xs.min, 1e-7
    assert_equal [0, 1, 2, 0, 2, 3], gr.blocks[0][:indices]
    assert_equal 'Sin material', gr.document['materials'][0]['name']
    assert_equal 'project', gr.document['nodes'][0]['kind']
    assert_equal 'SketchUp', gr.document['origin']['format']
  end

  def test_groups_apply_transformations_and_keep_hierarchy
    inner = Sketchup::Group.new('Mesa', [unit_square_face], Geom::Transformation.translation(100, 0, 0))
    model = Sketchup::Model.new([inner])
    gr, = export(model)
    nodes = gr.document['nodes']
    assert_equal ['Casa', 'Mesa'], nodes.map { |n| n['name'] }
    assert_equal 0, nodes[1]['parent']
    assert_equal 'element', nodes[1]['kind']
    shift = gr.document['origin_shift']
    x = gr.blocks[0][:positions][0] + shift[0]
    assert_in_delta 2.54, x, 1e-5
  end

  def test_materials_color_alpha_and_back_face
    glass = Sketchup::Material.new('Vidrio', [51, 102, 204], 0.4)
    face = unit_square_face
    face.back_material = glass
    gr, = export(Sketchup::Model.new([face]))
    material = gr.document['materials'][0]
    assert_equal 'Vidrio', material['name']
    assert_in_delta 0.2, material['color'][0], 1e-6
    assert_in_delta 0.4, material['color'][3], 1e-6
    assert_equal [0, 2, 1, 0, 3, 2], gr.blocks[0][:indices], 'solo la cara posterior pintada se invierte'
  end

  def test_mirrored_instances_flip_winding
    mirrored = Sketchup::ComponentInstance.new('Espejo', [unit_square_face], Geom::Transformation.scaling_x(-1))
    gr, = export(Sketchup::Model.new([mirrored]))
    assert_equal [0, 2, 1, 0, 3, 2], gr.blocks[0][:indices]
    assert_equal 'Espejo', gr.document['nodes'][1]['type_name']
  end

  def test_hidden_entities_and_tags_are_skipped_by_default_and_kept_on_request
    hidden = unit_square_face
    hidden.hidden = true
    off = unit_square_face(Sketchup::Layer.new('Apagada', false))
    model = Sketchup::Model.new([unit_square_face, hidden, off])
    _, result = export(model)
    assert_equal 1, result[:faces]
    assert_equal 2, result[:skipped_hidden]
    _, all = export(model, skip_hidden: false)
    assert_equal 3, all[:faces]
  end

  def test_tags_name_the_node_class
    door = Sketchup::Group.new('Puerta', [unit_square_face], Geom::Transformation.new, Sketchup::Layer.new('Puertas'))
    gr, = export(Sketchup::Model.new([door]))
    assert_equal 'Puertas', gr.document['nodes'][1]['class']
  end

  def test_empty_model_gives_a_spanish_error
    error = assert_raises(E::ExportError) { export(Sketchup::Model.new([])) }
    assert_match(/no tiene geometría/, error.message)
  end

  def test_document_counts_match_blocks
    gr, = export(Sketchup::Model.new([unit_square_face, Sketchup::Group.new('G', [unit_square_face])]))
    gr.document['meshes'].each_with_index do |meta, i|
      assert_equal meta['vertices'] * 3, gr.blocks[i][:positions].length
      assert_equal meta['indices'], gr.blocks[i][:indices].length
      assert_equal i, gr.blocks[i][:id]
    end
  end
end
