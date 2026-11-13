# encoding: UTF-8
# Escritor del contenedor .gr (GRRPROJ1). Especificación: docs/plugin-export-format.md

require 'json'
require 'zlib'
require 'stringio'
require 'digest'
require 'securerandom'

module GuanacasteRenderExport
  module GrWriter
    SIGNATURE = 'GRRPROJ1'.freeze
    HEADER_BYTES = 96
    CONTAINER_VERSION = 1
    SCHEMA_VERSION = 1
    KIND_MESH = 1

    module_function

    def gzip(data, level)
      io = StringIO.new
      io.set_encoding(Encoding::BINARY)
      writer = Zlib::GzipWriter.new(io, level)
      writer.write(data)
      writer.finish
      io.string
    end

    def sha256(data)
      Digest::SHA256.digest(data)
    end

    # Bloque de malla: u32 nv, u32 ni, f32 * 3nv, u32 * ni (little endian).
    def mesh_block(positions, indices)
      [positions.length / 3, indices.length].pack('VV') +
        positions.pack('e*') + indices.pack('V*')
    end

    # Escribe el archivo completo.
    # document: Hash del documento JSON; meshes: [{ positions: [Float], indices: [Integer] }]
    # (en el mismo orden que document['meshes']).
    def write(path, document, meshes, document_id = SecureRandom.random_bytes(16))
      json = JSON.generate(document).dup.force_encoding(Encoding::BINARY)
      json_gz = gzip(json, Zlib::DEFAULT_COMPRESSION)
      entries = []
      File.open(path, 'wb') do |file|
        file.write("\0".b * HEADER_BYTES)
        file.write(json_gz)
        meshes.each_with_index do |mesh, id|
          raw = mesh_block(mesh[:positions], mesh[:indices])
          gz = gzip(raw, Zlib::BEST_SPEED)
          entries << [KIND_MESH, id, file.pos, gz.bytesize, raw.bytesize, sha256(raw)]
          file.write(gz)
        end
        index_offset = file.pos
        entries.each { |entry| file.write(entry.pack('V V Q< Q< Q< a32')) }
        header = [SIGNATURE, CONTAINER_VERSION, 0, SCHEMA_VERSION, document_id,
                  json.bytesize, json_gz.bytesize, entries.length, 0,
                  sha256(json), index_offset].pack('a8 v v V a16 Q< Q< V V a32 Q<')
        file.seek(0)
        file.write(header)
      end
      nil
    end
  end
end
