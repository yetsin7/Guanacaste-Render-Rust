using System;
using System.Collections.Generic;
using GuanacasteRender.Revit.Format;

namespace GuanacasteRender.Revit.Export
{
    internal sealed partial class SceneCollector
    {
        public const string NoLevelName = "Sin nivel";
        public const string NoCategoryName = "Sin categoría";

        /// <summary>
        /// Builds the scene to write. Hierarchy: project, then one node per level (lowest first), then one group per
        /// category inside it, then the elements. Elements without a level hang from a "Sin nivel" group.
        /// X and Y are shifted by the center of the model so that float32 keeps millimeter precision far from the
        /// internal origin; Z is never shifted, so level elevations stay valid.
        /// </summary>
        public GrScene Build(string projectName, string application, string sourceVersion, string fileName, long unixTime, byte[] documentId)
        {
            var scene = new GrScene
            {
                Name = projectName,
                UnixTime = unixTime,
                Application = application,
                SourceVersion = sourceVersion,
                SourceFileName = fileName,
                DocumentId = documentId,
            };
            scene.Materials.AddRange(_materials);
            ComputeShift(scene);

            scene.Nodes.Add(new GrNode { Name = projectName, Class = "Revit", Kind = "project" });
            var levelNodes = new List<int>();
            for (int i = 0; i < _levels.Count; i++)
            {
                scene.Nodes.Add(new GrNode { Name = _levels[i].Name, Class = "Level", Kind = "level", Parent = 0, Level = i });
                int node = scene.Nodes.Count - 1;
                levelNodes.Add(node);
                scene.Levels.Add(new GrLevel { Name = _levels[i].Name, ElevationMeters = _levels[i].Elevation * FeetToMeters, Node = node });
            }

            int noLevelNode = -1;
            var groups = new Dictionary<string, int>();
            foreach (ElementState element in _finished)
            {
                string category = element.Category ?? NoCategoryName;
                int parentLevelNode;
                if (element.Level >= 0) parentLevelNode = levelNodes[element.Level];
                else
                {
                    if (noLevelNode < 0)
                    {
                        scene.Nodes.Add(new GrNode { Name = NoLevelName, Class = "", Kind = "group", Parent = 0 });
                        noLevelNode = scene.Nodes.Count - 1;
                    }
                    parentLevelNode = noLevelNode;
                }
                string groupKey = parentLevelNode + "|" + category;
                int group;
                if (!groups.TryGetValue(groupKey, out group))
                {
                    scene.Nodes.Add(new GrNode { Name = category, Class = category, Kind = "group", Parent = parentLevelNode, Level = element.Level });
                    group = scene.Nodes.Count - 1;
                    groups[groupKey] = group;
                }
                var node = new GrNode
                {
                    Name = (string.IsNullOrEmpty(element.Name) ? category : element.Name) + " [" + element.Id + "]",
                    Class = category,
                    Kind = "element",
                    TypeName = element.TypeName,
                    Parent = group,
                    Level = element.Level,
                };
                foreach (RawMesh raw in element.ByMaterial.Values)
                {
                    node.Meshes.Add(scene.Meshes.Count);
                    scene.Meshes.Add(Narrow(raw, scene.OriginShift));
                }
                scene.Nodes.Add(node);
            }
            return scene;
        }

        private void ComputeShift(GrScene scene)
        {
            double minX = double.MaxValue, maxX = double.MinValue, minY = double.MaxValue, maxY = double.MinValue;
            bool any = false;
            foreach (ElementState element in _finished)
                foreach (RawMesh mesh in element.ByMaterial.Values)
                    for (int i = 0; i < mesh.Positions.Count; i += 3)
                    {
                        any = true;
                        double x = mesh.Positions[i], y = mesh.Positions[i + 1];
                        if (x < minX) minX = x;
                        if (x > maxX) maxX = x;
                        if (y < minY) minY = y;
                        if (y > maxY) maxY = y;
                    }
            if (!any) return;
            scene.OriginShift[0] = (minX + maxX) / 2.0;
            scene.OriginShift[1] = (minY + maxY) / 2.0;
        }

        private static GrMesh Narrow(RawMesh raw, double[] shift)
        {
            var positions = new float[raw.Positions.Count];
            for (int i = 0; i < positions.Length; i += 3)
            {
                positions[i] = (float)(raw.Positions[i] - shift[0]);
                positions[i + 1] = (float)(raw.Positions[i + 1] - shift[1]);
                positions[i + 2] = (float)(raw.Positions[i + 2] - shift[2]);
            }
            return new GrMesh { Positions = positions, Indices = raw.Indices.ToArray(), Material = raw.Material };
        }

        public int ElementsWithGeometry { get { return _finished.Count; } }
    }
}
