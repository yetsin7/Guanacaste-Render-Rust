using System;
using System.Windows.Media;
using System.Windows.Media.Imaging;

namespace GuanacasteRender.Revit.Ui
{
    /// <summary>Draws the ribbon icon in memory (a green tile with a "G" ring), so the add-in ships no image files.</summary>
    internal static class IconFactory
    {
        public static BitmapSource Create(int size)
        {
            var pixels = new byte[size * size * 4];
            double center = (size - 1) / 2.0;
            double outer = size * 0.34, inner = size * 0.20;
            for (int y = 0; y < size; y++)
            {
                for (int x = 0; x < size; x++)
                {
                    double dx = x - center, dy = y - center;
                    double distance = Math.Sqrt(dx * dx + dy * dy);
                    bool ring = distance <= outer && distance >= inner;
                    bool gap = dx > 0 && dy < 0 && dy > -size * 0.14 && distance > inner * 0.9;
                    bool bar = dx > 0 && dx < outer && Math.Abs(dy - size * 0.02) < size * 0.05 && distance <= outer;
                    bool white = (ring && !gap) || bar;
                    double edge = Math.Min(x, Math.Min(y, Math.Min(size - 1 - x, size - 1 - y)));
                    double corner = size * 0.18;
                    bool inside = edge >= 0 && !(Math.Min(x, size - 1 - x) < corner && Math.Min(y, size - 1 - y) < corner
                        && Math.Sqrt(Math.Pow(corner - Math.Min(x, size - 1 - x), 2) + Math.Pow(corner - Math.Min(y, size - 1 - y), 2)) > corner);
                    int at = (y * size + x) * 4;
                    if (!inside) continue;
                    double t = (double)y / size;
                    pixels[at] = white ? (byte)255 : (byte)(70 - 25 * t);
                    pixels[at + 1] = white ? (byte)255 : (byte)(150 - 40 * t);
                    pixels[at + 2] = white ? (byte)255 : (byte)(60 - 20 * t);
                    pixels[at + 3] = 255;
                }
            }
            BitmapSource image = BitmapSource.Create(size, size, 96, 96, PixelFormats.Bgra32, null, pixels, size * 4);
            image.Freeze();
            return image;
        }
    }
}
