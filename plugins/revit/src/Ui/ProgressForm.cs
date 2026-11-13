using System;
using System.Diagnostics;
using System.Drawing;
using System.Windows.Forms;

namespace GuanacasteRender.Revit.Ui
{
    /// <summary>
    /// Small modeless progress window with a Cancel button. <see cref="CustomExporter"/> runs synchronously inside
    /// Revit, so the window is refreshed from the export callbacks with a throttled message pump.
    /// </summary>
    internal sealed class ProgressForm : Form
    {
        private readonly Label _title = new Label();
        private readonly Label _detail = new Label();
        private readonly ProgressBar _bar = new ProgressBar();
        private readonly Button _cancel = new Button();
        private readonly Stopwatch _clock = Stopwatch.StartNew();
        private long _lastPump = -1000;

        public bool CancelRequested { get; private set; }

        public ProgressForm(string title)
        {
            Text = "Guanacaste Render";
            FormBorderStyle = FormBorderStyle.FixedDialog;
            StartPosition = FormStartPosition.CenterScreen;
            MaximizeBox = false;
            MinimizeBox = false;
            ShowInTaskbar = false;
            TopMost = true;
            ClientSize = new Size(440, 130);

            _title.Text = title;
            _title.Font = new Font(Font, FontStyle.Bold);
            _title.SetBounds(16, 14, 408, 20);
            _detail.SetBounds(16, 40, 408, 18);
            _detail.AutoEllipsis = true;
            _bar.SetBounds(16, 64, 408, 18);
            _bar.Style = ProgressBarStyle.Marquee;
            _cancel.Text = "Cancelar";
            _cancel.SetBounds(344, 94, 80, 26);
            _cancel.Click += delegate
            {
                CancelRequested = true;
                _cancel.Enabled = false;
                _detail.Text = "Cancelando...";
            };
            Controls.AddRange(new Control[] { _title, _detail, _bar, _cancel });
            FormClosing += delegate(object sender, FormClosingEventArgs e)
            {
                if (e.CloseReason == CloseReason.UserClosing) { CancelRequested = true; e.Cancel = true; }
            };
        }

        /// <summary>Total number of elements, when known; switches the bar from marquee to percentage.</summary>
        public void SetTotal(int total)
        {
            if (total <= 0) return;
            _bar.Style = ProgressBarStyle.Continuous;
            _bar.Minimum = 0;
            _bar.Maximum = total;
        }

        /// <summary>Updates the window at most about 10 times a second and lets it process clicks.</summary>
        public void Report(int done, string detail)
        {
            long now = _clock.ElapsedMilliseconds;
            if (now - _lastPump < 100) return;
            _lastPump = now;
            if (_bar.Style == ProgressBarStyle.Continuous)
                _bar.Value = Math.Max(_bar.Minimum, Math.Min(_bar.Maximum, done));
            if (!CancelRequested)
                _detail.Text = string.IsNullOrEmpty(detail) ? "Leyendo la geometría de la vista..." : detail;
            Application.DoEvents();
        }

        /// <summary>Shows a status line and pumps messages once (used between phases).</summary>
        public void Phase(string text)
        {
            _bar.Style = ProgressBarStyle.Marquee;
            _detail.Text = text;
            _lastPump = -1000;
            Application.DoEvents();
        }
    }
}
