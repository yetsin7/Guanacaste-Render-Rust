using System.Collections.Generic;

namespace GuanacasteRender.Revit.Format
{
    /// <summary>Material of the exported scene. Color channels are 0..1.</summary>
    internal sealed class GrMaterial
    {
        public string Name;
        public float R, G, B, A;
    }

    /// <summary>Node of the hierarchy. Indices refer to <see cref="GrScene"/> lists; -1 means "none".</summary>
    internal sealed class GrNode
    {
        public string Name;
        /// <summary>Source class: for elements the Revit category name.</summary>
        public string Class;
        /// <summary>project, site, building, level, element or group (the values the Rust reader knows).</summary>
        public string Kind;
        public string TypeName;
        public int Parent = -1;
        public int Level = -1;
        public List<int> Meshes = new List<int>();
    }

    internal sealed class GrLevel
    {
        public string Name;
        /// <summary>Elevation in meters (the same Z as the vertices, since the origin shift never touches Z).</summary>
        public double ElevationMeters;
        public int Node;
    }

    /// <summary>Indexed triangle mesh in meters, Z up, already relative to the origin shift.</summary>
    internal sealed class GrMesh
    {
        /// <summary>x, y, z per vertex.</summary>
        public float[] Positions;
        /// <summary>Three indices per triangle, counter-clockwise seen from outside.</summary>
        public uint[] Indices;
        public int Material = -1;
    }

    /// <summary>Everything a .gr project written by this add-in contains. Pure data: no Revit types.</summary>
    internal sealed class GrScene
    {
        public string Name = "Revit";
        /// <summary>Creation and modification time, seconds since the Unix epoch.</summary>
        public long UnixTime;
        public string AppVersion = "revit-addin 1.0.0";
        /// <summary>Authoring application, e.g. "Autodesk Revit 2026".</summary>
        public string Application = "Autodesk Revit";
        public string SourceVersion = "";
        public string SourceFileName = "";
        public ulong SourceFileSize;
        /// <summary>World position in meters subtracted from every vertex (X and Y only).</summary>
        public double[] OriginShift = new double[3];
        public byte[] DocumentId = new byte[16];
        public List<GrLevel> Levels = new List<GrLevel>();
        public List<GrMaterial> Materials = new List<GrMaterial>();
        public List<GrNode> Nodes = new List<GrNode>();
        public List<GrMesh> Meshes = new List<GrMesh>();
    }
}
