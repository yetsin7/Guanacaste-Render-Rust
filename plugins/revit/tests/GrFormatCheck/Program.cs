using System;
using System.IO;
using System.Linq;
using GuanacasteRender.Revit.Format;

// Writes the deterministic sample project used by the Rust tests (src/features/projects/tests/revit_tests.rs) with the
// very same GrFile writer the Revit add-in uses. Needs no Revit installation.
//   dotnet run --project tests/GrFormatCheck -- tests/fixtures/sample.gr
internal static class Program
{
    private static int Main(string[] args)
    {
        string output = args.Length > 0 ? args[0] : "sample.gr";
        var scene = new GrScene
        {
            Name = "Casa de prueba",
            UnixTime = 1700000000,
            Application = "Autodesk Revit 2026",
            SourceVersion = "Revit 2026 (compilación 20260101)",
            SourceFileName = "casa.rvt",
            SourceFileSize = 1234567,
            OriginShift = new[] { 500.25, -300.5, 0.0 },
            DocumentId = Enumerable.Range(1, 16).Select(i => (byte)i).ToArray(),
        };
        scene.Levels.Add(new GrLevel { Name = "Nivel 1", ElevationMeters = 0.0, Node = 1 });
        scene.Levels.Add(new GrLevel { Name = "Nivel 2", ElevationMeters = 3.048, Node = 2 });
        scene.Materials.Add(new GrMaterial { Name = "Concreto", R = 0.5f, G = 0.5f, B = 0.5f, A = 1f });
        scene.Materials.Add(new GrMaterial { Name = "Vidrio", R = 0.6f, G = 0.8f, B = 0.9f, A = 0.25f });
        scene.Nodes.Add(new GrNode { Name = "Casa de prueba", Class = "Revit", Kind = "project" });
        scene.Nodes.Add(new GrNode { Name = "Nivel 1", Class = "Level", Kind = "level", Parent = 0, Level = 0 });
        scene.Nodes.Add(new GrNode { Name = "Nivel 2", Class = "Level", Kind = "level", Parent = 0, Level = 1 });
        scene.Nodes.Add(new GrNode { Name = "Muros", Class = "Muros", Kind = "group", Parent = 1, Level = 0 });
        var wall = new GrNode
        {
            Name = "Muro básico [100]", Class = "Muros", Kind = "element",
            TypeName = "Muro básico: Genérico - 200 mm", Parent = 3, Level = 0,
        };
        wall.Meshes.Add(0);
        scene.Nodes.Add(wall);
        scene.Nodes.Add(new GrNode { Name = "Ventanas", Class = "Ventanas", Kind = "group", Parent = 2, Level = 1 });
        var window = new GrNode { Name = "Ventana \"ñ\" [200]\t", Class = "Ventanas", Kind = "element", Parent = 5, Level = 1 };
        window.Meshes.Add(1);
        scene.Nodes.Add(window);
        scene.Meshes.Add(new GrMesh
        {
            Material = 0,
            Positions = new float[] { 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0 },
            Indices = new uint[] { 0, 1, 2, 0, 2, 3 },
        });
        scene.Meshes.Add(new GrMesh
        {
            Material = 1,
            Positions = new float[] { 0.1f, 0.2f, 3.048f, 2, 0, 3.048f, 2, 0, 4.5f },
            Indices = new uint[] { 0, 1, 2 },
        });

        GrFile.WriteFile(output, scene);
        Console.WriteLine(GrFile.DocumentJson(scene));
        Console.WriteLine("Escrito " + new FileInfo(output).Length + " bytes en " + output);
        return 0;
    }
}
