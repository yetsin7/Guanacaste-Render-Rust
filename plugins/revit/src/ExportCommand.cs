using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Windows.Forms;
using Autodesk.Revit.Attributes;
using Autodesk.Revit.DB;
using Autodesk.Revit.UI;
using GuanacasteRender.Revit.Export;
using GuanacasteRender.Revit.Format;
using GuanacasteRender.Revit.Ui;
using TaskDialog = Autodesk.Revit.UI.TaskDialog;

namespace GuanacasteRender.Revit
{
    /// <summary>
    /// "Exportar a Guanacaste Render": exports the visible geometry of a 3D view to a .gr project. Read only: it
    /// never opens a transaction and never modifies the Revit model.
    /// </summary>
    [Transaction(TransactionMode.ReadOnly)]
    public sealed class ExportCommand : IExternalCommand
    {
        public Result Execute(ExternalCommandData commandData, ref string message, ElementSet elements)
        {
            Document doc = commandData.Application.ActiveUIDocument != null
                ? commandData.Application.ActiveUIDocument.Document
                : null;
            if (doc == null)
            {
                message = "Abra un proyecto de Revit antes de exportar.";
                return Result.Failed;
            }

            try
            {
                View3D view = FindView(doc, commandData.Application.ActiveUIDocument.ActiveView);
                if (view == null)
                {
                    message = "El proyecto no tiene ninguna vista 3D utilizable. Cree una vista 3D (Vista > Vista 3D > Vista 3D predeterminada) y vuelva a intentarlo.";
                    return Result.Failed;
                }

                string path = AskPath(doc);
                if (path == null) return Result.Cancelled;

                return Run(commandData.Application.Application, doc, view, path, ref message);
            }
            catch (Autodesk.Revit.Exceptions.OperationCanceledException)
            {
                return Result.Cancelled;
            }
            catch (UnauthorizedAccessException)
            {
                message = "No se pudo escribir el archivo: no hay permiso en esa carpeta. Elija otra ubicación.";
                return Result.Failed;
            }
            catch (IOException ex)
            {
                message = "No se pudo escribir el archivo (¿está abierto en otro programa o el disco está lleno?): " + ex.Message;
                return Result.Failed;
            }
            catch (OutOfMemoryException)
            {
                message = "El modelo es demasiado grande para exportarlo de una vez. Use una vista 3D con caja de sección o oculte categorías que no necesite y repita la exportación.";
                return Result.Failed;
            }
            catch (Exception ex)
            {
                message = "La exportación falló: " + ex.Message;
                return Result.Failed;
            }
        }

        private static Result Run(Autodesk.Revit.ApplicationServices.Application app, Document doc, View3D view, string path, ref string message)
        {
            var clock = Stopwatch.StartNew();
            SceneCollector collector;
            using (var progress = new ProgressForm("Exportando a Guanacaste Render"))
            {
                progress.Show();
                progress.Phase("Contando elementos de la vista...");
                progress.SetTotal(new FilteredElementCollector(doc, view.Id).WhereElementIsNotElementType().GetElementCount());
                collector = new SceneCollector(doc, () => progress.CancelRequested, progress.Report);
                using (var exporter = new CustomExporter(doc, collector))
                {
                    exporter.ShouldStopOnError = false;
                    exporter.Export(view);
                }
                if (collector.WasCanceled || progress.CancelRequested) return Result.Cancelled;

                if (collector.ElementsWithGeometry == 0)
                {
                    message = "La vista \"" + view.Name + "\" no contiene geometría visible. Revise que las categorías no estén ocultas y que la caja de sección incluya el modelo.";
                    return Result.Failed;
                }
                progress.Phase("Escribiendo el archivo .gr...");
                string fileName = ModelFileName(doc);
                GrScene scene = collector.Build(
                    ProjectName(doc),
                    "Autodesk Revit " + app.VersionNumber,
                    "Revit " + app.VersionNumber + " (compilación " + app.VersionBuild + ")",
                    fileName,
                    DateTimeOffset.UtcNow.ToUnixTimeSeconds(),
                    Guid.NewGuid().ToByteArray());
                scene.SourceFileSize = SourceSize(doc);
                GrFile.WriteFile(path, scene);

                long triangles = scene.Meshes.Sum(m => (long)m.Indices.Length / 3);
                progress.Close();
                TaskDialog.Show("Guanacaste Render",
                    "Exportación completada." + Environment.NewLine + Environment.NewLine
                    + "Archivo: " + path + Environment.NewLine
                    + "Vista: " + view.Name + Environment.NewLine
                    + "Elementos con geometría: " + collector.ElementsWithGeometry + Environment.NewLine
                    + "Mallas: " + scene.Meshes.Count + "   Triángulos: " + triangles + Environment.NewLine
                    + "Materiales: " + scene.Materials.Count + "   Niveles: " + scene.Levels.Count + Environment.NewLine
                    + "Tamaño: " + (new FileInfo(path).Length / 1024.0 / 1024.0).ToString("0.0") + " MB   Tiempo: "
                    + clock.Elapsed.TotalSeconds.ToString("0.0") + " s"
                    + (collector.SkippedLinks > 0
                        ? Environment.NewLine + Environment.NewLine + "Se omitieron " + collector.SkippedLinks + " vínculo(s) de Revit: expórtelos desde su propio proyecto."
                        : ""));
            }
            return Result.Succeeded;
        }

        /// <summary>The active view when it is a usable 3D view; otherwise the default "{3D}" view, otherwise any 3D view.</summary>
        internal static View3D FindView(Document doc, Autodesk.Revit.DB.View active)
        {
            var current = active as View3D;
            if (current != null && !current.IsTemplate) return current;
            List<View3D> views = new FilteredElementCollector(doc).OfClass(typeof(View3D)).Cast<View3D>()
                .Where(v => !v.IsTemplate).ToList();
            return views.FirstOrDefault(v => v.Name == "{3D}")
                ?? views.FirstOrDefault(v => !v.IsPerspective)
                ?? views.FirstOrDefault();
        }

        private static string AskPath(Document doc)
        {
            string suggested = Sanitize(ProjectName(doc)) + ".gr";
            string folder = null;
            try { if (!string.IsNullOrEmpty(doc.PathName)) folder = Path.GetDirectoryName(doc.PathName); } catch { }
            using (var dialog = new SaveFileDialog())
            {
                dialog.Title = "Exportar a Guanacaste Render";
                dialog.Filter = "Proyecto de Guanacaste Render (*.gr)|*.gr";
                dialog.DefaultExt = "gr";
                dialog.AddExtension = true;
                dialog.OverwritePrompt = true;
                dialog.FileName = suggested;
                if (folder != null && Directory.Exists(folder)) dialog.InitialDirectory = folder;
                return dialog.ShowDialog() == DialogResult.OK ? dialog.FileName : null;
            }
        }

        private static string ProjectName(Document doc)
        {
            string name = null;
            try { name = doc.ProjectInformation != null ? doc.ProjectInformation.Name : null; } catch { }
            if (string.IsNullOrWhiteSpace(name)) name = doc.Title;
            return string.IsNullOrWhiteSpace(name) ? "Revit" : name;
        }

        private static string ModelFileName(Document doc)
        {
            try { if (!string.IsNullOrEmpty(doc.PathName)) return Path.GetFileName(doc.PathName); } catch { }
            return doc.Title;
        }

        private static ulong SourceSize(Document doc)
        {
            try { if (!string.IsNullOrEmpty(doc.PathName) && File.Exists(doc.PathName)) return (ulong)new FileInfo(doc.PathName).Length; } catch { }
            return 0;
        }

        private static string Sanitize(string name)
        {
            foreach (char bad in Path.GetInvalidFileNameChars()) name = name.Replace(bad, '_');
            return name.Trim().Length == 0 ? "modelo" : name.Trim();
        }
    }
}
