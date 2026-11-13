using System;
using System.Collections.Generic;
using Autodesk.Revit.DB;
using GuanacasteRender.Revit.Format;

namespace GuanacasteRender.Revit.Export
{
    /// <summary>
    /// Receives the geometry of a 3D view from <see cref="CustomExporter"/>. Revit internal units are feet and the
    /// model is already Z up, so the only conversion is feet to meters. Instances (families, groups, nested
    /// components) arrive with their own transform; the accumulated transform is applied to every polymesh.
    /// </summary>
    internal sealed partial class SceneCollector : IExportContext
    {
        public const double FeetToMeters = 0.3048;

        private readonly Document _doc;
        private readonly Func<bool> _cancelRequested;
        private readonly Action<int, string> _progress;
        private readonly Stack<Transform> _transforms = new Stack<Transform>();
        private readonly Stack<ElementState> _elements = new Stack<ElementState>();
        private readonly List<ElementState> _finished = new List<ElementState>();
        private readonly List<GrMaterial> _materials = new List<GrMaterial>();
        private readonly Dictionary<string, int> _materialKeys = new Dictionary<string, int>();
        private readonly Dictionary<ElementId, int> _levelIndex = new Dictionary<ElementId, int>();
        private readonly List<Level> _levels = new List<Level>();
        private int _currentMaterial = -1;
        private int _elementsSeen;
        private int _skippedLinks;

        /// <summary>One element in progress: its identity and the meshes accumulated per material.</summary>
        private sealed class ElementState
        {
            public ElementId Id;
            public string Name, Category, TypeName;
            public int Level = -1;
            public readonly Dictionary<int, RawMesh> ByMaterial = new Dictionary<int, RawMesh>();
        }

        /// <summary>Mesh under construction in double precision world meters; shifted and narrowed to float at the end.</summary>
        private sealed class RawMesh
        {
            public int Material;
            public readonly List<double> Positions = new List<double>();
            public readonly List<uint> Indices = new List<uint>();
        }

        public int ElementsSeen { get { return _elementsSeen; } }
        public int SkippedLinks { get { return _skippedLinks; } }
        public bool WasCanceled { get; private set; }

        public SceneCollector(Document doc, Func<bool> cancelRequested, Action<int, string> progress)
        {
            _doc = doc;
            _cancelRequested = cancelRequested;
            _progress = progress;
        }

        // ---------------------------------------------------------------- IExportContext

        public bool Start()
        {
            _transforms.Clear();
            _transforms.Push(Transform.Identity);
            var collector = new FilteredElementCollector(_doc).OfClass(typeof(Level));
            foreach (Element element in collector)
            {
                var level = element as Level;
                if (level != null) _levels.Add(level);
            }
            _levels.Sort((a, b) => a.Elevation.CompareTo(b.Elevation));
            for (int i = 0; i < _levels.Count; i++) _levelIndex[_levels[i].Id] = i;
            return true;
        }

        public void Finish() { }

        public bool IsCanceled()
        {
            if (!WasCanceled && _cancelRequested()) WasCanceled = true;
            return WasCanceled;
        }

        public RenderNodeAction OnViewBegin(ViewNode node) { return RenderNodeAction.Proceed; }

        public void OnViewEnd(ElementId elementId) { }

        public RenderNodeAction OnElementBegin(ElementId elementId)
        {
            _elementsSeen++;
            Element element = _doc.GetElement(elementId);
            var state = new ElementState { Id = elementId };
            if (element != null)
            {
                state.Category = element.Category != null ? element.Category.Name : null;
                state.Name = SafeName(element);
                state.TypeName = TypeNameOf(element);
                ElementId levelId = element.LevelId;
                int level;
                if (levelId != null && _levelIndex.TryGetValue(levelId, out level)) state.Level = level;
            }
            _elements.Push(state);
            _progress(_elementsSeen, state.Name);
            return RenderNodeAction.Proceed;
        }

        public void OnElementEnd(ElementId elementId)
        {
            if (_elements.Count == 0) return;
            ElementState state = _elements.Pop();
            if (state.ByMaterial.Count > 0) _finished.Add(state);
        }

        public RenderNodeAction OnInstanceBegin(InstanceNode node)
        {
            _transforms.Push(_transforms.Peek().Multiply(node.GetTransform()));
            return RenderNodeAction.Proceed;
        }

        public void OnInstanceEnd(InstanceNode node)
        {
            if (_transforms.Count > 1) _transforms.Pop();
        }

        /// <summary>Linked models live in another document, so their elements cannot be named or leveled here.</summary>
        public RenderNodeAction OnLinkBegin(LinkNode node)
        {
            _skippedLinks++;
            return RenderNodeAction.Skip;
        }

        public void OnLinkEnd(LinkNode node) { }

        public RenderNodeAction OnFaceBegin(FaceNode node) { return RenderNodeAction.Proceed; }

        public void OnFaceEnd(FaceNode node) { }

        public void OnLight(LightNode node) { }

        public void OnRPC(RPCNode node) { }

        public void OnMaterial(MaterialNode node)
        {
            Color color = node.Color;
            double transparency = node.Transparency;
            string name = null;
            string key;
            if (node.MaterialId != null && node.MaterialId != ElementId.InvalidElementId)
            {
                Element material = _doc.GetElement(node.MaterialId);
                if (material != null) name = material.Name;
            }
            float r = color != null ? color.Red / 255f : 0.7f;
            float g = color != null ? color.Green / 255f : 0.7f;
            float b = color != null ? color.Blue / 255f : 0.7f;
            float a = (float)Math.Max(0.0, Math.Min(1.0, 1.0 - transparency));
            if (name != null) key = "m:" + name + "|" + r + "|" + g + "|" + b + "|" + a;
            else
            {
                name = "Color #" + ColorHex(r, g, b) + (a < 1f ? " (transparente)" : "");
                key = "c:" + name + "|" + a;
            }
            int index;
            if (!_materialKeys.TryGetValue(key, out index))
            {
                index = _materials.Count;
                _materials.Add(new GrMaterial { Name = name, R = r, G = g, B = b, A = a });
                _materialKeys[key] = index;
            }
            _currentMaterial = index;
        }

        public void OnPolymesh(PolymeshTopology polymesh)
        {
            if (_elements.Count == 0 || polymesh.NumberOfFacets == 0) return;
            ElementState element = _elements.Peek();
            int material = _currentMaterial >= 0 ? _currentMaterial : DefaultMaterial();
            RawMesh mesh;
            if (!element.ByMaterial.TryGetValue(material, out mesh))
            {
                mesh = new RawMesh { Material = material };
                element.ByMaterial[material] = mesh;
            }
            Transform transform = _transforms.Peek();
            uint offset = (uint)(mesh.Positions.Count / 3);
            foreach (XYZ point in polymesh.GetPoints())
            {
                XYZ world = transform.OfPoint(point);
                mesh.Positions.Add(world.X * FeetToMeters);
                mesh.Positions.Add(world.Y * FeetToMeters);
                mesh.Positions.Add(world.Z * FeetToMeters);
            }
            // A mirrored transform flips the triangle winding; swap two corners to keep faces pointing outwards.
            bool mirrored = transform.Determinant < 0;
            foreach (PolymeshFacet facet in polymesh.GetFacets())
            {
                mesh.Indices.Add(offset + (uint)facet.V1);
                mesh.Indices.Add(offset + (uint)(mirrored ? facet.V3 : facet.V2));
                mesh.Indices.Add(offset + (uint)(mirrored ? facet.V2 : facet.V3));
            }
        }

        // ---------------------------------------------------------------- helpers

        private int DefaultMaterial()
        {
            const string key = "default";
            int index;
            if (!_materialKeys.TryGetValue(key, out index))
            {
                index = _materials.Count;
                _materials.Add(new GrMaterial { Name = "Sin material", R = 0.7f, G = 0.7f, B = 0.7f, A = 1f });
                _materialKeys[key] = index;
            }
            return index;
        }

        private static string ColorHex(float r, float g, float b)
        {
            return ((int)Math.Round(r * 255)).ToString("X2") + ((int)Math.Round(g * 255)).ToString("X2")
                + ((int)Math.Round(b * 255)).ToString("X2");
        }

        private static string SafeName(Element element)
        {
            try { return element.Name; } catch { return null; }
        }

        private string TypeNameOf(Element element)
        {
            try
            {
                ElementId typeId = element.GetTypeId();
                if (typeId == null || typeId == ElementId.InvalidElementId) return null;
                var type = _doc.GetElement(typeId) as ElementType;
                if (type == null) return null;
                string family = type.FamilyName;
                return string.IsNullOrEmpty(family) ? type.Name : family + ": " + type.Name;
            }
            catch { return null; }
        }
    }
}
