using System.Collections.Generic;
using System.Globalization;
using System.Text;

namespace GuanacasteRender.Revit.Format
{
    /// <summary>Minimal compact JSON writer (no dependencies). Numbers always use the invariant culture.</summary>
    internal sealed class JsonOut
    {
        private readonly StringBuilder _text = new StringBuilder();
        private readonly Stack<bool> _hasItems = new Stack<bool>();
        private bool _afterKey;

        public override string ToString() { return _text.ToString(); }

        /// <summary>Emits the comma that precedes a value or key, unless a key was just written.</summary>
        private void Value()
        {
            if (_afterKey) { _afterKey = false; return; }
            if (_hasItems.Count == 0) return;
            if (_hasItems.Pop()) _text.Append(',');
            _hasItems.Push(true);
        }

        public JsonOut BeginObject() { Value(); _text.Append('{'); _hasItems.Push(false); return this; }
        public JsonOut EndObject() { _hasItems.Pop(); _text.Append('}'); return this; }
        public JsonOut BeginArray() { Value(); _text.Append('['); _hasItems.Push(false); return this; }
        public JsonOut EndArray() { _hasItems.Pop(); _text.Append(']'); return this; }

        /// <summary>Writes <c>"name":</c>; the next value is written without a comma.</summary>
        public JsonOut Key(string name)
        {
            Value();
            AppendString(name);
            _text.Append(':');
            _afterKey = true;
            return this;
        }

        public JsonOut Str(string value) { Value(); AppendString(value ?? ""); return this; }
        public JsonOut Int(long value) { Value(); _text.Append(value.ToString(CultureInfo.InvariantCulture)); return this; }
        public JsonOut Num(double value) { Value(); _text.Append(Double(value)); return this; }
        public JsonOut Float(float value) { Value(); _text.Append(Single(value)); return this; }

        /// <summary>Shortest text that reads back as the same double (17 significant digits).</summary>
        public static string Double(double value)
        {
            if (double.IsNaN(value) || double.IsInfinity(value)) value = 0;
            return value.ToString("G17", CultureInfo.InvariantCulture);
        }

        /// <summary>Text that reads back as the same float (9 significant digits).</summary>
        public static string Single(float value)
        {
            if (float.IsNaN(value) || float.IsInfinity(value)) value = 0;
            return value.ToString("G9", CultureInfo.InvariantCulture);
        }

        private void AppendString(string value)
        {
            _text.Append('"');
            foreach (char c in value)
            {
                switch (c)
                {
                    case '"': _text.Append("\\\""); break;
                    case '\\': _text.Append("\\\\"); break;
                    case '\n': _text.Append("\\n"); break;
                    case '\r': _text.Append("\\r"); break;
                    case '\t': _text.Append("\\t"); break;
                    default:
                        if (c < 0x20) _text.Append("\\u").Append(((int)c).ToString("x4", CultureInfo.InvariantCulture));
                        else _text.Append(c);
                        break;
                }
            }
            _text.Append('"');
        }
    }
}
