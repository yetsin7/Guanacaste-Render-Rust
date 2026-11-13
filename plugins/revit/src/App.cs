using System;
using System.Reflection;
using Autodesk.Revit.UI;
using GuanacasteRender.Revit.Ui;

namespace GuanacasteRender.Revit
{
    /// <summary>Creates the "Guanacaste" ribbon tab with the export button when Revit starts.</summary>
    public sealed class App : IExternalApplication
    {
        private const string TabName = "Guanacaste";
        private const string PanelName = "Guanacaste Render";

        public Result OnStartup(UIControlledApplication application)
        {
            try
            {
                try { application.CreateRibbonTab(TabName); }
                catch (Autodesk.Revit.Exceptions.ArgumentException) { /* the tab already exists */ }

                RibbonPanel panel = application.CreateRibbonPanel(TabName, PanelName);
                var data = new PushButtonData(
                    "GuanacasteRender.Export",
                    "Exportar a" + Environment.NewLine + "Guanacaste Render",
                    Assembly.GetExecutingAssembly().Location,
                    typeof(ExportCommand).FullName)
                {
                    ToolTip = "Exporta la vista 3D al formato .gr de Guanacaste Render, sin pasar por IFC.",
                    LongDescription = "Recorre la geometría visible de la vista 3D activa (o de la vista {3D}) y escribe un proyecto "
                        + ".gr con mallas en metros, materiales (color y transparencia) y la jerarquía nivel / categoría / elemento.",
                    LargeImage = IconFactory.Create(32),
                    Image = IconFactory.Create(16),
                };
                panel.AddItem(data);
                return Result.Succeeded;
            }
            catch (Exception ex)
            {
                TaskDialog.Show("Guanacaste Render", "No se pudo crear la pestaña Guanacaste: " + ex.Message);
                return Result.Failed;
            }
        }

        public Result OnShutdown(UIControlledApplication application)
        {
            return Result.Succeeded;
        }
    }
}
