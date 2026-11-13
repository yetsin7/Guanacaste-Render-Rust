using System;
using System.IO;
using System.IO.Compression;
using System.Security.Cryptography;
using System.Text;

namespace GuanacasteRender.Revit.Format
{
    /// <summary>
    /// Writer of the Guanacaste Render project container (<c>.gr</c>, signature <c>GRRPROJ1</c>), byte for byte what
    /// <c>src/features/projects/services/container.rs</c> and <c>store.rs</c> read. All integers are little endian.
    /// <code>
    /// HEADER (96 bytes)
    ///    0  8  "GRRPROJ1"
    ///    8  2  container version (1)           10  2  reserved (0)
    ///   12  4  schema version (1)              16 16  document id
    ///   32  8  JSON length, uncompressed       40  8  JSON length, gzip
    ///   48  4  block count                     52  4  reserved (0)
    ///   56 32  SHA-256 of the uncompressed JSON
    ///   88  8  offset of the block index
    /// gzip(JSON) | gzip(block) ... | index (64 bytes per block)
    /// INDEX ENTRY: u32 kind (1 = mesh), u32 id (mesh number), u64 offset, u64 gzip length, u64 raw length, 32 SHA-256 of raw
    /// MESH BLOCK (raw): u32 vertices, u32 indices, f32 x,y,z per vertex, u32 per index
    /// </code>
    /// </summary>
    internal static class GrFile
    {
        public const string Signature = "GRRPROJ1";
        public const ushort ContainerVersion = 1;
        public const uint SchemaVersion = 1;
        public const int HeaderBytes = 96;
        public const uint KindMesh = 1;

        /// <summary>Writes the project to <paramref name="path"/> through a temporary file, so a failure never leaves a half-written .gr.</summary>
        public static void WriteFile(string path, GrScene scene)
        {
            string temporary = path + ".tmp";
            try
            {
                using (var stream = new FileStream(temporary, FileMode.Create, FileAccess.ReadWrite, FileShare.None))
                {
                    Write(stream, scene);
                }
                if (File.Exists(path)) File.Delete(path);
                File.Move(temporary, path);
            }
            catch
            {
                try { if (File.Exists(temporary)) File.Delete(temporary); } catch { }
                throw;
            }
        }

        /// <summary>Writes the whole container to a seekable stream.</summary>
        public static void Write(Stream output, GrScene scene)
        {
            byte[] json = new UTF8Encoding(false).GetBytes(DocumentJson(scene));
            byte[] jsonGz = Gzip(json, CompressionLevel.Optimal);
            using (var writer = new BinaryWriter(output, new UTF8Encoding(false), true))
            {
                writer.Write(new byte[HeaderBytes]);
                writer.Write(jsonGz);
                var entries = new IndexEntry[scene.Meshes.Count];
                for (int i = 0; i < scene.Meshes.Count; i++)
                {
                    byte[] raw = EncodeMesh(scene.Meshes[i]);
                    byte[] gz = Gzip(raw, CompressionLevel.Fastest);
                    entries[i] = new IndexEntry
                    {
                        Id = (uint)i,
                        Offset = (ulong)output.Position,
                        GzLength = (ulong)gz.Length,
                        RawLength = (ulong)raw.Length,
                        Sha = Sha256(raw),
                    };
                    writer.Write(gz);
                }
                ulong indexOffset = (ulong)output.Position;
                foreach (IndexEntry entry in entries)
                {
                    writer.Write(KindMesh);
                    writer.Write(entry.Id);
                    writer.Write(entry.Offset);
                    writer.Write(entry.GzLength);
                    writer.Write(entry.RawLength);
                    writer.Write(entry.Sha);
                }
                writer.Flush();

                output.Seek(0, SeekOrigin.Begin);
                writer.Write(Encoding.ASCII.GetBytes(Signature));
                writer.Write(ContainerVersion);
                writer.Write((ushort)0);
                writer.Write(SchemaVersion);
                writer.Write(scene.DocumentId);
                writer.Write((ulong)json.Length);
                writer.Write((ulong)jsonGz.Length);
                writer.Write((uint)entries.Length);
                writer.Write((uint)0);
                writer.Write(Sha256(json));
                writer.Write(indexOffset);
                writer.Flush();
                output.Seek(0, SeekOrigin.End);
            }
        }

        private struct IndexEntry
        {
            public uint Id;
            public ulong Offset, GzLength, RawLength;
            public byte[] Sha;
        }

        /// <summary>Raw mesh block: u32 vertex count, u32 index count, f32 positions, u32 indices.</summary>
        public static byte[] EncodeMesh(GrMesh mesh)
        {
            int vertices = mesh.Positions.Length / 3;
            using (var memory = new MemoryStream(8 + mesh.Positions.Length * 4 + mesh.Indices.Length * 4))
            using (var writer = new BinaryWriter(memory))
            {
                writer.Write((uint)vertices);
                writer.Write((uint)mesh.Indices.Length);
                foreach (float value in mesh.Positions) writer.Write(value);
                foreach (uint index in mesh.Indices) writer.Write(index);
                writer.Flush();
                return memory.ToArray();
            }
        }

        /// <summary>The JSON document (see <c>models/document.rs</c>): keys the Rust side defaults are left out.</summary>
        public static string DocumentJson(GrScene scene)
        {
            var j = new JsonOut();
            j.BeginObject();
            j.Key("meta").BeginObject()
                .Key("name").Str(scene.Name)
                .Key("created").Int(scene.UnixTime)
                .Key("modified").Int(scene.UnixTime)
                .Key("app_version").Str(scene.AppVersion)
                .EndObject();
            j.Key("origin").BeginObject()
                .Key("format").Str("Revit")
                .Key("version").Str(scene.SourceVersion)
                .Key("application").Str(scene.Application)
                .Key("file_name").Str(scene.SourceFileName)
                .Key("file_size").Int((long)scene.SourceFileSize)
                .Key("length_unit").Str("ft")
                .Key("unit_scale").Num(0.3048)
                .Key("imported").Int(scene.UnixTime)
                .EndObject();
            j.Key("origin_shift").BeginArray()
                .Num(scene.OriginShift[0]).Num(scene.OriginShift[1]).Num(scene.OriginShift[2])
                .EndArray();

            j.Key("levels").BeginArray();
            foreach (GrLevel level in scene.Levels)
                j.BeginObject()
                    .Key("name").Str(level.Name)
                    .Key("elevation").Num(level.ElevationMeters)
                    .Key("node").Int(level.Node)
                    .EndObject();
            j.EndArray();

            j.Key("materials").BeginArray();
            foreach (GrMaterial m in scene.Materials)
                j.BeginObject()
                    .Key("name").Str(m.Name)
                    .Key("color").BeginArray().Float(m.R).Float(m.G).Float(m.B).Float(m.A).EndArray()
                    .EndObject();
            j.EndArray();

            j.Key("nodes").BeginArray();
            foreach (GrNode n in scene.Nodes)
            {
                j.BeginObject()
                    .Key("name").Str(n.Name)
                    .Key("class").Str(n.Class ?? "")
                    .Key("kind").Str(n.Kind);
                if (n.TypeName != null) j.Key("type_name").Str(n.TypeName);
                if (n.Parent >= 0) j.Key("parent").Int(n.Parent);
                if (n.Meshes.Count > 0)
                {
                    j.Key("meshes").BeginArray();
                    foreach (int mesh in n.Meshes) j.Int(mesh);
                    j.EndArray();
                }
                if (n.Level >= 0) j.Key("level").Int(n.Level);
                j.EndObject();
            }
            j.EndArray();

            j.Key("meshes").BeginArray();
            foreach (GrMesh mesh in scene.Meshes)
            {
                j.BeginObject();
                if (mesh.Material >= 0) j.Key("material").Int(mesh.Material);
                j.Key("vertices").Int(mesh.Positions.Length / 3)
                    .Key("indices").Int(mesh.Indices.Length)
                    .EndObject();
            }
            j.EndArray();
            j.EndObject();
            return j.ToString();
        }

        private static byte[] Gzip(byte[] raw, CompressionLevel level)
        {
            using (var memory = new MemoryStream())
            {
                using (var gzip = new GZipStream(memory, level, true))
                {
                    gzip.Write(raw, 0, raw.Length);
                }
                return memory.ToArray();
            }
        }

        private static byte[] Sha256(byte[] data)
        {
            using (var sha = SHA256.Create()) return sha.ComputeHash(data);
        }
    }
}
